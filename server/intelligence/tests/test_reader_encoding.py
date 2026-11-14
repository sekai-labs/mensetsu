import httpx
import pytest

import src.fetcher.reader as reader_module
from src.fetcher.reader import SafeReaderFetcher


@pytest.mark.asyncio
async def test_fetch_document_uses_declared_response_charset(monkeypatch):
    expected_title = "求人情報"
    expected_text = "日本語の職務内容"
    html = (
        f"<html><head><title>{expected_title}</title></head>"
        f"<body><main><p>{expected_text}</p></main></body></html>"
    )
    response = httpx.Response(
        200,
        headers={"content-type": "text/html; charset=shift_jis"},
        content=html.encode("shift_jis"),
    )

    async def fetch_response(*_args, **_kwargs):
        return response

    monkeypatch.setattr(reader_module, "fetch_limited_response", fetch_response)
    monkeypatch.setattr(reader_module, "validate_url_for_ssrf", lambda _url: None)

    document = await SafeReaderFetcher().fetch_document("https://example.invalid/job")

    assert document.title == expected_title
    assert document.status_code == 200
    assert expected_text in document.markdown
