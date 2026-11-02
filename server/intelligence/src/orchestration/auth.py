import os


def get_internal_token() -> str:
    token = os.environ.get("INTERNAL_SERVICE_TOKEN")
    if not token or not token.strip():
        raise RuntimeError(
            "INTERNAL_SERVICE_TOKEN is unset or empty. Service refusing to start (fail-closed)."
        )
    return token.strip()
