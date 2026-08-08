# pasm Project Review

**Review date:** 2026-03-13  
**Scope:** Architecture, security, correctness, code quality, build/test health, deployment configuration, and maintainability.

## Executive summary

`pasm` is a small, understandable Rust password-manager prototype with a clean client/server split, parameterized SQL, client-side encryption, PostgreSQL ownership boundaries, and a build that passes formatting and strict Clippy checks. It is **not ready for production or exposure to an untrusted network**.

The largest risks are architectural rather than stylistic:

1. Master-password keys use fast, unsalted SHA-256 derivation, making offline guessing cheap.
2. The server stores raw bearer-token equivalents despite naming the column `auth_key_hash`.
3. Any authenticated user can retrieve every user's bearer token through `GET /auth/list` and impersonate them.
4. HTTP is the default and TLS is absent, exposing bearer tokens and ciphertext metadata in transit on non-local networks.
5. Encryption relies on `magic-crypt` rather than an explicitly authenticated AEAD construction with managed nonces and versioned metadata.
6. There are zero automated tests, so successful `cargo test` currently proves compilation only.

**Overall health:** Prototype / pre-production.  
**Security posture:** High risk if network-accessible; acceptable only for isolated experimentation with non-sensitive data.  
**Code quality:** Generally readable and lint-clean, but correctness and API design need stronger tests and several focused fixes.

## Review method and verification

Commands run from the repository root:

- `cargo fmt --check` — passed.
- `cargo clippy --all-targets --all-features -- -D warnings` — passed.
- `cargo test --all-targets --all-features` — passed with **0 tests** across the library and both binaries.
- `cargo audit` — not run because the `cargo-audit` subcommand is not installed.
- Git working tree was clean before this report and task list were created.

Not verified:

- End-to-end behavior against a running PostgreSQL instance.
- Docker image build and runtime behavior.
- Dependency advisories/CVEs.
- Load, concurrency, migration rollback, backup restore, or cross-platform behavior.

## Architecture overview

```text
Master password
      |
      +-- SHA-256 contexts --> API bearer token --> Axum API --> PostgreSQL
      |
      +-- SHA-256 context  --> encryption key --> magic-crypt --> ciphertext

Local client files                         Server database
~/.config/pasm/master.hash                 users.auth_key_hash (raw token)
~/.config/pasm/session                     entries.encrypted_value
(api token + encryption key)
```

Positive architectural properties:

- Password-entry plaintext is encrypted on the client before upload.
- Entries are scoped by a server-resolved user UUID in database queries.
- SQL parameters are bound rather than interpolated, substantially reducing SQL injection risk.
- Foreign keys use `ON DELETE CASCADE`, keeping account deletion coherent.
- The database is accessed behind a `Db` trait, which supports future test doubles.
- Configuration supports command-line, environment, and TOML sources.
- Schema creation is idempotent for the current single migration.

## Security findings

### Critical — Authenticated users can retrieve all bearer tokens

**Evidence:** `src/server/api/users.rs:6-14` calls `list_users`; `src/types/db.rs:198-205` returns every `auth_key_hash`. The route is available to every authenticated user in `src/server/mod.rs:56`.

The values are usable bearer tokens, not irreversible hashes. Any registered user can call `GET /auth/list`, receive all account credentials, then authenticate as another user and read, modify, delete, or back up that user's encrypted vault. Even though entry contents remain encrypted, account takeover permits destructive actions and ciphertext exfiltration for offline attacks.

**Recommendation:** Remove the endpoint immediately. If administrative user listing is required later, introduce explicit administrator authorization and return non-secret user identifiers/metadata only. Never return credential material.

### Critical — Bearer tokens are stored as plaintext-equivalent database values

**Evidence:** `src/types/db.rs:99-127` compares and inserts the received bearer token directly into `users.auth_key_hash`; the schema name in `src/server/sql/migrations/001_init.sql:7-11` is misleading. The source documentation itself acknowledges this in `src/server/sql/mod.rs:102`.

A database read compromise immediately becomes account compromise. Double-hashing on the client does not help because the final hash is itself the bearer credential.

**Recommendation:** Hash the presented token server-side before storage and lookup, preferably with a keyed server-side HMAC/pepper if operational key management is available. Use constant-time verification where direct secret comparisons remain. Migrate existing tokens deliberately and rename fields to describe actual semantics.

### High — Master-password derivation is fast and unsalted

**Evidence:** `src/client/auth/master.rs:59-75` derives keys using `SHA-256(context || password)` with fixed context strings; `src/client/auth/master.rs:200-216` derives the bearer token with another fast SHA-256 pass.

An attacker who obtains `master.hash`, a bearer token, or ciphertext can test password guesses cheaply. Fixed context strings provide domain separation but are not random salts and do not slow attacks. A second SHA-256 pass does not materially improve password resistance.

**Recommendation:** Use Argon2id with a unique random salt and calibrated memory/time costs. Derive independent authentication and encryption subkeys from the resulting master key with HKDF. Define a migration/version format before changing existing vaults.

### High — Transport security is absent and HTTP is the default

**Evidence:** Defaults and examples use `http://` in `README.md:26-31` and `README.md:69-74`; `src/server/mod.rs:66-69` binds a plain TCP listener and serves HTTP directly.

Bearer tokens can be captured and replayed by anyone able to observe traffic. Client-side encryption does not protect authentication credentials, entry names, response metadata, or destructive API calls.

**Recommendation:** Treat non-loopback HTTP as unsafe. Add TLS support or document a mandatory trusted TLS reverse proxy, default client URLs to HTTPS for remote use, and reject or prominently gate plaintext remote connections.

### High — Encryption design is not explicitly authenticated or versioned

**Evidence:** `src/utils/encrypt.rs:1-19` and `src/utils/decrypt.rs:1-24` delegate encryption to `magic-crypt`; the database stores only one opaque string in `src/server/sql/migrations/001_init.sql:14-24`.

The application does not explicitly manage an AEAD algorithm, nonce, authentication tag, algorithm/version identifier, or key version. This makes integrity guarantees dependent on a wrapper library's undocumented details and makes safe migration/rotation difficult. The comment claiming random IV behavior in `src/utils/encrypt.rs:5-7` should not substitute for a verified cryptographic format.

**Recommendation:** Move to a well-reviewed AEAD such as AES-256-GCM or XChaCha20-Poly1305, generate a fresh cryptographically secure nonce per entry encryption, authenticate relevant metadata, and persist a versioned envelope containing algorithm, nonce, ciphertext/tag, KDF version, and key version.

### High — Local session stores both reusable auth and encryption keys

**Evidence:** `src/client/auth/master.rs:18-27` defines a session containing the API token and encryption key; `src/client/auth/master.rs:166-185` writes both as plaintext JSON. Logout only deletes the file and does not revoke the server token (`src/client/auth/master.rs:328-337`).

Mode `0600` is a good baseline, but malware or any process running as the same user can recover complete long-lived access. File deletion may not securely erase storage. Auto-login keeps high-value keys persistently available.

**Recommendation:** Use an OS keyring/credential service, introduce expiring/revocable sessions, and make logout invalidate the server-side credential/session. Consider keeping the encryption key in memory only by default and zeroizing secret buffers where practical.

### High — Registration is unrestricted and lacks abuse controls

**Evidence:** `POST /auth` is public in `src/server/mod.rs:60-63`; `src/server/api/auth/register.rs:13-29` accepts any Bearer value and inserts a user. There is no rate limiting, token length/format validation, invitation mode, or account policy.

An attacker can create unlimited accounts and consume database/storage resources. They can also immediately exploit `/auth/list` after creating one account.

**Recommendation:** First remove secret enumeration. Then add registration policy (local-only, invitations, or disabled by default for single-user deployments), strict token/input limits, per-IP throttling, global capacity controls, and audit logging.

### Medium — Server-side backups create unmanaged sensitive artifacts

**Evidence:** `src/server/api/backup.rs:7-72` writes every encrypted entry to `/tmp/pasm/backups`, returns the absolute path, and does not set restrictive permissions, expire files, prevent accumulation, or provide a restore workflow.

The content is encrypted, but entry names and ciphertext are still sensitive and useful for offline attack. On shared hosts, default umask and `/tmp` behavior may expose data. Repeated requests cause unbounded disk growth. Files inside the current container are also ephemeral unless separately mounted.

**Recommendation:** Prefer streaming an authenticated export to the client. If server files are required, use a private configured directory, create files atomically with `0600`, enforce quotas and retention, avoid returning internal paths, and test restore integrity.

### Medium — User-controlled input has no explicit size limits

**Evidence:** `src/types/entry.rs:12-17` accepts unbounded strings; routes accept Axum JSON bodies without a configured body limit visible in `src/server/mod.rs:47-64`. Entry names are also placed into URL paths by `src/client/curl/requests.rs:132-143` and `src/client/curl/requests.rs:174-185` without URL encoding.

Large payloads can consume memory and database storage. Names containing `/`, `?`, `#`, `%`, spaces, or Unicode edge cases can fail or target unintended URLs.

**Recommendation:** Set request-body limits, cap entry-name and ciphertext lengths at both API and schema boundaries, validate token format/length, and use a native URL-aware HTTP client.

### Medium — Authentication does redundant database lookups

**Evidence:** Middleware checks token existence in `src/server/auth.rs:35-38`, then handlers look up the same token again, for example `src/server/api/delete.rs:18-22`.

This doubles authentication-path database work and stores the secret token in request extensions. It also makes authorization logic repetitive.

**Recommendation:** Resolve the user ID once in middleware and place a non-secret authenticated-principal type in request extensions. Handlers should consume that principal.

### Medium — No rate limiting, audit trail, security headers, or request tracing

The server has no visible rate limiter, structured security audit log, request IDs, tracing, timeout layer, concurrency limit, or graceful shutdown. These controls are especially important for an authentication service and are already recognized in `ROADMAP.md`.

**Recommendation:** Add bounded request timeouts and body sizes first, then rate limiting and structured tracing with secret redaction. Record account and entry mutations without logging tokens, ciphertext, or plaintext.

### Low — Error/status semantics are inconsistent

**Evidence:** Missing registration authentication returns `510 Not Extended` in `src/server/api/auth/register.rs:20-26`, where `401 Unauthorized` is expected. Entry deletion returns success even when no row exists because `src/types/db.rs:249-257` does not inspect `rows_affected`, and `src/server/api/delete.rs:24-28` always returns `200` after a successful query. `amend_entry` uses an error variant to represent successful creation at `src/types/db.rs:326-330`.

**Recommendation:** Define a typed API error/response model, use conventional HTTP statuses, and test every route's status and body contract.

## Correctness and code-quality review

### Strengths

- The project passes `rustfmt` and strict Clippy with warnings denied.
- Modules are small and responsibilities are mostly clear.
- SQL statements use bound parameters throughout `src/types/db.rs`.
- Database ownership is consistently scoped by `user_id` for entry operations.
- Errors are converted into a project-specific response type rather than broadly unwrapped in request handlers.
- Session and master-verifier files attempt `0600` permissions on Unix.
- README and roadmap are useful and unusually candid about missing production features.

### Issues

#### Zero automated test coverage

No `#[test]`, `#[tokio::test]`, or test modules were found. `cargo test` runs three empty test binaries. Cryptographic derivation, serialization, configuration precedence, authentication middleware, database behavior, route isolation, status codes, and CLI workflows are unprotected against regression.

#### Database upsert is race-prone and semantically awkward

`src/types/db.rs:280-330` performs `SELECT EXISTS` followed by `UPDATE` or `INSERT`. Concurrent calls can race. PostgreSQL already supports atomic `INSERT ... ON CONFLICT ... DO UPDATE`. The method also returns `Err(ServerStatus(CREATED, ...))` for a successful insert, conflating control flow and errors.

#### Delete does not distinguish missing entries

`src/types/db.rs:249-257` reports `Ok(())` regardless of affected rows, contradicting the route documentation in `src/server/api/delete.rs:9-10`.

#### Configuration and runtime failures panic

`src/utils/config.rs:126` panics when the database URL is absent, while `src/server/mod.rs:30-41` and `src/server/mod.rs:66-69` use `expect`/`unwrap`. Fail-fast startup is reasonable, but a password manager should return contextual errors and controlled exit codes rather than panic output.

#### Blocking filesystem work occurs in async request handling

`src/server/api/backup.rs:31-61` performs directory creation and file writing directly in an async handler. Large backups can block a Tokio worker. Streaming to the client would avoid this; otherwise use asynchronous filesystem APIs or `spawn_blocking`.

#### The CLI shells out to `curl`

`src/client/curl/requests.rs:1-330` relies on an external binary, manually parses status output, lacks a default timeout for most operations, and manually builds path URLs. A native Rust client would improve portability, URL encoding, timeout control, TLS configuration, connection reuse, and typed errors.

#### Secret-bearing strings are copied extensively

Tokens and encryption keys are ordinary `String`s and cloned/formatted into command arguments. This increases lifetime and exposure in process memory. Passing bearer tokens to `curl` also exposes them in the spawned process argument list on systems where process command lines are inspectable.

#### Documentation and implementation drift

- The README documents entry/auth routes but omits the implemented `/backup` route.
- The field `auth_key_hash` is not a server-side hash.
- `PgDb::connect` hardcodes five connections at `src/types/db.rs:79-87`, while server startup separately honors configurable pool size.
- Comments in `src/server/api/auth/register.rs:9-11` refer to a “server state” authentication key but the key comes from the request.

#### Migration strategy will not scale

Startup executes one embedded `001_init.sql` via `raw_sql` (`src/server/mod.rs:37-41`) rather than tracking applied migration versions. `CREATE TABLE IF NOT EXISTS` does not evolve existing schemas when columns or constraints change.

## Project health assessment

| Area | Rating | Notes |
|---|---|---|
| Build health | Good | Formatting and strict Clippy pass; project compiles. |
| Test health | Critical gap | Zero tests; no integration or end-to-end verification. |
| Security | High risk | Critical credential disclosure and weak password KDF. |
| Maintainability | Fair | Small, readable modules; some duplicated auth flow and semantic inconsistencies. |
| Dependency health | Unknown | Locked dependencies exist, but advisory scan was unavailable. |
| Documentation | Good for prototype | Clear README/roadmap, but some route and security semantics have drifted. |
| Operations | Weak | No graceful shutdown, observability, rate limits, durable backup design, or migration tracking. |
| Deployment | Development-only | Default credentials, published PostgreSQL port, HTTP-only service, container runs as root. |

## Deployment observations

- `docker-compose.yml:7-10` uses public default database credentials (`pasm`/`pasm`) and publishes PostgreSQL on host port 5432. This is convenient for development but unsafe as a production template.
- `Dockerfile:12-17` runs the application as root in the runtime image; add a dedicated unprivileged user and a read-only filesystem where practical.
- The runtime image installs `curl`, but only the client shells out to curl; the server image likely does not need it.
- Base images are floating tags (`rust:alpine`, `alpine:latest`, `postgres:17-alpine`), reducing reproducibility. Pin versions/digests and update intentionally.
- No container health check is defined for the application service.
- No CI workflow, license file, release automation, or contribution guide was observed at repository root.

## Recommended remediation order

### Immediate containment

1. Remove `/auth/list` and stop returning bearer credentials.
2. Document the application as local-development-only and avoid real credentials until critical issues are fixed.
3. Do not expose the API over plaintext remote networks.
4. Restrict or disable public registration.

### Security foundation

1. Implement Argon2id with unique salts and HKDF-separated keys.
2. Replace encryption with a versioned AEAD envelope.
3. Hash/pepper bearer credentials server-side and design revocable, expiring sessions.
4. Move local secrets to an OS credential store where available.
5. Add strict request/token/name/ciphertext limits and URL-safe HTTP handling.

### Correctness and confidence

1. Build unit tests for KDF/encryption/config/serialization.
2. Build database integration tests for user isolation, duplicate handling, atomic amend, and deletion semantics.
3. Build API tests for authentication and all status contracts, including attempts to cross account boundaries.
4. Add CI for format, Clippy, tests, dependency audit, and secret scanning.

### Operational maturity

1. Adopt tracked SQLx migrations.
2. Add structured tracing, request IDs, timeouts, graceful shutdown, and redaction.
3. Replace server-side backup files with authenticated export/import.
4. Harden container and Compose configuration.

## Conclusion

The repository is a promising prototype with a sound basic separation between client-side encryption and server-side storage, and its Rust code is presently clean under compiler/linter checks. However, the credential-listing endpoint and raw bearer-token storage are release-blocking vulnerabilities, while weak password derivation, absent TLS, unclear encryption integrity, and zero tests prevent trustworthy use as a password manager. Complete the P0 items in `todo.md` before adding roadmap features or using real secrets.
