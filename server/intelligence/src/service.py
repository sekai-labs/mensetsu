import asyncio

from fastapi import BackgroundTasks, FastAPI
from pydantic import BaseModel

from src.activities.progress import log_pipeline_step, update_workspace_progress
from src.activities.research_activities import (
    calculate_match_analysis_activity,
    construct_knowledge_graph_activity,
    generate_interview_questions_activity,
    generate_learning_units_activity,
    parse_job_description_activity,
    parse_resume_activity,
    research_company_activity,
)

app = FastAPI(title="Mensetsu Intelligence Service")

class TriggerWorkflowRequest(BaseModel):
    workspace_id: str
    company_name: str
    role_title: str
    company_urls: list[str] = []
    resume_text: str = ""
    resume_pdf_base64: str | None = None
    resume_filename: str | None = None
    job_description: str = ""
    jd_url: str | None = None
    seniority: str | None = None
    llm_config: dict | None = None
async def execute_pipeline_background(req: TriggerWorkflowRequest):
    ws_id = req.workspace_id
    try:
        await log_pipeline_step(ws_id, "preparing_input", "info", "Starting pipeline execution and input preparation")
        await update_workspace_progress(ws_id, "preparing_input", 5)
        await asyncio.sleep(0.1)

        await log_pipeline_step(ws_id, "analyzing_resume", "info", f"Extracting candidate competencies from resume (PDF: {req.resume_filename or 'text'})")
        await update_workspace_progress(ws_id, "analyzing_resume", 15)
        cv_res = await parse_resume_activity(
            ws_id,
            resume_text=req.resume_text,
            resume_pdf_base64=req.resume_pdf_base64,
            resume_filename=req.resume_filename,
        )
        await log_pipeline_step(ws_id, "analyzing_resume", "info", f"Extracted {len(cv_res.get('extracted_skills', []))} candidate skills")
        await asyncio.sleep(0.1)

        await log_pipeline_step(ws_id, "reading_job_description", "info", f"Parsing job requirements (URL: {req.jd_url or 'raw text'})")
        await update_workspace_progress(ws_id, "reading_job_description", 30)
        jd_res = await parse_job_description_activity(
            ws_id,
            jd_text=req.job_description,
            jd_url=req.jd_url,
        )
        await log_pipeline_step(ws_id, "reading_job_description", "info", f"Parsed {len(jd_res.get('requirements', []))} role requirements")
        await asyncio.sleep(0.1)

        await log_pipeline_step(ws_id, "researching_company", "info", f"Researching {req.company_name} engineering profile, architecture topology, and sources")
        await update_workspace_progress(ws_id, "researching_company", 45)
        company_res = await research_company_activity(ws_id, req.company_name, req.company_urls)
        await log_pipeline_step(ws_id, "researching_company", "info", f"Indexed {len(company_res.get('sources', []))} sources and {len(company_res.get('claims', []))} claims")
        await asyncio.sleep(0.1)

        await log_pipeline_step(ws_id, "mapping_resume_evidence", "info", "Computing deterministic match analysis and gap matrix")
        await update_workspace_progress(ws_id, "mapping_resume_evidence", 60)
        resume_skill_names = [s["name"] for s in cv_res.get("extracted_skills", [])]
        match_res = await calculate_match_analysis_activity(ws_id, resume_skill_names, jd_res.get("requirements", []))
        await log_pipeline_step(ws_id, "mapping_resume_evidence", "info", f"Match score: {match_res.get('overall_match_ratio', 0.0) * 100:.0f}%, {len(match_res.get('gaps', []))} gaps identified")
        await asyncio.sleep(0.1)

        await log_pipeline_step(ws_id, "constructing_knowledge_graph", "info", "Assembling isolated workspace competency DAG")
        await update_workspace_progress(ws_id, "constructing_knowledge_graph", 75)
        graph_res = await construct_knowledge_graph_activity(ws_id, req.company_name, resume_skill_names)
        await log_pipeline_step(ws_id, "constructing_knowledge_graph", "info", f"Persisted {len(graph_res.get('nodes', []))} nodes and {len(graph_res.get('edges', []))} edges")
        await asyncio.sleep(0.1)

        await log_pipeline_step(ws_id, "generating_lessons", "info", "Building tailored curriculum lessons for identified gaps")
        await update_workspace_progress(ws_id, "generating_lessons", 85)
        await generate_learning_units_activity(ws_id, match_res.get("gaps", []))
        await log_pipeline_step(ws_id, "generating_lessons", "info", "Curriculum lessons successfully persisted")
        await asyncio.sleep(0.1)

        await log_pipeline_step(ws_id, "building_interview_question_book", "info", "Generating targeted 5-tier question book with rubrics and pitfalls")
        await update_workspace_progress(ws_id, "building_interview_question_book", 95)
        q_res = await generate_interview_questions_activity(ws_id, req.company_name, req.role_title)
        await log_pipeline_step(ws_id, "building_interview_question_book", "info", f"Generated {len(q_res)} interview questions")
        await asyncio.sleep(0.1)

        await log_pipeline_step(ws_id, "ready", "info", "Workspace pipeline completed successfully")
        await update_workspace_progress(ws_id, "ready", 100)
    except Exception as e:
        err_msg = str(e)
        print(f"Error executing pipeline for {ws_id}: {err_msg}")
        await log_pipeline_step(ws_id, "failed", "error", f"Pipeline failed: {err_msg}", details={"error": err_msg})
        await update_workspace_progress(ws_id, "failed", 0, error_message=err_msg)

@app.post("/api/v1/workflows/trigger")
async def trigger_workflow(req: TriggerWorkflowRequest, background_tasks: BackgroundTasks):
    background_tasks.add_task(execute_pipeline_background, req)
    return {"status": "triggered", "workspace_id": req.workspace_id}

@app.get("/health")
async def health():
    return {"status": "healthy"}
