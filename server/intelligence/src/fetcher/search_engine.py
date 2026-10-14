import asyncio
import logging
import os
import time
from collections import defaultdict
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any, Literal

import httpx

from src.fetcher.ssrf_client import validate_url_for_ssrf

logger = logging.getLogger("mensetsu.search")


@dataclass
class SearchResult:
    title: str
    url: str
    snippet: str
    provider: str
    query: str = ""
    score: float | None = None
    published_date: str | None = None
    discovery_timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())



class TokenBucketRateLimiter:
    def __init__(self, rate: float = 1.0, burst: float = 2.0):
        self.rate = rate
        self.capacity = burst
        self.tokens = burst
        self.last_time = time.monotonic()
        self._lock = asyncio.Lock()

    async def acquire(self) -> None:
        async with self._lock:
            now = time.monotonic()
            elapsed = now - self.last_time
            self.last_time = now
            self.tokens = min(self.capacity, self.tokens + elapsed * self.rate)
            if self.tokens < 1.0:
                wait_time = (1.0 - self.tokens) / self.rate
                await asyncio.sleep(wait_time)
                self.tokens = 0.0
                self.last_time = time.monotonic()
            else:
                self.tokens -= 1.0


_PROVIDER_LIMITERS: dict[str, TokenBucketRateLimiter] = defaultdict(
    lambda: TokenBucketRateLimiter(rate=0.5, burst=1.0)
)


def _provider_score(raw: Any) -> float | None:
    try:
        return float(raw) if raw is not None else None
    except (TypeError, ValueError):
        return None


class AgentSearchEngine:
    def __init__(self, timeout: float = 12.0):
        self.timeout = timeout
        self.tavily_api_key = os.environ.get("TAVILY_API_KEY")
        self.brave_api_key = os.environ.get("BRAVE_API_KEY")
        self.serpapi_api_key = os.environ.get("SERPAPI_API_KEY")
        self.searxng_url = os.environ.get("SEARXNG_URL")
        self.history: list[dict[str, Any]] = []
        self._cache: dict[str, list[SearchResult]] = {}
        self.last_search_blocked: bool = False
    async def search(
        self,
        query: str,
        limit: int = 5,
        search_depth: Literal["basic", "advanced"] = "advanced",
    ) -> list[SearchResult]:
        return await self.search_web(query, limit=limit, search_depth=search_depth)

    async def search_web(
        self,
        query: str,
        limit: int = 5,
        search_depth: Literal["basic", "advanced"] = "advanced",
    ) -> list[SearchResult]:
        clean_q = query.strip()
        if not clean_q:
            return []

        cache_key = f"web:{clean_q}:{limit}"
        if cache_key in self._cache:
            return self._cache[cache_key]

        start_time = time.time()
        results: list[SearchResult] = []
        provider_used = "none"
        attempted_count = 0
        blocked_count = 0

        providers = [
            ("searxng", self.searxng_url, lambda: self._search_searxng(clean_q, limit)),
            ("tavily", self.tavily_api_key, lambda: self._search_tavily(clean_q, limit, search_depth)),
            ("brave", self.brave_api_key, lambda: self._search_brave(clean_q, limit)),
            ("serpapi", self.serpapi_api_key, lambda: self._search_serpapi(clean_q, limit)),
        ]
        for name, configured, call in providers:
            if results or not configured:
                continue
            attempted_count += 1
            try:
                await _PROVIDER_LIMITERS[name].acquire()
                found = await call()
            except Exception as e:
                blocked_count += 1
                logger.warning(f"provider blocked: {name} failed for '{clean_q}': {e}. Falling back.")
                continue
            if found is None:
                blocked_count += 1
            elif found:
                results = found
                provider_used = name

        self.last_search_blocked = attempted_count == 0 or blocked_count >= attempted_count
        deduped = self._deduplicate_results(results, query=clean_q)
        self.history.append(
            {
                "query": clean_q,
                "provider": provider_used,
                "results_count": len(deduped),
                "duration_seconds": round(time.time() - start_time, 3),
                "timestamp": datetime.now(timezone.utc).isoformat(),
            }
        )
        self._cache[cache_key] = deduped
        return deduped

    async def search_domain(self, domain: str, query: str, limit: int = 5) -> list[SearchResult]:
        clean_domain = domain.replace("https://", "").replace("http://", "").split("/")[0]
        structured_query = f"site:{clean_domain} {query}".strip()
        return await self.search_web(structured_query, limit=limit)

    async def search_recent(
        self, query: str, date_range: str = "y", limit: int = 5
    ) -> list[SearchResult]:
        structured_query = f"{query} recent developments updates".strip()
        return await self.search_web(structured_query, limit=limit)

    async def search_documents(
        self, query: str, file_types: list[str], limit: int = 5
    ) -> list[SearchResult]:
        ft_clause = " OR ".join([f"filetype:{ft}" for ft in file_types])
        structured_query = f"{query} ({ft_clause})".strip()
        return await self.search_web(structured_query, limit=limit)

    async def search_repositories(self, query: str, limit: int = 5) -> list[SearchResult]:
        structured_query = f"site:github.com {query}".strip()
        return await self.search_web(structured_query, limit=limit)

    def _deduplicate_results(self, results: list[SearchResult], query: str) -> list[SearchResult]:
        seen_urls = set()
        seen_titles = set()
        cleaned: list[SearchResult] = []

        for r in results:
            if not r.url:
                continue
            normalized_url = r.url.split("#")[0].rstrip("/")
            title_norm = r.title.strip().lower()

            if normalized_url in seen_urls:
                continue
            if title_norm and title_norm in seen_titles:
                continue

            seen_urls.add(normalized_url)
            if title_norm:
                seen_titles.add(title_norm)

            r.query = query
            cleaned.append(r)

        return cleaned

    async def _search_tavily(self, query: str, limit: int, depth: str) -> list[SearchResult]:
        url = "https://api.tavily.com/search"
        payload = {
            "api_key": self.tavily_api_key,
            "query": query,
            "search_depth": depth,
            "max_results": limit,
            "include_answer": False,
        }
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            resp = await client.post(url, json=payload)
            resp.raise_for_status()
            data = resp.json()
            out = []
            for item in data.get("results", []):
                u = item.get("url", "")
                try:
                    validate_url_for_ssrf(u)
                except Exception:
                    continue
                out.append(
                    SearchResult(
                        title=item.get("title", ""),
                        url=u,
                        snippet=item.get("content", ""),
                        provider="tavily",
                        query=query,
                        score=_provider_score(item.get("score")),
                        published_date=item.get("published_date"),
                    )
                )
            return out

    async def _search_brave(self, query: str, limit: int) -> list[SearchResult]:
        url = "https://api.search.brave.com/res/v1/web/search"
        headers = {
            "Accept": "application/json",
            "X-Subscription-Token": self.brave_api_key or "",
        }
        params = {"q": query, "count": limit}
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            resp = await client.get(url, headers=headers, params=params)
            resp.raise_for_status()
            data = resp.json()
            out = []
            for item in data.get("web", {}).get("results", [])[:limit]:
                u = item.get("url", "")
                try:
                    validate_url_for_ssrf(u)
                except Exception:
                    continue
                out.append(
                    SearchResult(
                        title=item.get("title", ""),
                        url=u,
                        snippet=item.get("description", ""),
                        provider="brave",
                        query=query,
                        score=None,
                        published_date=item.get("page_age"),
                    )
                )
            return out

    async def _search_serpapi(self, query: str, limit: int) -> list[SearchResult]:
        url = "https://serpapi.com/search.json"
        params = {
            "engine": "google",
            "q": query,
            "api_key": self.serpapi_api_key,
            "num": limit,
        }
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            resp = await client.get(url, params=params)
            resp.raise_for_status()
            data = resp.json()
            out = []
            for item in data.get("organic_results", [])[:limit]:
                u = item.get("link", "")
                try:
                    validate_url_for_ssrf(u)
                except Exception:
                    continue
                out.append(
                    SearchResult(
                        title=item.get("title", ""),
                        url=u,
                        snippet=item.get("snippet", ""),
                        provider="serpapi",
                        query=query,
                        score=None,
                        published_date=item.get("date"),
                    )
                )
            return out

    async def _search_searxng(self, query: str, limit: int) -> list[SearchResult] | None:
        url = f"{self.searxng_url.rstrip('/')}/search"
        params = {"q": query, "format": "json"}
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            resp = await client.get(url, params=params)
            if resp.status_code in (202, 403, 429) or resp.status_code >= 400:
                logger.warning(f"provider blocked: searxng returned status {resp.status_code}")
                return None
            data = resp.json()
            raw_results = data.get("results", [])
            if not raw_results and data.get("unresponsive_engines"):
                logger.warning(
                    f"provider blocked: searxng upstream engines unresponsive {data.get('unresponsive_engines')}"
                )
                return None
            out = []
            for item in raw_results[:limit]:
                u = item.get("url", "")
                try:
                    validate_url_for_ssrf(u)
                except Exception:
                    continue
                out.append(
                    SearchResult(
                        title=item.get("title", ""),
                        url=u,
                        snippet=item.get("content", ""),
                        provider="searxng",
                        query=query,
                        score=_provider_score(item.get("score")),
                        published_date=item.get("publishedDate"),
                    )
                )
            return out
