import traceback
from pathlib import Path

import psycopg2
from qdrant_client import QdrantClient

from datetime import timedelta, date, timezone
import time as time_to_sleep
import dotenv
import os

from sentence_transformers import SentenceTransformer

from db.database import ChunkRepository
from db.search_engine import ensure_collection
from downloaders import ingest_boe_date_to_qdrant
from datetime import datetime

def coerce_to_date(value, default: date | None = None) -> date:
    """Convierte value a date. Admite date, datetime, timestamp numérico o string ISO.
       Si no puede, devuelve default (o date.today() si default es None)."""
    if isinstance(value, date) and not isinstance(value, datetime):
        return value
    if isinstance(value, datetime):
        return value.date()
    if isinstance(value, (int, float)):
        return datetime.fromtimestamp(value, tz=timezone.utc).date()
    if isinstance(value, str):
        # intentos razonables
        try:
            return datetime.fromisoformat(value).date()
        except Exception:
            # formatos comunes adicionales
            for fmt in ("%Y-%m-%d", "%Y/%m/%d", "%Y-%m-%dT%H:%M:%S",
                        "%Y-%m-%dT%H:%M:%S.%fZ"):
                try:
                    return datetime.strptime(value, fmt).date()
                except ValueError:
                    pass
    return default or date.today()


BATCH_REFERENCE_DATE = Path('./reference_date_batch')
INIT_REFERENCE_DATA = "20250904"
CONSTANT_INIT_DATE = coerce_to_date("19900101")

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

def init():
    qdrant_host = os.environ.get("QDRANT_HOST", "0.0.0.0")
    qdrant_port = int(os.environ.get("QDRANT_PORT", 6334))
    collection_name = "boe_disposiciones"

    sql_host = os.environ.get("POSTGRES_HOST", "localhost")
    sql_port = int(os.environ.get("POSTGRES_PORT", 5432))
    sql_user = os.environ.get("POSTGRES_USER", "axum")
    sql_password = os.environ.get("POSTGRES_PASSWORD", "axum")
    sql_database = os.environ.get("POSTGRES_DB", "boe_info")

    embedder = SentenceTransformer("sentence-transformers/all-MiniLM-L6-v2")  # 384 dims
    client = QdrantClient(host=qdrant_host, grpc_port=qdrant_port, prefer_grpc=True)

    db_conn = psycopg2.connect(database=sql_database,
                               user=sql_user,
                               host=sql_host,
                               password=sql_password,
                               port=sql_port)

    print(f"Using this host and port for db {qdrant_host}:{qdrant_port} for collection {collection_name} ")
    print(f"SQL Database {sql_database} for sql {sql_host}:{sql_port} for database {sql_database} ")

    return (embedder, collection_name, client, db_conn)


def run_batch():
    loaded = dotenv.load_dotenv()
    print(loaded)
    (embedder, collection_name, client, db_conn) = init()
    ensure_collection(client, collection_name, vector_size=384)
    chunk_repo = ChunkRepository(db_conn)

    str_date = read_or_create(BATCH_REFERENCE_DATE, INIT_REFERENCE_DATA)
    ref_day = coerce_to_date(str_date)
    while ref_day >= CONSTANT_INIT_DATE:
        str_date = read_or_create(BATCH_REFERENCE_DATE, INIT_REFERENCE_DATA)
        ref_day = coerce_to_date(str_date)
        last_day_not_updated =  ref_day - timedelta(days=1)
        str_last_not_updated = last_day_not_updated.strftime("%Y%m%d")
        print(f"Retrieving data from {str_last_not_updated}...")
        retries = 0
        working = False
        while not working and retries < 20:
            try:
                ingest_boe_date_to_qdrant(client, chunk_repo,  embedder, str_last_not_updated, collection_name)
                time_to_sleep.sleep(1)
            except Exception as e:
                print(f"trying number {retries}")
                retries += 1
                traceback.print_exception(e)
            else:
                working = True
        str_date = write_or_create(BATCH_REFERENCE_DATE, str_last_not_updated)
        ref_day = coerce_to_date(str_date)
        print("sleeping for the next try in 60 seconds...")
        time_to_sleep.sleep(60)

def run():
    loaded = dotenv.load_dotenv()
    print(loaded)
    (embedder, collection_name, client, db_conn) = init()
    ensure_collection(client, collection_name, vector_size=384)
    chunk_repo = ChunkRepository(db_conn)
    # Obtener fecha actual en formato aaaammdd
    now = datetime.today()
    str_now = now.strftime("%Y%m%d")
    print(f"Trying to ingest data for day={str_now}...")
    ingest_boe_date_to_qdrant(client, chunk_repo, embedder, str_now, collection_name)









