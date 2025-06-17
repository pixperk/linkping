-- Add migration script here

CREATE TABLE IF NOT EXISTS links (
    id SERIAL PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    target_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);