# src/indexing.py

import json
import logging
import os
from uuid import NAMESPACE_URL, uuid5

from fastembed import TextEmbedding
from langchain_text_splitters import RecursiveCharacterTextSplitter
from qdrant_client import QdrantClient, models
from tokenizers import Tokenizer

from src.filters import build_payload

logger = logging.getLogger(__name__)

MODEL_NAME = "sentence-transformers/all-MiniLM-L6-v2"
VECTOR_SIZE = 384
BATCH_SIZE = 32


def create_text_splitter() -> RecursiveCharacterTextSplitter:
    tokenizer = Tokenizer.from_pretrained(MODEL_NAME)
    tokenizer.no_truncation()
    tokenizer.no_padding()

    def count_tokens(text: str) -> int:
        return len(
            tokenizer.encode(text, add_special_tokens=False).ids
        )

    return RecursiveCharacterTextSplitter(
        chunk_size=220,
        chunk_overlap=35,
        length_function=count_tokens,
        # Prioriza párrafos y frases antes de cortar por palabras.
        separators=["\n\n", "\n", ". ", "? ", "! ", "; ", " ", ""],
        keep_separator="end",
    )


class ReportIndexer:
    def __init__(self, collection: str, qdrant_client: QdrantClient, text_embedding: TextEmbedding):

        self.collection = collection

        self.client = qdrant_client
        self.embedding_model =text_embedding

        self.splitter = create_text_splitter()
        self.ensure_collection()

    def ensure_collection(self) -> None:
        if not self.client.collection_exists(self.collection):
            self.client.create_collection(
                collection_name=self.collection,
                vectors_config=models.VectorParams(
                    size=VECTOR_SIZE,
                    distance=models.Distance.COSINE,
                ),
            )

        vectors = (
            self.client.get_collection(self.collection)
            .config.params.vectors
        )

        if (
            not isinstance(vectors, models.VectorParams)
            or vectors.size != VECTOR_SIZE
            or vectors.distance != models.Distance.COSINE
        ):
            raise ValueError(
                f"La colección {self.collection!r} debe usar "
                f"vectores sin nombre de {VECTOR_SIZE} dimensiones "
                "y distancia COSINE"
            )

    def index_pdf(
        self,
        job_id: str,
        source: str,
        text: str,
    ) -> int:
        chunks = self.splitter.split_text(text)

        if not chunks:
            raise ValueError(
                f"El PDF {source!r} no contiene texto extraíble"
            )

        for start in range(0, len(chunks), BATCH_SIZE):
            batch = chunks[start:start + BATCH_SIZE]

            embeddings = list(
                self.embedding_model.embed(batch)
            )

            points = []

            for offset, (chunk, embedding) in enumerate(
                zip(batch, embeddings)
            ):
                chunk_index = start + offset

                # El mismo fragmento obtiene el mismo ID al reintentar.
                identity = json.dumps(
                    [
                        self.collection,
                        job_id,
                        source,
                        chunk_index,
                    ],
                    ensure_ascii=False,
                )

                payload = build_payload(
                    chunk_text=chunk,
                    document_id=str(uuid5(NAMESPACE_URL, f"{job_id}:{source}")),
                    chunk_index=chunk_index,
                    source=source,
                )

                payload.update({
                    "job_id": job_id,
                    "embedding_model": MODEL_NAME,
                })

                points.append(
                    models.PointStruct(
                        id=str(uuid5(NAMESPACE_URL, identity)),
                        vector=embedding.tolist(),
                        payload=payload,
                    )
                )

            self.client.upsert(
                collection_name=self.collection,
                points=points,
                wait=True,
            )

        # Si se reindexa una versión más corta, elimina los chunks
        # sobrantes de la versión anterior, después de guardar la nueva.
        self.client.delete(
            collection_name=self.collection,
            points_selector=models.FilterSelector(
                filter=models.Filter(
                    must=[
                        models.FieldCondition(
                            key="job_id",
                            match=models.MatchValue(value=job_id),
                        ),
                        models.FieldCondition(
                            key="source",
                            match=models.MatchValue(value=source),
                        ),
                        models.FieldCondition(
                            key="chunk_index",
                            range=models.Range(gte=len(chunks)),
                        ),
                    ]
                )
            ),
            wait=True,
        )

        logger.info(
            "PDF indexado: job_id=%s source=%s chunks=%s",
            job_id,
            source,
            len(chunks),
        )

        return len(chunks)