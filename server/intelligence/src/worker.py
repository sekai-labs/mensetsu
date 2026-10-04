import asyncio

from temporalio.client import Client
from temporalio.worker import Worker

from src.activities.research_activities import (
    calculate_match_analysis_activity,
    construct_knowledge_graph_activity,
    generate_interview_questions_activity,
    generate_learning_units_activity,
    parse_job_description_activity,
    parse_resume_activity,
    research_company_activity,
    update_workspace_progress,
)
from src.db.client import settings
from src.workflows.interview_workflow import InterviewPreparationWorkflow


async def main():
    print(f"Connecting Temporal worker to {settings.temporal_host}...")
    try:
        client = await Client.connect(settings.temporal_host)
        worker = Worker(
            client,
            task_queue=settings.task_queue,
            workflows=[InterviewPreparationWorkflow],
            activities=[
                update_workspace_progress,
                parse_resume_activity,
                parse_job_description_activity,
                research_company_activity,
                calculate_match_analysis_activity,
                construct_knowledge_graph_activity,
                generate_learning_units_activity,
                generate_interview_questions_activity,
            ],
        )
        print("Worker running on queue:", settings.task_queue)
        await worker.run()
    except Exception as e:
        print(f"Temporal server not running locally ({e}). Operating in standalone/trigger mode.")

if __name__ == "__main__":
    asyncio.run(main())
