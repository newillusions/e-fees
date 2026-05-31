mod auth;
mod config;
mod error;
mod health;
mod pagination;
mod routes;
mod schemas;
mod ssh;
mod validation;

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use axum::{middleware, response::Json, routing::get, Router};
use serde_json::Value;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use config::Config;

/// Shared application state available to all route handlers.
pub struct AppState {
    pub db: Surreal<surrealdb::engine::remote::ws::Client>,
    pub api_keys: HashSet<String>,
    pub folder_config: Option<config::FolderConfig>,
    pub started_at: Instant,
}

/// OpenAPI documentation for the e-fees API.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "E-Fees API",
        description = "REST API for managing fee proposals, projects, companies, and contacts.",
        version = "0.3.4",
        contact(name = "Emittiv", url = "https://emittiv.com"),
    ),
    paths(
        health::health,
        health::help,
        routes::stats::get_stats,
        routes::projects::list_projects,
        routes::projects::get_next_number,
        routes::projects::get_project,
        routes::projects::create_project,
        routes::projects::update_project,
        routes::projects::delete_project,
        routes::fees::list_fees,
        routes::fees::get_fee,
        routes::fees::create_fee,
        routes::fees::update_fee,
        routes::fees::delete_fee,
        routes::fee_export::export_fee_json,
        routes::fee_export::fee_json_status,
        routes::fee_export::export_indesign,
        routes::fee_export::export_template,
        routes::companies::list_companies,
        routes::companies::get_company,
        routes::companies::create_company,
        routes::companies::update_company,
        routes::companies::delete_company,
        routes::contacts::list_contacts,
        routes::contacts::get_contact,
        routes::contacts::create_contact,
        routes::contacts::update_contact,
        routes::contacts::delete_contact,
        routes::folders::create_folder,
        routes::documents::upload_document,
    ),
    tags(
        (name = "Health", description = "Service health checks"),
        (name = "Statistics", description = "Dashboard statistics"),
        (name = "Projects", description = "Project management"),
        (name = "Fees", description = "Fee proposal management"),
        (name = "Companies", description = "Company management"),
        (name = "Contacts", description = "Contact management"),
    ),
    security(("api_key" = [])),
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

/// Adds API key security scheme to the OpenAPI spec.
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
    // Load .env file — try /data/.env (Docker WORKDIR) then current dir (local dev)
    if dotenvy::from_path("/data/.env").is_err() {
        let _ = dotenvy::dotenv();
    }

    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt::init();

    // Load configuration from config.yaml + environment
    let config = Config::load();
    let port = config.port;
    info!(
        "Starting e-fees-api on port {} (db: {})",
        port, config.surreal_url
    );

    // Connect to SurrealDB via WebSocket
    let connection_address = config
        .surreal_url
        .strip_prefix("ws://")
        .or_else(|| config.surreal_url.strip_prefix("wss://"))
        .unwrap_or(&config.surreal_url);

    let db = Surreal::new::<Ws>(connection_address)
        .await
        .expect("Failed to connect to SurrealDB");

    // Authenticate with root credentials (matching existing e-fees Tauri app pattern)
    db.signin(Root {
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

    if config.folder_config.is_some() {
        info!("Folder creation enabled (NC_SSH_HOST configured)");
    } else {
        info!("Folder creation disabled (NC_SSH_HOST not set)");
    }

    // Build shared state (API keys validated at startup via Config::from_env)
    let state = Arc::new(AppState {
        db,
        api_keys: config.api_keys.into_iter().collect(),
        folder_config: config.folder_config,
        started_at: Instant::now(),
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Protected routes require API key authentication
    let protected = Router::new()
        .route("/stats", get(routes::stats::get_stats))
        .route(
            "/projects",
            get(routes::projects::list_projects).post(routes::projects::create_project),
        )
        .route(
            "/projects/next-number",
            get(routes::projects::get_next_number),
        )
        .route(
            "/projects/{id}",
            get(routes::projects::get_project)
                .put(routes::projects::update_project)
                .patch(routes::projects::update_project)
                .delete(routes::projects::delete_project),
        )
        .route(
            "/projects/{id}/folder",
            axum::routing::post(routes::folders::create_folder),
        )
        .route(
            "/projects/{id}/documents",
            axum::routing::post(routes::documents::upload_document),
        )
        .route(
            "/fees",
            get(routes::fees::list_fees).post(routes::fees::create_fee),
        )
        .route(
            "/fees/{id}",
            get(routes::fees::get_fee)
                .put(routes::fees::update_fee)
                .delete(routes::fees::delete_fee),
        )
        .route(
            "/fees/{id}/json-export",
            axum::routing::post(routes::fee_export::export_fee_json),
        )
        .route(
            "/fees/{id}/json-status",
            get(routes::fee_export::fee_json_status),
        )
        .route(
            "/fees/{id}/export/indesign",
            axum::routing::post(routes::fee_export::export_indesign),
        )
        .route(
            "/fees/{id}/export/template",
            axum::routing::post(routes::fee_export::export_template),
        )
        .route(
            "/companies",
            get(routes::companies::list_companies).post(routes::companies::create_company),
        )
        .route(
            "/companies/{id}",
            get(routes::companies::get_company)
                .put(routes::companies::update_company)
                .delete(routes::companies::delete_company),
        )
        .route(
            "/contacts",
            get(routes::contacts::list_contacts).post(routes::contacts::create_contact),
        )
        .route(
            "/contacts/{id}",
            get(routes::contacts::get_contact)
                .put(routes::contacts::update_contact)
                .delete(routes::contacts::delete_contact),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ));

    // Build router — health + docs + help are public, all other routes require auth
    let app = Router::new()
        .route("/health", get(health::health))
        .route("/api/health", get(health::health))
        .route("/openapi.json", get(openapi_json))
        .route("/help", get(health::help))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(protected)
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = format!("{}:{}", config.host, port);
    info!("Listening on {} (docs at /docs)", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server error");
}

/// Serve the OpenAPI spec as JSON (no auth required).
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
