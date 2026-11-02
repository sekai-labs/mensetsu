import io
import pytest
from pypdf import PdfWriter

from src.generation.documents import (
    MAX_RESUME_PDF_BYTES,
    ResumePdfError,
    extract_resume_pdf_bytes,
)


def _make_pdf(text: str) -> bytes:
    # Minimal single-page PDF with text stream
    # Using pypdf's PdfWriter with blank page, but to add text with pure pypdf or minimal raw pdf
    # Constructing a valid PDF with a simple content stream
    content = f"BT /F1 12 Tf 72 712 Td ({text}) Tj ET".encode("latin1")
    stream_len = len(content)
    pdf = (
        b"%PDF-1.4\n"
        b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n"
        b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n"
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n"
        b"4 0 obj\n<< /Length " + str(stream_len).encode("ascii") + b" >>\nstream\n"
        + content + b"\nendstream\nendobj\n"
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n"
        b"xref\n0 6\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n"
        b"0000000224 00000 n \n0000000300 00000 n \ntrailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n380\n%%EOF\n"
    )
    return pdf


def test_extract_resume_pdf_bytes_success():
    pdf_bytes = _make_pdf("Jane Doe Software Engineer")
    extracted = extract_resume_pdf_bytes(pdf_bytes, "sample.pdf")
    assert "Jane Doe Software Engineer" in extracted


def test_extract_resume_pdf_bytes_rejects_non_bytes():
    with pytest.raises(ResumePdfError, match="must be bytes"):
        extract_resume_pdf_bytes("not bytes", "sample.pdf")  # type: ignore


def test_extract_resume_pdf_bytes_rejects_oversized():
    oversized = b"%PDF" + b"A" * (MAX_RESUME_PDF_BYTES + 1)
    with pytest.raises(ResumePdfError, match="exceeds the 5 MB limit"):
        extract_resume_pdf_bytes(oversized, "large.pdf")


def test_extract_resume_pdf_bytes_rejects_missing_magic():
    not_pdf = b"NOT_A_PDF_DOCUMENT_CONTENT"
    with pytest.raises(ResumePdfError, match="missing %PDF header"):
        extract_resume_pdf_bytes(not_pdf, "sample.txt")


def test_extract_resume_pdf_bytes_rejects_corrupted_pdf():
    corrupted = b"%PDF-1.4\nsome garbage content not conforming to pdf syntax"
    with pytest.raises(ResumePdfError, match="could not be parsed as PDF"):
        extract_resume_pdf_bytes(corrupted, "corrupt.pdf")


def test_extract_resume_pdf_bytes_rejects_empty_extractable_text():
    # Empty page with no text stream
    writer = PdfWriter()
    writer.add_blank_page(width=100, height=100)
    out = io.BytesIO()
    writer.write(out)
    empty_pdf = out.getvalue()
    with pytest.raises(ResumePdfError, match="contains no extractable text"):
        extract_resume_pdf_bytes(empty_pdf, "scanned.pdf")
