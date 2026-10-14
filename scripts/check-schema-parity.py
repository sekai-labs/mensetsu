#!/usr/bin/env python3
import json
import re
import sys
import urllib.request
from base64 import b64encode
from pathlib import Path

URL = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:8000"
REPO = Path(__file__).resolve().parent.parent
AUTH = "Basic " + b64encode(b"root:root").decode()


def sql(body: str, ns: str | None = None, db: str | None = None) -> list:
    headers = {"Accept": "application/json", "Authorization": AUTH}
    if ns:
        headers["Surreal-NS"] = ns
    if db:
        headers["Surreal-DB"] = db
    req = urllib.request.Request(f"{URL}/sql", data=body.encode(), headers=headers, method="POST")
    with urllib.request.urlopen(req) as r:
        return json.load(r)


def retarget(text: str, ns: str) -> str:
    text = text.replace("USE NS mensetsu DB production;", f"USE NS {ns} DB p;")
    text = text.replace("DEFINE NAMESPACE IF NOT EXISTS mensetsu;", f"DEFINE NAMESPACE IF NOT EXISTS {ns};")
    text = text.replace("USE NS mensetsu;", f"USE NS {ns};")
    text = text.replace("DEFINE DATABASE IF NOT EXISTS production;", "DEFINE DATABASE IF NOT EXISTS p;")
    assert "mensetsu" not in text and "production" not in text, "unretargeted reference"
    return f"DEFINE NAMESPACE IF NOT EXISTS {ns}; USE NS {ns}; DEFINE DATABASE IF NOT EXISTS p; USE NS {ns} DB p;\n" + text


def apply(label: str, ns: str, files: list[Path]) -> None:
    sql(f"REMOVE NAMESPACE IF EXISTS {ns};")
    for f in files:
        res = sql(retarget(f.read_text(), ns))
        errs = [(i, r["result"]) for i, r in enumerate(res) if r["status"] == "ERR"]
        print(f"  [{label}] {f.name}: {len(res)} statements, {len(errs)} errors")
        for i, e in errs:
            print(f"      #{i}: {e}")


def snapshot(ns: str) -> dict:
    tables = sql("INFO FOR DB;", ns, "p")[0]["result"]["tables"]
    out = {}
    for name, tdef in sorted(tables.items()):
        info = sql(f"INFO FOR TABLE `{name}`;", ns, "p")[0]["result"]
        out[name] = {
            "table": re.sub(r"\s+", " ", tdef),
            "fields": {k: re.sub(r"\s+", " ", v) for k, v in info["fields"].items()},
            "indexes": {k: re.sub(r"\s+", " ", v) for k, v in info["indexes"].items()},
        }
    return out


def main() -> int:
    migrations = sorted((REPO / "infra/migrations").glob("[0-9][0-9][0-9]_*.surql"))
    print("migrations:", [m.name for m in migrations])
    apply("schema", "parity_schema", [REPO / "server/api/src/schema.surql"])
    apply("migrations", "parity_migr", migrations)
    a, b = snapshot("parity_schema"), snapshot("parity_migr")
    sql("REMOVE NAMESPACE IF EXISTS parity_schema; REMOVE NAMESPACE IF EXISTS parity_migr;")

    diffs = []
    for t in sorted(set(a) | set(b)):
        if t not in b:
            diffs.append(f"table {t}: only in schema.surql")
            continue
        if t not in a:
            diffs.append(f"table {t}: only in migrations")
            continue
        if a[t]["table"] != b[t]["table"]:
            diffs.append(f"table {t}:\n    schema: {a[t]['table']}\n    migr:   {b[t]['table']}")
        for kind in ("fields", "indexes"):
            for k in sorted(set(a[t][kind]) | set(b[t][kind])):
                x, y = a[t][kind].get(k), b[t][kind].get(k)
                if x != y:
                    diffs.append(f"{kind[:-1]} {t}.{k}:\n    schema: {x}\n    migr:   {y}")

    nf = sum(len(v["fields"]) for v in a.values())
    ni = sum(len(v["indexes"]) for v in a.values())
    print(f"schema.surql: {len(a)} tables, {nf} fields, {ni} indexes")
    print(f"migrations:   {len(b)} tables, {sum(len(v['fields']) for v in b.values())} fields, {sum(len(v['indexes']) for v in b.values())} indexes")
    if diffs:
        print(f"DIVERGED: {len(diffs)} difference(s)")
        for d in diffs:
            print("  " + d)
        return 1
    print("IDENTICAL")
    return 0


if __name__ == "__main__":
    sys.exit(main())
