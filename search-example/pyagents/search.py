import dotenv
from qdrant_client import QdrantClient
from qdrant_client.models import Filter, FieldCondition, MatchValue
from sentence_transformers import SentenceTransformer
import os


loaded = dotenv.load_dotenv()
print(loaded)


qdrant_host = os.environ.get("QDRANT_HOST","0.0.0.0")
qdrant_port = int(os.environ.get("QDRANT_PORT",6333))
collection_name= "boe_disposiciones"

client = QdrantClient(host=qdrant_host, port=qdrant_port)

query = "autoridad portuaria tarifas Baleares"

#encoder
enc = SentenceTransformer("sentence-transformers/all-MiniLM-L6-v2")

def search(query,collection_name,enc):
    hits = client.search(
        collection_name=collection_name,
        query_vector=enc.encode(query, normalize_embeddings=True).tolist(),
        limit=20,
        query_filter=Filter(
            must=[FieldCondition(key="doc_type", match=MatchValue(value="BOE"))]
        )
    )
    return hits

def print_hits(hits):
    for h in hits:
        p = h.payload
        print(round(h.score, 4), p.get("item_identificador"), p.get("item_titulo"))
        print(p.get("source_url"), f"page={p.get('page')}, chunk={p.get('chunk_idx')}")
        print()

#!/usr/bin/env python3
import argparse

def parse_args():
    p = argparse.ArgumentParser(
        prog="search",
        description="Search documents (text, vector, or hybrid)."
    )

    # Query
    p.add_argument("query", nargs="+", help="Search query string (one or more tokens).")

    # Mode
    mode = p.add_mutually_exclusive_group()
    mode.add_argument("--text",    action="store_true", help="Text/BM25 search.")
    mode.add_argument("--vector",  action="store_true", help="Semantic/vector search.")
    mode.add_argument("--hybrid",  action="store_true", help="Blend text + vector.")

    # General options
    p.add_argument("-k", "--top-k", type=int, default=10, help="Max results to return.")
    p.add_argument("--offset", type=int, default=0, help="Results offset (pagination).")
    p.add_argument("--filter", action="append", default=[],
                   help="Filter expression(s). Repeat for multiple filters.")
    p.add_argument("--sort", default=None, help="Sort by field (e.g. 'date:desc').")

    # Text (e.g., Meilisearch)
    p.add_argument("--meili-url", default="http://localhost:7700")
    p.add_argument("--meili-key", default=None)
    p.add_argument("--index", default="documents", help="Text index name.")

    # Vector (e.g., Qdrant)
    p.add_argument("--qdrant-url", default="http://localhost:6333")
    p.add_argument("--collection", default="documents_vec", help="Vector collection.")
    p.add_argument("--embedding-model", default="text-embedding-3-large",
                   help="Embedding model name.")
    p.add_argument("--dim", type=int, default=3072, help="Embedding dimension (if needed).")

    # Hybrid weight (0.0 uses pure text, 1.0 pure vector)
    p.add_argument("--alpha", type=float, default=0.5,
                   help="Hybrid mix weight (vector weight).")

    # Output
    p.add_argument("--json", action="store_true", help="Print JSON output.")
    p.add_argument("--pretty", action="store_true", help="Pretty-print results.")

    args = p.parse_args()

    # Sensible defaultsq
    if not (args.text or args.vector or args.hybrid):
        args.text = True  # default to text

    return args



