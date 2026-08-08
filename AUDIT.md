# pasm Codebase Audit

Date: 2026-07-17
Scope: pasm-core, pasm-cli, pasm-server, pasm-wasm

---

## Critical Bugs

### 1. `find` endpoint returns bare string, CLI never decrypts

- **Server**: `pasm-server/src/server/api/find.rs:26`
  `Json(result)` where `result: String` is the raw encrypted value. Produces a bare JSON
  string like `"\"encrypted_data\""`, not a structured `{"key":..., "value":...}` object.
- **CLI**: `pasm-cli/src/client/entry/ops.rs:52-58`
  Expects `parsed["value"]` to exist. It never does, so the fallback at line 58 returns
  the raw response. `deserialize_entry` at line 60 is dead code in this path.
- **Impact**: `find` has never actually decrypted and displayed an entry.

### 2. `delete` always returns 200 OK

- **Server**: `pasm-server/src/types/db.rs:342-349`
  `remove_entry` runs `DELETE FROM entries WHERE ...` but never checks `rows_affected()`.
  Deleting a non-existent entry returns `Ok(())`.
- **Doc**: `pasm-server/src/server/api/delete.rs:11`
  Claims "if key doesnot exist return Error:404, NOT_FOUND" — does not match the implementation.

### 3. Hardcoded default admin password

- **Server**: `pasm-server/src/server/mod.rs:48-57`
  Creates admin user with `derive_api_key("admin")` on every startup. The key is derived
  from a well-known value and printed to the console at line 106 with a "change it
  immediately" warning that is never enforced.

---

## Logic / API Issues

### 4. `amend_entry` returns `Err` for success (created)

- **Server**: `pasm-server/src/types/db.rs:410-416`
  When `amend_entry` creates a new entry (didn't exist before), it returns
  `Err(PasmResult::ServerStatus(StatusCode::CREATED, ...))`. The handler at
  `pasm-server/src/server/api/amend.rs:20-22` treats any `Err` as failure, so this
  works by accident — the error path returns CREATED to the client. Fragile and
  semantically wrong: a successful creation should be `Ok(())`.

### 5. `register` returns HTTP 501 for missing auth

- **Server**: `pasm-server/src/server/api/auth/register.rs:26`
  Returns `StatusCode::NOT_EXTENDED` (501) when no Bearer token is provided. Should be
  `StatusCode::UNAUTHORIZED` (401) or `StatusCode::BAD_REQUEST` (400).

### 6. Backup files world-readable

- **Server**: `pasm-server/src/server/api/backup.rs:32`
  Writes encrypted entries to `/tmp/pasm/backups/` with default filesystem permissions
  (0644). Any user on the system can read these files. No `set_restricted_permissions`
  equivalent is applied.

### 7. Auth keys stored as plaintext in PostgreSQL

- **Server**: `pasm-server/src/server/sql/mod.rs:102-103`
  Comment acknowledges: "`auth_key_hash` is currently stored as plaintext (the raw
  bearer token)." Column name says "hash" but it's the raw key.

### 8. Password "hash" is encrypt-and-compare, not a real hash

- **CLI**: `pasm-cli/src/client/auth/master.rs:89-96`
  `store_password_hash` encrypts the known plaintext `"pasm::verify"` using the password
  as an AES-256 key via MagicCrypt. No salt, no iteration, no real hashing. `verify_password`
  at line 109-119 decrypts and compares. This is not a password hash.

### 9. Session `refresh` does not verify token ownership

- **Server**: `pasm-server/src/server/api/auth/session.rs:88-101`
  Anyone with a valid refresh token can exchange it for a new session. There is no
  verification that the caller matches the token's owner.

### 10. No rate limiting on public endpoints

- **Server**: `pasm-server/src/server/mod.rs:95-100`
  `/auth`, `/auth/session`, `/auth/refresh`, `/health` are public with no rate limiting.
  Session creation and token refresh are particularly sensitive to brute-force.

---

## Dead Code

### 11. `PgDb::connect()` — never called

- **Server**: `pasm-server/src/types/db.rs:134-141`
  Defined but never called. The server creates the pool directly via `PgPoolOptions` at
  `server/mod.rs:31-35`. Also hardcodes `.max_connections(5)` instead of using
  `config::max_connections()`.

### 12. `has_entry()` in `Db` trait — never called

- **Server**: `pasm-server/src/types/db.rs:52` (trait), `331` (impl)
  Defined in the trait and implemented for `PgDb`, but no caller exists anywhere in
  the project.

### 13. `PgDb::pool()` accessor — never called

- **Server**: `pasm-server/src/types/db.rs:143-145`
  Returns `&PgPool`. No caller exists.

### 14. CLI `find` strips `"entry:"` prefix that server never adds

- **CLI**: `pasm-cli/src/client/entry/ops.rs:92`
  `raw_name.strip_prefix("entry:").unwrap_or(raw_name)` — the server stores entry names
  as-is from `payload.key`, and the CLI sends `&details.name` without any prefix. This
  code never fires.

---

## Docs vs Reality

### 15. README, ROADMAP, report.md all say SHA-256

- `README.md:146` describes SHA-256 key derivation as current.
- `README.md:153` lists Argon2id as "Not implemented / future".
- `ROADMAP.md:54` lists Argon2id as a TODO.
- `report.md:12,45,85` references SHA-256.
- **Code**: `pasm-core/src/utils/crypto.rs` already uses Argon2id.

### 16. `register.rs` doc says "from the server state" but it's from the request

- **Server**: `pasm-server/src/server/api/auth/register.rs:10-13`
  Says "Registers a new user with the authentication key from the server state" and
  "creates a new user with a generated UUID." The key comes from the request's Bearer
  token, not server state. UUID generation is handled by PostgreSQL, not the server.

### 17. `create.rs` doc has backwards error description

- **Server**: `pasm-server/src/server/api/create.rs:7`
  "if key doesnot exist returns Error:409, `CONFLICT`" — 409 is returned when the key
  *already exists*, not when it doesn't.

### 18. `delete.rs` doc promises 404 but implementation returns 200

- **Server**: `pasm-server/src/server/api/delete.rs:11`
  "if key doesnot exist return Error:404, NOT_FOUND" — implementation always returns
  `Ok(())` regardless of whether any rows were affected.

### 19. `auth/update` and `auth/remove` docs say "current user" but routes are admin-only

- **Server**: `pasm-server/src/server/api/auth/update.rs:7-8`, `remove.rs:6-8`
  Say "current user" but these routes are in `admin_routes` at `server/mod.rs:82-92`.
- **CLI**: `pasm-cli/src/client/cli/commands.rs:34-37,178`
  `UpdateAuth`, `RemoveAuth`, `ListUsers` are presented to all users without indicating
  they require admin. A regular user gets a 403 with no helpful message.

---

## Code Smells & Inconsistencies

### 20. `PasmResult` is named like a `Result`, not an error

- **Core**: `pasm-core/src/types/error.rs:9`
  The error enum is named `PasmResult`. Everywhere it's used as `Result<T, PasmResult>`,
  which reads as "Result of PasmResult". Should be `PasmError`.

### 21. Inconsistent variable naming: `user` / `user_id` / `uid`

- **Server**: `amend.rs:15` and `list.rs:12` use `let user =`, but `create.rs:15`,
  `delete.rs:19`, `find.rs:17` use `let user_id =`. `remove.rs:10` and `update.rs:11`
  use `Extension(uid)`. All hold the same thing: a user UUID string.

### 22. Bearer token extraction duplicated in 3 places

- **Server**: `auth.rs:19-33`, `register.rs:19-23`, `session.rs:33-40`
  Each extracts the Bearer token from `Authorization` header with nearly identical code.

### 23. `--addr` / `--config` CLI flag parsing duplicated

- **Server**: `pasm-server/src/main.rs:16-34`
- **CLI**: `pasm-cli/src/client/cli/commands.rs:59-82`
  Both parse `--addr` and `--config` flags with similar loop logic. The server version
  uses `std::env::set_var` which is not thread-safe.

### 24. Duplicate `set_restricted_permissions` inline code

- **CLI**: `master.rs:64-74` and `ops.rs:185-193`
  Both implement the same `#[cfg(unix)] { set_mode(0o600) }` pattern inline.

### 25. Duplicate dependency declarations across Cargo.toml files

- `serde`, `serde_json`, `magic-crypt`, `sha2`, `tokio` are declared in multiple crate
  `Cargo.toml` files with no workspace-level `[workspace.dependencies]`.

### 26. `Cargo.lock` committed for workspace with libraries

- Root `Cargo.lock` is committed. Standard for binaries, less common for libraries.

---

## Test Coverage Gaps

### 27. No handler tests for any API endpoints
All server route handlers (`create`, `find`, `list`, `delete`, `amend`, `backup`, `health`,
`users`, `register`, `update`, `remove`, `set_admin`) have zero tests that exercise the
handler logic. Only serde of response types is tested.

### 28. No auth middleware tests
`auth.rs` (Redis lookup + DB fallback) and `admin_auth.rs` (role check) have no tests.

### 29. No config tests
`pasm-core/src/utils/config.rs` has no unit tests for any function.

### 30. No curl/HTTP client tests
`pasm-cli/src/client/curl/requests.rs` — all 15+ functions are untested.

### 31. No prompt/input tests
`pasm-cli/src/client/input/prompts.rs` — no tests.

### 32. No display/response tests
`pasm-cli/src/client/response/display.rs` — no tests.

### 33. `login()` is untested
`pasm-cli/src/client/auth/master.rs:187-263` — the most complex function in the CLI,
combining interactive prompts, password hashing, API registration, and session storage.
Zero tests due to its interactive nature.

### 34. WASM encrypt/decrypt wrappers untested
`pasm-wasm/src/lib.rs` — only key derivation functions are tested. `encrypt_entry` and
`decrypt_entry` have no tests (requires wasm32 target for `JsValue`).

---

## Security Notes

| # | Issue | File | Severity |
|---|-------|------|----------|
| 1 | Default admin password `"admin"` | `server/mod.rs:48-57` | High |
| 2 | Auth keys in plaintext DB column | `sql/mod.rs:102-103` | High |
| 3 | Password "hash" is encrypt-no-salt | `master.rs:89-96` | High |
| 4 | Backup files world-readable | `backup.rs:32` | Medium |
| 5 | `std::env::set_var` not thread-safe | `main.rs:22`, `config.rs:42,92` | Medium |
| 6 | No rate limiting on public routes | `server/mod.rs:95-100` | Medium |
| 7 | No CSRF on session endpoints | `server/mod.rs:95-100` | Medium |
| 8 | Redis stores tokens in plaintext | `session.rs:56-72` | Medium |
| 9 | Hardcoded DB creds in docker-compose | `docker-compose.yml:8-9,25` | Low |
| 10 | Fixed deterministic salt for Argon2id | `crypto.rs:21` | Medium |
