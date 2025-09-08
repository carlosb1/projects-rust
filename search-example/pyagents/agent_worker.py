import asyncio
import pathlib
import traceback
from pathlib import Path
import uuid
from typing import Dict, Any, List, Tuple, Optional, Iterable

import httpx
from httpx._urlparse import urlparse
from pypdf import PdfReader
from qdrant_client import QdrantClient

from datetime import datetime, timedelta, time
import dotenv
import os

from sentence_transformers import SentenceTransformer
from tqdm import tqdm

from db import upsert_chunks, ensure_collection
from parser import flatten_boe_payload
from datetime import datetime


def str_pregenerate_date(d) -> str:
    return d.strftime("%Y%m%d")
def decrement_days(d, count):
    return d - timedelta(days=count)



def gather_unique_pdfs(rows: List[Dict[str, Optional[str]]]) -> Dict[str, List[int]]:
    """
    Return dict: url -> list of row indices that reference it.
    We include diario PDF and item PDFs.
    """
    by_url: Dict[str, List[int]] = {}
    for i, r in enumerate(rows):
        for key in ("diario_pdf_url", "item_pdf_url"):
            u = r.get(key)
            if not u:
                continue
            by_url.setdefault(u, []).append(i)
    return by_url

async def download_pdfs(urls: Iterable[str], out_dir: str, concurrency: int = 6) -> Dict[str, Optional[str]]:
    outp: Dict[str, Optional[str]] = {}
    out_path = pathlib.Path(out_dir)
    out_path.mkdir(parents=True, exist_ok=True)

    limits = httpx.Limits(max_keepalive_connections=concurrency, max_connections=concurrency)
    async with httpx.AsyncClient(limits=limits, headers={"User-Agent": "httpx/boe-ingestor"}) as client:
        sem = asyncio.Semaphore(concurrency)

        async def sem_task(u: str):
            async with sem:
                p = await download_one(client, u, out_path)
                outp[u] = str(p) if p is not None else None

        tasks = [asyncio.create_task(sem_task(u)) for u in urls]
        for f in tqdm(asyncio.as_completed(tasks), total=len(tasks), desc="Downloading PDFs"):
            await f

    return outp

def safe_pdf_filename(url: str) -> str:
    """
    Prefer the last path component; fallback to UUID.
    """
    p = urlparse(url)
    base = pathlib.Path(p.path).name
    if base and base.lower().endswith(".pdf"):
        return base
    return f"{uuid.uuid4().hex}.pdf"



async def download_one(client: httpx.AsyncClient, url: str, out_dir: pathlib.Path) -> Optional[pathlib.Path]:
    try:
        name = safe_pdf_filename(url)
        path = out_dir / name
        if path.exists() and path.stat().st_size > 0:
            return path
        r = await client.get(url, timeout=60.0, follow_redirects=True)
        r.raise_for_status()
        path.write_bytes(r.content)
        return path
    except Exception as e:
        print(f"[WARN] Failed download {url}: {e}")
        return None

def extract_pdf_text_pages(pdf_path: str) -> List[Tuple[int, str]]:
    """
    Returns list of (page_number, text). page_number starts at 1.
    """
    pages: List[Tuple[int, str]] = []
    try:
        reader = PdfReader(pdf_path)
        for i, page in enumerate(reader.pages, start=1):
            txt = page.extract_text() or ""
            pages.append((i, txt))
    except Exception as e:
        print(f"[WARN] Failed to read {pdf_path}: {e}")
    return pages


def fetch_boe_sumario(date_yyyymmdd: str) -> Dict[str, Any]:
    """
    GET with httpx.get; follow redirects like `curl -L`.
    """
    url = f"https://www.boe.es/datosabiertos/api/boe/sumario/{date_yyyymmdd}"
    headers = {
        "Accept": "application/json",
        "User-Agent": "httpx/boe-example",
    }
    # httpx.get is synchronous; follow_redirects replicates curl -L
    resp = httpx.get(url, headers=headers, timeout=30.0, follow_redirects=True)
    resp.raise_for_status()  # raise on 4xx/5xx
    return resp.json()


def chunk_text(text: str, chunk_chars: int = 1000, overlap: int = 200) -> List[str]:
    if not text:
        return []
    chunks = []
    step = max(1, chunk_chars - overlap)
    for start in range(0, len(text), step):
        chunk = text[start:start + chunk_chars]
        if chunk.strip():
            chunks.append(chunk)
    return chunks


def ingest_boe_date_to_qdrant(
    client: QdrantClient,
    embedder: SentenceTransformer,
    date_yyyymmdd: str,
    out_dir: str = "./boe_pdfs",
    collection_name: str = "boe_disposiciones",
    chunk_chars: int = 1000,
    overlap: int = 200,
):
    # fetch & parse
    payload = fetch_boe_sumario(date_yyyymmdd)
    rows = flatten_boe_payload(payload)
    if not rows:
        print("No rows found. Exiting.")
        return

    # unique pdfs
    url_to_rows = gather_unique_pdfs(rows)
    urls = list(url_to_rows.keys())
    print(f"Found {len(urls)} unique PDFs")

    # download
    pdf_map = asyncio.run(download_pdfs(urls, out_dir=out_dir, concurrency=6))

    # prepare chunks
    chunk_payloads: List[Tuple[str, Dict[str, Any]]] = []
    for url, path in pdf_map.items():
        if not path:
            continue
        pages = extract_pdf_text_pages(path)
        # link back to any rows that reference this PDF (may be multiple)
        ref_indices = url_to_rows.get(url, [])
        # derive some shared fields from the first referencing row (payload will include them all individually below)
        for page_num, txt in pages:
            pieces = chunk_text(txt, chunk_chars=chunk_chars, overlap=overlap)
            for idx, piece in enumerate(pieces):
                for r_index in ref_indices:
                    r = rows[r_index]
                    meta = {
                        # Provenance
                        "source_url": url,
                        "source_pdf_path": str(path),
                        "page": page_num,
                        "chunk_idx": idx,
                        # BOE fields
                        "publicacion": r.get("publicacion"),
                        "fecha_publicacion": r.get("fecha_publicacion"),
                        "diario_numero": r.get("diario_numero"),
                        "diario_identificador": r.get("diario_identificador"),
                        "seccion_codigo": r.get("seccion_codigo"),
                        "seccion_nombre": r.get("seccion_nombre"),
                        "departamento_codigo": r.get("departamento_codigo"),
                        "departamento_nombre": r.get("departamento_nombre"),
                        "epigrafe_nombre": r.get("epigrafe_nombre"),
                        "epigrafe_identificador": r.get("epigrafe_identificador"),
                        "item_identificador": r.get("item_identificador"),
                        "item_titulo": r.get("item_titulo"),
                        # Useful helpers
                        "doc_type": "BOE",
                        "date": date_yyyymmdd,
                    }
                    chunk_payloads.append((piece, meta))

    if not chunk_payloads:
        print("No chunks produced. Exiting.")
        return


    upsert_chunks(client, collection_name, embedder, chunk_payloads, batch_size=128)

    print(f"Done. Upserted {len(chunk_payloads)} chunks into '{collection_name}'.")


def run():
    qdrant_host = os.environ.get("QDRANT_HOST","0.0.0.0")
    qdrant_port = int(os.environ.get("QDRANT_PORT",6333))
    collection_name= "boe_disposiciones"
    loaded = dotenv.load_dotenv()
    print(loaded)
    # Qdrant: ensure collection & upsert
    print("Trying to load sentence transformer")
    embedder = SentenceTransformer("sentence-transformers/all-MiniLM-L6-v2", device="cpu")  # 384 dims
    print("Trying to initialize client for qdrant...")
    client = QdrantClient(host=qdrant_host, port=qdrant_port)
    print("Fixing database collection")
    ensure_collection(client, collection_name, vector_size=384)


    # Obtener fecha actual en formato aaaammdd
    now = datetime.today()
    str_now = now.strftime("%Y%m%d")
    print(f"Trying to ingest data for day={str_now}...")
    ingest_boe_date_to_qdrant(client, embedder, str_now, collection_name)


BATCH_REFERENCE_DATE = Path('./reference_date_batch')
INIT_REFERENCE_DATA = "20250904"

def write_or_create(path: Path, new_text: str, encoding="utf-8"):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(new_text, encoding=encoding)
    return new_text

def read_or_create(path: Path, default_entry: str, encoding="utf-8"):
    if path.exists():
        return path.read_text(encoding=encoding)
    else:
        # create parent dirs if missing
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(default_entry, encoding=encoding)
        return default_entry

def run_batch():
    qdrant_host = os.environ.get("QDRANT_HOST","0.0.0.0")
    qdrant_port = int(os.environ.get("QDRANT_PORT",6333))
    collection_name= "boe_disposiciones"


    embedder = SentenceTransformer("sentence-transformers/all-MiniLM-L6-v2")  # 384 dims
    client = QdrantClient(host=qdrant_host, port=qdrant_port)
    ensure_collection(client, collection_name, vector_size=384)

    last_day_not_updated = read_or_create(BATCH_REFERENCE_DATE, INIT_REFERENCE_DATA) - timedelta(day=1)
    retries = 0
    while retries < 20:
        try:
            str_last_not_updated = last_day_not_updated.strftime("%Y%m%d")
            print(f"Retrieving data from {str_last_not_updated}...")
            ingest_boe_date_to_qdrant(client, embedder, str_last_not_updated, collection_name)
            write_or_create(BATCH_REFERENCE_DATE, str_last_not_updated) - timedelta(day=1)
            print("sleeping")
            time.sleep(60)
        except Exception as e:
            retries+=1
            traceback.print_exception(e)









