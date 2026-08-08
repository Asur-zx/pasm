-- Migration 001: Initial schema for PostgreSQL-backed pasm.
-- Idempotent — safe to run multiple times (uses IF NOT EXISTS).

-- Maps auth keys to user IDs.
-- `gen_random_uuid()` is built-in since PostgreSQL 13 (no extension needed).
CREATE TABLE IF NOT EXISTS users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    auth_key_hash TEXT NOT NULL UNIQUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_users_auth_key_hash ON users(auth_key_hash);

-- Stores encrypted password entries per user.
CREATE TABLE IF NOT EXISTS entries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entry_name      TEXT NOT NULL,
    encrypted_value TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, entry_name)
);

CREATE INDEX IF NOT EXISTS idx_entries_user_id ON entries(user_id);
CREATE INDEX IF NOT EXISTS idx_entries_user_name ON entries(user_id, entry_name);
