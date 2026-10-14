import base64
import binascii
import io
from pathlib import Path

from docx import Document
from pypdf import PdfReader
from pypdf.errors import PdfReadError

MAX_RESUME_PDF_BYTES = 5 * 1024 * 1024
PDF_MAGIC = b"%PDF"


class ResumePdfError(ValueError):
    pass


def extract_resume_pdf_text(resume_pdf_base64: str, resume_filename: str | None) -> str:
    label = resume_filename or "uploaded resume"
    if len(resume_pdf_base64) > (MAX_RESUME_PDF_BYTES * 4) // 3 + 4:
        raise ResumePdfError(f"{label} exceeds the {MAX_RESUME_PDF_BYTES // (1024 * 1024)} MB limit.")
    try:
        pdf_bytes = base64.b64decode(resume_pdf_base64, validate=True)
    except (binascii.Error, ValueError) as e:
        raise ResumePdfError(f"{label} is not valid base64: {e}") from e
    if len(pdf_bytes) > MAX_RESUME_PDF_BYTES:
        raise ResumePdfError(f"{label} exceeds the {MAX_RESUME_PDF_BYTES // (1024 * 1024)} MB limit.")
    if not pdf_bytes.startswith(PDF_MAGIC):
        raise ResumePdfError(f"{label} is not a PDF (missing %PDF header).")
    try:
        text = DocumentParser.extract_text_from_bytes(pdf_bytes, "resume.pdf").strip()
    except PdfReadError as e:
        raise ResumePdfError(f"{label} could not be parsed as PDF: {e}") from e
    if not text:
        raise ResumePdfError(
            f"{label} contains no extractable text (likely a scanned image); upload a text-based PDF or paste the resume text."
        )
    return text


class DocumentParser:
    @classmethod
    def extract_text_from_bytes(cls, content: bytes, filename: str) -> str:
        ext = Path(filename).suffix.lower()
        if ext == ".pdf":
            reader = PdfReader(io.BytesIO(content))
            pages = []
            for page in reader.pages:
                text = page.extract_text()
                if text:
                    pages.append(text)
            return "\n\n".join(pages)
        elif ext in [".docx", ".doc"]:
            doc = Document(io.BytesIO(content))
            return "\n\n".join([p.text for p in doc.paragraphs if p.text.strip()])
        else:
            return content.decode("utf-8", errors="replace")


async def parse_document_file(content: bytes, filename: str) -> str:
    return DocumentParser.extract_text_from_bytes(content, filename)
