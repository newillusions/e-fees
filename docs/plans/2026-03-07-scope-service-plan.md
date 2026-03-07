# E-Fees Scope Service Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a standalone `e-fees-scope` microservice that manages a clause library, assembles numbered scope documents, ingests historical PDFs into a corpus, and uses LLM (Ollama) for contextual scope text polishing.

**Architecture:** Rust/axum REST API on port 3201, SurrealDB for storage, Ollama for LLM, Docling-Serve for PDF extraction. Replicates e-fees-api patterns (auth, error handling, config, pagination). Lives as a workspace member alongside e-fees-api.

**Tech Stack:** Rust, axum 0.8, SurrealDB 3.0, utoipa 5, reqwest (for Ollama/Docling HTTP calls), tokio

**Design doc:** `docs/plans/2026-03-07-scope-service-design.md`

---

## Task 1: Scaffold the Crate

**Files:**
- Create: `e-fees-scope/Cargo.toml`
- Create: `e-fees-scope/src/main.rs`
- Create: `e-fees-scope/src/auth.rs`
- Create: `e-fees-scope/src/config.rs`
- Create: `e-fees-scope/src/error.rs`
- Create: `e-fees-scope/src/schemas.rs`
- Create: `e-fees-scope/src/routes/mod.rs`
- Modify: `Cargo.toml` (root workspace)

**Step 1: Add e-fees-scope to workspace**

In root `Cargo.toml`, add `"e-fees-scope"` to the `members` array:

```toml
[workspace]
members = [
  "src-tauri",
  "crates/e-fees-core",
  "e-fees-api",
  "e-fees-scope",
]
resolver = "2"
```

**Step 2: Create `e-fees-scope/Cargo.toml`**

```toml
[package]
name = "e-fees-scope"
version = "0.1.0"
edition = "2021"
description = "Scope/deliverables service for e-fees"

[dependencies]
e-fees-core = { path = "../crates/e-fees-core", features = ["openapi"] }
axum = "0.8"
tokio = { version = "1", features = ["full"] }
surrealdb = { version = "3.0", features = ["protocol-ws", "rustls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower-http = { version = "0.6", features = ["cors"] }
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = "0.4"
utoipa = "5"
utoipa-swagger-ui = { version = "9", features = ["axum"] }
reqwest = { version = "0.12", features = ["json"] }
uuid = { version = "1", features = ["v4"] }

[dev-dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

Note: `reqwest` in main deps is for Ollama/Docling HTTP calls. `uuid` is for corpus record IDs.

**Step 3: Create `e-fees-scope/src/auth.rs`**

Copy verbatim from `e-fees-api/src/auth.rs` — identical API key middleware:

```rust
use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::AppState;

pub async fn require_api_key(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match request.headers().get("X-API-Key") {
        Some(key) if state.api_keys.contains(key.to_str().unwrap_or("")) => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
```

**Step 4: Create `e-fees-scope/src/config.rs`**

```rust
use std::env;

pub struct Config {
    pub surreal_url: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    pub api_keys: Vec<String>,
    pub port: u16,
    pub ollama_url: String,
    pub ollama_model: String,
    pub docling_url: String,
    pub corpus_path: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let raw_keys = env::var("API_KEY").expect("API_KEY required");
        let api_keys: Vec<String> = raw_keys
            .split(',')
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();
        assert!(!api_keys.is_empty(), "API_KEY must contain at least one non-empty key");

        Self {
            surreal_url: env::var("SURREAL_URL").expect("SURREAL_URL required"),
            surreal_ns: env::var("SURREAL_NS").unwrap_or_else(|_| "emittiv".into()),
            surreal_db: env::var("SURREAL_DB").unwrap_or_else(|_| "projects".into()),
            surreal_user: env::var("SURREAL_USER").expect("SURREAL_USER required"),
            surreal_pass: env::var("SURREAL_PASS").expect("SURREAL_PASS required"),
            api_keys,
            port: env::var("API_PORT")
                .unwrap_or_else(|_| "3201".into())
                .parse()
                .expect("Invalid API_PORT"),
            ollama_url: env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://10.0.21.20:11434".into()),
            ollama_model: env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "qwen3:4b".into()),
            docling_url: env::var("DOCLING_URL")
                .unwrap_or_else(|_| "http://10.0.21.42:5001".into()),
            corpus_path: env::var("CORPUS_PATH").ok(),
        }
    }
}
```

**Step 5: Create `e-fees-scope/src/error.rs`**

```rust
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub struct ApiError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn not_found(entity: &str, id: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found".into(),
            message: format!("{} '{}' not found", entity, id),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request".into(),
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error".into(),
            message: message.into(),
        }
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "service_unavailable".into(),
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(json!({
                "error": self.code,
                "message": self.message
            })),
        )
            .into_response()
    }
}

impl From<surrealdb::Error> for ApiError {
    fn from(e: surrealdb::Error) -> Self {
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "database_error".into(),
            message: e.to_string(),
        }
    }
}
```

**Step 6: Create `e-fees-scope/src/schemas.rs`**

```rust
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub database: String,
    pub ollama: String,
}

#[derive(Serialize, ToSchema)]
pub struct ClauseResponse {
    pub id: String,
    pub category: String,
    pub subcategory: Option<String>,
    pub title: String,
    pub body: String,
    pub sort_order: i64,
    pub is_default: bool,
    pub status: String,
    pub version: i64,
}

#[derive(Serialize, ToSchema)]
pub struct CategoryCount {
    pub category: String,
    pub count: u64,
}

#[derive(Serialize, ToSchema)]
pub struct ScopeAssemblyResponse {
    pub fee_id: String,
    pub generated_text: String,
    pub llm_polished: bool,
    pub clause_count: u64,
}

#[derive(Serialize, ToSchema)]
pub struct CorpusDocResponse {
    pub id: String,
    pub filename: String,
    pub project_number: Option<String>,
    pub project_name: Option<String>,
    pub section_count: u64,
}
```

**Step 7: Create `e-fees-scope/src/routes/mod.rs`**

```rust
pub mod clauses;
```

**Step 8: Create minimal `e-fees-scope/src/main.rs`**

```rust
mod auth;
mod config;
mod error;
mod routes;
mod schemas;

use std::collections::HashSet;
use std::sync::Arc;

use axum::{extract::State, middleware, response::Json, routing::get, Router};
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
    ),
    tags(
        (name = "Health", description = "Service health"),
        (name = "Clauses", description = "Clause library CRUD"),
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
        username: &config.surreal_user,
        password: &config.surreal_pass,
    })
    .await
    .expect("Failed to authenticate with SurrealDB");

    db.use_ns(&config.surreal_ns)
        .use_db(&config.surreal_db)
        .await
        .expect("Failed to select namespace/database");

    info!("Connected to SurrealDB {}/{}", config.surreal_ns, config.surreal_db);

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
        .route("/clauses", get(routes::clauses::list_clauses).post(routes::clauses::create_clause))
        .route("/clauses/categories", get(routes::clauses::list_categories))
        .route(
            "/clauses/{id}",
            get(routes::clauses::get_clause)
                .put(routes::clauses::update_clause)
                .delete(routes::clauses::delete_clause),
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
```

**Step 9: Create stub `e-fees-scope/src/routes/clauses.rs`**

Create with empty handler stubs so the crate compiles:

```rust
use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::AppState;

#[utoipa::path(get, path = "/clauses", tag = "Clauses",
    responses((status = 200, description = "List clauses")),
    security(("api_key" = [])))]
pub async fn list_clauses(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "data": [], "total": 0 })))
}

#[utoipa::path(get, path = "/clauses/{id}", tag = "Clauses",
    params(("id" = String, Path, description = "Clause ID")),
    responses((status = 200, description = "Get clause")),
    security(("api_key" = [])))]
pub async fn get_clause(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Err(ApiError::not_found("Clause", "stub"))
}

#[utoipa::path(post, path = "/clauses", tag = "Clauses",
    responses((status = 201, description = "Create clause")),
    security(("api_key" = [])))]
pub async fn create_clause(
    State(_state): State<Arc<AppState>>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "data": {} })))
}

#[utoipa::path(put, path = "/clauses/{id}", tag = "Clauses",
    params(("id" = String, Path, description = "Clause ID")),
    responses((status = 200, description = "Update clause")),
    security(("api_key" = [])))]
pub async fn update_clause(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "data": {} })))
}

#[utoipa::path(delete, path = "/clauses/{id}", tag = "Clauses",
    params(("id" = String, Path, description = "Clause ID")),
    responses((status = 200, description = "Archive clause")),
    security(("api_key" = [])))]
pub async fn delete_clause(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "deleted": true })))
}

#[utoipa::path(get, path = "/clauses/categories", tag = "Clauses",
    responses((status = 200, description = "List categories")),
    security(("api_key" = [])))]
pub async fn list_categories(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "data": [] })))
}
```

**Step 10: Verify it compiles**

Run: `cargo check -p e-fees-scope`
Expected: Compiles with no errors (warnings OK)

**Step 11: Commit**

```bash
git add e-fees-scope/ Cargo.toml
git commit -m "feat(scope): scaffold e-fees-scope crate with stubs"
```

---

## Task 2: SurrealDB Schema Definition

**Files:**
- Create: `e-fees-scope/schema.surql`

**Step 1: Write the schema DDL file**

```surql
-- E-Fees Scope Service Schema
-- Run against SurrealDB: ns=emittiv, db=projects

-- ============================================================
-- Clause library
-- ============================================================
DEFINE TABLE clause SCHEMAFULL;

DEFINE FIELD category    ON clause TYPE string;
DEFINE FIELD subcategory ON clause TYPE option<string>;
DEFINE FIELD title       ON clause TYPE string;
DEFINE FIELD body        ON clause TYPE string;
DEFINE FIELD conditions  ON clause TYPE option<object>;
DEFINE FIELD sort_order  ON clause TYPE int;
DEFINE FIELD tags        ON clause TYPE option<array<string>>;
DEFINE FIELD is_default  ON clause TYPE bool DEFAULT true;
DEFINE FIELD status      ON clause TYPE string DEFAULT "active";
DEFINE FIELD version     ON clause TYPE int DEFAULT 1;
DEFINE FIELD created_at  ON clause TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at  ON clause TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_clause_category ON clause FIELDS category;
DEFINE INDEX idx_clause_status   ON clause FIELDS status;

-- ============================================================
-- Scope assembly (generated scope per fee)
-- ============================================================
DEFINE TABLE scope_assembly SCHEMAFULL;

DEFINE FIELD fee_id         ON scope_assembly TYPE record<fee>;
DEFINE FIELD clauses        ON scope_assembly TYPE array;
DEFINE FIELD generated_text ON scope_assembly TYPE string;
DEFINE FIELD numbering      ON scope_assembly TYPE option<object>;
DEFINE FIELD llm_model      ON scope_assembly TYPE option<string>;
DEFINE FIELD llm_polished   ON scope_assembly TYPE bool DEFAULT false;
DEFINE FIELD created_at     ON scope_assembly TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at     ON scope_assembly TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_scope_fee ON scope_assembly FIELDS fee_id UNIQUE;

-- ============================================================
-- Proposal corpus (historical PDFs for RAG)
-- ============================================================
DEFINE TABLE proposal_corpus SCHEMAFULL;

DEFINE FIELD filename       ON proposal_corpus TYPE string;
DEFINE FIELD project_number ON proposal_corpus TYPE option<string>;
DEFINE FIELD project_name   ON proposal_corpus TYPE option<string>;
DEFINE FIELD extracted_text ON proposal_corpus TYPE string;
DEFINE FIELD sections       ON proposal_corpus TYPE option<array>;
DEFINE FIELD metadata       ON proposal_corpus TYPE option<object>;
DEFINE FIELD embedding      ON proposal_corpus TYPE option<array<float>>;
DEFINE FIELD created_at     ON proposal_corpus TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_corpus_project ON proposal_corpus FIELDS project_number;
```

**Step 2: Apply schema to SurrealDB**

Run against prod (10.0.23.11):
```bash
curl -s -X POST "http://10.0.23.11:8000/sql" \
  -u "root:$(grep SURREAL_PASS /Volumes/base/dev/app/e-fees/e-fees-api/.env | cut -d= -f2)" \
  -H "surreal-ns: emittiv" \
  -H "surreal-db: projects" \
  -H "Accept: application/json" \
  -H "Content-Type: text/plain" \
  --data-binary @e-fees-scope/schema.surql
```

Expected: All DEFINE statements succeed (no errors).

**Step 3: Verify tables exist**

```bash
curl -s -X POST "http://10.0.23.11:8000/sql" \
  -u "root:..." \
  -H "surreal-ns: emittiv" -H "surreal-db: projects" \
  -H "Accept: application/json" -H "Content-Type: text/plain" \
  -d "INFO FOR DB;"
```

Expected: `clause`, `scope_assembly`, `proposal_corpus` tables listed.

**Step 4: Commit**

```bash
git add e-fees-scope/schema.surql
git commit -m "feat(scope): add SurrealDB schema for clause, scope_assembly, proposal_corpus"
```

---

## Task 3: Clause CRUD — Full Implementation

**Files:**
- Create: `e-fees-scope/src/models.rs`
- Modify: `e-fees-scope/src/routes/clauses.rs`
- Modify: `e-fees-scope/src/main.rs` (add `mod models`)
- Create: `e-fees-scope/tests/clause_tests.rs`

**Step 1: Write clause integration tests**

Create `e-fees-scope/tests/clause_tests.rs`:

```rust
//! Integration tests for clause CRUD endpoints.
//! Run: API_BASE_URL=http://localhost:3201 API_KEY=test-key cargo test -p e-fees-scope --test clause_tests -- --test-threads=1

use reqwest::Client;
use serde_json::{json, Value};

fn base_url() -> String {
    std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:3201".into())
}

fn api_key() -> String {
    std::env::var("API_KEY").unwrap_or_else(|_| "test-key".into())
}

fn client() -> Client {
    Client::new()
}

#[tokio::test]
async fn test_health() {
    let res = client()
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["service"], "e-fees-scope");
    assert!(body["database"].as_str().is_some());
}

#[tokio::test]
async fn test_clauses_require_auth() {
    let res = client()
        .get(format!("{}/clauses", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn test_create_clause() {
    let res = client()
        .post(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .json(&json!({
            "category": "DELETE ME - Test Category",
            "title": "DELETE ME - Test Clause",
            "body": "Test clause body for {{project_name}}.",
            "sort_order": 1,
            "is_default": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["data"]["id"].as_str().is_some());
    assert_eq!(body["data"]["category"], "DELETE ME - Test Category");
    assert_eq!(body["data"]["version"], 1);

    // Cleanup
    let id = body["data"]["id"].as_str().unwrap();
    let key = id.split(':').nth(1).unwrap_or(id);
    client()
        .delete(format!("{}/clauses/{}", base_url(), key))
        .header("X-API-Key", api_key())
        .send()
        .await
        .unwrap();
}

#[tokio::test]
async fn test_create_and_get_clause() {
    // Create
    let res = client()
        .post(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .json(&json!({
            "category": "DELETE ME - Get Test",
            "title": "DELETE ME - Get Clause",
            "body": "Body text.",
            "sort_order": 1,
            "is_default": false,
            "subcategory": "Subsection A",
            "tags": ["test", "delete-me"],
            "conditions": { "regions": ["UAE"] }
        }))
        .send()
        .await
        .unwrap();
    let created: Value = res.json().await.unwrap();
    let id = created["data"]["id"].as_str().unwrap();
    let key = id.split(':').nth(1).unwrap_or(id);

    // Get
    let res = client()
        .get(format!("{}/clauses/{}", base_url(), key))
        .header("X-API-Key", api_key())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["data"]["subcategory"], "Subsection A");
    assert_eq!(body["data"]["tags"][0], "test");
    assert_eq!(body["data"]["conditions"]["regions"][0], "UAE");

    // Cleanup
    client()
        .delete(format!("{}/clauses/{}", base_url(), key))
        .header("X-API-Key", api_key())
        .send()
        .await
        .unwrap();
}

#[tokio::test]
async fn test_update_clause_increments_version() {
    // Create
    let res = client()
        .post(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .json(&json!({
            "category": "DELETE ME - Update Test",
            "title": "DELETE ME - Update Clause",
            "body": "Original body.",
            "sort_order": 1
        }))
        .send()
        .await
        .unwrap();
    let created: Value = res.json().await.unwrap();
    let id = created["data"]["id"].as_str().unwrap();
    let key = id.split(':').nth(1).unwrap_or(id);
    assert_eq!(created["data"]["version"], 1);

    // Update
    let res = client()
        .put(format!("{}/clauses/{}", base_url(), key))
        .header("X-API-Key", api_key())
        .json(&json!({
            "body": "Updated body.",
            "title": "DELETE ME - Updated Title"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["data"]["version"], 2);
    assert_eq!(updated["data"]["body"], "Updated body.");

    // Cleanup
    client()
        .delete(format!("{}/clauses/{}", base_url(), key))
        .header("X-API-Key", api_key())
        .send()
        .await
        .unwrap();
}

#[tokio::test]
async fn test_delete_clause_archives() {
    // Create
    let res = client()
        .post(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .json(&json!({
            "category": "DELETE ME - Archive Test",
            "title": "DELETE ME - Archive Clause",
            "body": "Will be archived.",
            "sort_order": 1
        }))
        .send()
        .await
        .unwrap();
    let created: Value = res.json().await.unwrap();
    let id = created["data"]["id"].as_str().unwrap();
    let key = id.split(':').nth(1).unwrap_or(id);

    // Delete (soft)
    let res = client()
        .delete(format!("{}/clauses/{}", base_url(), key))
        .header("X-API-Key", api_key())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["archived"], true);

    // Verify archived — GET should still work but show status=archived
    let res = client()
        .get(format!("{}/clauses/{}", base_url(), key))
        .header("X-API-Key", api_key())
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["data"]["status"], "archived");

    // Hard cleanup (direct DB delete via API won't be available, but the test data has DELETE ME prefix)
}

#[tokio::test]
async fn test_list_clauses_with_category_filter() {
    // Create two clauses in different categories
    let cat = format!("DELETE ME - Filter Test {}", chrono::Utc::now().timestamp_millis());

    client()
        .post(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .json(&json!({
            "category": &cat,
            "title": "DELETE ME - Filter A",
            "body": "A",
            "sort_order": 1
        }))
        .send()
        .await
        .unwrap();

    client()
        .post(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .json(&json!({
            "category": "DELETE ME - Other Category",
            "title": "DELETE ME - Filter B",
            "body": "B",
            "sort_order": 1
        }))
        .send()
        .await
        .unwrap();

    // Filter by category
    let res = client()
        .get(format!("{}/clauses?category={}", base_url(), urlencoding::encode(&cat)))
        .header("X-API-Key", api_key())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 1);
    for item in data {
        assert_eq!(item["category"].as_str().unwrap(), cat);
    }
}

#[tokio::test]
async fn test_list_categories() {
    let res = client()
        .get(format!("{}/clauses/categories", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["data"].is_array());
}
```

Note: Add `urlencoding = "2"` to `[dev-dependencies]` in Cargo.toml.

**Step 2: Run tests to verify they fail**

Run: `cargo test -p e-fees-scope --test clause_tests -- --test-threads=1 2>&1 | head -30`
Expected: Tests fail (stubs return wrong data, or service not running yet)

**Step 3: Create `e-fees-scope/src/models.rs`**

```rust
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};
use surrealdb_types::Datetime;

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Clause {
    pub id: RecordId,
    pub category: String,
    pub subcategory: Option<String>,
    pub title: String,
    pub body: String,
    pub conditions: Option<surrealdb_types::Value>,
    pub sort_order: i64,
    pub tags: Option<Vec<String>>,
    pub is_default: bool,
    pub status: String,
    pub version: i64,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Deserialize)]
pub struct NewClause {
    pub category: String,
    pub subcategory: Option<String>,
    pub title: String,
    pub body: String,
    pub conditions: Option<serde_json::Value>,
    pub sort_order: i64,
    pub tags: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub is_default: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateClause {
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub sort_order: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub is_default: Option<bool>,
}
```

**Step 4: Implement full clause CRUD in `e-fees-scope/src/routes/clauses.rs`**

Replace the stubs with full implementations. Each handler follows the e-fees-api pattern:
- Validate input
- Parameterized SurrealQL query
- Convert result to JSON
- Return `Result<Json<Value>, ApiError>`

```rust
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use e_fees_core::models::{dbvalue_to_json, record_id_string, record_key_string};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::warn;

use crate::error::ApiError;
use crate::models::{Clause, NewClause, UpdateClause};
use crate::AppState;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ClauseListParams {
    pub category: Option<String>,
    pub status: Option<String>,
    pub tag: Option<String>,
}

fn clause_to_json(c: &Clause) -> Value {
    json!({
        "id": record_id_string(&c.id),
        "category": c.category,
        "subcategory": c.subcategory,
        "title": c.title,
        "body": c.body,
        "conditions": c.conditions.as_ref().map(dbvalue_to_json),
        "sort_order": c.sort_order,
        "tags": c.tags,
        "is_default": c.is_default,
        "status": c.status,
        "version": c.version,
        "created_at": c.created_at.to_string(),
        "updated_at": c.updated_at.to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/clauses",
    tag = "Clauses",
    params(ClauseListParams),
    responses(
        (status = 200, description = "List of clauses"),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_clauses(
    State(state): State<Arc<AppState>>,
    params: Query<ClauseListParams>,
) -> Result<Json<Value>, ApiError> {
    let mut where_parts: Vec<String> = vec![];
    let mut binds: Vec<(String, Value)> = vec![];

    if let Some(ref cat) = params.category {
        where_parts.push("category = $filter_cat".into());
        binds.push(("filter_cat".into(), json!(cat)));
    }
    if let Some(ref status) = params.status {
        where_parts.push("status = $filter_status".into());
        binds.push(("filter_status".into(), json!(status)));
    } else {
        // Default: only active
        where_parts.push("status = $filter_status".into());
        binds.push(("filter_status".into(), json!("active")));
    }
    if let Some(ref tag) = params.tag {
        where_parts.push("tags CONTAINS $filter_tag".into());
        binds.push(("filter_tag".into(), json!(tag)));
    }

    let where_clause = if where_parts.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_parts.join(" AND "))
    };

    let query = format!(
        "SELECT * FROM clause{} ORDER BY sort_order ASC, category ASC",
        where_clause
    );

    let mut builder = state.db.query(&query);
    for (name, value) in &binds {
        builder = builder.bind((name.clone(), value.clone()));
    }

    let mut response = builder.await?;
    let clauses: Vec<Clause> = response.take(0)?;
    let data: Vec<Value> = clauses.iter().map(clause_to_json).collect();

    Ok(Json(json!({
        "data": data,
        "total": data.len()
    })))
}

#[utoipa::path(
    get,
    path = "/clauses/{id}",
    tag = "Clauses",
    params(("id" = String, Path, description = "Clause record key")),
    responses(
        (status = 200, description = "Single clause", body = crate::schemas::ClauseResponse),
        (status = 404, description = "Not found", body = crate::schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn get_clause(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let clause: Option<Clause> = state.db.select(("clause", &*id)).await?;
    match clause {
        Some(c) => Ok(Json(json!({ "data": clause_to_json(&c) }))),
        None => Err(ApiError::not_found("Clause", &id)),
    }
}

#[utoipa::path(
    post,
    path = "/clauses",
    tag = "Clauses",
    responses(
        (status = 200, description = "Created clause", body = crate::schemas::ClauseResponse),
        (status = 400, description = "Validation error", body = crate::schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn create_clause(
    State(state): State<Arc<AppState>>,
    Json(body): Json<NewClause>,
) -> Result<Json<Value>, ApiError> {
    if body.category.trim().is_empty() {
        return Err(ApiError::bad_request("'category' must not be empty"));
    }
    if body.title.trim().is_empty() {
        return Err(ApiError::bad_request("'title' must not be empty"));
    }
    if body.body.trim().is_empty() {
        return Err(ApiError::bad_request("'body' must not be empty"));
    }

    let query = "CREATE clause SET \
        category = $category, \
        subcategory = $subcategory, \
        title = $title, \
        body = $body, \
        conditions = $conditions, \
        sort_order = $sort_order, \
        tags = $tags, \
        is_default = $is_default, \
        status = 'active', \
        version = 1, \
        created_at = time::now(), \
        updated_at = time::now()";

    let conditions_db = body.conditions.as_ref()
        .map(|c| e_fees_core::models::json_to_dbvalue(c))
        .unwrap_or(surrealdb_types::Value::None);

    let mut response = state
        .db
        .query(query)
        .bind(("category", &body.category))
        .bind(("subcategory", &body.subcategory))
        .bind(("title", &body.title))
        .bind(("body", &body.body))
        .bind(("conditions", conditions_db))
        .bind(("sort_order", body.sort_order))
        .bind(("tags", &body.tags))
        .bind(("is_default", body.is_default))
        .await?;

    let created: Option<Clause> = response.take(0)?;
    match created {
        Some(c) => Ok(Json(json!({ "data": clause_to_json(&c) }))),
        None => Err(ApiError::bad_request("Failed to create clause")),
    }
}

#[utoipa::path(
    put,
    path = "/clauses/{id}",
    tag = "Clauses",
    params(("id" = String, Path, description = "Clause record key")),
    responses(
        (status = 200, description = "Updated clause", body = crate::schemas::ClauseResponse),
        (status = 404, description = "Not found", body = crate::schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn update_clause(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateClause>,
) -> Result<Json<Value>, ApiError> {
    // Build SET clauses dynamically for provided fields
    let mut set_parts: Vec<String> = vec![];
    let mut binds: Vec<(String, Value)> = vec![];

    if let Some(ref cat) = body.category {
        set_parts.push("category = $u_category".into());
        binds.push(("u_category".into(), json!(cat)));
    }
    if let Some(ref sub) = body.subcategory {
        set_parts.push("subcategory = $u_subcategory".into());
        binds.push(("u_subcategory".into(), json!(sub)));
    }
    if let Some(ref title) = body.title {
        set_parts.push("title = $u_title".into());
        binds.push(("u_title".into(), json!(title)));
    }
    if let Some(ref b) = body.body {
        set_parts.push("body = $u_body".into());
        binds.push(("u_body".into(), json!(b)));
    }
    if let Some(ref cond) = body.conditions {
        set_parts.push("conditions = $u_conditions".into());
        binds.push(("u_conditions".into(), cond.clone()));
    }
    if let Some(order) = body.sort_order {
        set_parts.push("sort_order = $u_sort_order".into());
        binds.push(("u_sort_order".into(), json!(order)));
    }
    if let Some(ref tags) = body.tags {
        set_parts.push("tags = $u_tags".into());
        binds.push(("u_tags".into(), json!(tags)));
    }
    if let Some(def) = body.is_default {
        set_parts.push("is_default = $u_is_default".into());
        binds.push(("u_is_default".into(), json!(def)));
    }

    if set_parts.is_empty() {
        return Err(ApiError::bad_request("No fields to update"));
    }

    // Always bump version and updated_at
    set_parts.push("version = version + 1".into());
    set_parts.push("updated_at = time::now()".into());

    let query = format!(
        "UPDATE clause:{} SET {}; SELECT * FROM clause:{};",
        id,
        set_parts.join(", "),
        id
    );

    let mut builder = state.db.query(&query);
    for (name, value) in &binds {
        builder = builder.bind((name.clone(), value.clone()));
    }

    let mut response = builder.await?;
    let updated: Option<Clause> = response.take(1)?;

    match updated {
        Some(c) => Ok(Json(json!({ "data": clause_to_json(&c) }))),
        None => Err(ApiError::not_found("Clause", &id)),
    }
}

#[utoipa::path(
    delete,
    path = "/clauses/{id}",
    tag = "Clauses",
    params(("id" = String, Path, description = "Clause record key")),
    responses(
        (status = 200, description = "Clause archived"),
        (status = 404, description = "Not found", body = crate::schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn delete_clause(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let query = format!(
        "UPDATE clause:{} SET status = 'archived', updated_at = time::now(); \
         SELECT * FROM clause:{};",
        id, id
    );
    let mut response = state.db.query(&query).await?;
    let clause: Option<Clause> = response.take(1)?;

    match clause {
        Some(_) => Ok(Json(json!({ "archived": true, "id": format!("clause:{}", id) }))),
        None => Err(ApiError::not_found("Clause", &id)),
    }
}

#[utoipa::path(
    get,
    path = "/clauses/categories",
    tag = "Clauses",
    responses(
        (status = 200, description = "Category list with counts"),
    ),
    security(("api_key" = []))
)]
pub async fn list_categories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let query = "SELECT category, count() AS count FROM clause WHERE status = 'active' GROUP BY category ORDER BY category ASC";
    let mut response = state.db.query(query).await?;
    let rows: Vec<Value> = response.take(0)?;

    Ok(Json(json!({ "data": rows })))
}
```

**Step 5: Add `mod models` to main.rs**

Add `mod models;` after the existing module declarations in `e-fees-scope/src/main.rs`.

**Step 6: Run tests to verify they pass**

Run: `cargo test -p e-fees-scope --test clause_tests -- --test-threads=1`
Expected: All tests pass (requires the service to be running with DB access)

**Step 7: Commit**

```bash
git add e-fees-scope/
git commit -m "feat(scope): implement clause CRUD with integration tests"
```

---

## Task 4: Corpus Ingestion Routes

**Files:**
- Create: `e-fees-scope/src/routes/corpus.rs`
- Modify: `e-fees-scope/src/routes/mod.rs`
- Modify: `e-fees-scope/src/main.rs` (add routes)
- Modify: `e-fees-scope/src/models.rs` (add corpus models)

**Step 1: Add corpus models to `models.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ProposalCorpus {
    pub id: RecordId,
    pub filename: String,
    pub project_number: Option<String>,
    pub project_name: Option<String>,
    pub extracted_text: String,
    pub sections: Option<surrealdb_types::Value>,
    pub metadata: Option<surrealdb_types::Value>,
    pub embedding: Option<Vec<f64>>,
    pub created_at: Datetime,
}
```

**Step 2: Create `e-fees-scope/src/routes/corpus.rs`**

Implement:
- `POST /corpus/ingest` — accepts `{ "file_path": "/data/rfps/file.pdf" }`, sends to Docling-Serve, extracts text, stores in DB
- `POST /corpus/ingest-batch` — accepts `{ "directory": "/data/rfps" }`, ingests all PDFs in directory
- `GET /corpus` — list ingested documents
- `GET /corpus/{id}` — get single document with sections
- `GET /corpus/search` — vector similarity search (placeholder until embeddings are generated)

Key pattern for Docling-Serve call:
```rust
let docling_res = state.http
    .post(format!("{}/convert", state.docling_url))
    .json(&json!({
        "input": { "path": file_path },
        "options": { "output_format": "text" }
    }))
    .timeout(std::time::Duration::from_secs(120))
    .send()
    .await
    .map_err(|e| ApiError::service_unavailable(format!("Docling error: {}", e)))?;
```

**Step 3: Add `pub mod corpus;` to `routes/mod.rs`**

**Step 4: Wire corpus routes in `main.rs`**

Add to protected router:
```rust
.route("/corpus", get(routes::corpus::list_corpus).post(routes::corpus::ingest))
.route("/corpus/ingest-batch", post(routes::corpus::ingest_batch))
.route("/corpus/search", get(routes::corpus::search_corpus))
.route("/corpus/{id}", get(routes::corpus::get_corpus_doc))
```

Add routes to OpenAPI `paths(...)`.

**Step 5: Commit**

```bash
git add e-fees-scope/
git commit -m "feat(scope): add corpus ingestion and search routes"
```

---

## Task 5: Scope Generation Routes

**Files:**
- Create: `e-fees-scope/src/routes/scope.rs`
- Create: `e-fees-scope/src/llm.rs`
- Modify: `e-fees-scope/src/routes/mod.rs`
- Modify: `e-fees-scope/src/main.rs`
- Modify: `e-fees-scope/src/models.rs`

**Step 1: Create `e-fees-scope/src/llm.rs` — Ollama client**

```rust
use reqwest::Client;
use serde_json::{json, Value};
use crate::error::ApiError;

pub async fn polish_scope(
    http: &Client,
    ollama_url: &str,
    model: &str,
    project_context: &Value,
    raw_scope: &str,
    similar_examples: &[String],
) -> Result<String, ApiError> {
    let examples_text = if similar_examples.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nReference examples from similar past proposals:\n{}",
            similar_examples.join("\n---\n")
        )
    };

    let prompt = format!(
        "Given this project context:\n{}\n\n\
         Refine these scope of services clauses for professional tone and project specificity. \
         Maintain the exact structure, numbering, and deliverables. Do not add or remove items. \
         Only improve language, specificity, and professionalism.\n\n\
         Scope text:\n{}\n{}",
        serde_json::to_string_pretty(project_context).unwrap_or_default(),
        raw_scope,
        examples_text
    );

    let body = json!({
        "model": model,
        "prompt": prompt,
        "system": "You are a senior lighting design consultant writing scope of services \
                    for fee proposals. Write in clear, professional English. \
                    Be specific to the project context provided.",
        "stream": false,
        "think": false,
        "options": {
            "temperature": 0.3
        }
    });

    let res = http
        .post(format!("{}/api/generate", ollama_url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| ApiError::service_unavailable(format!("Ollama error: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(ApiError::service_unavailable(format!(
            "Ollama returned {}: {}",
            status, text
        )));
    }

    let result: Value = res
        .json()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to parse Ollama response: {}", e)))?;

    result["response"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| ApiError::internal("Ollama response missing 'response' field"))
}
```

**Step 2: Create `e-fees-scope/src/routes/scope.rs`**

Implement:
- `POST /scope/generate` — accepts `{ "fee_id": "fee:xxx" }`, fetches fee+project, selects clauses, numbers them, optionally polishes with LLM
- `GET /scope/{fee_id}` — get existing scope assembly
- `PUT /scope/{fee_id}` — update scope (manual overrides)
- `POST /scope/{fee_id}/regenerate` — re-polish with LLM
- `GET /scope/{fee_id}/export` — export as InDesign-ready JSON

Key logic in `generate`:
1. Fetch fee from DB: `SELECT * FROM fee:$fee_key`
2. Fetch project: `SELECT * FROM $fee.project_id`
3. Select default + condition-matched clauses
4. Auto-number hierarchically by category
5. Substitute `{{placeholders}}`
6. Optionally call `llm::polish_scope()`
7. Store in `scope_assembly`

**Step 3: Add scope assembly model to `models.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ScopeAssembly {
    pub id: RecordId,
    pub fee_id: RecordId,
    pub clauses: surrealdb_types::Value,
    pub generated_text: String,
    pub numbering: Option<surrealdb_types::Value>,
    pub llm_model: Option<String>,
    pub llm_polished: bool,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}
```

**Step 4: Wire scope routes in `main.rs`**

Add to protected router:
```rust
.route("/scope/generate", post(routes::scope::generate_scope))
.route("/scope/{fee_id}", get(routes::scope::get_scope).put(routes::scope::update_scope))
.route("/scope/{fee_id}/regenerate", post(routes::scope::regenerate_scope))
.route("/scope/{fee_id}/export", get(routes::scope::export_scope))
```

**Step 5: Commit**

```bash
git add e-fees-scope/
git commit -m "feat(scope): add scope generation, LLM polish, and export routes"
```

---

## Task 6: Dockerfile and Deployment

**Files:**
- Create: `e-fees-scope/Dockerfile`
- Create: `e-fees-scope/.env.example`

**Step 1: Create Dockerfile**

```dockerfile
FROM rust:1.89-slim AS builder

WORKDIR /app

# Minimal workspace (only scope + core)
RUN printf '[workspace]\nmembers = ["crates/e-fees-core", "e-fees-scope"]\nresolver = "2"\n' > Cargo.toml

COPY Cargo.lock .
COPY crates/ crates/
COPY e-fees-scope/ e-fees-scope/

# curl needed by utoipa-swagger-ui build script
RUN apt-get update && apt-get install -y curl pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

RUN cargo build --release -p e-fees-scope

# Runtime
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/e-fees-scope /usr/local/bin/

EXPOSE 3201

CMD ["e-fees-scope"]
```

**Step 2: Create `.env.example`**

```env
SURREAL_URL=ws://10.0.23.11:8000
SURREAL_USER=root
SURREAL_PASS=changeme
SURREAL_NS=emittiv
SURREAL_DB=projects

API_KEY=efees-scope-2026-changeme
API_PORT=3201

OLLAMA_URL=http://10.0.21.20:11434
OLLAMA_MODEL=qwen3:4b

DOCLING_URL=http://10.0.21.42:5001

# Optional: mount point for PDF corpus
CORPUS_PATH=/data/rfps
```

**Step 3: Commit**

```bash
git add e-fees-scope/Dockerfile e-fees-scope/.env.example
git commit -m "feat(scope): add Dockerfile and env example"
```

---

## Task 7: Build, Deploy, and Verify

**Step 1: Build Docker image on AI server**

SSH to AI server and build:
```bash
ssh root@10.0.20.11
cd /mnt/user/appdata/e-fees-scope/source
git clone https://forge.mms.name/emittiv/e-fees.git . || git pull
docker build -f e-fees-scope/Dockerfile -t e-fees-scope:v0.1.0 --no-cache .
```

**Step 2: Create .env file**

```bash
mkdir -p /mnt/user/appdata/e-fees-scope
cat > /mnt/user/appdata/e-fees-scope/.env << 'EOF'
SURREAL_URL=ws://10.0.23.11:8000
SURREAL_USER=root
SURREAL_PASS=<actual password>
SURREAL_NS=emittiv
SURREAL_DB=projects
API_KEY=efees-scope-2026-<generate>
API_PORT=3201
OLLAMA_URL=http://10.0.21.20:11434
OLLAMA_MODEL=qwen3:4b
DOCLING_URL=http://10.0.21.42:5001
CORPUS_PATH=/data/rfps
EOF
```

**Step 3: Run container**

```bash
docker run -d \
  --name e-fees-scope \
  --network br0 --ip 10.0.21.81 \
  -p 3201:3201 \
  --env-file /mnt/user/appdata/e-fees-scope/.env \
  -v "/mnt/user/emittiv/nc/__groupfolders/1/01 Projects/01 RFPs/99 All RFPs:/data/rfps:ro" \
  --restart unless-stopped \
  e-fees-scope:v0.1.0
```

**Step 4: Verify health**

```bash
curl -s http://10.0.21.81:3201/health | jq .
```

Expected:
```json
{
  "status": "ok",
  "service": "e-fees-scope",
  "version": "0.1.0",
  "database": "connected",
  "ollama": "connected"
}
```

**Step 5: Verify Swagger UI**

Open `http://10.0.21.81:3201/docs/` — should show OpenAPI docs with Clauses endpoints.

**Step 6: Run integration tests against deployed service**

```bash
API_BASE_URL=http://10.0.21.81:3201 API_KEY=efees-scope-2026-... cargo test -p e-fees-scope --test clause_tests -- --test-threads=1
```

Expected: All tests pass.

---

## Task 8: Seed Clause Library from PDFs

**Step 1: Batch-ingest the 66 PDFs**

```bash
curl -s -X POST http://10.0.21.81:3201/corpus/ingest-batch \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"directory": "/data/rfps"}'
```

**Step 2: Verify corpus**

```bash
curl -s http://10.0.21.81:3201/corpus \
  -H "X-API-Key: $API_KEY" | jq '.total'
```

Expected: 66 (or close to it — some PDFs may fail extraction).

**Step 3: Review extracted sections**

Pick a document and inspect:
```bash
curl -s http://10.0.21.81:3201/corpus/<id> \
  -H "X-API-Key: $API_KEY" | jq '.data.sections'
```

**Step 4: Manually seed initial clauses**

Based on reviewed corpus, create the first set of standard clauses via API:
```bash
curl -s -X POST http://10.0.21.81:3201/clauses \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "category": "Concept Design",
    "title": "Lighting Concept Development",
    "body": "Development of the overall lighting concept for {{project_name}}, including mood and atmosphere studies, initial fixture palette, and control strategy outline.",
    "sort_order": 1,
    "is_default": true,
    "tags": ["concept", "standard"]
  }'
```

Repeat for each standard clause identified from the corpus.

**Step 5: Commit any adjustments**

```bash
git add -A && git commit -m "chore(scope): post-deployment adjustments"
```

---

## Execution Notes

- **Tasks 1-3 are the critical path** — get the crate scaffolded, schema applied, and clause CRUD working
- **Task 4** (corpus) can be done in parallel with Task 5 (scope generation) by different agents
- **Task 6-7** (Docker/deploy) must wait for at least Tasks 1-3
- **Task 8** (seeding) is the final step and may require iteration based on PDF extraction quality
- **SurrealDB v3 gotchas**: Use `NONE` not `NULL` for optional fields, `i64` not `i32`, `f64` requires `Number::Float`
- **Ollama gotcha**: Must set `"think": false` in request body for Qwen3.5
- **Docling-Serve**: At 10.0.21.42:5001, may need `curl` test first to verify it's responsive
