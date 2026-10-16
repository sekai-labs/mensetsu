import json
from pathlib import Path

from src.contracts.export_schema import generate_workflows_v1_schema


def test_workflows_v1_schema_is_not_stale():
    repo_root = Path(__file__).resolve().parents[3]
    schema_path = repo_root / "contracts" / "rest" / "schema" / "workflows.v1.json"

    assert schema_path.exists(), f"Committed schema file does not exist at {schema_path}"

    committed_content = schema_path.read_text(encoding="utf-8")
    committed_json = json.loads(committed_content)

    generated_schema = generate_workflows_v1_schema()
    generated_formatted = json.dumps(generated_schema, indent=2, sort_keys=True) + "\n"
    generated_json = json.loads(generated_formatted)

    assert committed_json == generated_json, (
        "The committed workflows.v1.json schema is out of date. "
        "Run `PYTHONPATH=. uv run python src/contracts/export_schema.py` to regenerate."
    )
