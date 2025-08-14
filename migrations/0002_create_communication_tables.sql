-- 0002_create_communication_tables.sql
-- Tables for communication plugin: blogs and stories

CREATE TABLE IF NOT EXISTS blogs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    body TEXT NOT NULL,
    author TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS blogs_created_at_idx ON blogs (created_at DESC);
CREATE INDEX IF NOT EXISTS blogs_is_active_idx ON blogs (is_active);

CREATE TABLE IF NOT EXISTS stories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT,
    media_url TEXT NOT NULL,
    caption TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    created_by TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS stories_created_at_idx ON stories (created_at DESC);
CREATE INDEX IF NOT EXISTS stories_is_active_idx ON stories (is_active);
