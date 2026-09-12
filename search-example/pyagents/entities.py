from dataclasses import dataclass
from uuid import UUID


@dataclass
class Entity:
   value: str

@dataclass
class Quantity:
    value: str

@dataclass
class DBEntry:
    id: UUID
    source_url: str
    page: int
    chunk_idx: int
    text: str

