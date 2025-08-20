-- 0004_create_tangle_blocks.sql
-- Table for storing Tangle / DAG blocks


CREATE TABLE IF NOT EXISTS public.tangle_blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parents TEXT[] NOT NULL,
    data JSONB NOT NULL,
    signature BYTEA NOT NULL,
    public_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_tangle_blocks_created_at ON public.tangle_blocks (created_at DESC);
-- GIN index for array membership queries on parents
CREATE INDEX IF NOT EXISTS idx_tangle_blocks_parents_gin ON public.tangle_blocks USING GIN (parents);
