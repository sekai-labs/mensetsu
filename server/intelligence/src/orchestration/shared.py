from src.contracts.v1 import WorkflowDefinition
from src.orchestration.ledger import SurrealLedgerRepository, set_event_publisher
from src.orchestration.redis_queue import RedisStreamJobQueue

default_ledger = SurrealLedgerRepository()
job_queue = RedisStreamJobQueue()
set_event_publisher(job_queue.publish_event)


class _LedgerProxy:
    def __getattr__(self, name):
        from src.agents.runtime import current_ledger_var

        ctx_ledger = current_ledger_var.get()
        target = ctx_ledger if ctx_ledger is not None else default_ledger
        return getattr(target, name)


ledger = _LedgerProxy()


def get_ledger() -> SurrealLedgerRepository:
    from src.agents.runtime import current_ledger_var

    ctx_ledger = current_ledger_var.get()
    return ctx_ledger if ctx_ledger is not None else default_ledger


DEFINITIONS = [
    WorkflowDefinition(
        id="interview-preparation-static",
        name="Static Interview Preparation Workflow",
        type="static",
        version="1.0.0",
        description="Deterministic fan-out/fan-in research and question generation pipeline",
        default_max_retries=3,
        supported_agents=[
            "company-research",
            "job-analysis",
            "technical-research",
            "systems-deep-dive",
            "evidence-evaluator",
            "curriculum",
            "interview-generation",
        ],
    ),
    WorkflowDefinition(
        id="interview-preparation-dynamic",
        name="Dynamic Interview Preparation Workflow",
        type="dynamic",
        version="1.0.0",
        description="Adaptive planner-driven DAG research and study path compiler",
        default_max_retries=3,
        supported_agents=[
            "planner",
            "company-research",
            "job-analysis",
            "technical-research",
            "systems-deep-dive",
            "evidence-evaluator",
            "curriculum",
            "interview-generation",
        ],
    ),
    WorkflowDefinition(
        id="interview-preparation-hybrid",
        name="Hybrid Interview Preparation Workflow",
        type="hybrid",
        version="1.0.0",
        description="Adaptive prompt-directed research DAG combined with deterministic interview preparation milestones",
        default_max_retries=3,
        supported_agents=[
            "planner",
            "company-research",
            "job-analysis",
            "technical-research",
            "systems-deep-dive",
            "evidence-evaluator",
            "curriculum",
            "interview-generation",
        ],
    ),
]
