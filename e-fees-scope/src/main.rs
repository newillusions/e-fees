mod auth;
mod config;
mod error;
mod llm;
mod models;
mod routes;
mod schemas;

use std::collections::HashSet;
use std::sync::Arc;

use axum::{extract::State, middleware, response::Json, routing::{get, post}, Router};
use serde_json::{json, Value};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use config::Config;

pub struct AppState {
    pub db: Surreal<Client>,
    pub api_keys: HashSet<String>,
    pub ollama_url: String,
    pub ollama_model: String,
    pub docling_url: String,
    pub http: reqwest::Client,
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "E-Fees Scope Service",
        description = "Scope/deliverables management, clause library, and proposal corpus",
        version = "0.1.0",
        contact(name = "Emittiv", url = "https://emittiv.com"),
    ),
    paths(
        health,
        routes::clauses::list_clauses,
        routes::clauses::get_clause,
        routes::clauses::create_clause,
        routes::clauses::update_clause,
        routes::clauses::delete_clause,
        routes::clauses::list_categories,
        routes::corpus::list_corpus,
        routes::corpus::ingest,
        routes::corpus::ingest_batch,
        routes::corpus::search_corpus,
        routes::corpus::get_corpus_doc,
        routes::scope::generate_scope,
        routes::scope::get_scope,
        routes::scope::update_scope,
        routes::scope::regenerate_scope,
        routes::scope::export_scope,
    ),
    tags(
        (name = "Health", description = "Service health"),
        (name = "Clauses", description = "Clause library CRUD"),
        (name = "Corpus", description = "Proposal corpus / knowledge base"),
        (name = "Scope", description = "Scope generation and assembly"),
    ),
    security(("api_key" = [])),
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("X-API-Key"),
                    ),
                ),
            );
        }
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let config = Config::from_env();
    let port = config.port;

    info!("Connecting to SurrealDB at {}", config.surreal_url);
    let db = Surreal::new::<Ws>(&config.surreal_url)
        .await
        .expect("Failed to connect to SurrealDB");

    db.signin(Root {
        username: config.surreal_user.clone(),
        password: config.surreal_pass.clone(),
    })
    .await
    .expect("Failed to authenticate with SurrealDB");

    db.use_ns(&config.surreal_ns)
        .use_db(&config.surreal_db)
        .await
        .expect("Failed to select namespace/database");

    info!(
        "Connected to SurrealDB {}/{}",
        config.surreal_ns, config.surreal_db
    );

    let state = Arc::new(AppState {
        db,
        api_keys: config.api_keys.into_iter().collect(),
        ollama_url: config.ollama_url,
        ollama_model: config.ollama_model,
        docling_url: config.docling_url,
        http: reqwest::Client::new(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let protected = Router::new()
        .route(
            "/clauses",
            get(routes::clauses::list_clauses).post(routes::clauses::create_clause),
        )
        .route(
            "/clauses/categories",
            get(routes::clauses::list_categories),
        )
        .route(
            "/clauses/{id}",
            get(routes::clauses::get_clause)
                .put(routes::clauses::update_clause)
                .delete(routes::clauses::delete_clause),
        )
        .route(
            "/corpus",
            get(routes::corpus::list_corpus).post(routes::corpus::ingest),
        )
        .route("/corpus/ingest-batch", post(routes::corpus::ingest_batch))
        .route("/corpus/search", get(routes::corpus::search_corpus))
        .route("/corpus/{id}", get(routes::corpus::get_corpus_doc))
        .route("/scope/generate", post(routes::scope::generate_scope))
        .route(
            "/scope/{fee_id}",
            get(routes::scope::get_scope).put(routes::scope::update_scope),
        )
        .route(
            "/scope/{fee_id}/regenerate",
            post(routes::scope::regenerate_scope),
        )
        .route(
            "/scope/{fee_id}/export",
            get(routes::scope::export_scope),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ));

    let app = Router::new()
        .route("/health", get(health))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(protected)
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("E-Fees Scope Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service health", body = schemas::HealthResponse),
    )
)]
async fn health(State(state): State<Arc<AppState>>) -> Json<Value> {
    let db_ok = state.db.health().await.is_ok();

    let ollama_ok = state
        .http
        .get(format!("{}/api/tags", state.ollama_url))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    let status = if db_ok && ollama_ok {
        "ok"
    } else if db_ok {
        "degraded"
    } else {
        "unhealthy"
    };

    Json(json!({
        "status": status,
        "service": "e-fees-scope",
        "version": env!("CARGO_PKG_VERSION"),
        "database": if db_ok { "connected" } else { "disconnected" },
        "ollama": if ollama_ok { "connected" } else { "disconnected" },
    }))
}
