CREATE TABLE entity_mentions (
    id UUID PRIMARY KEY,
    text TEXT NOT NULL,
    entity_type TEXT,
    start_offset INTEGER NOT NULL,
    end_offset INTEGER NOT NULL,
    source_uri TEXT NOT NULL,
    chunk_idx INTEGER NOT NULL,

    CHECK (start_offset >= 0),
    CHECK (end_offset > start_offset),

    UNIQUE (
        source_uri,
        chunk_idx,
        start_offset,
        end_offset
    )
);

CREATE TABLE quantity_mentions (
    id UUID PRIMARY KEY,
    value NUMERIC NOT NULL,
    unit TEXT,
    source_uri TEXT NOT NULL,
    chunk_idx INTEGER NOT NULL,

    original_text TEXT NOT NULL,
    start_offset INTEGER NOT NULL,
    end_offset INTEGER NOT NULL,

    quantity_type TEXT,

    CHECK (start_offset >= 0),
    CHECK (end_offset > start_offset),

    UNIQUE (
        source_uri,
        chunk_idx,
        start_offset,
        end_offset
    )
);