# Container Standards Compliance — e-fees-api & e-fees-scope

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring both e-fees API containers into full compliance with Emittiv Container & API Standards (6/6 checks passing).

**Architecture:** Both services are Rust/axum 0.8 with identical patterns. We add a shared `health` module to each with `uptime`, `checked_at`, `dependencies` fields, plus `/api/health`, `/help`, and `/openapi.json` routes. The health routes are Actix-only in container-utils, so we implement directly in each service (Option B — no incomplete dependency). Config migration to YAML is deferred to a separate plan since the Rust shared library needs an axum port first.

**Tech Stack:** Rust, axum 0.8, utoipa 5, chrono 0.4, serde_json, tokio

**Scope decision:** The `emittiv-container-utils` Rust crate health routes are Actix-only and incompatible with axum. ConfigManager is framework-agnostic but YAML config migration is a separate, larger effort. This plan covers endpoint compliance only (checks 1-4, 6). Config migration (check 5) will be a follow-up plan after the library gets an axum feature flag.

---

## File Structure

### e-fees-api (new/modified files)
| File | Action | Responsibility |
|------|--------|---------------|
| `e-fees-api/src/health.rs` | **Create** | Health handler with `uptime`, `checked_at`, `dependencies`; `/help` handler; startup time tracking |
| `e-fees-api/src/main.rs` | **Modify** | Add `mod health`, track startup `Instant`, add `/api/health`, `/help`, `/openapi.json` routes, update `AppState` |
| `e-fees-api/src/schemas.rs` | **Modify** | Update `HealthResponse` schema + add `HelpResponse`, `DependencyStatus` schemas |
| `e-fees-api/tests/integration_tests.rs` | **Modify** | Add tests for new/updated endpoints |

### e-fees-scope (new/modified files)
| File | Action | Responsibility |
|------|--------|---------------|
| `e-fees-scope/src/health.rs` | **Create** | Health handler with `uptime`, `checked_at`, `dependencies`; `/help` handler; Ollama dependency check |
| `e-fees-scope/src/main.rs` | **Modify** | Add `mod health`, track startup `Instant`, add `/api/health`, `/help`, `/openapi.json` routes, update `AppState` |
| `e-fees-scope/src/schemas.rs` | **Modify** | Update `HealthResponse` schema + add `HelpResponse`, `DependencyStatus` schemas |
| `e-fees-scope/tests/clause_tests.rs` | **Modify** | Add tests for new/updated endpoints |

### Wiki
| Resource | Action |
|----------|--------|
| KB wiki page `e-fees-scope` | **Create** | Service documentation following container standards template |

---

## Task 1: e-fees-api — Upgrade `/health` endpoint

**Files:**
- Modify: `e-fees-api/src/main.rs:24-29` (AppState struct)
- Create: `e-fees-api/src/health.rs`
- Modify: `e-fees-api/src/schemas.rs:39-46` (HealthResponse)
- Modify: `e-fees-api/tests/integration_tests.rs`

### Step-by-step

- [ ] **Step 1: Write failing tests for upgraded health response**

Add to `e-fees-api/tests/integration_tests.rs` after the existing `test_health_response_format` test:

```rust
#[tokio::test]
async fn test_health_has_uptime() {
    verify_not_production();
    let client = Client::new();
    let body: serde_json::Value = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body["uptime"].is_number(), "missing 'uptime' field (should be seconds as number)");
    assert!(body["uptime"].as_f64().unwrap() >= 0.0, "uptime must be non-negative");
}

#[tokio::test]
async fn test_health_has_checked_at() {
    verify_not_production();
    let client = Client::new();
    let body: serde_json::Value = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body["checked_at"].is_string(), "missing 'checked_at' field");
    let ts = body["checked_at"].as_str().unwrap();
    // Must be ISO 8601 parseable
    assert!(chrono::DateTime::parse_from_rfc3339(ts).is_ok(), "checked_at must be RFC3339: got {}", ts);
}

#[tokio::test]
async fn test_health_has_dependencies() {
    verify_not_production();
    let client = Client::new();
    let body: serde_json::Value = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body["dependencies"].is_object(), "missing 'dependencies' object");
    assert!(body["dependencies"]["surrealdb"].is_object(), "missing 'surrealdb' dependency");
    assert!(body["dependencies"]["surrealdb"]["status"].is_string(), "missing dependency status");
}
```

- [ ] **Step 2: Add chrono dev-dependency**

Add to `e-fees-api/Cargo.toml` `[dev-dependencies]`:
```toml
chrono = "0.4"
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `API_BASE_URL=http://10.0.21.80:3200 API_KEY=efees-api-2026-k8x9m4pq SURREAL_URL=ws://10.0.21.8:8000 cargo test -p e-fees-api --test integration_tests test_health_has -- --test-threads=1`

Expected: 3 FAIL — missing `uptime`, `checked_at`, `dependencies` fields.

- [ ] **Step 4: Update AppState to track startup time**

In `e-fees-api/src/main.rs`, add to the imports:
```rust
use std::time::Instant;
```

Update `AppState` struct (line 25-29):
```rust
pub struct AppState {
    pub db: Surreal<surrealdb::engine::remote::ws::Client>,
    pub api_keys: HashSet<String>,
    pub folder_config: Option<config::FolderConfig>,
    pub started_at: Instant,
}
```

Update the state construction (line 152-156):
```rust
let state = Arc::new(AppState {
    db,
    api_keys: config.api_keys.into_iter().collect(),
    folder_config: config.folder_config,
    started_at: Instant::now(),
});
```

- [ ] **Step 5: Create health.rs with upgraded handler**

Create `e-fees-api/src/health.rs`:

```rust
use std::sync::Arc;
use std::time::Instant;

use axum::{extract::State, response::Json};
use serde_json::{json, Value};

use crate::AppState;

/// Health check endpoint (no auth required).
///
/// Returns service status with uptime, timestamp, and dependency health.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service healthy or degraded", body = crate::schemas::HealthResponse),
        (status = 503, description = "Service unhealthy"),
    )
)]
pub async fn health(State(state): State<Arc<AppState>>) -> (axum::http::StatusCode, Json<Value>) {
    let start = Instant::now();
    let db_ok = state.db.health().await.is_ok();
    let db_latency = start.elapsed().as_millis() as f64;

    let status = if db_ok { "ok" } else { "error" };
    let http_status = if db_ok {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    let body = json!({
        "status": status,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": state.started_at.elapsed().as_secs_f64(),
        "checked_at": chrono::Utc::now().to_rfc3339(),
        "dependencies": {
            "surrealdb": {
                "status": if db_ok { "ok" } else { "error" },
                "latency_ms": db_latency,
            }
        }
    });

    (http_status, Json(body))
}
```

- [ ] **Step 6: Wire health.rs into main.rs**

In `e-fees-api/src/main.rs`:
- Add `mod health;` after `mod config;` (line 2)
- Remove the inline `health` function (lines 241-259)
- Update the router (line 222) to use `health::health`:
```rust
.route("/health", get(health::health))
```
- Update the `#[openapi(paths(...))]` list — change `health,` to `health::health,`

- [ ] **Step 7: Update HealthResponse schema**

Replace `HealthResponse` in `e-fees-api/src/schemas.rs` (lines 39-46):

```rust
/// Dependency health status.
#[derive(Serialize, ToSchema)]
pub struct DependencyStatus {
    pub status: String,
    pub latency_ms: f64,
}

/// Health check response (Container Standards compliant).
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: f64,
    pub checked_at: String,
    pub dependencies: std::collections::HashMap<String, DependencyStatus>,
}
```

- [ ] **Step 8: Build and verify compilation**

Run: `cargo build -p e-fees-api`
Expected: Compiles successfully.

- [ ] **Step 9: Deploy updated container and run tests**

Build and deploy to AI server, then run:
```bash
API_BASE_URL=http://10.0.21.80:3200 API_KEY=efees-api-2026-k8x9m4pq SURREAL_URL=ws://10.0.21.8:8000 cargo test -p e-fees-api --test integration_tests test_health -- --test-threads=1
```
Expected: All health tests PASS (existing + 3 new).

- [ ] **Step 10: Also update existing test to not check removed `service`/`database` top-level fields**

The `test_health_response_format` test (line 69-90) checks for `service` and `database` fields that we removed. Update it:

```rust
#[tokio::test]
async fn test_health_response_format() {
    verify_not_production();
    let client = Client::new();
    let resp = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(body["status"].is_string(), "missing 'status' field");
    assert!(body["version"].is_string(), "missing 'version' field");
    assert!(body["uptime"].is_number(), "missing 'uptime' field");
    assert!(body["checked_at"].is_string(), "missing 'checked_at' field");
    assert!(body["dependencies"].is_object(), "missing 'dependencies' field");
}
```

- [ ] **Step 11: Commit**

```bash
git add e-fees-api/src/health.rs e-fees-api/src/main.rs e-fees-api/src/schemas.rs e-fees-api/tests/integration_tests.rs e-fees-api/Cargo.toml
git commit -m "feat(api): upgrade /health to container standards (uptime, checked_at, dependencies)"
```

---

## Task 2: e-fees-api — Add `/api/health` and `/openapi.json` routes

**Files:**
- Modify: `e-fees-api/src/main.rs:220-226` (router)
- Modify: `e-fees-api/tests/integration_tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `e-fees-api/tests/integration_tests.rs`:

```rust
#[tokio::test]
async fn test_api_health_alias() {
    verify_not_production();
    let client = Client::new();
    let resp = client
        .get(format!("{}/api/health", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["status"].is_string(), "/api/health must return same schema as /health");
    assert!(body["uptime"].is_number(), "/api/health must include uptime");
}

#[tokio::test]
async fn test_openapi_json_endpoint() {
    verify_not_production();
    let client = Client::new();
    let resp = client
        .get(format!("{}/openapi.json", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["openapi"].is_string(), "must have 'openapi' field");
    assert!(body["paths"].is_object(), "must have 'paths' field");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p e-fees-api --test integration_tests test_api_health_alias -- --test-threads=1`
Run: `cargo test -p e-fees-api --test integration_tests test_openapi_json -- --test-threads=1`
Expected: Both FAIL (404).

- [ ] **Step 3: Add routes to main.rs**

Update the public router section in `e-fees-api/src/main.rs` (around line 221):

```rust
let app = Router::new()
    .route("/health", get(health::health))
    .route("/api/health", get(health::health))
    .route("/openapi.json", get(openapi_json))
    .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
    .merge(protected)
    .layer(cors)
    .with_state(state);
```

Add the `openapi_json` handler after the `main` function:

```rust
async fn openapi_json() -> Json<Value> {
    let spec = ApiDoc::openapi();
    Json(serde_json::to_value(spec).unwrap())
}
```

- [ ] **Step 4: Build and deploy, run tests**

Run: `cargo build -p e-fees-api`
Deploy and run all tests:
```bash
API_BASE_URL=http://10.0.21.80:3200 API_KEY=efees-api-2026-k8x9m4pq SURREAL_URL=ws://10.0.21.8:8000 cargo test -p e-fees-api --test integration_tests -- --test-threads=1
```
Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add e-fees-api/src/main.rs e-fees-api/tests/integration_tests.rs
git commit -m "feat(api): add /api/health alias and /openapi.json endpoint"
```

---

## Task 3: e-fees-api — Add `/help` endpoint

**Files:**
- Modify: `e-fees-api/src/health.rs` (add help handler)
- Modify: `e-fees-api/src/main.rs` (add route)
- Modify: `e-fees-api/src/schemas.rs` (add HelpResponse)
- Modify: `e-fees-api/tests/integration_tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `e-fees-api/tests/integration_tests.rs`:

```rust
#[tokio::test]
async fn test_help_endpoint() {
    verify_not_production();
    let client = Client::new();
    let resp = client
        .get(format!("{}/help", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["service"].is_string(), "must have 'service'");
    assert!(body["version"].is_string(), "must have 'version'");
    assert!(body["description"].is_string(), "must have 'description'");
    assert!(body["endpoints"].is_array(), "must have 'endpoints' array");
    assert!(!body["endpoints"].as_array().unwrap().is_empty(), "endpoints must not be empty");
}

#[tokio::test]
async fn test_help_no_auth_required() {
    verify_not_production();
    let client = Client::new();
    let resp = client
        .get(format!("{}/help", base_url()))
        .send()
        .await
        .unwrap();
    // /help should be public — no auth needed
    assert_ne!(resp.status(), 401, "/help should not require auth");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Expected: FAIL (404).

- [ ] **Step 3: Add help handler to health.rs**

Append to `e-fees-api/src/health.rs`:

```rust
/// Self-documentation endpoint (no auth required).
///
/// Returns service metadata and endpoint listing generated from OpenAPI spec.
#[utoipa::path(
    get,
    path = "/help",
    tag = "Health",
    responses(
        (status = 200, description = "Service documentation", body = crate::schemas::HelpResponse),
    )
)]
pub async fn help() -> Json<Value> {
    let spec = crate::ApiDoc::openapi();

    let methods: &[(&str, fn(&utoipa::openapi::path::PathItem) -> &Option<utoipa::openapi::path::Operation>)] = &[
        ("GET", |p| &p.get),
        ("POST", |p| &p.post),
        ("PUT", |p| &p.put),
        ("DELETE", |p| &p.delete),
        ("PATCH", |p| &p.patch),
    ];

    let endpoints: Vec<Value> = spec
        .paths
        .paths
        .iter()
        .flat_map(|(path, item)| {
            methods.iter().filter_map(move |(method, getter)| {
                getter(item).as_ref().map(|op| {
                    json!({
                        "method": method,
                        "path": path,
                        "description": op.description.as_deref()
                            .or(op.summary.as_deref())
                            .unwrap_or(""),
                        "auth": !matches!(path.as_str(),
                            "/health" | "/api/health" | "/help" | "/openapi.json" | "/docs"),
                    })
                })
            })
        })
        .collect();

    Json(json!({
        "service": "e-fees-api",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "REST API for managing fee proposals, projects, companies, and contacts.",
        "config_file": "/app/config/config.yaml",
        "endpoints": endpoints,
    }))
}
```

- [ ] **Step 4: Add HelpResponse schema**

Add to `e-fees-api/src/schemas.rs`:

```rust
/// Help/self-documentation response.
#[derive(Serialize, ToSchema)]
pub struct HelpResponse {
    pub service: String,
    pub version: String,
    pub description: String,
    pub config_file: String,
    pub endpoints: Vec<HelpEndpoint>,
}

/// Single endpoint in help response.
#[derive(Serialize, ToSchema)]
pub struct HelpEndpoint {
    pub method: String,
    pub path: String,
    pub description: String,
    pub auth: bool,
}
```

- [ ] **Step 5: Register the route and utoipa path**

In `e-fees-api/src/main.rs`:
- Add `/help` route to the public router:
```rust
.route("/help", get(health::help))
```
- Add `health::help,` to the `#[openapi(paths(...))]` list

- [ ] **Step 6: Build, deploy, run tests**

```bash
cargo build -p e-fees-api
```
Deploy and run all tests. Expected: All PASS including new help tests.

- [ ] **Step 7: Commit**

```bash
git add e-fees-api/src/health.rs e-fees-api/src/main.rs e-fees-api/src/schemas.rs e-fees-api/tests/integration_tests.rs
git commit -m "feat(api): add /help self-documentation endpoint"
```

---

## Task 4: e-fees-scope — Upgrade `/health` endpoint

Same pattern as Task 1 but for e-fees-scope, with Ollama as an additional dependency.

**Files:**
- Modify: `e-fees-scope/src/main.rs:24-32` (AppState)
- Create: `e-fees-scope/src/health.rs`
- Modify: `e-fees-scope/src/schemas.rs:10-17` (HealthResponse)
- Modify: `e-fees-scope/tests/clause_tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `e-fees-scope/tests/clause_tests.rs` after the existing health test:

```rust
#[tokio::test]
async fn test_health_has_uptime() {
    let client = Client::new();
    let body: serde_json::Value = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body["uptime"].is_number(), "missing 'uptime' field");
}

#[tokio::test]
async fn test_health_has_checked_at() {
    let client = Client::new();
    let body: serde_json::Value = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body["checked_at"].is_string(), "missing 'checked_at' field");
}

#[tokio::test]
async fn test_health_has_dependencies() {
    let client = Client::new();
    let body: serde_json::Value = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body["dependencies"].is_object(), "missing 'dependencies' object");
    assert!(body["dependencies"]["surrealdb"].is_object(), "missing 'surrealdb' dependency");
    assert!(body["dependencies"]["ollama"].is_object(), "missing 'ollama' dependency");
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
API_BASE_URL=http://10.0.21.81:3201 API_KEY=efees-scope-2026-s7k2m9xp cargo test -p e-fees-scope --test clause_tests test_health_has -- --test-threads=1
```
Expected: 3 FAIL.

- [ ] **Step 3: Update AppState**

In `e-fees-scope/src/main.rs`, add `use std::time::Instant;` to imports and add `started_at: Instant` to AppState:

```rust
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
```

Update state construction (line 140-148) to add `started_at: Instant::now(),`.

- [ ] **Step 4: Create health.rs with Ollama + SurrealDB dependency checks**

Create `e-fees-scope/src/health.rs`:

```rust
use std::sync::Arc;
use std::time::Instant;

use axum::{extract::State, response::Json};
use serde_json::{json, Value};

use crate::AppState;

#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service healthy or degraded", body = crate::schemas::HealthResponse),
        (status = 503, description = "Service unhealthy"),
    )
)]
pub async fn health(State(state): State<Arc<AppState>>) -> (axum::http::StatusCode, Json<Value>) {
    let db_start = Instant::now();
    let db_ok = state.db.health().await.is_ok();
    let db_latency = db_start.elapsed().as_millis() as f64;

    let ollama_start = Instant::now();
    let ollama_ok = state
        .http
        .get(format!("{}/api/tags", state.ollama_url))
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    let ollama_latency = ollama_start.elapsed().as_millis() as f64;

    let status = if db_ok && ollama_ok {
        "ok"
    } else if db_ok {
        "degraded"
    } else {
        "error"
    };

    let http_status = if db_ok {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    let body = json!({
        "status": status,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": state.started_at.elapsed().as_secs_f64(),
        "checked_at": chrono::Utc::now().to_rfc3339(),
        "dependencies": {
            "surrealdb": {
                "status": if db_ok { "ok" } else { "error" },
                "latency_ms": db_latency,
            },
            "ollama": {
                "status": if ollama_ok { "ok" } else { "unreachable" },
                "latency_ms": ollama_latency,
            }
        }
    });

    (http_status, Json(body))
}
```

- [ ] **Step 5: Wire health.rs into main.rs**

In `e-fees-scope/src/main.rs`:
- Add `mod health;` after `mod config;`
- Remove inline `health` function (lines 242-277)
- Update router: `.route("/health", get(health::health))`
- Update `#[openapi(paths(...))]`: change `health,` to `health::health,`

- [ ] **Step 6: Update HealthResponse schema**

Replace in `e-fees-scope/src/schemas.rs`:

```rust
/// Dependency health status.
#[derive(Serialize, ToSchema)]
pub struct DependencyStatus {
    pub status: String,
    pub latency_ms: f64,
}

/// Health check response (Container Standards compliant).
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: f64,
    pub checked_at: String,
    pub dependencies: std::collections::HashMap<String, DependencyStatus>,
}
```

- [ ] **Step 7: Build, deploy, run tests**

```bash
cargo build -p e-fees-scope
```
Deploy and run: Expected all PASS.

- [ ] **Step 8: Update existing health test**

Update the existing health format test in `clause_tests.rs` to match the new schema (check for `status`, `version`, `uptime`, `checked_at`, `dependencies` instead of `service`, `database`, `ollama`).

- [ ] **Step 9: Commit**

```bash
git add e-fees-scope/src/health.rs e-fees-scope/src/main.rs e-fees-scope/src/schemas.rs e-fees-scope/tests/clause_tests.rs
git commit -m "feat(scope): upgrade /health to container standards (uptime, checked_at, dependencies)"
```

---

## Task 5: e-fees-scope — Add `/api/health`, `/help`, `/openapi.json` routes

Same pattern as Tasks 2+3 for e-fees-scope.

**Files:**
- Modify: `e-fees-scope/src/health.rs` (add help handler)
- Modify: `e-fees-scope/src/main.rs` (add routes)
- Modify: `e-fees-scope/src/schemas.rs` (add HelpResponse)
- Modify: `e-fees-scope/tests/clause_tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `e-fees-scope/tests/clause_tests.rs`:

```rust
#[tokio::test]
async fn test_api_health_alias() {
    let client = Client::new();
    let resp = client
        .get(format!("{}/api/health", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["status"].is_string());
    assert!(body["uptime"].is_number());
}

#[tokio::test]
async fn test_openapi_json_endpoint() {
    let client = Client::new();
    let resp = client
        .get(format!("{}/openapi.json", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["openapi"].is_string());
    assert!(body["paths"].is_object());
}

#[tokio::test]
async fn test_help_endpoint() {
    let client = Client::new();
    let resp = client
        .get(format!("{}/help", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["service"].is_string());
    assert!(body["version"].is_string());
    assert!(body["description"].is_string());
    assert!(body["endpoints"].is_array());
    assert!(!body["endpoints"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_help_no_auth_required() {
    let client = Client::new();
    let resp = client
        .get(format!("{}/help", base_url()))
        .send()
        .await
        .unwrap();
    assert_ne!(resp.status(), 401);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Expected: FAIL (404 or 401).

- [ ] **Step 3: Add help handler to health.rs**

Append to `e-fees-scope/src/health.rs`:

```rust
#[utoipa::path(
    get,
    path = "/help",
    tag = "Health",
    responses(
        (status = 200, description = "Service documentation", body = crate::schemas::HelpResponse),
    )
)]
pub async fn help() -> Json<Value> {
    let spec = crate::ApiDoc::openapi();

    let methods: &[(&str, fn(&utoipa::openapi::path::PathItem) -> &Option<utoipa::openapi::path::Operation>)] = &[
        ("GET", |p| &p.get),
        ("POST", |p| &p.post),
        ("PUT", |p| &p.put),
        ("DELETE", |p| &p.delete),
        ("PATCH", |p| &p.patch),
    ];

    let endpoints: Vec<Value> = spec
        .paths
        .paths
        .iter()
        .flat_map(|(path, item)| {
            methods.iter().filter_map(move |(method, getter)| {
                getter(item).as_ref().map(|op| {
                    json!({
                        "method": method,
                        "path": path,
                        "description": op.description.as_deref()
                            .or(op.summary.as_deref())
                            .unwrap_or(""),
                        "auth": !matches!(path.as_str(),
                            "/health" | "/api/health" | "/help" | "/openapi.json" | "/docs"),
                    })
                })
            })
        })
        .collect();

    Json(json!({
        "service": "e-fees-scope",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Scope/deliverables management, clause library, and proposal corpus.",
        "config_file": "/app/config/config.yaml",
        "endpoints": endpoints,
    }))
}
```

- [ ] **Step 4: Add schemas**

Add `HelpResponse` and `HelpEndpoint` to `e-fees-scope/src/schemas.rs` (same as e-fees-api schemas from Task 3).

- [ ] **Step 5: Register routes**

In `e-fees-scope/src/main.rs`, update the public router:
```rust
let app = Router::new()
    .route("/health", get(health::health))
    .route("/api/health", get(health::health))
    .route("/help", get(health::help))
    .route("/openapi.json", get(openapi_json))
    .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
    .merge(protected)
    .layer(cors)
    .with_state(state);
```

Add `openapi_json` handler and register `health::help` in `#[openapi(paths(...))]`.

- [ ] **Step 6: Build, deploy, run tests**

Expected: All PASS.

- [ ] **Step 7: Commit**

```bash
git add e-fees-scope/src/health.rs e-fees-scope/src/main.rs e-fees-scope/src/schemas.rs e-fees-scope/tests/clause_tests.rs
git commit -m "feat(scope): add /api/health, /help, /openapi.json endpoints"
```

---

## Task 6: Create e-fees-scope wiki page

**Files:**
- None (KB wiki only)

- [ ] **Step 1: Create wiki page**

Use `kb_wiki_create_page` with:
- `slug`: `e-fees-scope`
- `title`: `E-Fees Scope Service Reference`
- `owner_instance`: `e-fees`
- `tags`: `["api", "rest", "rust", "axum", "e-fees", "emittiv", "scope", "clauses", "deliverables"]`

Content following the container standards wiki template:

```markdown
# e-fees-scope

## Purpose
Scope/deliverables management, clause library, and proposal corpus for the E-Fees system. Provides LLM-powered scope text generation from clause libraries and ingested fee proposal PDFs.

## Endpoints
See running /help endpoint at http://10.0.21.81:3201/help for auto-generated list.

Key route groups:
- /clauses — Clause library CRUD (6 endpoints)
- /corpus — Proposal corpus ingestion and search (6 endpoints)
- /scope — Scope generation, assembly, export (5 endpoints)
- /deliverables — Deliverable library CRUD + analytics (5 endpoints)
- /stages — Stage configuration (2 endpoints)
- /scope/assemble, /scope/save — Deliverable assembly (3 endpoints)

## Configuration
| Key | Type | Default | Source | Description |
|-----|------|---------|--------|-------------|
| SURREAL_URL | string | — | env (required) | SurrealDB WebSocket URL |
| SURREAL_USER | string | — | env (required) | DB root username |
| SURREAL_PASS | string | — | env (required) | DB root password |
| API_KEY | string | — | env (required) | Comma-separated API keys |
| SURREAL_NS | string | emittiv | env | Namespace |
| SURREAL_DB | string | projects | env | Database |
| API_PORT | int | 3201 | env | Listen port |
| OLLAMA_URL | string | http://10.0.21.20:11434 | env | Ollama LLM endpoint |
| OLLAMA_MODEL | string | qwen3:4b | env | LLM model for scope polish |
| DOCLING_URL | string | http://10.0.21.42:5001 | env | Docling document processor |
| STIRLING_URL | string | http://10.0.21.41:8080 | env | Stirling PDF converter |
| CORPUS_PATH | string | — | env (optional) | PDF corpus mount point |

## Dependencies
| Service | URL | Required | Notes |
|---------|-----|----------|-------|
| SurrealDB | ws://10.0.23.11:8000 | Yes | ns:emittiv, db:projects |
| Ollama | http://10.0.21.20:11434 | No | Degraded mode if unavailable |
| Docling | http://10.0.21.42:5001 | No | For PDF text extraction |
| Stirling | http://10.0.21.41:8080 | No | For PDF→PNG conversion |

## Deployment
- Container: e-fees-scope
- IP: 10.0.21.81
- Port: 3201
- Appdata: /mnt/user/appdata/e-fees-scope/
- Template: e-fees-scope.xml (Forgejo docker-templates)
- Swagger: http://10.0.21.81:3201/docs/
```

- [ ] **Step 2: Verify wiki page creation**

Run `kb_wiki_search` for "e-fees-scope" and confirm it appears.

- [ ] **Step 3: Commit** (no code changes, wiki is KB-only)

---

## Task 7: Run full compliance check and fix version numbers

**Files:**
- Modify: `e-fees-api/Cargo.toml:3` (version)
- Modify: `e-fees-scope/Cargo.toml:3` (version)

- [ ] **Step 1: Run compliance check on both containers**

```bash
/container-standards --check 10.0.21.80:3200
/container-standards --check 10.0.21.81:3201
```

Expected: 5/6 for both (config.yaml still missing — deferred).

- [ ] **Step 2: Bump Cargo.toml versions to match deployed reality**

The OpenAPI specs and /health report `CARGO_PKG_VERSION`. Currently both say `0.1.0`. Bump to reflect the actual release:

`e-fees-api/Cargo.toml` line 3: `version = "0.2.0"`
`e-fees-scope/Cargo.toml` line 3: `version = "0.2.0"`

- [ ] **Step 3: Rebuild, redeploy, verify version in /health response**

```bash
curl -s http://10.0.21.80:3200/health | jq .version
# Expected: "0.2.0"
curl -s http://10.0.21.81:3201/health | jq .version
# Expected: "0.2.0"
```

- [ ] **Step 4: Commit**

```bash
git add e-fees-api/Cargo.toml e-fees-scope/Cargo.toml
git commit -m "chore(api,scope): bump version to 0.2.0 for container standards compliance"
```

---

## Task 8: Update e-fees-api wiki page

**Files:**
- None (KB wiki only)

- [ ] **Step 1: Update existing e-fees-api wiki page**

Use `kb_wiki_update_section` to add/update sections for the new endpoints (`/api/health`, `/help`, `/openapi.json`) and note the container standards compliance status.

- [ ] **Step 2: Verify**

Run `kb_wiki_get("e-fees-api")` and confirm new endpoints are documented.

---

## Summary

| Task | Service | What | Commit |
|------|---------|------|--------|
| 1 | e-fees-api | Upgrade `/health` (uptime, checked_at, dependencies) | `feat(api): upgrade /health` |
| 2 | e-fees-api | Add `/api/health` + `/openapi.json` | `feat(api): add /api/health alias and /openapi.json` |
| 3 | e-fees-api | Add `/help` | `feat(api): add /help self-documentation` |
| 4 | e-fees-scope | Upgrade `/health` (uptime, checked_at, dependencies) | `feat(scope): upgrade /health` |
| 5 | e-fees-scope | Add `/api/health` + `/help` + `/openapi.json` | `feat(scope): add /api/health, /help, /openapi.json` |
| 6 | — | Create e-fees-scope wiki page | (KB only) |
| 7 | both | Version bump + full compliance check | `chore: bump version` |
| 8 | — | Update e-fees-api wiki page | (KB only) |

## Deployment Procedure (referenced by "deploy" steps in tasks above)

Both containers run on the AI server (10.0.20.11) with Docker on br0 network. To deploy updated code:

```bash
# 1. SSH to AI server
ssh root@10.0.20.11

# 2. Pull latest code (source is cloned on Unraid)
# e-fees-api:
cd /mnt/user/appdata/e-fees-api/source && git pull

# e-fees-scope:
cd /mnt/user/appdata/e-fees-scope/source && git pull

# 3. Rebuild Docker image (from the source dir containing Dockerfile)
# e-fees-api:
cd /mnt/user/appdata/e-fees-api/source && docker build -f e-fees-api/Dockerfile -t e-fees-api:latest .

# e-fees-scope:
cd /mnt/user/appdata/e-fees-scope/source && docker build -f e-fees-scope/Dockerfile -t e-fees-scope:latest .

# 4. Restart container (Unraid manages via XML template — use docker restart, NOT docker rm)
docker restart e-fees-api
docker restart e-fees-scope

# 5. Verify health
curl -s http://10.0.21.80:3200/health | jq .
curl -s http://10.0.21.81:3201/health | jq .
```

**CRITICAL:** Never use `docker rm` or `docker stop` on Unraid containers — use `docker restart` only. XML templates own the lifecycle.

**NOTE:** The source repos on the AI server are clones from Forgejo. Code must be pushed to `forge.mms.name/emittiv/fee-prop` first, then pulled on the server. The Dockerfiles use a minimal workspace Cargo.toml that only includes `e-fees-core` + the target crate (no Tauri deps).

---

**Deferred to follow-up plan:**
- Config migration from `.env` → `config.yaml` (requires axum port of `emittiv-container-utils` Rust crate health routes, or manual ConfigManager integration)
- Background dependency checks with caching (currently inline per-request)
- Docker HEALTHCHECK directive update

**Expected final compliance:**
- e-fees-api: **5/6** (missing config.yaml)
- e-fees-scope: **5/6** (missing config.yaml)
