//! SQL reference — every database operation with its query.
//!
//! ## Schema
//!
//! Two tables (see `migrations/001_init.sql`):
//! - `users`       — auth key → user ID mapping
//! - `entries`     — per-user encrypted password entries

// ──────────────────────────────────────────────
// User operations
// ──────────────────────────────────────────────

// * `get_user_id_by_authkey(authkey)`
//   ```sql
//   SELECT id::text FROM users WHERE auth_key_hash = $1;
//   ```
//   Returns `NotFound` when no row matches.
//
// * `auth_key_exists(authkey)`
//   ```sql
//   SELECT 1 FROM users WHERE auth_key_hash = $1;
//   ```
//
// * `register_auth(auth_key)`
//   ```sql
//   INSERT INTO users (auth_key_hash) VALUES ($1)
//   ON CONFLICT (auth_key_hash) DO NOTHING;
//   ```
//   Returns `Conflict` when the auth_key_hash already exists.
//
// * `update_auth(old_key, new_key)`
//   ```sql
//   UPDATE users SET auth_key_hash = $2 WHERE id = $1::uuid;
//   ```
//   Preceded by existence checks for both old and new keys.
//
// * `remove_user(auth_key)`
//   ```sql
//   DELETE FROM users WHERE auth_key_hash = $1;
//   ```
//   Entries cascade via `ON DELETE CASCADE`.
//
// * `list_users()`
//   ```sql
//   SELECT auth_key_hash FROM users ORDER BY created_at;
//   ```

// ──────────────────────────────────────────────
// Entry operations
// ──────────────────────────────────────────────

// * `add_entry(user_id, entry_name, encrypted_data)`
//   ```sql
//   INSERT INTO entries (user_id, entry_name, encrypted_value)
//   VALUES ($1::uuid, $2, $3)
//   ON CONFLICT (user_id, entry_name) DO NOTHING;
//   ```
//   Returns `Conflict` when the (user_id, entry_name) pair already exists.
//
// * `get_entry(user_id, entry_name)`
//   ```sql
//   SELECT encrypted_value FROM entries
//   WHERE user_id = $1::uuid AND entry_name = $2;
//   ```
//   Returns `NotFound` when no matching row.
//
// * `remove_entry(user_id, entry_name)`
//   ```sql
//   DELETE FROM entries WHERE user_id = $1::uuid AND entry_name = $2;
//   ```
//
// * `list_entries(user_id)`
//   ```sql
//   SELECT entry_name, encrypted_value FROM entries
//   WHERE user_id = $1::uuid ORDER BY entry_name;
//   ```
//
// * `amend_entry(user_id, entry_name, encrypted_data)`
//   ```sql
//   -- If entry exists:
//   UPDATE entries SET encrypted_value = $3, updated_at = now()
//   WHERE user_id = $1::uuid AND entry_name = $2;
//   -- If new:
//   INSERT INTO entries (user_id, entry_name, encrypted_value)
//   VALUES ($1::uuid, $2, $3);
//   ```
//   Returns `Created` for a new entry, `Ok` for an update.

// ──────────────────────────────────────────────
// Notes
// ──────────────────────────────────────────────
//
// 1. `user_id` is a PostgreSQL `UUID` column, referenced by foreign key
//    from `entries` with `ON DELETE CASCADE`.
//
// 2. `auth_key_hash` is currently stored as plaintext (the raw bearer token).
//    The server should hash incoming tokens with SHA-256 before querying
//    (see Phase 2 of the roadmap).
//
// 3. The `UNIQUE(user_id, entry_name)` constraint replaces sled's
//    `compare_and_swap` — the DB rejects duplicates at the constraint level.
//
// 4. Per-user isolation is enforced via `WHERE user_id = $1` on every
//    entry query, replacing sled's per-user tree structure.
