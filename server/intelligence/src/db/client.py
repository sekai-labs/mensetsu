import json
import os
from typing import Any

import httpx
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    surreal_url: str = "http://127.0.0.1:8000"
    surreal_endpoint: str | None = None
    surreal_user: str = "root"
    surreal_pass: str = "root"
    surreal_ns: str = os.environ.get("SURREAL_NS", "mensetsu")
    surreal_db: str = os.environ.get("SURREAL_DB", "production")

    def get_surreal_url(self) -> str:
        return self.surreal_endpoint or self.surreal_url


settings = Settings()


class SurrealQueryError(RuntimeError):
    pass


class SurrealDirectClient:
    def __init__(self, settings: Settings | None = None):
        st = settings or Settings(
            surreal_ns=os.environ.get("SURREAL_NS", "mensetsu"),
            surreal_db=os.environ.get("SURREAL_DB", "production"),
        )
        self.base_url = st.get_surreal_url().rstrip("/")
        self.auth = (st.surreal_user, st.surreal_pass)
        self.headers = {
            "surreal-ns": st.surreal_ns,
            "surreal-db": st.surreal_db,
            "Accept": "application/json",
        }

    async def execute_query(self, query: str, vars: dict | None = None) -> list[dict]:
        full_query = query
        if vars:

            def clean_value(val: Any) -> Any:
                if isinstance(val, str):
                    try:
                        return val.encode("utf-16", "surrogatepass").decode("utf-16")
                    except Exception:
                        return val.encode("utf-8", "ignore").decode("utf-8", "ignore")
                elif isinstance(val, dict):
                    return {k: clean_value(v) for k, v in val.items()}
                elif isinstance(val, list):
                    return [clean_value(v) for v in val]
                return val

            def drop_none(val: Any) -> Any:
                if isinstance(val, dict):
                    return {k: drop_none(v) for k, v in val.items() if v is not None}
                if isinstance(val, list):
                    return [drop_none(v) for v in val]
                return val

            let_lines = []
            for k, v in vars.items():
                if v is None:
                    let_lines.append("LET $" + str(k) + " = NONE;")
                    continue
                val_json = json.dumps(drop_none(clean_value(v)), default=str, ensure_ascii=False)
                let_lines.append("LET $" + str(k) + " = " + val_json + ";")
            full_query = "\n".join(let_lines) + "\n" + query
        async with httpx.AsyncClient(timeout=10.0) as client:
            resp = await client.post(
                f"{self.base_url}/sql",
                auth=self.auth,
                headers=self.headers,
                content=full_query.encode("utf-8", "ignore"),
            )
            if resp.status_code >= 400:
                raise SurrealQueryError(f"SurrealDB HTTP {resp.status_code}: {resp.text[:500]}")
            results = resp.json()
            failures = [
                (idx, item.get("result") or item.get("detail"))
                for idx, item in enumerate(results if isinstance(results, list) else [])
                if isinstance(item, dict) and item.get("status") == "ERR"
            ]
            if failures:
                root = next(
                    (f for f in failures if "not executed" not in str(f[1]).lower()),
                    failures[0],
                )
                raise SurrealQueryError(
                    f"SurrealDB statement {root[0]} failed: {root[1]} ({len(failures)} failed statements)"
                )
            return results
