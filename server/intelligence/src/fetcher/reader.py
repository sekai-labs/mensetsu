import re

from bs4 import BeautifulSoup, NavigableString, Tag

from src.fetcher.ssrf_client import SafeResearchFetcher, validate_url_for_ssrf


class MarkdownReader:

    @staticmethod
    def html_to_markdown(html: str, max_chars: int=25000) -> tuple[str, str]:
        soup = BeautifulSoup(html, 'html.parser')
        title = soup.title.string.strip() if soup.title and soup.title.string else 'Untitled Page'
        for tag in soup(['script', 'style', 'noscript', 'iframe', 'svg', 'nav', 'footer', 'header', 'aside', 'form', 'button']):
            tag.decompose()
        noise_pattern = re.compile('cookie|banner|ad-|advertisement|sidebar|popup|modal|newsletter|subscribe|sharing|social', re.I)
        for noise in soup.find_all(class_=noise_pattern):
            noise.decompose()
        for noise in soup.find_all(id=noise_pattern):
            noise.decompose()
        main_content = soup.find('article') or soup.find('main') or soup.find('div', role='main') or soup.body or soup
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
            if name in ['h1', 'h2', 'h3', 'h4', 'h5', 'h6']:
                level = int(name[1])
                prefix = '#' * level
                text = node.get_text(separator=' ', strip=True)
                if text:
                    md_lines.append(f'\n\n{prefix} {text}\n')
                return
            if name == 'p':
                text = node.get_text(separator=' ', strip=True)
                if text:
                    md_lines.append(f'\n\n{text}\n')
                return
            if name == 'pre' or name == 'code':
                code_text = node.get_text()
                if '\n' in code_text:
                    md_lines.append(f'\n\n```\n{code_text.strip()}\n```\n')
                else:
                    md_lines.append(f' `{code_text.strip()}` ')
                return
            if name == 'li':
                text = node.get_text(separator=' ', strip=True)
                if text:
                    md_lines.append(f'\n- {text}')
                return
            if name == 'blockquote':
                text = node.get_text(separator=' ', strip=True)
                if text:
                    md_lines.append(f'\n> {text}\n')
                return
            for child in node.children:
                recurse(child)
        recurse(main_content)
        markdown = ''.join(md_lines)
        markdown = re.sub('\\n{3,}', '\n\n', markdown).strip()
        return (title, markdown[:max_chars])

class SafeReaderFetcher:

    def __init__(self, timeout_seconds: float = 12.0):
        self.timeout = timeout_seconds
        self.fetcher = SafeResearchFetcher(timeout_seconds=timeout_seconds)

    async def fetch_markdown(self, url: str) -> tuple[str, str]:
        validate_url_for_ssrf(url)
        import httpx
        async with httpx.AsyncClient(timeout=self.timeout, follow_redirects=False) as client:
            current_url = url
            resp = None
            for _ in range(5):
                validate_url_for_ssrf(current_url)
                resp = await client.get(
                    current_url,
                    headers={'User-Agent': 'MensetsuResearchReader/2.0 (+https://mensetsu.local)'}
                )
                if resp.is_redirect:
                    location = resp.headers.get('Location')
                    if not location:
                        break
                    current_url = str(httpx.URL(current_url).join(location))
                    continue
                resp.raise_for_status()
                break
            if resp is None:
                raise ValueError(f"Failed to fetch content from {url}")
            html = resp.text
        return MarkdownReader.html_to_markdown(html)
