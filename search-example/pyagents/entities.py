from dataclasses import dataclass


@dataclass
class Entity:
   value: str

@dataclass
class Quantity:
    value: str

@dataclass
class DBEntry:
    source_url: str
    chunk_idx: int
    text: str

