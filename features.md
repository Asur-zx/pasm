# Features — Completed

Completed features and improvements in pasm, organized by area.

---

## Workspace Restructure

- **Crate split into workspace** — Monolithic `src/` replaced with
  `pasm-core`, `pasm-server`, `pasm-cli`, `pasm-wasm`.
- **Shared `pasm-core`** — Types (`detail`, `entry`, `error`) and utils
  (`encrypt`, `decrypt`, `serialize`, `deserialize`, `crypto`, `config`)
  shared by all consumers. `error.rs` feature-gates `IntoResponse` behind
  `server` feature; `config` behind `native` feature.
- **Key derivation extracted** — `derive_api_key`, `derive_key`, `hash_key`
  moved to `pasm_core::utils::crypto`, removing the duplicated copies in
  `master.rs` and `server/mod.rs`.
- **`pasm-wasm` crate** — `#[wasm_bindgen]` stub exposing
  `encrypt_entry`, `decrypt_entry`, `derive_api_key`, `derive_encr_key`.
  Build with `wasm-pack build pasm-wasm --target web`.

---

## Admin Role System

- **`002_admin_role.sql` migration** — Adds `roles TEXT[] DEFAULT ARRAY['user']`
  column to `users` table. Idempotent (`ADD COLUMN IF NOT EXISTS`).
- **`AuthInfo` struct** — `{ token, roles }` injected into request extensions
  by the auth middleware.
- **`admin_auth` middleware** — Reads `AuthInfo` from extensions, returns
  `403` if `"admin"` not in roles. Zero extra DB queries (roles already
  fetched by auth middleware).
- **Route split** — `auth/update`, `auth/remove`, `auth/list`, `auth/set-admin`
  require both auth + admin middleware. Entry routes require auth only.
- **Bootstrap admin** — Server creates `admin` user (`['user','admin']` roles,
  password `admin`) on first start via `ON CONFLICT DO NOTHING`.
- **Client `set-admin` command** — `pasm_client set-admin <key>` calls
  `POST /auth/set-admin`.

---

## Foundation & Configuration

- **Health endpoint** — `GET /health` returns `status`, `version`,
  `uptime_seconds`, `database` connectivity, and `timestamp`. Returns
  `200 OK` when healthy, `503` when database is unreachable.

- **Client connectivity check** — Before every command, `pasm_client`
  calls `GET /health` with a 3-second curl timeout. If the server
  doesn't respond with `200`, the client prints the server URL and
  exits immediately instead of sending the actual request.

- **Environment-based config** — `server_url()` / `set_server_url()`
  (reads `PASM_SERVER_URL`), `server_addr()` (reads `PASM_SERVER_ADDR`),
  `db_path()` (reads `PASM_DB_PATH`). Server and client both read from
  this shared module.

- **CLI args for both binaries** — `pasm_client --addr <host:port>` and
  `pasm_server --addr <host:port>` override the env var/default at
  startup. Client addr sets `PASM_SERVER_URL`, server addr sets
  `PASM_SERVER_ADDR`.

- **TOML config file** — `~/.config/pasm/config.toml` with `server_url`,
  `server_addr`, `database_url`, `max_connections`. Overridable via
  `PASM_CONFIG` env var or `--config <path>` CLI flag. Created
  automatically on first `pasm_client login`.

- **Categorized CLI help** — Usage text organized into Global options,
  Session management, Entry management, Account management sections.

- **Descriptive error responses** — Server 401 responses include the
  reason (missing token, invalid key). Client `run_curl` includes the
  server body in HTTP error messages. Errors print to stderr in red.

---

## Database (PostgreSQL)

- **PostgreSQL migration** — Sled replaced with `sqlx::PgPool` connection
  pool. Configurable `max_connections` (default 5).

- **Schema migration** — Idempotent `001_init.sql` with `users` and
  `entries` tables, UUID primary keys, `ON DELETE CASCADE`, unique
  constraint on `(user_id, entry_name)`.

- **`Db` trait** — 15 async methods in `types/db.rs`, object-safe via
  `#[async_trait]`, backed by `PgDb` or mockable for tests.

- **`PgDb` implementation** — All queries use `sqlx::query` with `$N`
  positional binds. Pool configured on server startup.

- **Dependencies pruned** — `sled` and `uuid` crates removed.

---

## Backup & Restore

- **Server backup endpoint** — `GET /backup` dumps all encrypted entries
  to `/tmp/pasm/backups/<user_id>_<timestamp>.json` with entry count and
  file size.

- **Client `backup` command** — Calls server backup and also saves a
  local copy to `~/.config/pasm/backups/backup_<timestamp>.json` with
  0600 permissions.

- **Restore from file** — `pasm_client --loadfile <path>` reads a local
  backup JSON and restores all entries to the server via
  `POST /entry/amend`.

---

## Operations

- **Docker Compose** — Multi-service setup with PostgreSQL 17 Alpine and
  pasm server. Healthcheck waits for PG before starting pasm. Persistent
  `pgdata` volume.

- **Dockerfile** — Multi-stage Alpine build. Builder compiles
  `pasm_server` with musl + openssl; runtime image is a minimal Alpine
  with just the binary and TLS certs.

---

## Bug Fixes

- **Amend entry** — `amend_entry` now correctly returns `Err(CREATED)`
  for new entries, `Ok(())` for updates (was reversed before migration).
