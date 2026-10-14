import asyncio
import hashlib
import logging
import re
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any, Literal
from urllib.parse import urlparse

import httpx
from bs4 import BeautifulSoup, NavigableString, Tag

from src.fetcher.ssrf_client import SafeResearchFetcher, SSRFSecurityError, validate_url_for_ssrf

logger = logging.getLogger("mensetsu.reader")


@dataclass
class FetchedDocument:
    url: str
    canonical_url: str
    title: str
    markdown: str
    raw_html: str = ""
    domain: str = ""
    content_hash: str = ""
    retrieval_method: Literal["http_readability", "pdf", "unsupported"] = "http_readability"
    status_code: int = 200
    access_status: Literal[
        "success", "blocked_or_captcha", "not_found", "ssrf_violation", "timeout", "error"
    ] = "success"
    structured_data: list[dict[str, Any]] = field(default_factory=list)
    retrieved_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())


class MarkdownReader:
    @staticmethod
    def html_to_markdown(
        html: str, max_chars: int = 35000
    ) -> tuple[str, str, list[dict[str, Any]]]:
        soup = BeautifulSoup(html, "html.parser")
        title = soup.title.string.strip() if soup.title and soup.title.string else "Untitled Page"

        structured_data = []
        for script in soup.find_all("script", type="application/ld+json"):
            import json

            try:
                if script.string:
                    data = json.loads(script.string)
                    if isinstance(data, list):
                        structured_data.extend(data)
                    elif isinstance(data, dict):
                        structured_data.append(data)
            except Exception:
                pass

        for tag in soup(
            [
                "script",
                "style",
                "noscript",
                "iframe",
                "svg",
                "nav",
                "footer",
                "header",
                "aside",
                "form",
                "button",
            ]
        ):
            tag.decompose()

        noise_pattern = re.compile(
            r"cookie|banner|ad-|advertisement|sidebar|popup|modal|newsletter|subscribe|sharing|social",
            re.I,
        )
        for noise in soup.find_all(class_=noise_pattern):
            noise.decompose()
        for noise in soup.find_all(id=noise_pattern):
            noise.decompose()

        main_content = (
            soup.find("article")
            or soup.find("main")
            or soup.find("div", role="main")
            or soup.find(
                "div", class_=re.compile(r"content|article|post|job-description|description", re.I)
            )
            or soup.body
            or soup
        )

        md_lines = []

        def recurse(node):
            if isinstance(node, NavigableString):
                text = str(node)
                if text.strip():
                    md_lines.append(text)
                return
            if not isinstance(node, Tag):
                return

            name = node.name.lower()
            if name in ["h1", "h2", "h3", "h4", "h5", "h6"]:
                level = int(name[1])
                prefix = "#" * level
                text = node.get_text(separator=" ", strip=True)
                if text:
                    md_lines.append(f"\n\n{prefix} {text}\n")
                return
            if name == "p":
                text = node.get_text(separator=" ", strip=True)
                if text:
                    md_lines.append(f"\n\n{text}\n")
                return
            if name in ["pre", "code"]:
                code_text = node.get_text()
                if "\n" in code_text:
                    md_lines.append(f"\n\n```\n{code_text.strip()}\n```\n")
                else:
                    md_lines.append(f" `{code_text.strip()}` ")
                return
            if name == "li":
                text = node.get_text(separator=" ", strip=True)
                if text:
                    md_lines.append(f"\n- {text}")
                return
            if name == "blockquote":
                text = node.get_text(separator=" ", strip=True)
                if text:
                    md_lines.append(f"\n> {text}\n")
                return

            for child in node.children:
                recurse(child)

        recurse(main_content)
        markdown = "".join(md_lines)
        markdown = re.sub(r"\n{3,}", "\n\n", markdown).strip()
        return title, markdown[:max_chars], structured_data


class SafeReaderFetcher:
    def __init__(self, timeout_seconds: float = 12.0):
        self.timeout = timeout_seconds
        self.fetcher = SafeResearchFetcher(timeout_seconds=timeout_seconds)
        self._domain_locks: dict[str, asyncio.Lock] = {}

    def _get_domain_lock(self, domain: str) -> asyncio.Lock:
        if domain not in self._domain_locks:
            self._domain_locks[domain] = asyncio.Lock()
        return self._domain_locks[domain]

    async def fetch_document(self, url: str) -> FetchedDocument:
        validate_url_for_ssrf(url)
        parsed = urlparse(url)
        domain = parsed.netloc

        lock = self._get_domain_lock(domain)
        async with lock:
            await asyncio.sleep(0.1)

            raw_html = ""
            canonical_url = url
            retrieval_method: Literal["http_readability", "pdf", "unsupported"] = "http_readability"
            access_status: Literal[
                "success", "blocked_or_captcha", "not_found", "ssrf_violation", "timeout", "error"
            ] = "success"

            try:
                async with httpx.AsyncClient(
                    timeout=self.timeout, follow_redirects=False
                ) as client:
                    current_url = url
                    resp = None
                    for _ in range(5):
                        validate_url_for_ssrf(current_url)
                        resp = await client.get(
                            current_url,
                            headers={
                                "User-Agent": (
                                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                                    "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 MensetsuBot/2.0"
                                ),
                                "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                            },
                        )
                        if resp.is_redirect:
                            loc = resp.headers.get("Location")
                            if not loc:
                                break
                            current_url = str(httpx.URL(current_url).join(loc))
                            continue
                        break

                    if resp is None:
                        raise ValueError(f"Failed to fetch content from {url}")

                    canonical_url = current_url
                    if resp.status_code in [401, 403, 429]:
                        access_status = "blocked_or_captcha"
                    elif resp.status_code == 404:
                        access_status = "not_found"
                    elif resp.status_code >= 400:
                        access_status = "error"

                    raw_html = resp.text
            except SSRFSecurityError:
                return FetchedDocument(
                    url=url,
                    canonical_url=url,
                    title="Blocked by Security Policy",
                    markdown="",
                    domain=domain,
                    access_status="ssrf_violation",
                    retrieval_method="unsupported",
                )
            except Exception as e:
                logger.warning(f"HTTP fetch failed for {url}: {e}")
                access_status = "error"

            title, markdown, structured_data = MarkdownReader.html_to_markdown(raw_html)
            content_hash = hashlib.sha256(markdown.encode("utf-8")).hexdigest()

            return FetchedDocument(
                url=url,
                canonical_url=canonical_url,
                title=title,
                markdown=markdown,
                raw_html=raw_html,
                domain=domain,
                content_hash=content_hash,
                retrieval_method=retrieval_method,
                access_status=access_status,
                structured_data=structured_data,
            )

    async def fetch_markdown(self, url: str) -> tuple[str, str]:
        doc = await self.fetch_document(url)
        return doc.title, doc.markdown
