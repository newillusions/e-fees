mod auth;
mod config;
mod error;
mod health;
mod llm;
mod models;
mod routes;
mod schemas;

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use axum::{middleware, response::Json, routing::get, routing::post, Router};
use serde_json::Value;
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
    pub stirling_url: String,
    pub http: reqwest::Client,
    pub started_at: Instant,
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "E-Fees Scope Service",
        description = "Scope/deliverables management, clause library, and proposal corpus",
        version = "0.2.0",
        contact(name = "Emittiv", url = "https://emittiv.com"),
    ),
    paths(
        health::health,
        health::help,
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
        routes::corpus::extract_clauses,
        routes::scope::generate_scope,
        routes::scope::get_scope,
        routes::scope::update_scope,
        routes::scope::regenerate_scope,
        routes::scope::export_scope,
        routes::stages::list_stages,
        routes::stages::update_stage,
        routes::deliverables::list_deliverables,
        routes::deliverables::get_deliverable,
        routes::deliverables::create_deliverable,
        routes::deliverables::update_deliverable,
        routes::deliverables::delete_deliverable,
        routes::deliverables::deliverable_analytics,
        routes::assembly::assemble_deliverables,
        routes::assembly::save_scope_builder,
        routes::assembly::get_scope_deliverables,
    ),
    tags(
        (name = "Health", description = "Service health"),
        (name = "Assembly", description = "Deliverable assembly and scope builder"),
        (name = "Clauses", description = "Clause library CRUD"),
        (name = "Corpus", description = "Proposal corpus / knowledge base"),
        (name = "Deliverables", description = "Deliverable library CRUD"),
        (name = "Scope", description = "Scope generation and assembly"),
        (name = "Stages", description = "Stage configuration"),
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

    let config = Config::load();
    let port = config.port;

    info!("Connecting to SurrealDB at {}", config.surreal_url);
    let connection_address = config
        .surreal_url
        .strip_prefix("ws://")
        .or_else(|| config.surreal_url.strip_prefix("wss://"))
        .unwrap_or(&config.surreal_url);

    let db = Surreal::new::<Ws>(connection_address)
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
        stirling_url: config.stirling_url,
        http: reqwest::Client::new(),
        started_at: Instant::now(),
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
        .route("/corpus/extract-clauses", post(routes::corpus::extract_clauses))
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
        .route(
            "/scope/assemble",
            post(routes::assembly::assemble_deliverables),
        )
        .route(
            "/scope/save",
            post(routes::assembly::save_scope_builder),
        )
        .route(
            "/scope/{fee_id}/deliverables",
            get(routes::assembly::get_scope_deliverables),
        )
        .route("/stages", get(routes::stages::list_stages))
        .route(
            "/stages/{canonical_name}",
            axum::routing::put(routes::stages::update_stage),
        )
        .route(
            "/deliverables",
            get(routes::deliverables::list_deliverables).post(routes::deliverables::create_deliverable),
        )
        .route(
            "/deliverables/analytics",
            get(routes::deliverables::deliverable_analytics),
        )
        .route(
            "/deliverables/{id}",
            get(routes::deliverables::get_deliverable)
                .put(routes::deliverables::update_deliverable)
                .delete(routes::deliverables::delete_deliverable),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ));

    let app = Router::new()
        .route("/health", get(health::health))
        .route("/api/health", get(health::health))
        .route("/help", get(health::help))
        .route("/openapi.json", get(openapi_json))
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

async fn openapi_json() -> (axum::http::StatusCode, Json<Value>) {
    let spec = ApiDoc::openapi();
    match serde_json::to_value(spec) {
        Ok(val) => (axum::http::StatusCode::OK, Json(val)),
        Err(e) => {
            tracing::error!("Failed to serialize OpenAPI spec: {e}");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to generate OpenAPI specification"})),
            )
        }
    }
}
