from src.orchestration.sanitizer import sanitize_data, sanitize_text


def test_secret_key_sanitization():
    fake_key = "sk-" + "1234567890abcdef" * 2
    raw = f"Here is my secret: {fake_key} and password."
    sanitized = sanitize_text(raw)
    assert "sk-" not in sanitized
    assert "[REDACTED_API_KEY]" in sanitized


def test_auth_header_and_pii_sanitization():
    raw_dict = {
        "authorization": "Bearer super_secret_jwt_token_12345",
        "email": "candidate@example.com",
        "phone": "+1 555-019-2834",
        "normal": "Safe normal description",
    }
    sanitized = sanitize_data(raw_dict)
    assert sanitized["authorization"] == "[REDACTED]"
    assert "[REDACTED_EMAIL]" in sanitized["email"]
    assert "[REDACTED_PHONE]" in sanitized["phone"]
    assert sanitized["normal"] == "Safe normal description"
