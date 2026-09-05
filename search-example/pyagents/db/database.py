from dataclasses import dataclass
import psycopg2
from entities import DBEntry

class ChunkRepository:
    def __init__(
        self,
        conn
    ):
        self.conn = conn


    def create_table(self):
        with self.conn.cursor() as cursor:
            cursor.execute(
                """
                CREATE TABLE IF NOT EXISTS chunks (
                    id BIGSERIAL PRIMARY KEY,
                    source_url TEXT NOT NULL,
                    chunk_idx INTEGER NOT NULL,
                    text TEXT NOT NULL,

                    UNIQUE(source_url, chunk_idx)
                )
                """
            )

        self.conn.commit()

    def insert(self, db_entries: list[DBEntry]):
        with self.conn.cursor() as cursor:
            cursor.executemany(
                """
                INSERT INTO chunks (
                    source_url,
                    chunk_idx,
                    text
                )
                VALUES (%s, %s, %s)
                ON CONFLICT (source_url, chunk_idx)
                DO UPDATE SET
                    text = EXCLUDED.text
                """,
                [
                    (
                        entry.source_url,
                        entry.chunk_idx,
                        entry.text,
                    )
                    for entry in db_entries
                ],
            )

        self.conn.commit()

    def delete_by_source(self, source_url: str):
        with self.conn.cursor() as cursor:
            cursor.execute(
                """
                DELETE FROM chunks
                WHERE source_url = %s
                """,
                (source_url,),
            )

        self.conn.commit()

    def drop_table(self):
        with self.conn.cursor() as cursor:
            cursor.execute(
                """
                DROP TABLE IF EXISTS chunks
                """
            )

        self.conn.commit()

    def close(self):
        self.conn.close()