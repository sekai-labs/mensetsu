from typing import Literal

from pydantic import BaseModel, Field
from temporalio import activity

from src.activities.persistence import persist_record
from src.db.client import SurrealDirectClient
from src.llm.roles import ModelRole
from src.llm.router import router

db_client = SurrealDirectClient()

class InterviewQuestionSpec(BaseModel):
    category: Literal['resume_deep_dive', 'project_deep_dive', 'technical_fundamentals', 'target_stack', 'system_design', 'debugging', 'behavioral'] = 'system_design'
    question: str
    difficulty: Literal['easy', 'medium', 'hard'] = 'medium'
    skills_tested: list[str] = Field(default_factory=list)
    interviewer_intent: str
    answer_outline: list[str] = Field(default_factory=list)
    strong_answer_rubric: list[str] = Field(default_factory=list)
    common_weaknesses: list[str] = Field(default_factory=list)
    follow_up_questions: list[str] = Field(default_factory=list)

class QuestionBookOutput(BaseModel):
    questions: list[InterviewQuestionSpec] = Field(default_factory=list)

def heuristic_questions_fallback(company_name: str, target_role: str) -> QuestionBookOutput:
    return QuestionBookOutput(questions=[
        InterviewQuestionSpec(
            category='resume_deep_dive',
            question="In your previous systems, you implemented asynchronous task processing and state transitions. When worker nodes crashed midway through execution, how did you prevent duplicate message dispatch, avoid zombie leases, and recover in-flight transactions without losing linearizable state?",
            difficulty='hard',
            skills_tested=['Distributed Systems', 'Idempotency', 'Lease Renewal', 'Fault Tolerance'],
            interviewer_intent="Evaluates candidate's real-world systems debugging experience under dirty crashes and network partitions.",
            answer_outline=[
                '1. Explain heartbeat mechanisms and bounded lease duration (e.g. 30s TTL with renew interval 10s).',
                '2. Discuss fencing tokens to reject stale writes from zombie nodes after split-brain recovery.',
                '3. Describe transactional outbox and unique idempotency deduplication keys in storage.',
                '4. Outline dead letter queue routing with backoff policies for unrecoverable errors.'
            ],
            strong_answer_rubric=[
                'Articulates split-brain scenarios and fencing tokens rather than relying on client wall-clock time.',
                'Demonstrates explicit distinction between at-least-once transport and exactly-once processing via idempotency keys.',
                'Addresses cascading crash loops caused by poisoned payloads.'
            ],
            common_weaknesses=[
                'Assuming TCP connection termination automatically notifies master during network partitions.',
                'Relying solely on local in-memory lock without distributed coordination.'
            ],
            follow_up_questions=[
                'What happens if the heartbeat renew RPC succeeds but the database write fails?',
                'How do you handle worker scale-in when 500 tasks are currently executing?'
            ]
        ),
        InterviewQuestionSpec(
            category='system_design',
            question=f"Design an end-to-end event ingestion and analysis platform for {company_name} that processes 200,000 requests/sec with p99 latency under 20ms and multi-region active-active durability.",
            difficulty='hard',
            skills_tested=['System Design', 'Ingress Architecture', 'Partitioning', 'Event Sourcing'],
            interviewer_intent=f"Probes architectural intuition around network ingress, partitioning, queue backpressure, and cross-region consensus at {company_name}.",
            answer_outline=[
                '1. Clarify ingress requirements, payload sizes, read vs write ratios, and consistency models.',
                '2. Propose Anycast routing, global API gateways, and asynchronous buffering tier (e.g. Kafka/Redpanda).',
                '3. Design consistent hashing key distribution to prevent hotspotting.',
                '4. Outline storage partitioning with WAL replication and multi-tier read caches.'
            ],
            strong_answer_rubric=[
                'Calculates realistic capacity numbers (200k req/sec * 1KB payload = ~200MB/s ingress throughput).',
                'Proactively addresses hot partitions and backpressure mitigation (load shedding, client exponential backoff).',
                'Explains cross-region conflict resolution strategies (CRDTs or single-leader region per tenant).'
            ],
            common_weaknesses=[
                'Proposing direct synchronous writes to a relational database without buffering.',
                'Overlooking cross-region egress latency and replication lag.'
            ],
            follow_up_questions=[
                'If one region experiences a 30-second fiber cut, how do client clients failover without transaction drops?',
                'How do you secure tenant data isolation in this shared ingestion pipeline?'
            ]
        ),
        InterviewQuestionSpec(
            category='technical_fundamentals',
            question="Explain the Linux memory model, page faults, and how memory-mapped I/O (mmap) interacts with the kernel page cache under high read concurrency.",
            difficulty='medium',
            skills_tested=['Linux Internals', 'Virtual Memory', 'mmap', 'Kernel Concurrency'],
            interviewer_intent="Verifies systems fundamentals and low-level understanding of kernel virtual memory management and page cache contention.",
            answer_outline=[
                '1. Walk through virtual address translation via page tables, TLB cache, and MMU.',
                '2. Explain major vs minor page faults and disk seek penalties.',
                '3. Describe how mmap avoids user-to-kernel buffer copies compared to read()/write().',
                '4. Analyze TLB shootdowns and lock contention in page eviction under memory pressure.'
            ],
            strong_answer_rubric=[
                'Distinguishes between anonymous memory and file-backed pages.',
                'Explains madvise flags (MADV_WILLNEED, MADV_DONTNEED, MADV_HUGEPAGE) for memory-intensive servers.',
                'Points out writeback synchronization risks and msync semantics.'
            ],
            common_weaknesses=[
                'Confusing physical RAM addresses with virtual memory allocations.',
                'Believing mmap avoids all kernel overhead or eliminates disk I/O.'
            ],
            follow_up_questions=[
                'Why would a database choose direct I/O (O_DIRECT) over mmap?',
                'How does NUMA node locality affect memory latency in multi-socket cloud instances?'
            ]
        ),
        InterviewQuestionSpec(
            category='target_stack',
            question=f"When building high-throughput microservices in Rust or Go for {company_name}, how do you manage connection pooling, gracefully drain in-flight requests during rolling updates, and propagate distributed trace contexts across gRPC boundaries?",
            difficulty='medium',
            skills_tested=['Rust/Go Runtimes', 'gRPC', 'Connection Pooling', 'OpenTelemetry'],
            interviewer_intent=f"Tests practical proficiency with {company_name}'s production microservices stack and observability conventions.",
            answer_outline=[
                '1. Discuss keep-alive pings, idle connection timeouts, and max concurrent streams per HTTP/2 connection.',
                '2. Outline SIGTERM trap handling, readiness probe de-registration, and connection draining timers.',
                '3. Explain W3C TraceContext propagation via gRPC metadata headers (traceparent and tracestate).'
            ],
            strong_answer_rubric=[
                'Mentions Kubernetes preStop hooks to prevent race condition between service endpoint removal and pod termination.',
                'Explains backpressure propagation on bounded channel queues to prevent out-of-memory crashes.',
                'Articulates context cancellation propagation across asynchronous futures/goroutines.'
            ],
            common_weaknesses=[
                'Terminating process immediately upon receiving SIGINT without draining active HTTP/gRPC requests.',
                'Neglecting to handle connection churn against upstream DNS updates.'
            ],
            follow_up_questions=[
                'How do you detect memory leaks or unbounded channel growth in production?',
                'What happens if an upstream client disconnects mid-computation?'
            ]
        ),
        InterviewQuestionSpec(
            category='debugging',
            question="A production service suddenly exhibits a spike in p99 latency from 15ms to 2,500ms while CPU and memory usage remain at only 25%. Walk me through your step-by-step triage methodology to identify and resolve the root cause.",
            difficulty='hard',
            skills_tested=['Incident Triage', 'Observability', 'Lock Contention', 'Downstream Saturation'],
            interviewer_intent="Evaluates diagnostic methodology, hypothesis formation under pressure, and production debugging maturity.",
            answer_outline=[
                '1. Assess blast radius, active alerts, error budgets, and recent deployments/config changes.',
                '2. Check upstream request rate and inspect distributed traces for high-duration spans.',
                '3. Check blocking I/O: connection pool starvation, database lock contention, or slow external third-party calls.',
                '4. Examine thread dumps, off-CPU profiling, and garbage collection / allocator lock delays.',
                '5. Execute immediate mitigation (canary rollback, traffic diversion, circuit tripping) before root-cause deep dive.'
            ],
            strong_answer_rubric=[
                'Prioritizes customer mitigation (rollback / circuit breaker) before extensive code forensics.',
                'Leverages distributed trace spans to immediately isolate which downstream dependency or mutex is blocked.',
                'Identifies off-CPU wait time as the primary suspect when CPU utilization is low but latency is high.'
            ],
            common_weaknesses=[
                'Guessing and rebooting servers without capturing thread dumps, logs, or metrics.',
                'Focusing exclusively on CPU-bound profiling when CPU utilization is clearly idle.'
            ],
            follow_up_questions=[
                'If the database shows zero slow queries, what other downstream resource could cause thread starvation?',
                'How would you write a regression prevention test or alert for this scenario?'
            ]
        )
    ])
def deduplicate_questions(questions: list[InterviewQuestionSpec]) -> list[InterviewQuestionSpec]:
    seen = set()
    deduped = []
    for q in questions:
        norm = ' '.join(q.question.strip().lower().split())
        if norm and norm not in seen:
            seen.add(norm)
            deduped.append(q)
    return deduped

@activity.defn
async def generate_interview_questions_activity(workspace_id: str, company_name: str, target_role: str) -> list[dict]:
    system_prompt = f'You are a bar-raiser technical interviewer at {company_name} interviewing a candidate for {target_role}. Generate a diverse set of rigorous interview questions covering system design, fundamentals, target stack, and deep dives. Each question must contain an interviewer intent, detailed answer outline, strong rubric points, common weaknesses, and follow-up probes. Every question MUST be distinct and unique.'
    try:
        book = await router.generate_structured(schema=QuestionBookOutput, system_prompt=system_prompt, user_prompt=f'Generate interview questions for {company_name} ({target_role}).', role=ModelRole.QUESTIONS)
        if not book.questions:
            book = heuristic_questions_fallback(company_name, target_role)
    except Exception:
        book = heuristic_questions_fallback(company_name, target_role)
    unique_questions = deduplicate_questions(book.questions)
    persisted = []
    for idx, q in enumerate(unique_questions):
        q_id = f'q_{workspace_id}_{idx}'
        q_dict = {'id': q_id, 'workspace_id': workspace_id, 'category': q.category, 'question': q.question, 'difficulty': q.difficulty, 'skills_tested': q.skills_tested, 'interviewer_intent': q.interviewer_intent, 'answer_outline': q.answer_outline, 'strong_answer_rubric': q.strong_answer_rubric, 'common_weaknesses': q.common_weaknesses, 'follow_up_questions': q.follow_up_questions}
        await persist_record(db_client=db_client, table='interview_question', record_id=q_id, fields=q_dict)
        persisted.append(q_dict)
    return persisted
