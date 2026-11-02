import io

from pypdf import PdfReader

MAX_RESUME_PDF_BYTES = 5 * 1024 * 1024
PDF_MAGIC = b"%PDF"


class ResumePdfError(ValueError):
    pass


def extract_resume_pdf_bytes(content: bytes, filename: str | None = None) -> str:
    label = filename or "uploaded resume"
    if not isinstance(content, (bytes, bytearray)):
        raise ResumePdfError(f"{label} must be bytes.")
    if len(content) > MAX_RESUME_PDF_BYTES:
        raise ResumePdfError(
            f"{label} exceeds the {MAX_RESUME_PDF_BYTES // (1024 * 1024)} MB limit."
        )
    if not content.startswith(PDF_MAGIC):
        raise ResumePdfError(f"{label} is not a PDF (missing %PDF header).")
    try:
        reader = PdfReader(io.BytesIO(content))
        text = "\n\n".join(
            page_text for page in reader.pages if (page_text := page.extract_text())
        ).strip()
    except Exception as e:
        raise ResumePdfError(f"{label} could not be parsed as PDF: {e}") from e
    if not text:
        raise ResumePdfError(
            f"{label} contains no extractable text (likely a scanned image); upload a text-based PDF or paste the resume text."
        )
    return text
