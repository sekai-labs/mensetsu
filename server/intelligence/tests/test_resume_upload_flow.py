import asyncio
from unittest.mock import AsyncMock, MagicMock, patch
import pytest

from src.contracts.v1 import WorkflowRun
from src.orchestration.static_workflow import StaticWorkflowState, build_static_workflow


def _make_pdf(text: str) -> bytes:
    content = f"BT /F1 12 Tf 72 712 Td ({text}) Tj ET".encode("latin1")
    stream_len = len(content)
    return (
        b"%PDF-1.4\n"
        b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n"
        b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n"
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n"
        b"4 0 obj\n<< /Length " + str(stream_len).encode("ascii") + b" >>\nstream\n"
        + content + b"\nendstream\nendobj\n"
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n"
        b"xref\n0 6\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n"
        b"0000000224 00000 n \n0000000300 00000 n \ntrailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n380\n%%EOF\n"
    )


@pytest.mark.asyncio
async def test_static_normalize_with_resume_upload_success(monkeypatch):
    monkeypatch.setenv("INTERNAL_SERVICE_TOKEN", "test-token")
    monkeypatch.setenv("S3_DOWNLOAD_ALLOWED_ORIGIN", "http://minio:9000")
    monkeypatch.setenv("API_GRPC_ENDPOINT", "localhost:50052")

    pdf_data = _make_pdf("Built reliable event pipelines in Rust")

    mock_resp = MagicMock()
    mock_resp.url = "http://minio:9000/mensetsu-resumes/resumes/final/test.pdf"
    mock_resp.filename = "resume.pdf"
    mock_resp.size_bytes = len(pdf_data)

    mock_stub = MagicMock()
    mock_stub.GetResumeDownloadUrl = AsyncMock(return_value=mock_resp)

    mock_ledger = MagicMock()
    mock_ledger.get_task = AsyncMock(return_value=None)
    mock_ledger.next_task_attempt = AsyncMock(return_value=1)
    mock_ledger.save_task = AsyncMock()
    mock_ledger.append_event_transactional = AsyncMock()
    mock_ledger.update_workflow_run = AsyncMock()
    mock_ledger.db = MagicMock()
    mock_ledger.db.execute_query = AsyncMock(return_value=[])

    class MockStreamResponse:
        status_code = 200
        async def __aenter__(self):
            return self
        async def __aexit__(self, exc_type, exc_val, exc_tb):
            pass
        async def aiter_bytes(self):
            yield pdf_data

    class MockAsyncClient:
        def __init__(self, **kwargs):
            pass
        async def __aenter__(self):
            return self
        async def __aexit__(self, exc_type, exc_val, exc_tb):
            pass
        def stream(self, method, url):
            return MockStreamResponse()

    with patch("src.grpc.v1.resume_storage_pb2_grpc.ResumeStorageServiceStub", return_value=mock_stub), \
         patch("httpx.AsyncClient", MockAsyncClient), \
         patch("src.generation.resume.parse_resume_activity", new_callable=AsyncMock) as mock_parse:

        mock_parse.return_value = {
            "extracted_skills": [{"name": "Rust"}],
            "candidate_name": "Test Candidate",
            "summary": "Rust Developer",
            "raw_text": "Built reliable event pipelines in Rust",
        }

        graph = build_static_workflow(checkpointer=None, ledger=mock_ledger, db_client=mock_ledger.db, mode="normalize_only")
        res = await graph.ainvoke(
            StaticWorkflowState(
                run_id="run_test_pdf_upload",
                workspace_id="ws_test_123",
                user_id="usr_test_123",
                company_name="Acme",
                role_title="Senior Rust Engineer",
                resume_upload_id="upl_test_123",
                job_description="Seeking Rust engineer",
            ),
            config={"configurable": {"thread_id": "test_thread"}},
        )

        assert "Built reliable event pipelines in Rust" in res["resume_text"]
        upsert_calls = mock_ledger.db.execute_query.call_args_list
        assert any("UPSERT type::record('resume'" in str(call) for call in upsert_calls)


@pytest.mark.asyncio
async def test_static_normalize_rejects_disallowed_s3_origin(monkeypatch):
    monkeypatch.setenv("INTERNAL_SERVICE_TOKEN", "test-token")
    monkeypatch.setenv("S3_DOWNLOAD_ALLOWED_ORIGIN", "http://minio:9000")
    monkeypatch.setenv("API_GRPC_ENDPOINT", "localhost:50052")

    mock_resp = MagicMock()
    mock_resp.url = "http://attacker.com/evil.pdf"
    mock_resp.filename = "resume.pdf"
    mock_resp.size_bytes = 100

    mock_stub = MagicMock()
    mock_stub.GetResumeDownloadUrl = AsyncMock(return_value=mock_resp)

    mock_ledger = MagicMock()
    mock_ledger.get_task = AsyncMock(return_value=None)
    mock_ledger.next_task_attempt = AsyncMock(return_value=1)
    mock_ledger.save_task = AsyncMock()
    mock_ledger.append_event_transactional = AsyncMock()
    mock_ledger.update_workflow_run = AsyncMock()
    mock_ledger.db = MagicMock()

    with patch("src.grpc.v1.resume_storage_pb2_grpc.ResumeStorageServiceStub", return_value=mock_stub):
        graph = build_static_workflow(checkpointer=None, ledger=mock_ledger, db_client=mock_ledger.db, mode="normalize_only")
        with pytest.raises(Exception, match="origin 'http://attacker.com' does not strictly match allowed origin"):
            await graph.ainvoke(
                StaticWorkflowState(
                    run_id="run_test_ssrf",
                    workspace_id="ws_test_123",
                    user_id="usr_test_123",
                    company_name="Acme",
                    role_title="Senior Rust Engineer",
                    resume_upload_id="upl_test_123",
                    job_description="Seeking Rust engineer",
                ),
                config={"configurable": {"thread_id": "test_thread"}},
            )
