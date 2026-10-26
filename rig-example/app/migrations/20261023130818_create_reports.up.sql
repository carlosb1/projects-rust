-- Add migration script here
CREATE TYPE report_status AS ENUM (
    'pending',
    'indexing',
    'processing',
    'completed',
    'failed'
);

CREATE TABLE reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id UUID NOT NULL UNIQUE,
    path TEXT NOT NULL,
    status report_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
