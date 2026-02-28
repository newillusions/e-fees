mod auth;
mod config;
mod error;
mod routes;

use std::sync::Arc;

use axum::{extract::State, middleware, response::Json, routing::get, Router};
use serde_json::{json, Value};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Database;
use surrealdb::Surreal;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use config::Config;

/// Shared application state available to all route handlers.
pub struct AppState {
    pub db: Surreal<surrealdb::engine::remote::ws::Client>,
}

#[tokio::main]
async fn main() {
    // Load .env file (silently ignore if missing)
    let _ = dotenvy::dotenv();

    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt::init();

    // Load configuration from environment
    let config = Config::from_env();
    let port = config.port;
    info!(
        "Starting e-fees-api on port {} (db: {})",
        port, config.surreal_url
    );

    // Connect to SurrealDB via WebSocket
    // The existing e-fees app strips the ws:// prefix before passing to Surreal::new
    let connection_address = config
        .surreal_url
        .strip_prefix("ws://")
        .or_else(|| config.surreal_url.strip_prefix("wss://"))
        .unwrap_or(&config.surreal_url);

    let db = Surreal::new::<Ws>(connection_address)
        .await
        .expect("Failed to connect to SurrealDB");

    // Authenticate with database-level credentials (matching existing e-fees pattern)
    db.signin(Database {
        namespace: config.surreal_ns.clone(),
        database: config.surreal_db.clone(),
        username: config.surreal_user.clone(),
        password: config.surreal_pass.clone(),
    })
    .await
    .expect("Failed to authenticate with SurrealDB");

    // Select namespace and database
    db.use_ns(&config.surreal_ns)
        .use_db(&config.surreal_db)
        .await
        .expect("Failed to select namespace/database");

    info!(
        "Connected to SurrealDB at {} (ns: {}, db: {})",
        config.surreal_url, config.surreal_ns, config.surreal_db
    );

    // Build shared state
    let state = Arc::new(AppState { db });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router — .with_state() converts Router<Arc<AppState>> to Router<()>
    let app = Router::new()
        .route("/health", get(health))
        .route("/stats", get(routes::stats::get_stats))
        .route("/projects", get(routes::projects::list_projects))
        .route("/projects/{id}", get(routes::projects::get_project))
        .route("/fees", get(routes::fees::list_fees))
        .route("/fees/{id}", get(routes::fees::get_fee))
        .route("/companies", get(routes::companies::list_companies))
        .route("/companies/{id}", get(routes::companies::get_company))
        .route("/contacts", get(routes::contacts::list_contacts))
        .layer(middleware::from_fn(auth::require_api_key))
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = format!("0.0.0.0:{}", port);
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

/// Health check endpoint.
///
/// Returns service status, name, and version.
/// Also verifies the SurrealDB connection is alive.
async fn health(State(state): State<Arc<AppState>>) -> Json<Value> {
    let db_ok = state.db.health().await.is_ok();

    Json(json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "service": "e-fees-api",
        "version": env!("CARGO_PKG_VERSION"),
        "database": if db_ok { "connected" } else { "disconnected" },
    }))
}
