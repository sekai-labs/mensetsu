import io
from pathlib import Path

from docx import Document
from pypdf import PdfReader


class DocumentParser:

    @classmethod
    def extract_text_from_bytes(cls, content: bytes, filename: str) -> str:
        ext = Path(filename).suffix.lower()
        if ext == '.pdf':
            reader = PdfReader(io.BytesIO(content))
            pages = []
            for page in reader.pages:
                text = page.extract_text()
                if text:
                    pages.append(text)
            return '\n\n'.join(pages)
        elif ext in ['.docx', '.doc']:
            doc = Document(io.BytesIO(content))
            return '\n\n'.join([p.text for p in doc.paragraphs if p.text.strip()])
        else:
            return content.decode('utf-8', errors='replace')

async def parse_document_file(content: bytes, filename: str) -> str:
    return DocumentParser.extract_text_from_bytes(content, filename)
