use axum::http::StatusCode;
use sqlx::PgPool;
use sqlx::Row;

use pasm_core::types::{entry::RequestData, error::PasmResult};

/// Database operations trait.
///
/// Abstracts all persistence operations so the rest of the application is
/// agnostic about the database backend. Currently implemented by [`PgDb`].
#[async_trait::async_trait]
pub trait Db: Send + Sync + 'static {
    /// Health check: verifies the database is reachable.
    async fn ping(&self) -> Result<(), PasmResult>;

    /// Looks up a user ID by their authentication key.
    async fn get_user_id_by_authkey(&self, authkey: &str) -> Result<String, PasmResult>;

    /// Checks whether a given authentication key exists in the database.
    async fn auth_key_exists(&self, authkey: &str) -> Result<bool, PasmResult>;

    /// Returns the roles array for a user, or `None` if the key doesn't exist.
    async fn get_user_roles(&self, authkey: &str) -> Result<Option<Vec<String>>, PasmResult>;

    /// Sets the roles for a target user to `['user', 'admin']`.
    async fn set_admin_role(&self, target_auth_key: &str) -> Result<(), PasmResult>;

    /// Registers a new authentication key, returning a generated user ID.
    async fn register_auth(&self, auth_key: &str) -> PasmResult;

    /// Replaces an old authentication key with a new one.
    async fn update_auth(&self, auth_key: &str, new_auth: &str) -> PasmResult;

    /// Removes a user and all their entries.
    async fn remove_user(&self, auth_key: &str) -> PasmResult;

    /// Lists every registered authentication key.
    async fn list_users(&self) -> Result<Vec<String>, PasmResult>;

    /// Inserts a new encrypted entry for a user.
    async fn add_entry(
        &self,
        user_id: &str,
        entry_name: &str,
        encrypted_data: &str,
    ) -> Result<(), PasmResult>;

    /// Retrieves an entry's encrypted value by name.
    async fn get_entry(&self, user_id: &str, entry_name: &str) -> Result<String, PasmResult>;

    /// Removes a single entry.
    async fn remove_entry(&self, user_id: &str, entry_name: &str) -> Result<(), PasmResult>;

    /// Returns all entries belonging to a user.
    async fn list_entries(&self, user_id: &str) -> Result<Vec<RequestData>, PasmResult>;

    /// Creates or overwrites an entry.
    async fn amend_entry(
        &self,
        user_id: &str,
        entry_name: &str,
        encrypted_data: &str,
    ) -> Result<(), PasmResult>;
}

/// Splits a PostgreSQL `array_to_string` result into a `Vec<String>`.
///
/// A role array like `'{user,admin}'` is converted to `"user,admin"` by
/// the SQL function `array_to_string`, then parsed by this function.
fn parse_roles_csv(roles_str: &str) -> Vec<String> {
    if roles_str.is_empty() {
        Vec::new()
    } else {
        roles_str.split(',').map(|s| s.to_string()).collect()
    }
}

/// Classifies the result of an INSERT … ON CONFLICT DO NOTHING operation.
///
/// * `rows_affected > 0` — row was inserted → `OK`
/// * `rows_affected == 0` — row already existed → `CONFLICT`
fn register_result(rows_affected: u64) -> PasmResult {
    if rows_affected > 0 {
        PasmResult::ServerStatus(
            StatusCode::OK,
            "registered new authentication token!".to_string(),
        )
    } else {
        PasmResult::ServerStatus(StatusCode::CONFLICT, "auth key already exists".to_string())
    }
}

/// Classifies the result of a DELETE … WHERE query by `rows_affected`.
///
/// * `rows_affected > 0` — row was deleted → `OK` (empty body)
/// * `rows_affected == 0` — no matching row → `NOT_FOUND`
fn remove_user_result(rows_affected: u64) -> PasmResult {
    if rows_affected > 0 {
        PasmResult::ServerStatus(StatusCode::OK, String::new())
    } else {
        PasmResult::ServerStatus(StatusCode::NOT_FOUND, "auth key not found".to_string())
    }
}

/// Classifies the result of an INSERT … ON CONFLICT DO NOTHING entry operation.
///
/// * `rows_affected > 0` — entry was inserted → `Ok(())`
/// * `rows_affected == 0` — entry already exists → `Err(CONFLICT)`
fn add_entry_result(rows_affected: u64) -> Result<(), PasmResult> {
    if rows_affected > 0 {
        Ok(())
    } else {
        Err(PasmResult::ServerStatus(
            StatusCode::CONFLICT,
            "entry already exists".to_string(),
        ))
    }
}

/// Classifies the result of a DELETE entry query by `rows_affected`.
///
/// * `rows_affected > 0` — entry was deleted → `Ok(())`
/// * `rows_affected == 0` — no matching row → `Err(NOT_FOUND)`
fn remove_entry_result(rows_affected: u64) -> Result<(), PasmResult> {
    if rows_affected > 0 {
        Ok(())
    } else {
        Err(PasmResult::ServerStatus(
            StatusCode::NOT_FOUND,
            "entry not found".to_string(),
        ))
    }
}

/// PostgreSQL-backed database.
#[derive(Clone)]
pub struct PgDb {
    pool: PgPool,
}

impl PgDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Db for PgDb {
    async fn ping(&self) -> Result<(), PasmResult> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })?;
        Ok(())
    }

    async fn get_user_id_by_authkey(&self, authkey: &str) -> Result<String, PasmResult> {
        let user_id: Option<String> =
            sqlx::query_scalar("SELECT id::text FROM users WHERE auth_key_hash = $1")
                .bind(authkey)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })?;

        match user_id {
            Some(id) => Ok(id),
            None => Err(PasmResult::ServerStatus(
                StatusCode::NOT_FOUND,
                "user_id not found".to_string(),
            )),
        }
    }

    async fn auth_key_exists(&self, authkey: &str) -> Result<bool, PasmResult> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE auth_key_hash = $1)")
            .bind(authkey)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })
    }

    async fn get_user_roles(&self, authkey: &str) -> Result<Option<Vec<String>>, PasmResult> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT array_to_string(roles, ',') FROM users WHERE auth_key_hash = $1",
        )
        .bind(authkey)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })?;

        match row {
            Some((roles_str,)) => Ok(Some(parse_roles_csv(&roles_str))),
            None => Ok(None),
        }
    }

    async fn set_admin_role(&self, target_auth_key: &str) -> Result<(), PasmResult> {
        let affected = sqlx::query(
            "UPDATE users SET roles = ARRAY['user','admin']::TEXT[] WHERE auth_key_hash = $1",
        )
        .bind(target_auth_key)
        .execute(&self.pool)
        .await
        .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })?;

        if affected.rows_affected() == 0 {
            return Err(PasmResult::ServerStatus(
                StatusCode::NOT_FOUND,
                "auth key not found".to_string(),
            ));
        }
        Ok(())
    }

    async fn register_auth(&self, auth_key: &str) -> PasmResult {
        let result = sqlx::query(
            "INSERT INTO users (auth_key_hash) VALUES ($1) ON CONFLICT (auth_key_hash) DO NOTHING",
        )
        .bind(auth_key)
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) => register_result(res.rows_affected()),
            Err(e) => PasmResult::DatabaseError { err: e.to_string() },
        }
    }

    async fn update_auth(&self, auth_key: &str, new_auth: &str) -> PasmResult {
        let user_id = match self.get_user_id_by_authkey(auth_key).await {
            Ok(id) => id,
            Err(e) => return e,
        };

        let new_taken = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE auth_key_hash = $1)",
        )
        .bind(new_auth)
        .fetch_one(&self.pool)
        .await;

        let new_taken = match new_taken {
            Ok(v) => v,
            Err(e) => return PasmResult::DatabaseError { err: e.to_string() },
        };

        if new_taken {
            return PasmResult::ServerStatus(
                StatusCode::CONFLICT,
                "new auth key already exists".to_string(),
            );
        }

        let result = sqlx::query("UPDATE users SET auth_key_hash = $2 WHERE id = $1::uuid")
            .bind(&user_id)
            .bind(new_auth)
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => PasmResult::ServerStatus(StatusCode::OK, String::new()),
            Err(e) => PasmResult::DatabaseError { err: e.to_string() },
        }
    }

    async fn remove_user(&self, auth_key: &str) -> PasmResult {
        let result = sqlx::query("DELETE FROM users WHERE auth_key_hash = $1")
            .bind(auth_key)
            .execute(&self.pool)
            .await;

        match result {
            Ok(res) => remove_user_result(res.rows_affected()),
            Err(e) => PasmResult::DatabaseError { err: e.to_string() },
        }
    }

    async fn list_users(&self) -> Result<Vec<String>, PasmResult> {
        let keys: Vec<String> =
            sqlx::query_scalar("SELECT auth_key_hash FROM users ORDER BY created_at")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })?;

        Ok(keys)
    }

    async fn add_entry(
        &self,
        user_id: &str,
        entry_name: &str,
        encrypted_data: &str,
    ) -> Result<(), PasmResult> {
        let result = sqlx::query(
            "INSERT INTO entries (user_id, entry_name, encrypted_value) \
             VALUES ($1::uuid, $2, $3) \
             ON CONFLICT (user_id, entry_name) DO NOTHING",
        )
        .bind(user_id)
        .bind(entry_name)
        .bind(encrypted_data)
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) => add_entry_result(res.rows_affected()),
            Err(e) => Err(PasmResult::DatabaseError { err: e.to_string() }),
        }
    }

    async fn get_entry(&self, user_id: &str, entry_name: &str) -> Result<String, PasmResult> {
        let value: Option<String> = sqlx::query_scalar(
            "SELECT encrypted_value FROM entries WHERE user_id = $1::uuid AND entry_name = $2",
        )
        .bind(user_id)
        .bind(entry_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })?;

        match value {
            Some(v) => Ok(v),
            None => Err(PasmResult::ServerStatus(
                StatusCode::NOT_FOUND,
                entry_name.to_string(),
            )),
        }
    }

    async fn remove_entry(&self, user_id: &str, entry_name: &str) -> Result<(), PasmResult> {
        let result =
            sqlx::query("DELETE FROM entries WHERE user_id = $1::uuid AND entry_name = $2")
                .bind(user_id)
                .bind(entry_name)
                .execute(&self.pool)
                .await
                .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })?;

        remove_entry_result(result.rows_affected())
    }

    async fn list_entries(&self, user_id: &str) -> Result<Vec<RequestData>, PasmResult> {
        let rows = sqlx::query(
            "SELECT entry_name, encrypted_value FROM entries \
             WHERE user_id = $1::uuid ORDER BY entry_name",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })?;

        let entries: Vec<RequestData> = rows
            .iter()
            .map(|row| RequestData {
                key: row.get(0),
                value: row.get(1),
            })
            .collect();

        Ok(entries)
    }

    async fn amend_entry(
        &self,
        user_id: &str,
        entry_name: &str,
        encrypted_data: &str,
    ) -> Result<(), PasmResult> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM entries WHERE user_id = $1::uuid AND entry_name = $2)",
        )
        .bind(user_id)
        .bind(entry_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })?;

        if exists {
            sqlx::query(
                "UPDATE entries SET encrypted_value = $3, updated_at = now() \
                 WHERE user_id = $1::uuid AND entry_name = $2",
            )
            .bind(user_id)
            .bind(entry_name)
            .bind(encrypted_data)
            .execute(&self.pool)
            .await
            .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })?;

            Ok(())
        } else {
            sqlx::query(
                "INSERT INTO entries (user_id, entry_name, encrypted_value) \
                 VALUES ($1::uuid, $2, $3)",
            )
            .bind(user_id)
            .bind(entry_name)
            .bind(encrypted_data)
            .execute(&self.pool)
            .await
            .map_err(|e| PasmResult::DatabaseError { err: e.to_string() })?;

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_roles_csv ──────────────────────────────────────────

    #[test]
    fn parse_roles_csv_empty() {
        assert!(parse_roles_csv("").is_empty());
    }

    #[test]
    fn parse_roles_csv_single() {
        assert_eq!(parse_roles_csv("user"), vec!["user"]);
    }

    #[test]
    fn parse_roles_csv_multiple() {
        assert_eq!(parse_roles_csv("user,admin"), vec!["user", "admin"]);
    }

    #[test]
    fn parse_roles_csv_trailing_comma() {
        assert_eq!(parse_roles_csv("user,admin,"), vec!["user", "admin", ""]);
    }

    // ── register_result ──────────────────────────────────────────

    #[test]
    fn register_result_inserted() {
        let res = register_result(1);
        assert!(matches!(res, PasmResult::ServerStatus(StatusCode::OK, _)));
        if let PasmResult::ServerStatus(_, msg) = &res {
            assert_eq!(msg, "registered new authentication token!");
        }
    }

    #[test]
    fn register_result_conflict() {
        let res = register_result(0);
        assert!(matches!(
            res,
            PasmResult::ServerStatus(StatusCode::CONFLICT, _)
        ));
        if let PasmResult::ServerStatus(_, msg) = &res {
            assert_eq!(msg, "auth key already exists");
        }
    }

    // ── remove_user_result ───────────────────────────────────────

    #[test]
    fn remove_user_result_deleted() {
        let res = remove_user_result(1);
        assert!(matches!(res, PasmResult::ServerStatus(StatusCode::OK, _)));
    }

    #[test]
    fn remove_user_result_not_found() {
        let res = remove_user_result(0);
        assert!(matches!(
            res,
            PasmResult::ServerStatus(StatusCode::NOT_FOUND, _)
        ));
        if let PasmResult::ServerStatus(_, msg) = &res {
            assert!(msg.contains("not found"));
        }
    }

    // ── add_entry_result ─────────────────────────────────────────

    #[test]
    fn add_entry_result_inserted() {
        assert!(add_entry_result(1).is_ok());
    }

    #[test]
    fn add_entry_result_conflict() {
        let res = add_entry_result(0);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(
            err,
            PasmResult::ServerStatus(StatusCode::CONFLICT, _)
        ));
    }

    // ── remove_entry_result ──────────────────────────────────────

    #[test]
    fn remove_entry_result_deleted() {
        assert!(remove_entry_result(1).is_ok());
    }

    #[test]
    fn remove_entry_result_not_found() {
        let res = remove_entry_result(0);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(
            err,
            PasmResult::ServerStatus(StatusCode::NOT_FOUND, _)
        ));
    }
}
