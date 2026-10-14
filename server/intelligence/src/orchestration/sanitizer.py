import re
from typing import Any

SECRET_KEY_PATTERN = re.compile(r"sk-[A-Za-z0-9]{20,}")
AUTH_HEADER_PATTERN = re.compile(r"(Bearer\s+)[A-Za-z0-9_\-\.]{10,}", re.IGNORECASE)
EMAIL_PATTERN = re.compile(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+")
PHONE_PATTERN = re.compile(r"\b(?:\+?\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b")


def sanitize_text(text: str) -> str:

    if not isinstance(text, str):
        return text
    text = SECRET_KEY_PATTERN.sub("[REDACTED_API_KEY]", text)
    text = AUTH_HEADER_PATTERN.sub(r"\1[REDACTED_TOKEN]", text)
    text = EMAIL_PATTERN.sub("[REDACTED_EMAIL]", text)
    text = PHONE_PATTERN.sub("[REDACTED_PHONE]", text)
    return text


def sanitize_data(data: Any) -> Any:

    if isinstance(data, dict):
        sanitized = {}
        for k, v in data.items():
            k_lower = str(k).lower()

            if (
                any(
                    k_lower == s or k_lower.endswith(f"_{s}")
                    for s in ("authorization", "api_key", "secret", "password", "token")
                )
                and "tokens" not in k_lower
            ):
                sanitized[k] = "[REDACTED]"
            else:
                sanitized[k] = sanitize_data(v)
        return sanitized
    elif isinstance(data, list):
        return [sanitize_data(item) for item in data]
    elif isinstance(data, str):
        return sanitize_text(data)
    return data
