"""Artifact persistence and graph projections."""

from src.contracts.v1 import (
    Artifact,
    DelegationGraphEdge,
    DelegationGraphEdgeData,
    DelegationGraphNode,
    DelegationGraphNodeData,
    DelegationGraphResponse,
    TaskGraphEdge,
    TaskGraphNode,
    TaskGraphNodeData,
    TaskGraphResponse,
)
from src.orchestration.id_validator import validate_id


class ArtifactMixin:
    async def save_artifact(self, artifact: Artifact) -> None:
        clean_id = validate_id(artifact.id, "artifact_id")
        clean_run_id = validate_id(artifact.run_id, "run_id")
        query = """
        UPSERT type::record('artifact', $art_id) SET
            run_id = type::record('workflow_run', $run_id),
            task_id = $task_id,
            agent_execution_id = $agent_execution_id,
            type = $type,
            title = $title,
            version = $version,
            evidence_ids = $evidence_ids,
            source_links = $source_links,
            content = $content,
            created_at = time::now();
        """
        vars_payload = {
            "art_id": clean_id,
            "run_id": clean_run_id,
            "task_id": artifact.task_id or "",
            "agent_execution_id": artifact.agent_execution_id or "",
            "type": artifact.type,
            "title": artifact.title,
            "version": artifact.version,
            "evidence_ids": artifact.evidence_ids,
            "source_links": [sl.model_dump() for sl in artifact.source_links],
            "content": artifact.content
            if isinstance(artifact.content, dict)
            else {"text": artifact.content},
        }
        if artifact.surreal_record_id:
            query = query.replace(
                "created_at = time::now();",
                "surreal_record_id = $surreal_record_id, created_at = time::now();",
            )
            vars_payload["surreal_record_id"] = artifact.surreal_record_id
        if artifact.evidence:
            query = query.replace(
                "created_at = time::now();", "evidence = $evidence, created_at = time::now();"
            )
            vars_payload["evidence"] = artifact.evidence.model_dump()
        await self.db.execute_query(query, vars=vars_payload)

    async def list_artifacts_for_run(self, run_id: str) -> list[Artifact]:
        clean_run_id = validate_id(run_id, "run_id")
        query = "SELECT * FROM artifact WHERE run_id = type::record('workflow_run', $run_id);"
        res = await self.db.execute_query(query, vars={"run_id": clean_run_id})
        items = []
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    for r in item["result"]:
                        row = dict(r)
                        row["id"] = row.get("id", "").replace("artifact:", "")
                        if "run_id" in row and isinstance(row["run_id"], str):
                            row["run_id"] = row["run_id"].replace("workflow_run:", "")
                        items.append(Artifact.model_validate(row))
        return items

    async def get_task_graph(self, run_id: str) -> TaskGraphResponse:
        tasks = await self.list_tasks_for_run(run_id)
        nodes: list[TaskGraphNode] = []
        edges: list[TaskGraphEdge] = []

        for t in tasks:
            node = TaskGraphNode(
                id=t.id,
                label=t.name,
                data=TaskGraphNodeData(
                    status=t.status,
                    assigned_agent=t.assigned_agent,
                    attempt=t.attempt,
                    task_type=t.task_type,
                    required=t.required,
                    added_in_revision=t.added_in_revision,
                    execution_ids=t.execution_ids,
                    group=t.task_type,
                ),
            )
            nodes.append(node)
            for dep in t.dependencies:
                edges.append(
                    TaskGraphEdge(
                        id=f"edge_{dep}_to_{t.id}",
                        source=dep,
                        target=t.id,
                        kind="dependency",
                    )
                )

        return TaskGraphResponse(nodes=nodes, edges=edges)

    async def get_delegation_graph(self, run_id: str) -> DelegationGraphResponse:
        delegations = await self.list_delegations_for_run(run_id)
        nodes: list[DelegationGraphNode] = []
        edges: list[DelegationGraphEdge] = []
        seen_agents: set[str] = set()

        for d in delegations:
            if d.parent_execution_id not in seen_agents:
                seen_agents.add(d.parent_execution_id)
                nodes.append(
                    DelegationGraphNode(
                        id=d.parent_execution_id,
                        label=d.parent_agent_id,
                        data=DelegationGraphNodeData(
                            status="completed",
                            task_id=d.delegation_task_id,
                            agent_id=d.parent_agent_id,
                        ),
                    )
                )
            if d.child_execution_id not in seen_agents:
                seen_agents.add(d.child_execution_id)
                nodes.append(
                    DelegationGraphNode(
                        id=d.child_execution_id,
                        label=d.child_agent_id,
                        data=DelegationGraphNodeData(
                            status=d.status
                            if d.status
                            in ("pending", "running", "completed", "failed", "cancelled")
                            else "completed",
                            task_id=d.delegation_task_id,
                            agent_id=d.child_agent_id,
                        ),
                    )
                )

            edges.append(
                DelegationGraphEdge(
                    id=f"del_edge_{d.id}",
                    source=d.parent_execution_id,
                    target=d.child_execution_id,
                    kind="delegation",
                    data=DelegationGraphEdgeData(
                        delegation_id=d.id,
                        objective=d.objective,
                        status=d.status,
                        created_at=d.created_at,
                        parent_agent_id=d.parent_agent_id,
                        child_agent_id=d.child_agent_id,
                    ),
                )
            )
        return DelegationGraphResponse(nodes=nodes, edges=edges)
