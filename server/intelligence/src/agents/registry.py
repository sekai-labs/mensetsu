import logging
from pathlib import Path

import yaml
from pydantic import BaseModel, Field

logger = logging.getLogger("mensetsu.agents.registry")


class AgentDefinition(BaseModel):
    id: str
    name: str
    description: str
    allowed_tools: list[str] = Field(default_factory=list)
    delegation_permissions: list[str] = Field(default_factory=list)
    max_budget: float = 1.0
    tool_calls_budget: int = 50
    model_calls_budget: int = 50
    wall_time_seconds: int = 360
    llm_request_timeout_s: float = Field(default=90.0, gt=0)
    max_concurrency: int = Field(default=3, ge=1)
    instructions: str = ""


class AgentRegistry:
    def __init__(self, agents_dir: Path | None = None):
        self.agents_dir = agents_dir or (Path(__file__).resolve().parents[2] / "agents")
        self._agents: dict[str, AgentDefinition] = {}
        self.reload()

    def reload(self) -> None:
        self._agents.clear()
        if not self.agents_dir.exists():
            raise RuntimeError(
                f"Agents directory {self.agents_dir} does not exist. Failing closed at startup."
            )

        KNOWN_TOOLS = {
            "web_search",
            "read_source",
            "save_evidence",
            "save_finding",
            "update_notes",
            "delegate_task",
            "evaluate_findings",
        }

        for subdir in self.agents_dir.iterdir():
            if subdir.is_dir():
                yaml_file = subdir / "agent.yaml"
                md_file = subdir / "instructions.md"
                if not yaml_file.exists():
                    raise RuntimeError(f"Agent directory {subdir} is missing agent.yaml.")
                if not md_file.exists():
                    raise RuntimeError(f"Agent directory {subdir} is missing instructions.md.")

                data = yaml.safe_load(yaml_file.read_text(encoding="utf-8"))
                instructions = md_file.read_text(encoding="utf-8").strip()
                if not instructions:
                    raise RuntimeError(f"Agent {subdir} instructions.md is empty.")

                defn = AgentDefinition(**data, instructions=instructions)

                for tool in defn.allowed_tools:
                    if tool not in KNOWN_TOOLS:
                        raise RuntimeError(f"Agent {defn.id} declares unknown tool '{tool}'.")

                self._agents[defn.id] = defn

        for defn in self._agents.values():
            for target in defn.delegation_permissions:
                if target not in self._agents:
                    raise RuntimeError(
                        f"Agent {defn.id} has unknown delegation permission '{target}'."
                    )

    def get(self, agent_id: str) -> AgentDefinition:
        if agent_id not in self._agents:
            raise KeyError(f"Unknown agent '{agent_id}'. Must be a registered agent.")
        return self._agents[agent_id]

    def list_all(self) -> list[AgentDefinition]:
        return list(self._agents.values())


registry = AgentRegistry()
