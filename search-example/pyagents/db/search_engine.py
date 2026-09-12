import time
import uuid
from typing import List, Tuple, Dict, Any

from qdrant_client import QdrantClient
from qdrant_client.conversions.common_types import PointStruct
from qdrant_client.http.models import Distance, VectorParams
from sentence_transformers import SentenceTransformer
from tqdm import tqdm

def wait_for_qdrant(client: QdrantClient, retries: int = 10):
    for attempt in range(retries):
        try:
            client.get_collections()
            return
        except Exception as e:
            if attempt == retries - 1:
                raise

            wait = min(2 ** attempt, 10)
            print(f"Qdrant not ready: {e}. Retrying in {wait}s...")
            time.sleep(wait)

def upsert_chunks(
    client: QdrantClient,
    collection: str,
    embedder: SentenceTransformer,
    chunks: List[Tuple[str, Dict[str, Any]]],  # (text, payload)
    batch_size: int = 128,
):
    # Embed and upload in batches
    for i in tqdm(range(0, len(chunks), batch_size), desc="Upserting to Qdrant"):
        batch = chunks[i:i+batch_size]
        texts = [t for t, _ in batch]
        vecs = embedder.encode(texts, show_progress_bar=False, convert_to_numpy=True, normalize_embeddings=True)
        points = []
        for (text, payload), vec in zip(batch, vecs):
            pid = uuid.uuid5(uuid.NAMESPACE_URL, payload["source_url"] + f'/{payload["page"]}/{payload["chunk_idx"]}')
            p = PointStruct(id=str(pid), vector=vec.tolist(), payload={**payload, "text": text})
            print("inserting pid=", pid)
            points.append(p)
        print("Upserting chunk #", i)
        client.upsert(collection_name=collection, points=points)

def ensure_collection(client: QdrantClient, name: str, vector_size: int = 384):
    # Create if missing
    existing = {c.name for c in client.get_collections().collections}
    if name not in existing:
        client.recreate_collection(  # recreate ensures exact params; safe if new
            collection_name=name,
            vectors_config=VectorParams(size=vector_size, distance=Distance.COSINE),
        )
