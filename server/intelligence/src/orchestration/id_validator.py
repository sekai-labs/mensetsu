import re

ID_PATTERN = re.compile(r"^[A-Za-z0-9_\-]+$")


def validate_id(record_id: str, field_name: str = "id") -> str:

    clean = record_id.split(":")[-1].replace("`", "").strip()
    if not ID_PATTERN.match(clean):
        raise ValueError(f"Invalid {field_name} '{record_id}': must match ^[A-Za-z0-9_\\-]+$")
    return clean
