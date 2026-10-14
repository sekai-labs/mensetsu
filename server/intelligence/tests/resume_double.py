import re
import uuid

from langchain_core.language_models.chat_models import BaseChatModel
from langchain_core.messages import AIMessage, ToolMessage
from langchain_core.outputs import ChatGeneration, ChatResult

RESUME_LABEL = "CANDIDATE_RESUME"

KNOWN_SKILLS = (
    ("Distributed Systems", "Architecture"),
    ("Rust", "Systems"),
    ("Python", "Backend"),
    ("Kubernetes", "Cloud"),
    ("PostgreSQL", "Database"),
    ("Docker", "DevOps"),
    ("TypeScript", "Frontend"),
    ("Vue", "Frontend"),
    ("Kafka", "Messaging"),
    ("Linux", "Systems"),
    ("Go", "Systems"),
    ("C++", "Systems"),
    ("Ceph", "Storage"),
    ("Paxos", "Architecture"),
)


def resume_block(messages) -> str | None:
    for m in messages:
        content = getattr(m, "content", "")
        if not isinstance(content, str):
            continue
        match = re.search(rf"<{RESUME_LABEL}>.*?\n---\n(.*)\n</{RESUME_LABEL}>", content, re.S)
        if match:
            return match.group(1)
    return None


def resume_extraction_payload(resume_text: str) -> dict:
    lines = [line.strip() for line in resume_text.splitlines() if line.strip()]
    skills = []
    for name, category in KNOWN_SKILLS:
        pattern = re.compile(rf"(?<![\w+]){re.escape(name)}(?![\w+])", re.I)
        quote = next(
            (line for line in lines if pattern.search(line) and "[REDACTED" not in line), None
        )
        if quote:
            skills.append(
                {
                    "name": name,
                    "category": category,
                    "evidence": [
                        {
                            "skill_or_topic": name,
                            "quote": quote,
                            "context": "Stated in resume",
                            "confidence": 0.9,
                        }
                    ],
                    "confidence": 0.9,
                }
            )
    return {
        "candidate_name": None,
        "summary": None,
        "skills": skills,
        "experience_highlights": [],
    }


def resume_extraction_response(messages) -> ChatResult | None:
    block = resume_block(messages)
    if block is None:
        return None
    msg = AIMessage(
        content="",
        tool_calls=[
            {
                "name": "DynamicResumeExtraction",
                "args": resume_extraction_payload(block),
                "id": f"tc_resume_{uuid.uuid4().hex[:6]}",
                "type": "tool_call",
            }
        ],
    )
    return ChatResult(generations=[ChatGeneration(message=msg)])


class ResumeDoubleModel(BaseChatModel):
    fabricated_attempts: int = 0
    calls: int = 0
    seen_errors: list[str] = []

    @property
    def _llm_type(self) -> str:
        return "resume_double_model"

    def bind_tools(self, tools, **kwargs):
        return self

    def _generate(self, messages, stop=None, run_manager=None, **kwargs):
        raise NotImplementedError

    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        self.calls += 1
        for m in messages:
            if isinstance(m, ToolMessage):
                self.seen_errors.append(str(m.content))
        result = resume_extraction_response(messages)
        if result is None:
            raise AssertionError("ResumeDoubleModel only answers resume extraction prompts")
        if self.calls <= self.fabricated_attempts:
            call = result.generations[0].message.tool_calls[0]
            for skill in call["args"]["skills"]:
                for ev in skill["evidence"]:
                    ev["quote"] = f"Referenced {skill['name']} in resume."
        return result
