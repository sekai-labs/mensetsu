import logging
import os
import re
from dataclasses import dataclass
from typing import Literal
from urllib.parse import quote_plus

import httpx
from bs4 import BeautifulSoup

from src.fetcher.ssrf_client import validate_url_for_ssrf

logger = logging.getLogger('mensetsu.search')

@dataclass
class SearchResult:
    title: str
    url: str
    snippet: str
    score: float = 0.9

class AgentSearchEngine:

    def __init__(self, timeout: float=12.0):
        self.timeout = timeout
        self.tavily_api_key = os.environ.get('TAVILY_API_KEY')
        self.serpapi_api_key = os.environ.get('SERPAPI_API_KEY')
        self.searxng_url = os.environ.get('SEARXNG_URL')

    async def search(self, query: str, limit: int=5, search_depth: Literal['basic', 'advanced']='advanced') -> list[SearchResult]:
        results: list[SearchResult] = []
        if self.tavily_api_key:
            try:
                results = await self._search_tavily(query, limit, search_depth)
                if results:
                    return results
            except Exception as e:
                logger.warning(f"Tavily search failed for '{query}': {e}. Falling back.")
        if self.serpapi_api_key:
            try:
                results = await self._search_serpapi(query, limit)
                if results:
                    return results
            except Exception as e:
                logger.warning(f"SerpAPI search failed for '{query}': {e}. Falling back.")
        if self.searxng_url:
            try:
                results = await self._search_searxng(query, limit)
                if results:
                    return results
            except Exception as e:
                logger.warning(f"SearXNG search failed for '{query}': {e}. Falling back.")
        try:
            results = await self._search_duckduckgo(query, limit)
            if results:
                return results
        except Exception as e:
            logger.warning(f"DuckDuckGo search failed for '{query}': {e}.")
        return results

    async def _search_tavily(self, query: str, limit: int, depth: str) -> list[SearchResult]:
        url = 'https://api.tavily.com/search'
        payload = {'api_key': self.tavily_api_key, 'query': query, 'search_depth': depth, 'max_results': limit, 'include_answer': False}
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            resp = await client.post(url, json=payload)
            resp.raise_for_status()
            data = resp.json()
            out = []
            for item in data.get('results', []):
                out.append(SearchResult(title=item.get('title', ''), url=item.get('url', ''), snippet=item.get('content', ''), score=float(item.get('score', 0.9))))
            return out

    async def _search_serpapi(self, query: str, limit: int) -> list[SearchResult]:
        url = 'https://serpapi.com/search.json'
        params = {'engine': 'google', 'q': query, 'api_key': self.serpapi_api_key, 'num': limit}
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            resp = await client.get(url, params=params)
            resp.raise_for_status()
            data = resp.json()
            out = []
            for item in data.get('organic_results', [])[:limit]:
                out.append(SearchResult(title=item.get('title', ''), url=item.get('link', ''), snippet=item.get('snippet', ''), score=0.9))
            return out

    async def _search_searxng(self, query: str, limit: int) -> list[SearchResult]:
        url = f"{self.searxng_url.rstrip('/')}/search"
        params = {'q': query, 'format': 'json'}
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            resp = await client.get(url, params=params)
            resp.raise_for_status()
            data = resp.json()
            out = []
            for item in data.get('results', [])[:limit]:
                out.append(SearchResult(title=item.get('title', ''), url=item.get('url', ''), snippet=item.get('content', ''), score=float(item.get('score', 0.85))))
            return out

    async def _search_duckduckgo(self, query: str, limit: int) -> list[SearchResult]:
        url = f'https://html.duckduckgo.com/html/?q={quote_plus(query)}'
        headers = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36', 'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8', 'Accept-Language': 'en-US,en;q=0.5'}
        async with httpx.AsyncClient(timeout=self.timeout, follow_redirects=True) as client:
            resp = await client.get(url, headers=headers)
            if resp.status_code != 200:
                return []
            soup = BeautifulSoup(resp.text, 'html.parser')
            results: list[SearchResult] = []
            for result_div in soup.find_all('div', class_=re.compile('result\\b')):
                if len(results) >= limit:
                    break
                title_elem = result_div.find('a', class_='result__a')
                snippet_elem = result_div.find('a', class_='result__snippet')
                if not title_elem:
                    continue
                raw_href = title_elem.get('href', '')
                actual_url = raw_href
                if 'uddg=' in raw_href:
                    import urllib.parse
                    parsed = urllib.parse.urlparse(raw_href)
                    qs = urllib.parse.parse_qs(parsed.query)
                    if 'uddg' in qs:
                        actual_url = qs['uddg'][0]
                if not actual_url.startswith('http'):
                    continue
                try:
                    validate_url_for_ssrf(actual_url)
                except Exception:
                    continue
                title = title_elem.get_text(strip=True)
                snippet = snippet_elem.get_text(strip=True) if snippet_elem else ''
                results.append(SearchResult(title=title, url=actual_url, snippet=snippet, score=0.85))
            return results
