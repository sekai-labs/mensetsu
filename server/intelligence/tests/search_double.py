from src.fetcher.search_engine import SearchResult


class InMemorySearchEngine:
    def __init__(self, corpus: list[dict[str, str]] | None = None):
        self.corpus = corpus or [
            {
                "title": "About Python",
                "url": "https://www.python.org/about/",
                "snippet": "Python is a programming language used to build distributed systems.",
            },
            {
                "title": "Python Blogs",
                "url": "https://www.python.org/blogs/",
                "snippet": "Engineering posts about runtime internals and concurrency.",
            },
        ]
        self.queries: list[str] = []

    async def search_web(self, query: str, limit: int = 5, search_depth: str = "advanced") -> list[SearchResult]:
        self.queries.append(query)
        return [
            SearchResult(
                title=item["title"],
                url=item["url"],
                snippet=item["snippet"],
                provider="in_memory_test",
                query=query,
            )
            for item in self.corpus[:limit]
        ]

    async def search(self, query: str, limit: int = 5, search_depth: str = "advanced") -> list[SearchResult]:
        return await self.search_web(query, limit=limit, search_depth=search_depth)


def install_search_double() -> InMemorySearchEngine:
    from src.agents.runtime import agent_runtime

    engine = InMemorySearchEngine()
    agent_runtime.set_search_engine(engine)
    return engine
