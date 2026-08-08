# TODO

## Redis Layer
- [x] Add `redis` crate to pasm-server
- [x] Add `PASM_REDIS_URL` env var to config
- [x] Add Redis connection to `PasmState`
- [x] Auth middleware: Redis-first lookup (fall back to DB)

## Session & Refresh Keys
- [x] `POST /auth/session` endpoint — takes api_key, returns session token + refresh token
- [x] `POST /auth/refresh` endpoint — takes refresh token, returns new session token
- [x] Session token: short TTL (24h), stored in Redis as `session:<token> -> api_key`
- [x] Refresh token: longer TTL (7d), stored in Redis as `refresh:<token> -> api_key`
- [ ] Frontend: store refresh token in localStorage, session token in memory
- [ ] CLI: store refresh token in session file, re-auth transparently on expiry

## Database Restructure
- [ ] Review current schema (single `users` table with `auth_key_hash`)
- [ ] Split into three tables:
  - `users` — id, created_at
  - `authentications` — id, user_id, auth_key_hash, label, created_at (one user can have multiple auth keys)
  - `entries` — id, user_id, entry_name, encrypted_value, created_at, updated_at
- [ ] Migration script: `003_restructure.sql`
- [ ] Update all DB queries in `types/db.rs`

## Key Rotation
- [ ] `POST /auth/rotate` endpoint — takes old api_key + new api_key, re-links entries
- [ ] Optionally re-encrypt all entries with new encr_key (client-side)
- [ ] Invalidate all session tokens for that user on rotation

## Rate Limiting
- [ ] Per IP: track request count per IP in Redis, block after N/min
- [ ] Per user: track request count per user_id in Redis, block after M/min
- [ ] Configurable limits via env vars (`PASM_RATE_LIMIT_IP`, `PASM_RATE_LIMIT_USER`)
- [ ] Axum middleware layer for rate limiting
