from datetime import timedelta

from temporalio import workflow
from temporalio.common import RetryPolicy

with workflow.unsafe.imports_passed_through():
    from src.activities import (
        calculate_match_analysis_activity,
        construct_knowledge_graph_activity,
        generate_interview_questions_activity,
        generate_learning_units_activity,
        parse_job_description_activity,
        parse_resume_activity,
        research_company_activity,
        update_workspace_progress,
    )


@workflow.defn
class InterviewPreparationWorkflow:
    @workflow.run
    async def run(self, input_data: dict) -> dict:
        workspace_id = input_data["workspace_id"]
        company_name = input_data.get("company_name", "Target Company")
        role_title = input_data.get("role_title", "Software Engineer")
        resume_text = input_data.get("resume_text", "")
        jd_text = input_data.get("job_description", "")
        company_urls = input_data.get("company_urls", [])

        default_retry = RetryPolicy(
            initial_interval=timedelta(seconds=1),
            backoff_coefficient=2.0,
            maximum_interval=timedelta(seconds=30),
            maximum_attempts=3,
        )

                            
        await workflow.execute_activity(
            update_workspace_progress,
            args=[workspace_id, "preparing_input", 5],
            start_to_close_timeout=timedelta(seconds=15),
            retry_policy=default_retry,
        )

                             
        await workflow.execute_activity(
            update_workspace_progress,
            args=[workspace_id, "analyzing_resume", 15],
            start_to_close_timeout=timedelta(seconds=15),
            retry_policy=default_retry,
        )
        cv_result = await workflow.execute_activity(
            parse_resume_activity,
            args=[workspace_id, resume_text],
            start_to_close_timeout=timedelta(seconds=60),
            retry_policy=default_retry,
        )

                                    
        await workflow.execute_activity(
            update_workspace_progress,
            args=[workspace_id, "reading_job_description", 30],
            start_to_close_timeout=timedelta(seconds=15),
            retry_policy=default_retry,
        )
        jd_result = await workflow.execute_activity(
            parse_job_description_activity,
            args=[workspace_id, jd_text],
            start_to_close_timeout=timedelta(seconds=60),
            retry_policy=default_retry,
        )

                                
        await workflow.execute_activity(
            update_workspace_progress,
            args=[workspace_id, "researching_company", 45],
            start_to_close_timeout=timedelta(seconds=15),
            retry_policy=default_retry,
        )
        await workflow.execute_activity(
            research_company_activity,
            args=[workspace_id, company_name, company_urls],
            start_to_close_timeout=timedelta(seconds=90),
            retry_policy=default_retry,
        )

                                                   
        await workflow.execute_activity(
            update_workspace_progress,
            args=[workspace_id, "mapping_resume_evidence", 60],
            start_to_close_timeout=timedelta(seconds=15),
            retry_policy=default_retry,
        )
        resume_skill_names = [s["name"] for s in cv_result.get("extracted_skills", [])]
        match_result = await workflow.execute_activity(
            calculate_match_analysis_activity,
            args=[workspace_id, resume_skill_names, jd_result.get("requirements", [])],
            start_to_close_timeout=timedelta(seconds=60),
            retry_policy=default_retry,
        )

                                         
        await workflow.execute_activity(
            update_workspace_progress,
            args=[workspace_id, "constructing_knowledge_graph", 75],
            start_to_close_timeout=timedelta(seconds=15),
            retry_policy=default_retry,
        )
        await workflow.execute_activity(
            construct_knowledge_graph_activity,
            args=[workspace_id, company_name, resume_skill_names],
            start_to_close_timeout=timedelta(seconds=60),
            retry_policy=default_retry,
        )

                                            
        await workflow.execute_activity(
            update_workspace_progress,
            args=[workspace_id, "generating_lessons", 85],
            start_to_close_timeout=timedelta(seconds=15),
            retry_policy=default_retry,
        )
        await workflow.execute_activity(
            generate_learning_units_activity,
            args=[workspace_id, match_result.get("gaps", [])],
            start_to_close_timeout=timedelta(seconds=90),
            retry_policy=default_retry,
        )

                                             
        await workflow.execute_activity(
            update_workspace_progress,
            args=[workspace_id, "building_interview_question_book", 95],
            start_to_close_timeout=timedelta(seconds=15),
            retry_policy=default_retry,
        )
        await workflow.execute_activity(
            generate_interview_questions_activity,
            args=[workspace_id, company_name, role_title],
            start_to_close_timeout=timedelta(seconds=90),
            retry_policy=default_retry,
        )

                  
        await workflow.execute_activity(
            update_workspace_progress,
            args=[workspace_id, "ready", 100],
            start_to_close_timeout=timedelta(seconds=15),
            retry_policy=default_retry,
        )

        return {"status": "ready", "workspace_id": workspace_id}
