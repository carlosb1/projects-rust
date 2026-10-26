from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Tuple, List
from zipfile import ZipFile
import logging
from pypdf import PdfReader

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def unzip_report(zip_path: Path, destination: Path) -> list[Path]:
    destination = destination.resolve()
    destination.mkdir(parents=True, exist_ok=True)

    with ZipFile(zip_path) as archive:
        for entry in archive.infolist():
            target = (destination / entry.filename).resolve()
            if not target.is_relative_to(destination):
                raise ValueError(f"Invalid path for ZIP: {entry.filename}")

        archive.extractall(destination)

    return sorted(
        path
        for path in destination.rglob("*")
        if path.is_file() and path.suffix.lower() == ".pdf"
    )


def read_pdf(pdf_path: Path) -> str:
    logger.info("Reading PDF: %s", pdf_path.name)

    with pdf_path.open("rb") as pdf_file:
        reader = PdfReader(pdf_file)
        return "\n\n".join(
            page.extract_text() or ""
            for page in reader.pages
        )



