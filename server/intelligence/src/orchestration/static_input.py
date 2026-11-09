"""Input normalization node for the static workflow."""
import logging
import time
from typing import Any

from src.contracts.v1 import WorkflowTask
from src.orchestration.static_support import StaticNodeSupport, StaticWorkflowState

logger = logging.getLogger("mensetsu.orchestration.static_workflow")


def build_input_nodes(support: StaticNodeSupport):
    ledger = support.ledger
    db = support.db
    check_lease = support.check_lease
    set_phase = support.set_phase
    restored_job_description_text = support.restored_job_description_text
    fetch_job_description = support.fetch_job_description

    async def normalize_input_node(state: StaticWorkflowState) -> dict[str, Any]:
        check_lease()
        normalize_task_id = f"task_normalize_{state.run_id[:8]}"
        existing = await ledger.get_task(state.run_id, normalize_task_id)
        if existing is not None and existing.status == "completed":
            from src.generation.resume import load_parsed_resume

            stored_cv = existing.output_data.get("parsed_resume")
            restored_cv = (
                state.parsed_resume
                or (
                    stored_cv if isinstance(stored_cv, dict) and stored_cv.get("raw_text") else None
                )
                or await load_parsed_resume(state.workspace_id, db_client=db)
            )
            if restored_cv is None:
                raise RuntimeError(
                    f"Run {state.run_id}: normalize task is completed but its parsed CV is not recoverable."
                )
            return {
                "parsed_resume": restored_cv,
                "resume_text": state.resume_text.strip() or restored_cv["raw_text"],
                "job_description": state.job_description.strip()
                or await restored_job_description_text(state.run_id, existing),
            }
        await set_phase(state, "analyzing_resume", 5)
        start_t = time.perf_counter()
        task = WorkflowTask(
            id=f"task_normalize_{state.run_id[:8]}",
            run_id=state.run_id,
            task_type="normalization",
            name="Normalize CV and JD inputs",
            assigned_agent="job-analysis",
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(task.run_id, task.id)
        task.attempt = first_attempt
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={
                "task_id": task.id,
                "attempt": first_attempt,
                "assigned_agent": task.assigned_agent,
            },
        )

        from src.generation.documents import extract_resume_pdf_bytes
        from src.generation.persistence import persist_record
        from src.generation.resume import parse_resume_activity

        try:
            resume_text = state.resume_text.strip()
            resume_source = "text" if resume_text else None

            if state.resume_upload_id:
                import os
                from urllib.parse import urlparse

                import httpx

                import grpc
                from src.grpc.v1 import resume_storage_pb2, resume_storage_pb2_grpc
                from src.orchestration.auth import get_internal_token

                grpc_endpoint = (
                    os.environ.get("API_GRPC_ENDPOINT")
                    or os.environ.get("API_GRPC_BIND")
                    or "api:50052"
                )
                if grpc_endpoint.startswith("http://"):
                    grpc_endpoint = grpc_endpoint[len("http://") :]
                elif grpc_endpoint.startswith("https://"):
                    grpc_endpoint = grpc_endpoint[len("https://") :]

                internal_token = get_internal_token()
                channel = grpc.aio.insecure_channel(grpc_endpoint)
                try:
                    stub = resume_storage_pb2_grpc.ResumeStorageServiceStub(channel)
                    metadata = (("authorization", f"Bearer {internal_token}"),)
                    req = resume_storage_pb2.GetResumeDownloadUrlRequest(
                        workspace_id=state.workspace_id,
                        user_id=state.user_id,
                        resume_upload_id=state.resume_upload_id,
                    )
                    resp = await stub.GetResumeDownloadUrl(req, metadata=metadata, timeout=10.0)
                except grpc.RpcError as rpc_err:
                    code = rpc_err.code()
                    details = rpc_err.details()
                    raise RuntimeError(f"Resume storage gRPC error ({code.name}): {details}") from rpc_err
                finally:
                    await channel.close()

                download_url = resp.url
                declared_filename = resp.filename
                declared_size = resp.size_bytes

                allowed_origin = os.environ.get("S3_DOWNLOAD_ALLOWED_ORIGIN", "").rstrip("/")
                if not allowed_origin:
                    raise ValueError("S3_DOWNLOAD_ALLOWED_ORIGIN is unset or empty; refusing download.")

                parsed_url = urlparse(download_url)
                actual_origin = f"{parsed_url.scheme}://{parsed_url.netloc}".rstrip("/")
                if actual_origin != allowed_origin:
                    raise ValueError(
                        f"Download URL origin '{actual_origin}' does not strictly match allowed origin '{allowed_origin}'."
                    )

                max_allowed = 5 * 1024 * 1024
                async with httpx.AsyncClient(follow_redirects=False, timeout=30.0, trust_env=False) as client:
                    async with client.stream("GET", download_url) as stream_resp:
                        if stream_resp.status_code != 200:
                            raise RuntimeError(
                                f"Failed to download resume from S3 (HTTP {stream_resp.status_code})"
                            )
                        content_chunks: list[bytes] = []
                        total_downloaded = 0
                        async for chunk in stream_resp.aiter_bytes():
                            total_downloaded += len(chunk)
                            if total_downloaded > max_allowed:
                                raise ValueError(f"Downloaded resume exceeds {max_allowed} bytes limit.")
                            content_chunks.append(chunk)
                        pdf_bytes = b"".join(content_chunks)

                if len(pdf_bytes) != declared_size:
                    raise ValueError(
                        f"Downloaded size ({len(pdf_bytes)} bytes) does not match declared size ({declared_size} bytes)."
                    )

                resume_text = extract_resume_pdf_bytes(
                    pdf_bytes, filename=state.resume_filename or declared_filename
                )
                resume_source = "pdf"

                await persist_record(
                    db_client=db,
                    table="resume",
                    record_id=f"res_{state.workspace_id}",
                    fields={
                        "workspace_id": state.workspace_id,
                        "raw_text": resume_text,
                    },
                )
            elif not resume_text:
                resume_text = f"Candidate applying for {state.role_title} at {state.company_name}. Background in software engineering and technology."
                resume_source = "default"

            job_description = state.job_description.strip()
            jd_evidence_id = None
            if not job_description and state.jd_url:
                job_description, jd_evidence_id = await fetch_job_description(state, task.id)

            parsed_cv = await parse_resume_activity(
                workspace_id=state.workspace_id,
                resume_text=resume_text,
                db_client=db,
            )
        except Exception as e:
            task.status = "failed"
            task.error = str(e)
            await ledger.save_task(task)
            await ledger.append_event_transactional(
                run_id=state.run_id,
                event_type="task.failed",
                task_id=task.id,
                payload={
                    "task_id": task.id,
                    "attempt": first_attempt,
                    "error": str(e),
                    "can_retry": False,
                },
            )
            raise

        skills_count = len(parsed_cv.get("extracted_skills", []))
        dur = (time.perf_counter() - start_t) * 1000
        task.status = "completed"
        task.output_data = {
            "skills_count": skills_count,
            "resume_source": resume_source,
            "job_description_source": "url" if jd_evidence_id else "text",
            "jd_evidence_id": jd_evidence_id,
            "parsed_resume": parsed_cv,
            "job_description_text": job_description,
        }
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.completed",
            task_id=task.id,
            payload={
                "task_id": task.id,
                "duration_ms": dur,
                "output_summary": f"CV ({resume_source}) extracted {skills_count} skills.",
            },
        )
        await set_phase(state, "researching_company", 15)
        return {
            "parsed_resume": parsed_cv,
            "resume_text": resume_text,
            "job_description": job_description,
        }

    return normalize_input_node
