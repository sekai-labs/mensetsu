import hmac
import os


def get_internal_token() -> str:
    token = os.environ.get("INTERNAL_SERVICE_TOKEN")
    if not token or not token.strip():
        raise RuntimeError(
            "INTERNAL_SERVICE_TOKEN is unset or empty. Service refusing to start (fail-closed)."
        )
    return token.strip()


def validate_internal_token(authorization: str | None) -> bool:
    if not authorization:
        return False
    try:
        expected_token = get_internal_token()
    except RuntimeError:
        return False

    parts = authorization.split(" ", 1)
    if len(parts) != 2 or parts[0].lower() != "bearer":
        return False

    provided_token = parts[1].strip()
    return bool(provided_token and hmac.compare_digest(provided_token, expected_token))
