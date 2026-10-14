import hmac
import os

from fastapi import Header, HTTPException, status


def get_internal_token() -> str:
    token = os.environ.get("INTERNAL_SERVICE_TOKEN")
    if not token or not token.strip():
        raise RuntimeError(
            "INTERNAL_SERVICE_TOKEN is unset or empty. Service refusing to start (fail-closed)."
        )
    return token.strip()


async def verify_internal_auth(authorization: str | None = Header(None)) -> None:
    expected_token = get_internal_token()

    if not authorization:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Missing Authorization header",
        )

    parts = authorization.split(" ", 1)
    if len(parts) != 2 or parts[0].lower() != "bearer":
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid authorization scheme; Bearer token required",
        )

    provided_token = parts[1].strip()
    if not provided_token or not hmac.compare_digest(provided_token, expected_token):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid internal service token",
        )
