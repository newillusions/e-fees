# Config YAML Migration + Expanded Smoke Test Suite — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port emittiv-container-utils to support axum, migrate e-fees services to YAML config, and expand Tauri MCP smoke tests from 11 to 52 checks.

**Architecture:** Two independent workstreams. WS1 (Tasks 1-6): restructure container-utils crate with feature flags, add axum routes, integrate ConfigManager into e-fees-api and e-fees-scope. WS2 (Tasks 7-12): add CRUD, UI state, integration, and regression test modules to the smoke test suite, update the skill for loop mode.

**Tech Stack:** Rust (axum 0.8, serde_yaml, notify v6), TypeScript (Tauri MCP execute_js), SurrealDB v3.0.4

**Spec:** `docs/superpowers/specs/2026-03-22-config-yaml-and-smoke-tests-design.md`

**Parallelism:** WS1 and WS2 are fully independent — can be assigned to separate agents. Within WS1, tasks are sequential. Within WS2, tasks 8-11 are independent after task 7.

---

## File Structure

### WS1: Container-Utils + Config YAML

| File | Action | Responsibility |
|------|--------|---------------|
| `/Volumes/base/dev/container-utils/rust/src/lib.rs` | **Modify** | Feature-gated re-exports |
| `/Volumes/base/dev/container-utils/rust/src/health.rs` | **Create** | Shared response types + compute_status helper |
| `/Volumes/base/dev/container-utils/rust/src/routes_actix.rs` | **Rename** | From `health_routes.rs`, zero logic changes |
| `/Volumes/base/dev/container-utils/rust/src/routes_axum.rs` | **Create** | Axum health/help handlers |
| `/Volumes/base/dev/container-utils/rust/Cargo.toml` | **Modify** | Add feature flags, axum dep |
| `/Volumes/base/dev/container-utils/rust/tests/routes_axum_test.rs` | **Create** | Tests for axum health routes |
| `e-fees-api/Cargo.toml` | **Modify** | Add container-utils dependency |
| `e-fees-api/src/config.rs` | **Modify** | Add ConfigManager loading alongside env |
| `e-fees-api/src/health.rs` | **Modify** | Use shared types from container-utils |
| `e-fees-api/src/main.rs` | **Modify** | Wire ConfigManager into AppState |
| `e-fees-api/config.yaml` | **Create** | YAML config for API service |
| `e-fees-scope/Cargo.toml` | **Modify** | Add container-utils dependency |
| `e-fees-scope/src/config.rs` | **Modify** | Add ConfigManager loading |
| `e-fees-scope/src/health.rs` | **Modify** | Use shared types from container-utils |
| `e-fees-scope/src/main.rs` | **Modify** | Wire ConfigManager into AppState |
| `e-fees-scope/config.yaml` | **Create** | YAML config for scope service |

### WS2: Smoke Test Expansion

| File | Action | Responsibility |
|------|--------|---------------|
| `e2e-mcp/suites/helpers/smoke-checks.ts` | **Modify** | Update `__TAURI__` → `__TAURI_INTERNALS__` |
| `e2e-mcp/suites/helpers/crud-checks.ts` | **Create** | CRUD pipeline (company→contact→project→fee→cleanup) |
| `e2e-mcp/suites/helpers/ui-state-checks.ts` | **Create** | Modal, form, filter, keyboard, bulk checks |
| `e2e-mcp/suites/helpers/integration-checks.ts` | **Create** | Status transitions, connection, settings, detail panel |
| `e2e-mcp/suites/helpers/regression-checks.ts` | **Create** | RecordId, infinity, full_name, legacy status checks |
| `e2e-mcp/suites/run-smoke.ts` | **Modify** | Import new modules, update CHECK_ORDER |
| `.claude/commands/smoke-test.md` | **Modify** | Add `--loop` flag documentation |

---

## WS1: Container-Utils Axum Port + Config YAML

### Task 1: Restructure container-utils crate — feature flags + shared types

**Files:**
- Modify: `/Volumes/base/dev/container-utils/rust/Cargo.toml`
- Rename: `/Volumes/base/dev/container-utils/rust/src/health_routes.rs` → `routes_actix.rs`
- Create: `/Volumes/base/dev/container-utils/rust/src/health.rs`
- Modify: `/Volumes/base/dev/container-utils/rust/src/lib.rs`

- [ ] **Step 1: Update Cargo.toml with feature flags**

```toml
[package]
name = "emittiv-container-utils"
version = "0.2.0"
edition = "2021"
description = "Container utilities for Emittiv services — ConfigManager + health routes"

[features]
default = ["actix"]
actix = ["dep:actix-web", "dep:actix-rt"]
axum = ["dep:axum"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
notify = "6"
tokio = { version = "1", features = ["full"] }
log = "0.4"
env_logger = "0.11"
chrono = { version = "0.4", features = ["serde"] }
futures = "0.3"

# Framework-specific (optional)
actix-web = { version = "4", optional = true }
actix-rt = { version = "2", optional = true }
axum = { version = "0.8", optional = true }

[dev-dependencies]
actix-test = "0.1"
tempfile = "3"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full", "test-util"] }
```

- [ ] **Step 2: Rename health_routes.rs to routes_actix.rs**

Run: `cd /Volumes/base/dev/container-utils/rust && mv src/health_routes.rs src/routes_actix.rs`

- [ ] **Step 3: Create shared health types in `src/health.rs`**

```rust
use std::collections::HashMap;
use serde::Serialize;

/// Health check response — shared across actix and axum routes.
#[derive(Serialize, Clone, Debug)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: f64,
    pub checked_at: String,
    pub dependencies: HashMap<String, DependencyStatus>,
}

/// Status of a single dependency.
#[derive(Serialize, Clone, Debug)]
pub struct DependencyStatus {
    pub status: String,
    pub latency_ms: f64,
}

/// Help endpoint response.
#[derive(Serialize, Clone, Debug)]
pub struct HelpResponse {
    pub service: String,
    pub version: String,
    pub endpoints: Vec<EndpointInfo>,
}

/// Single endpoint description for /help.
#[derive(Serialize, Clone, Debug)]
pub struct EndpointInfo {
    pub method: String,
    pub path: String,
    pub description: String,
    pub auth_required: bool,
}

/// Compute health status from dependency results.
///
/// Returns ("error", true) if any critical dependency is down.
/// Returns ("degraded", false) if only non-critical deps are down.
/// Returns ("ok", false) if all deps are up.
pub fn compute_status(
    deps: &HashMap<String, DependencyStatus>,
    critical_deps: &[&str],
) -> (String, bool) {
    let any_critical_down = critical_deps.iter().any(|name| {
        deps.get(*name).map_or(true, |d| d.status != "ok")
    });
    if any_critical_down {
        return ("error".to_string(), true);
    }
    let any_down = deps.values().any(|d| d.status != "ok");
    if any_down {
        return ("degraded".to_string(), false);
    }
    ("ok".to_string(), false)
}
```

- [ ] **Step 4: Update lib.rs with feature-gated re-exports**

```rust
pub mod config_manager;
pub mod health;

#[cfg(feature = "actix")]
pub mod routes_actix;

#[cfg(feature = "axum")]
pub mod routes_axum;

// Backwards compatibility re-export (deprecated)
#[cfg(feature = "actix")]
#[deprecated(note = "Use routes_actix instead")]
pub mod health_routes {
    pub use crate::routes_actix::*;
}
```

- [ ] **Step 5: Verify actix feature still compiles**

Run: `cd /Volumes/base/dev/container-utils/rust && cargo build --features actix`
Expected: Compiles successfully (routes_actix.rs is the renamed health_routes.rs)

- [ ] **Step 6: Verify axum feature compiles (will fail — routes_axum.rs not yet created)**

Run: `cd /Volumes/base/dev/container-utils/rust && cargo build --no-default-features --features axum`
Expected: Fails with "file not found: routes_axum" — confirms feature gating works

- [ ] **Step 7: Commit**

```bash
cd /Volumes/base/dev/container-utils/rust
git add -A
git commit -m "refactor: restructure crate with feature flags and shared health types

Extract shared HealthResponse/DependencyStatus/HelpResponse types into
health.rs. Rename health_routes.rs to routes_actix.rs behind actix feature
flag. Add axum feature flag (module not yet implemented).

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2: Implement axum health routes + tests

**Files:**
- Create: `/Volumes/base/dev/container-utils/rust/src/routes_axum.rs`
- Create: `/Volumes/base/dev/container-utils/rust/tests/routes_axum_test.rs`

- [ ] **Step 1: Write failing test for axum health route**

Create `/Volumes/base/dev/container-utils/rust/tests/routes_axum_test.rs`:

```rust
#[cfg(feature = "axum")]
mod axum_tests {
    use std::collections::HashMap;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::time::Instant;
    use std::future::Future;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use emittiv_container_utils::health::{EndpointInfo, HealthResponse};
    use emittiv_container_utils::routes_axum::{AxumHealthState, health_router};
    use tower::ServiceExt;

    fn make_state(dep_ok: bool) -> Arc<AxumHealthState> {
        Arc::new(AxumHealthState {
            started_at: Instant::now(),
            version: "0.2.0-test".to_string(),
            service_name: "test-service".to_string(),
            dep_checkers: vec![(
                "test_db".to_string(),
                true, // critical
                Box::new(move || -> Pin<Box<dyn Future<Output = (bool, f64)> + Send>> {
                    Box::pin(async move { (dep_ok, 1.5) })
                }),
            )],
            endpoints: vec![EndpointInfo {
                method: "GET".to_string(),
                path: "/health".to_string(),
                description: "Health check".to_string(),
                auth_required: false,
            }],
        })
    }

    #[tokio::test]
    async fn test_health_ok() {
        let state = make_state(true);
        let app = health_router().with_state(state);
        let resp = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: HealthResponse = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        assert_eq!(body.status, "ok");
        assert!(body.uptime >= 0.0);
        assert!(body.dependencies.contains_key("test_db"));
    }

    #[tokio::test]
    async fn test_health_critical_dep_down() {
        let state = make_state(false);
        let app = health_router().with_state(state);
        let resp = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body: HealthResponse = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        assert_eq!(body.status, "error");
    }

    #[tokio::test]
    async fn test_health_api_health_alias() {
        let state = make_state(true);
        let app = health_router().with_state(state);
        let resp = app
            .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_help_endpoint() {
        let state = make_state(true);
        let app = health_router().with_state(state);
        let resp = app
            .oneshot(Request::builder().uri("/help").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        assert_eq!(body["service"], "test-service");
        assert!(body["endpoints"].is_array());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Volumes/base/dev/container-utils/rust && cargo test --no-default-features --features axum -- axum_tests`
Expected: Fails — `routes_axum` module doesn't exist yet

- [ ] **Step 3: Implement routes_axum.rs**

Create `/Volumes/base/dev/container-utils/rust/src/routes_axum.rs`:

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;

use crate::health::{
    compute_status, DependencyStatus, EndpointInfo, HealthResponse, HelpResponse,
};

/// Axum-specific health state. Services embed this in their AppState.
pub struct AxumHealthState {
    pub started_at: Instant,
    pub version: String,
    pub service_name: String,
    /// (name, critical, async checker returning (ok, latency_ms))
    pub dep_checkers:
        Vec<(String, bool, Box<dyn Fn() -> Pin<Box<dyn Future<Output = (bool, f64)> + Send>> + Send + Sync>)>,
    /// Populated at startup from OpenAPI spec introspection.
    pub endpoints: Vec<EndpointInfo>,
}

/// Returns a Router with /health, /api/health, and /help routes.
pub fn health_router() -> Router<Arc<AxumHealthState>> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/health", get(health_handler))
        .route("/help", get(help_handler))
}

async fn health_handler(State(state): State<Arc<AxumHealthState>>) -> impl IntoResponse {
    let mut deps = std::collections::HashMap::new();
    let mut critical_names: Vec<&str> = Vec::new();

    // Run all dependency checks concurrently
    let checks: Vec<_> = state
        .dep_checkers
        .iter()
        .map(|(name, critical, checker)| {
            let name = name.clone();
            let critical = *critical;
            let fut = checker();
            async move { (name, critical, fut.await) }
        })
        .collect();

    let results = futures::future::join_all(checks).await;

    for (name, critical, (ok, latency_ms)) in &results {
        deps.insert(
            name.clone(),
            DependencyStatus {
                status: if *ok { "ok" } else { "error" }.to_string(),
                latency_ms: *latency_ms,
            },
        );
        if *critical {
            critical_names.push(name.as_str());
        }
    }

    let (status, is_error) = compute_status(&deps, &critical_names);
    let http_status = if is_error {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    let response = HealthResponse {
        status,
        version: state.version.clone(),
        uptime: state.started_at.elapsed().as_secs_f64(),
        checked_at: chrono::Utc::now().to_rfc3339(),
        dependencies: deps,
    };

    (http_status, Json(response))
}

async fn help_handler(State(state): State<Arc<AxumHealthState>>) -> Json<HelpResponse> {
    Json(HelpResponse {
        service: state.service_name.clone(),
        version: state.version.clone(),
        endpoints: state.endpoints.clone(),
    })
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /Volumes/base/dev/container-utils/rust && cargo test --no-default-features --features axum -- axum_tests`
Expected: All 4 tests pass

- [ ] **Step 5: Verify actix tests still pass**

Run: `cd /Volumes/base/dev/container-utils/rust && cargo test --features actix`
Expected: All existing tests pass

- [ ] **Step 6: Commit**

```bash
cd /Volumes/base/dev/container-utils/rust
git add -A
git commit -m "feat: add axum health routes with AxumHealthState contract

Implements /health, /api/health, /help handlers for axum 0.8.
Uses on-request async dependency checks with concurrent execution
via futures::join_all. 4 tests covering ok/error/alias/help responses.

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3: Integrate container-utils into e-fees-api

**Files:**
- Modify: `e-fees-api/Cargo.toml`
- Modify: `e-fees-api/src/config.rs`
- Modify: `e-fees-api/src/health.rs`
- Modify: `e-fees-api/src/main.rs`
- Create: `e-fees-api/config.yaml`

- [ ] **Step 1: Add container-utils dependency to e-fees-api/Cargo.toml**

Add to `[dependencies]` section:
```toml
emittiv-container-utils = { path = "../../../container-utils/rust", default-features = false, features = ["axum"] }
```

Note: path is relative from `e-fees-api/` (3 levels up to `/Volumes/base/dev/`, then into `container-utils/rust/`). Verify with `ls ../../../container-utils/rust/Cargo.toml` from the e-fees-api directory.

- [ ] **Step 2: Create config.yaml**

Create `e-fees-api/config.yaml`:
```yaml
server:
  port: 3200
  host: "0.0.0.0"

database:
  url: "ws://10.0.23.11:8000"
  namespace: "emittiv"
  database: "projects"

log_level: "info"
```

- [ ] **Step 3: Update config.rs to load from ConfigManager with env fallback for secrets**

Replace `e-fees-api/src/config.rs` contents. The ConfigManager handles non-secret config from YAML; secrets stay in env vars:

```rust
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use emittiv_container_utils::config_manager::ConfigManager;

/// Nextcloud folder creation config (optional — only if NC_SSH_HOST is set).
pub struct FolderConfig {
    pub ssh_host: String,
    pub ssh_user: String,
    pub ssh_key: String,
    pub script_path: String,
    pub nc_base_path: String,
}

/// API server configuration.
pub struct Config {
    pub surreal_url: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    pub api_keys: Vec<String>,
    pub port: u16,
    pub folder_config: Option<FolderConfig>,
    pub config_manager: Arc<ConfigManager>,
}

impl Config {
    pub fn load() -> Self {
        // ConfigManager for non-secret runtime config
        let mut defaults = HashMap::new();
        defaults.insert("server.port".into(), serde_yaml::Value::Number(3200.into()));
        defaults.insert("server.host".into(), serde_yaml::Value::String("0.0.0.0".into()));
        defaults.insert("database.url".into(), serde_yaml::Value::String("ws://10.0.23.11:8000".into()));
        defaults.insert("database.namespace".into(), serde_yaml::Value::String("emittiv".into()));
        defaults.insert("database.database".into(), serde_yaml::Value::String("projects".into()));
        defaults.insert("log_level".into(), serde_yaml::Value::String("info".into()));

        let cm = Arc::new(ConfigManager::new(
            "config.yaml".to_string(),
            defaults,
            HashMap::new(),
            2,
        ));
        cm.start_watching(); // start_watching requires self: &Arc<Self>

        // Read values from ConfigManager (YAML or defaults)
        let surreal_url = cm.get_str("database.url").unwrap_or_else(|| "ws://10.0.23.11:8000".into());
        let surreal_ns = cm.get_str("database.namespace").unwrap_or_else(|| "emittiv".into());
        let surreal_db = cm.get_str("database.database").unwrap_or_else(|| "projects".into());
        let port = cm.get_i64("server.port").unwrap_or(3200) as u16;

        // Secrets from env vars only
        let surreal_user = env::var("SURREAL_USER").expect("SURREAL_USER required");
        let surreal_pass = env::var("SURREAL_PASS").expect("SURREAL_PASS required");
        let raw_keys = env::var("API_KEY").expect("API_KEY required");
        let api_keys: Vec<String> = raw_keys
            .split(',')
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();
        assert!(!api_keys.is_empty(), "API_KEY must contain at least one non-empty key");

        let folder_config = env::var("NC_SSH_HOST").ok().map(|ssh_host| FolderConfig {
            ssh_host,
            ssh_user: env::var("NC_SSH_USER").unwrap_or_else(|_| "root".into()),
            ssh_key: env::var("NC_SSH_KEY").unwrap_or_else(|_| "/root/.ssh/id_ed25519".into()),
            script_path: env::var("NC_SCRIPT_PATH")
                .unwrap_or_else(|_| "/mnt/user/appdata/scripts/nc-project-create.sh".into()),
            nc_base_path: env::var("NC_BASE_PATH")
                .unwrap_or_else(|_| "/mnt/user/emittiv/nc/__groupfolders/1/01 Projects".into()),
        });

        Self {
            surreal_url,
            surreal_ns,
            surreal_db,
            surreal_user,
            surreal_pass,
            api_keys,
            port,
            folder_config,
            config_manager: cm,
        }
    }
}
```

- [ ] **Step 4: Update health.rs to use shared typed structs instead of json!()**

The existing `e-fees-api/src/health.rs` builds responses with `serde_json::json!({...})` returning `Json<Value>`. It has no local typed structs. The change is to replace the `json!` construction with the shared typed structs from `emittiv_container_utils::health`.

Key changes:
- Add `use emittiv_container_utils::health::{HealthResponse, DependencyStatus, HelpResponse, EndpointInfo, compute_status};`
- In `health()` handler: replace `Json(json!({...}))` with `Json(HealthResponse { status, version, uptime, checked_at, dependencies })` where `dependencies` is a `HashMap<String, DependencyStatus>`
- In `help()` handler: replace `Json(json!({...}))` with `Json(HelpResponse { service, version, endpoints })` where `endpoints` is built from utoipa OpenAPI introspection mapped into `Vec<EndpointInfo>`
- Use `compute_status()` to determine status string and HTTP status code, replacing the inline if/else logic
- The handler function signatures (`State<Arc<AppState>>`) stay unchanged — `AppState` does NOT change to `AxumHealthState` here. The shared crate's `AxumHealthState` is for services using the crate's pre-built router; e-fees keeps its own handlers but uses the shared response types
- Update `schemas.rs` to reference the shared types in utoipa `#[utoipa::path]` response annotations if needed

- [ ] **Step 5: Update main.rs — replace `Config::from_env()` with `Config::load()`**

In `e-fees-api/src/main.rs`, change:
```rust
let config = Config::from_env();
```
to:
```rust
let config = Config::load();
```

- [ ] **Step 6: Verify it compiles**

Run: `cd /Volumes/base/dev/app/e-fees && cargo build -p e-fees-api`
Expected: Compiles successfully

- [ ] **Step 7: Run existing integration tests**

Run: `API_BASE_URL=http://10.0.21.80:3200 API_KEY=efees-api-2026-k8x9m4pq cargo test -p e-fees-api --test integration_tests -- --test-threads=1`
Expected: All 72 tests pass (tests hit the deployed container, not local build — confirms shared types are compatible)

- [ ] **Step 8: Commit**

```bash
git add e-fees-api/Cargo.toml e-fees-api/src/config.rs e-fees-api/src/health.rs e-fees-api/src/main.rs e-fees-api/config.yaml
git commit -m "feat(api): integrate container-utils ConfigManager + shared health types

Migrate from .env-only config to ConfigManager YAML with env fallback
for secrets. Use shared HealthResponse/DependencyStatus types from
emittiv-container-utils crate.

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4: Integrate container-utils into e-fees-scope

**Files:**
- Modify: `e-fees-scope/Cargo.toml`
- Modify: `e-fees-scope/src/config.rs`
- Modify: `e-fees-scope/src/health.rs`
- Modify: `e-fees-scope/src/main.rs`
- Create: `e-fees-scope/config.yaml`

Same pattern as Task 3 but with additional config values for ollama, docling, stirling.

- [ ] **Step 1: Add container-utils dependency to e-fees-scope/Cargo.toml**

```toml
emittiv-container-utils = { path = "../../../container-utils/rust", default-features = false, features = ["axum"] }
```

- [ ] **Step 2: Create config.yaml for scope service**

Create `e-fees-scope/config.yaml`:
```yaml
server:
  port: 3201
  host: "0.0.0.0"

database:
  url: "ws://10.0.23.11:8000"
  namespace: "emittiv"
  database: "projects"

ollama:
  url: "http://10.0.21.50:11434"
  model: "qwen3.5:9b"

docling:
  url: "http://10.0.21.42:5001"

stirling:
  url: "http://10.0.21.41:8080"

log_level: "info"
```

- [ ] **Step 3: Update config.rs with ConfigManager + scope-specific config**

Same pattern as Task 3 Step 3, but add:
```rust
let ollama_url = cm.get_str("ollama.url").unwrap_or_else(|| "http://10.0.21.50:11434".into());
let ollama_model = cm.get_str("ollama.model").unwrap_or_else(|| "qwen3.5:9b".into());
let docling_url = cm.get_str("docling.url").unwrap_or_else(|| "http://10.0.21.42:5001".into());
let stirling_url = cm.get_str("stirling.url").unwrap_or_else(|| "http://10.0.21.41:8080".into());
```

- [ ] **Step 4: Update health.rs to use shared types**

Same pattern as Task 3 Step 4 — replace `json!({...})` construction with shared typed structs (`HealthResponse`, `DependencyStatus`). Keep the handler logic (SurrealDB + Ollama concurrent dependency checks via `tokio::join!`). Use `compute_status()` with `critical_deps: &["surrealdb"]` (Ollama is non-critical).

- [ ] **Step 5: Update main.rs**

Replace `Config::from_env()` with `Config::load()`.

- [ ] **Step 6: Verify compilation**

Run: `cd /Volumes/base/dev/app/e-fees && cargo build -p e-fees-scope`
Expected: Compiles

- [ ] **Step 7: Commit**

```bash
git add e-fees-scope/Cargo.toml e-fees-scope/src/config.rs e-fees-scope/src/health.rs e-fees-scope/src/main.rs e-fees-scope/config.yaml
git commit -m "feat(scope): integrate container-utils ConfigManager + shared health types

Add YAML config for scope service including ollama, docling, stirling
URLs. Use shared health types from emittiv-container-utils crate.

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5: Update Docker containers with config.yaml volume mounts

**Files:** Docker build commands (no file changes in repo)

- [ ] **Step 1: Build and test e-fees-api locally**

Run: `cd /Volumes/base/dev/app/e-fees && cargo build -p e-fees-api --release`

- [ ] **Step 2: Build and test e-fees-scope locally**

Run: `cd /Volumes/base/dev/app/e-fees && cargo build -p e-fees-scope --release`

- [ ] **Step 3: Document deployment commands**

The container rebuild will need config.yaml bind-mounted. Document in HANDOVER.md:
```
Docker rebuild commands (run on AI server):
  e-fees-api: mount /mnt/user/appdata/e-fees-api/config.yaml:/app/config.yaml
  e-fees-scope: mount /mnt/user/appdata/e-fees-scope/config.yaml:/app/config.yaml
```

Note: Actual container deployment deferred to deploy session — requires SSH to AI server and Unraid template updates.

- [ ] **Step 4: Commit any remaining changes**

---

### Task 6: Verify full WS1 — both features build, all tests pass

- [ ] **Step 1: container-utils actix build**

Run: `cd /Volumes/base/dev/container-utils/rust && cargo build --features actix`

- [ ] **Step 2: container-utils axum build**

Run: `cd /Volumes/base/dev/container-utils/rust && cargo build --no-default-features --features axum`

- [ ] **Step 3: container-utils all tests**

Run: `cd /Volumes/base/dev/container-utils/rust && cargo test --all-features`

- [ ] **Step 4: e-fees-api build**

Run: `cargo build -p e-fees-api`

- [ ] **Step 5: e-fees-scope build**

Run: `cargo build -p e-fees-scope`

- [ ] **Step 6: e-fees core tests**

Run: `cargo test -p e-fees-core`

---

## WS2: Expanded Smoke Test Suite

### Task 7: Unify __TAURI_INTERNALS__ in existing smoke-checks.ts

**Files:**
- Modify: `e2e-mcp/suites/helpers/smoke-checks.ts`

- [ ] **Step 1: Replace all `window.__TAURI__` with `window.__TAURI_INTERNALS__`**

Run a global replace in `e2e-mcp/suites/helpers/smoke-checks.ts`:
- Find: `window.__TAURI__`
- Replace: `window.__TAURI_INTERNALS__`

- [ ] **Step 2: Verify no `__TAURI__` references remain (except __TAURI_INTERNALS__)**

Run: `grep -n '__TAURI__' e2e-mcp/suites/helpers/smoke-checks.ts | grep -v '__TAURI_INTERNALS__'`
Expected: No output

- [ ] **Step 3: Commit**

```bash
git add e2e-mcp/suites/helpers/smoke-checks.ts
git commit -m "refactor(smoke): unify __TAURI_INTERNALS__ across all test files

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 8: CRUD checks module

**Files:**
- Create: `e2e-mcp/suites/helpers/crud-checks.ts`

- [ ] **Step 1: Create crud-checks.ts with sequential pipeline**

Create `e2e-mcp/suites/helpers/crud-checks.ts` with these exported checks:

- `crudCompany` — Create company with `{ company: { name: "DELETE ME - Test Company {ts}", country: "Test" } }`, read back via `get_companies`, update name, verify update, store ID in `window.__CRUD_IDS.company`
- `crudContact` — Create contact linked to `window.__CRUD_IDS.company`, store ID
- `crudProject` — Create project with number `{ country: "971", year: "26", sequence: 99 }` (sequence is integer, not string), store ID
- `crudFee` — Create fee linked to stored IDs, `issue_date: "202603"`, store ID
- `crudCleanup` — Delete in reverse order (fee→project→contact→company), then scan for orphaned "DELETE ME" entities

Each check follows the pattern:
```typescript
export const crudCompany = `(async () => {
  try {
    if (!window.__CRUD_IDS) window.__CRUD_IDS = {};
    // Cleanup any orphaned test data first
    const existing = await window.__TAURI_INTERNALS__.invoke('get_companies');
    for (const c of existing.filter(x => x.name?.startsWith('DELETE ME'))) {
      try { await window.__TAURI_INTERNALS__.invoke('delete_company', { id: c.id }); } catch(e) {}
    }
    // Create
    const ts = Date.now();
    const created = await window.__TAURI_INTERNALS__.invoke('create_company', {
      company: { name: 'DELETE ME - Test Company ' + ts, country: 'Test' }
    });
    // ... read back, update, verify ...
    window.__CRUD_IDS.company = /* extracted ID */;
    window.__SMOKE_RESULT = JSON.stringify({ check: 'crud_company', pass: true, details: { id: ... } });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({ check: 'crud_company', pass: false, error: e.message });
  }
})()`;
```

Use the exact Tauri invoke parameter names from the spec: `{ company: {...} }`, `{ contact: {...} }`, `{ project: {...} }`, `{ fee: {...} }`.

**Important:** CRUD checks store created IDs in `window.__CRUD_IDS` which persists across `execute_js` calls because SPA hash navigation does not cause full page reloads. The Phase 5 UI checks (which run before CRUD) use `window.location.hash` for navigation — this is safe. However, if any check triggers a hard page reload, `window.__CRUD_IDS` will be wiped. Each CRUD check should fall back to scanning for "DELETE ME" entities if `window.__CRUD_IDS` is missing.

- [ ] **Step 2: Verify file exports all 5 checks**

Run: `grep 'export const' e2e-mcp/suites/helpers/crud-checks.ts`
Expected: `crudCompany`, `crudContact`, `crudProject`, `crudFee`, `crudCleanup`

- [ ] **Step 3: Commit**

```bash
git add e2e-mcp/suites/helpers/crud-checks.ts
git commit -m "feat(smoke): add CRUD checks module — sequential pipeline with cleanup

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 9: UI state checks module

**Files:**
- Create: `e2e-mcp/suites/helpers/ui-state-checks.ts`

- [ ] **Step 1: Create ui-state-checks.ts with 10 checks**

Each check is an async IIFE string. Key checks:
- `modalOpenClose` — Navigate to `/companies`, click button with text "New", wait 500ms, check for modal element, press Escape or click close, verify modal gone
- `formValidation` — Open new company modal, submit without filling fields, check for validation error indicators
- `searchFilter` — Navigate to `/projects`, set search input value, wait 500ms, compare list count before/after
- `dropdownFilter` — Navigate to `/projects`, select a status filter, verify filtered results
- `keyboardNav1` through `keyboardNav5` — Dispatch keyboard events (Meta+1 through Meta+5), verify hash route changes
- `bulkSelect` — Navigate to `/projects`, click first checkbox, verify bulk action bar appears

- [ ] **Step 2: Verify file exports all 10 checks**

- [ ] **Step 3: Commit**

```bash
git add e2e-mcp/suites/helpers/ui-state-checks.ts
git commit -m "feat(smoke): add UI state checks — modals, forms, filters, keyboard shortcuts

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 10: Integration checks module

**Files:**
- Create: `e2e-mcp/suites/helpers/integration-checks.ts`

- [ ] **Step 1: Create integration-checks.ts with 6 checks**

- `statusTransition` — **Read-only**: Load fees, verify at least one has status "Draft" and at least one has a non-Draft status. Does NOT mutate data (previous design mutated live data with no rollback — changed to read-only validation).
- `feeProjectMapping` — **Read-only**: Load fees with status "Accepted", verify their linked projects have status "Awarded". Validates the mapping logic without mutating data.
- `connectionIndicator` — Check DOM for connection status element showing "connected" or green indicator
- `entityCountConsistency` — Compare `get_stats()` with actual counts from `get_projects()`, `get_companies()`, etc.
- `settingsModal` — Trigger settings modal (if accessible), verify DB config is displayed
- `detailPanel` — Navigate to `/projects`, click first project card, check for detail panel DOM element

- [ ] **Step 2: Commit**

```bash
git add e2e-mcp/suites/helpers/integration-checks.ts
git commit -m "feat(smoke): add integration checks — status transitions, connection, settings

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 11: Regression checks module

**Files:**
- Create: `e2e-mcp/suites/helpers/regression-checks.ts`

- [ ] **Step 1: Create regression-checks.ts with 8 checks**

Each targets a specific past bug:
- `recordidV3` — Load projects, verify ID fields are strings (not `{tb:"projects", id:{...}}` objects)
- `feeNoInfinity` — Load all fees, JSON.stringify each, verify no "-Infinity" or "NaN" in output
- `contactFullName` — Load contacts, verify `full_name` field exists and is non-empty on each
- `feeDeser` — Load all fees, verify all pricing fields are numbers or null (not -Infinity)
- `companyIdExtract` — Load companies, verify each has a string ID (not an object)
- `feeStatusLegacy` — Load fees, verify no status is "Awarded" or "Lost" (legacy values)
- `projectStatusLegacy` — Load projects, verify no status is "Accepted" or "Rejected" (legacy values)
- `navOrder` — Check sidebar DOM for navigation items, verify order: Dashboard, Projects, Companies, Contacts, Proposals

- [ ] **Step 2: Commit**

```bash
git add e2e-mcp/suites/helpers/regression-checks.ts
git commit -m "feat(smoke): add regression checks — RecordId, infinity, legacy status guards

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 12: Update run-smoke.ts and smoke-test skill

**Files:**
- Modify: `e2e-mcp/suites/run-smoke.ts`
- Modify: `.claude/commands/smoke-test.md`

- [ ] **Step 1: Update run-smoke.ts — import new modules and update CHECKS + CHECK_ORDER**

Add imports from all new modules. Add their exported checks to the `CHECKS` object. Update `CHECK_ORDER` to the full 52-check sequence from the spec:

```typescript
// Phase 1-4: existing checks (unchanged)
// Phase 5: UI State (from ui-state-checks.ts)
// Phase 6: CRUD pipeline (from crud-checks.ts)
// Phase 7: Integration (from integration-checks.ts)
// Phase 8: Regression (from regression-checks.ts)
```

- [ ] **Step 2: Update smoke-test.md skill — add --loop documentation**

Add to `.claude/commands/smoke-test.md`:
```markdown
## Loop Mode

- `/smoke-test --loop` — repeat every 10 minutes
- `/smoke-test --loop 5m` — repeat every 5 minutes

Uses the `/loop` skill infrastructure. Reports only failures after first run.
Prerequisites: App running via `npm run tauri:dev`.
```

- [ ] **Step 3: Commit**

```bash
git add e2e-mcp/suites/run-smoke.ts .claude/commands/smoke-test.md
git commit -m "feat(smoke): update orchestrator with 52-check suite and loop mode

Co-Authored-By: Claude <noreply@anthropic.com>"
```

- [ ] **Step 4: Run full smoke test suite (requires app running)**

Run: `/smoke-test`
Expected: All 52 checks pass against dev DB

---

## Post-Implementation

After all tasks complete:
1. Run `/smoke-test` to validate the expanded suite
2. Verify container-utils builds with both features
3. Update HANDOVER.md with completion status
4. Save key findings to KB via `kb_observe`
