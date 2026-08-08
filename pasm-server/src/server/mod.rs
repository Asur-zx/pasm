use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

use crate::{
    server::api::{
        amend,
        auth::{register, remove, session, set_admin, update},
        backup, create, delete, find, health, list, users,
    },
    types::{db::PgDb, state::PasmState},
};

pub mod admin_auth;
pub mod api;
pub mod auth;
pub mod sql;

/// This function is the main entry point to server listener
/// It loads runtime variables, defines routes and starts listener and starts server
pub async fn run() {
    dotenv().ok();

    let database_url = pasm_core::utils::config::database_url();

    let pool = PgPoolOptions::new()
        .max_connections(pasm_core::utils::config::max_connections())
        .connect(&database_url)
        .await
        .expect("failed to connect to PostgreSQL");

    // Run idempotent schema migrations on startup
    sqlx::raw_sql(include_str!("sql/migrations/001_init.sql"))
        .execute(&pool)
        .await
        .expect("failed to run database migrations");

    sqlx::raw_sql(include_str!("sql/migrations/002_admin_role.sql"))
        .execute(&pool)
        .await
        .expect("failed to run 002_admin_role migration");

    if let Ok(admin_password) = std::env::var("PASM_BOOTSTRAP_ADMIN_PASSWORD") {
        let admin_api_key = pasm_core::utils::crypto::derive_api_key(&admin_password);
        sqlx::query(
            "INSERT INTO users (auth_key_hash, roles) VALUES ($1, $2) ON CONFLICT (auth_key_hash) DO NOTHING",
        )
        .bind(&admin_api_key)
        .bind(vec!["user".to_string(), "admin".to_string()])
        .execute(&pool)
        .await
        .ok();
    }

    let redis_url = pasm_core::utils::config::redis_url();
    let redis_client = redis::Client::open(redis_url).expect("failed to connect to Redis");
    let redis_conn = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("failed to create Redis connection manager");

    let state = PasmState {
        db: PgDb::new(pool),
        redis: redis_conn,
        started_at: std::time::Instant::now(),
    };

    // Routes requiring only a valid auth key
    let protected_routes = Router::new()
        .route("/entries", get(list::call))
        .route("/entry", post(create::call))
        .route("/entry/amend", post(amend::call))
        .route("/entry/{name}", delete(delete::call).get(find::call))
        .route("/backup", get(backup::call))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state.clone(), auth::call));

    // Routes requiring admin role (auth middleware runs first, then admin)
    let admin_routes = Router::new()
        .route("/auth/update", post(update::call))
        .route("/auth/remove", delete(remove::call))
        .route("/auth/list", get(users::call))
        .route("/auth/set-admin", post(set_admin::call))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state.clone(), auth::call))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            admin_auth::call,
        ));

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/auth", post(register::call))
        .route("/auth/session", post(session::create))
        .route("/auth/refresh", post(session::refresh))
        .route("/health", get(health::call))
        .with_state(state.clone());

    let app = public_routes.merge(protected_routes).merge(admin_routes);
    let bind_addr = pasm_core::utils::config::server_addr();
    let listener = TcpListener::bind(&bind_addr).await.unwrap();
    println!("Server listening on {bind_addr}");
    if std::env::var("PASM_BOOTSTRAP_ADMIN_PASSWORD").is_ok() {
        println!("Bootstrap admin user ensured from PASM_BOOTSTRAP_ADMIN_PASSWORD.");
    }
    axum::serve(listener, app).await.unwrap();
}
