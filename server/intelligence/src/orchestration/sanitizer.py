import re
from typing import Any

SECRET_KEY_PATTERN = re.compile(r"sk-[A-Za-z0-9]{20,}")
AUTH_HEADER_PATTERN = re.compile(r"(Bearer\s+)[A-Za-z0-9_\-\.]{10,}", re.IGNORECASE)
EMAIL_PATTERN = re.compile(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+")
PHONE_PATTERN = re.compile(r"\b(?:\+?\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b")
SENSITIVE_KEY_PATTERN = re.compile(
    r"(?:^|_)(?:authorization|api_?key|private_?key|secret|password|credential|token)s?(?:_|$)"
)
TOKEN_COUNT_KEYS = frozenset(
    {
        "completion_tokens",
        "input_tokens",
        "output_tokens",
        "prompt_tokens",
        "token_count",
        "tokens_in",
        "tokens_out",
        "tokens_used",
        "total_tokens",
    }
)

def sanitize_text(text: str) -> str:
    if not isinstance(text, str):
        return text
    text = SECRET_KEY_PATTERN.sub("[REDACTED_API_KEY]", text)
    text = AUTH_HEADER_PATTERN.sub(r"\1[REDACTED_TOKEN]", text)
    text = EMAIL_PATTERN.sub("[REDACTED_EMAIL]", text)
    text = PHONE_PATTERN.sub("[REDACTED_PHONE]", text)
    return text



def _normalize_key(key: Any) -> str:
    separated = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", str(key))
    return re.sub(r"[^A-Za-z0-9]+", "_", separated).strip("_").lower()


def sanitize_data(data: Any) -> Any:
    if isinstance(data, dict):
        sanitized = {}
        for key, value in data.items():
            normalized_key = _normalize_key(key)
            if normalized_key not in TOKEN_COUNT_KEYS and SENSITIVE_KEY_PATTERN.search(
                normalized_key
            ):
                sanitized[key] = "[REDACTED]"
            else:
                sanitized[key] = sanitize_data(value)
        return sanitized
    if isinstance(data, list):
        return [sanitize_data(item) for item in data]
    if isinstance(data, str):
        return sanitize_text(data)
    return data
