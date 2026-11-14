from src.fetcher.search_engine import AgentSearchEngine, SearchResult


def test_search_deduplication_handles_null_text_and_keeps_distinct_urls_with_same_title():
    results = [
        SearchResult(
            title=None,
            url="https://example.com/a#section",
            snippet=None,
            provider="test",
        ),
        SearchResult(
            title="Same title",
            url="https://example.com/b",
            snippet="Distinct page",
            provider="test",
        ),
        SearchResult(
            title="Duplicate URL",
            url="https://example.com/a",
            snippet="Duplicate result",
            provider="test",
        ),
    ]

    deduplicated = AgentSearchEngine()._deduplicate_results(results, query="search")

    assert [result.url for result in deduplicated] == [
        "https://example.com/a#section",
        "https://example.com/b",
    ]
    assert deduplicated[0].title == ""
    assert deduplicated[0].snippet == ""
    assert all(result.query == "search" for result in deduplicated)
