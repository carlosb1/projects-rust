import asyncio
import pathlib
import uuid
from dataclasses import dataclass
from typing import Dict, Any, List, Tuple, Optional, Iterable

import httpx
from httpx._urlparse import urlparse
from pypdf import PdfReader
from qdrant_client import QdrantClient


from datetime import timedelta
import shutil

from qdrant_client.conversions.common_types import PointStruct
from sentence_transformers import SentenceTransformer
from torch.ao.quantization.backend_config.utils import entry_to_pretty_str
from tqdm import tqdm

import filters
from db.database import ChunkRepository
from db.search_engine import upsert_chunks
from entities import DBEntry
from parser import flatten_boe_payload

from tools import create_chunks


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

def extract_pdf_text_pages(pdf_path: str) -> Tuple[List[Tuple[int, str]],str]:
    """
    Returns list of (page_number, text). page_number starts at 1.
    """
    pages: List[Tuple[int, str]] = []
    content: str = ""
    try:
        reader = PdfReader(pdf_path)
        #TODO add here the page for range of text.
        for i, page in enumerate(reader.pages, start=1):
            #TODO fix and get a whole text
            txt = page.extract_text() or ""
            pages.append((i, txt))
            content += txt
    except Exception as e:
        print(f"[WARN] Failed to read {pdf_path}: {e}")
    return (pages, content)


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




def ingest_boe_date_to_qdrant(
    client: QdrantClient,
    embedder: SentenceTransformer,
    date_yyyymmdd: str,
    out_dir: str = "./boe_pdfs",
    collection_name: str = "boe_disposiciones",
    chunk_chars: int = 2500,
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
        (pages, content) = extract_pdf_text_pages(path)
        pieces = create_chunks(content, chunk_chars)
        # link back to any rows that reference this PDF (may be multiple)
        ref_indices = url_to_rows.get(url, [])
        # derive some shared fields from the first referencing row (payload will include them all individually below)
#        for page_num, txt in pages:
#        pieces = chunk_text(txt, chunk_chars=chunk_chars, overlap=overlap)
        for idx, piece in enumerate(pieces):
            # check text
            generic_entities = filters.spacy_extract(piece)
            date_entities = filters.search_dates(piece)
            ###
            for r_index in ref_indices:
                r = rows[r_index]
                meta = {
                    # Provenance
                    "publicacion": r.get("publicacion"),
                    "fecha_publicacion": r.get("fecha_publicacion"),
                    "diario_numero": r.get("diario_numero"),
                    "diario_identificador": r.get("diario_identificador"),
                    "seccion_codigo": r.get("seccion_codigo"),
                    "seccion_nombre": r.get("seccion_nombre"),
                    "departamento_codigo": r.get("departamento_codigo"),
                    "departamento_nombre": r.get("departamento_nombre"),
                    "epigrafe_nombre": r.get("epigrafe_nombre"),
                    "iteme_url": url,
                    "chunk_idx": idx,
                    **generic_entities,
                    "dates": date_entities,
                    # BOE fields
                    "public_identificador": r.get("item_identificador"),
                    "item_titulo": r.get("item_titulo"),
                    "paragraph": piece
                }
                chunk_payloads.append((piece, meta))

    if not chunk_payloads:
        print("No chunks produced. Exiting.")
        return


    upsert_chunks(client, collection_name, embedder, chunk_payloads, batch_size=128)
    #update_databases()
    shutil.rmtree(out_dir, ignore_errors=True)

    print(f"Done. Upserted {len(chunk_payloads)} chunks into '{collection_name}'.")

def new_uuid(payload: Dict[str, Any]) -> uuid.UUID:
    return uuid.uuid5(uuid.NAMESPACE_URL, payload["source_url"] + f'/{payload["page"]}/{payload["chunk_idx"]}')

def update_databases(
    qdrant_client: QdrantClient,
    chunk_repo: ChunkRepository,
    collection: str,
    embedder: SentenceTransformer,
    chunks: List[Tuple[str, Dict[str, Any]]],  # (text, payload)
    batch_size: int = 128,
):
    # Embed and upload in batches
    for i in tqdm(range(0, len(chunks), batch_size), desc="Upserting to databases"):
        batch = chunks[i:i+batch_size]

        texts = [t for t, _ in batch]
        vecs = embedder.encode(texts, show_progress_bar=False, convert_to_numpy=True, normalize_embeddings=True)
        points_to_qdrant = []
        db_inserts: List[DBEntry] = []
        for (text, payload), vec in zip(batch, vecs):
            pid = new_uuid(payload)
            # qdrant entry
            p = PointStruct(id=str(pid), vector=vec.tolist(), payload={**payload, "text": text})
            points_to_qdrant.append(p)
            # db entry
            entry = DBEntry(id=pid, source_url=payload["source_url"], page=payload["page"], chunk_idx=payload["chunk_idx"], text=text)
            db_inserts.append(entry)

        chunk_repo.insert(db_inserts)
        qdrant_client.upsert(collection_name=collection, points=points_to_qdrant)
