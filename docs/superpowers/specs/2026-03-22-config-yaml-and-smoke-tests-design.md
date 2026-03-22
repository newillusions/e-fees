# Design Spec: Config YAML Migration + Expanded Smoke Test Suite

**Date:** 2026-03-22
**Status:** Draft
**Scope:** e-fees-api, e-fees-scope, container-utils (Rust), e2e-mcp smoke tests

---

## Workstream 1: Container-Utils Axum Port + Config YAML Migration

### Problem

The `emittiv-container-utils` Rust crate (`/Volumes/base/dev/container-utils/rust/`) provides two capabilities:
1. **ConfigManager** — YAML config loading, hot-reload via `notify` v6, per-key callbacks, schema validation
2. **Health routes** — `/health`, `/api/health`, `/help`, `/openapi.json`

Both are currently **Actix-Web 4 only**. E-fees services use **axum 0.8**. The health routes were implemented directly in each service as a workaround (`e-fees-api/src/health.rs`, `e-fees-scope/src/health.rs`). Config is still `.env`-based.

### Goal

1. Make `emittiv-container-utils` support axum alongside actix
2. Migrate e-fees-api and e-fees-scope from `.env` to `config.yaml` using the shared ConfigManager
3. Replace hand-rolled health modules with the shared crate's axum routes

### Architecture

Split the crate into framework-agnostic core + framework-specific route modules:

```
container-utils/rust/src/
├── lib.rs                (re-exports, feature-gated)
├── config_manager.rs     (UNCHANGED — already framework-agnostic)
├── health.rs             (NEW — shared types: HealthResponse, DependencyStatus, HealthState)
├── routes_actix.rs       (RENAMED from health_routes.rs — existing actix routes, untouched)
└── routes_axum.rs        (NEW — axum equivalents of routes_actix.rs)
```

#### Feature Flags

```toml
[features]
default = ["actix"]
actix = ["actix-web", "actix-rt"]
axum = ["dep:axum"]
```

- `actix` (default): existing behavior, nothing breaks for current consumers
- `axum`: new axum route handlers

#### Shared Types (`health.rs`)

Framework-agnostic response types only. Each framework module handles its own dependency checking pattern (actix uses background thread with `Arc<RwLock<HashMap>>`, axum uses on-request async checks). The shared types are the JSON response structures:

```rust
#[derive(Serialize, Clone)]
pub struct HealthResponse {
    pub status: String,        // "ok" | "degraded" | "error"
    pub version: String,
    pub uptime: f64,           // seconds
    pub checked_at: String,    // ISO 8601
    pub dependencies: HashMap<String, DependencyStatus>,
}

#[derive(Serialize, Clone)]
pub struct DependencyStatus {
    pub status: String,        // "ok" | "error"
    pub latency_ms: f64,
}

#[derive(Serialize, Clone)]
pub struct HelpResponse {
    pub service: String,
    pub version: String,
    pub endpoints: Vec<EndpointInfo>,
}

#[derive(Serialize, Clone)]
pub struct EndpointInfo {
    pub method: String,
    pub path: String,
    pub description: String,
    pub auth_required: bool,
}

/// Helper to compute health status from dependency results
pub fn compute_status(deps: &HashMap<String, DependencyStatus>, critical_deps: &[&str]) -> (String, StatusCode) {
    // "error" + 503 if any critical dep is down
    // "degraded" + 200 if only non-critical deps are down
    // "ok" + 200 if all deps are up
}
```

**Design decision:** The actix routes (`routes_actix.rs`) keep their existing dependency checking model unchanged — background thread with `Arc<RwLock<HashMap<String, Value>>>`. The axum routes (`routes_axum.rs`) use on-request async checks matching the current e-fees pattern. Only the response types and status computation logic are shared. This avoids forcing actix consumers to change.

#### Axum Routes (`routes_axum.rs`)

```rust
pub fn health_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    // AppState must contain Arc<HealthState>
{
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/health", get(health_handler))
        .route("/help", get(help_handler))
}
```

The health handler runs all dependency checkers concurrently via `futures::join_all`, computes status (ok/degraded/error), and returns 503 if any critical dependency fails.

#### ConfigManager Integration

ConfigManager is already framework-agnostic. E-fees services add it to their AppState:

```rust
// In e-fees-api main.rs
use std::collections::HashMap;

let mut defaults = HashMap::new();
defaults.insert("server.port".into(), serde_yaml::Value::Number(3200.into()));
defaults.insert("server.host".into(), serde_yaml::Value::String("0.0.0.0".into()));
defaults.insert("database.url".into(), serde_yaml::Value::String("ws://10.0.23.11:8000".into()));
defaults.insert("database.namespace".into(), serde_yaml::Value::String("emittiv".into()));
defaults.insert("database.database".into(), serde_yaml::Value::String("projects".into()));
defaults.insert("log_level".into(), serde_yaml::Value::String("info".into()));

// ConfigManager::new returns Arc<ConfigManager>
let config = ConfigManager::new(
    "config.yaml".to_string(),
    defaults,
    HashMap::new(),  // empty schema = no validation
    2,               // poll interval seconds
);
// start_watching requires Arc<Self>
config.start_watching();
```

**Note:** ConfigManager constructor signature uses `HashMap<String, serde_yaml::Value>` for defaults and `HashMap<String, SchemaEntry>` for schema (not `Option`). The `new()` method returns `Arc<ConfigManager>`, and `start_watching()` requires `self: &Arc<Self>`.

#### Config YAML Format

```yaml
# config.yaml for e-fees-api
server:
  port: 3200
  host: "0.0.0.0"

database:
  url: "ws://10.0.23.11:8000"
  namespace: "emittiv"
  database: "projects"

log_level: "info"
```

**Secrets stay in `.env`** — database password, API keys. ConfigManager handles non-secret runtime config. The service reads `.env` for secrets at startup, ConfigManager for everything else.

#### Config Coverage

**e-fees-api** non-secret config (4 values): server port, host, DB URL, DB namespace/database, log level
**e-fees-scope** non-secret config (8 values): above + `ollama_url`, `ollama_model`, `docling_url`, `stirling_url`

All non-secret runtime config moves to `config.yaml`. Secrets (DB password, API keys) stay in `.env`.

```yaml
# config.yaml for e-fees-scope (extends e-fees-api format)
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

#### Migration Strategy

1. Add `config.yaml` support — service reads YAML at startup, compiled-in defaults for any missing keys (ConfigManager handles this natively)
2. `.env` shrinks to secrets-only (`SURREALDB_PASS`, `API_KEY`)
3. Docker containers mount `config.yaml` as a volume for hot-reload

**No dual-read fallback layer** — ConfigManager's built-in defaults cover the case where `config.yaml` doesn't exist yet. The service just works with defaults until a YAML file is provided.

#### Docker Hot-Reload Note

`notify`'s `inotify` backend does not reliably detect changes to bind-mounted files edited on the host (inode replacement by editors like vim). The ConfigManager's polling fallback (mtime-based, 2-second interval) handles this. Hot-reload in Docker will use polling, not inotify. This is fine for config changes.

### What Changes for Existing Actix Consumers

Minimal. `routes_actix.rs` is the renamed `health_routes.rs` with zero logic changes. The `actix` feature is the default. A deprecation re-export in `lib.rs` preserves `use emittiv_container_utils::health_routes::*` imports.

### `/help` Endpoint Strategy

The existing e-fees `/help` endpoints dynamically introspect the utoipa OpenAPI spec to list all endpoints with auth requirements. The shared crate's actix `/help` returns a static list. **The axum routes module provides a `help_handler` that accepts a `Vec<EndpointInfo>` via AppState** — the service populates this from its OpenAPI spec at startup. This preserves the dynamic listing behavior without hardcoding endpoints in the crate.

### Dependencies Added to Crate

```toml
[dependencies]
axum = { version = "0.8", optional = true }
futures = "0.3"  # for join_all in dependency checks
```

**Note:** `tower` is not needed — axum 0.8 re-exports everything required for basic routing.

---

## Workstream 2: Expanded Tauri MCP Smoke Test Suite

### Problem

Current smoke test suite has 11 checks covering DB connection, data counts, status validation, and route navigation. No CRUD workflow testing, no UI state validation, no regression coverage for known past bugs.

### Goal

Expand to ~52 checks (11 existing + 41 new) covering CRUD workflows, UI state, integration triggers, and known regressions. Add optional loop mode for continuous validation during development.

### Architecture

New test modules alongside existing ones:

```
e2e-mcp/suites/
├── run-smoke.ts            (MODIFIED — imports new suites, updated CHECK_ORDER)
├── helpers/
│   ├── smoke-checks.ts     (EXISTING — DB, data counts, statuses)
│   ├── dom-checks.ts       (EXISTING — navigation, DOM validation)
│   ├── crud-checks.ts      (NEW — CRUD workflows)
│   ├── ui-state-checks.ts  (NEW — modals, forms, filters, shortcuts)
│   ├── integration-checks.ts (NEW — API/backend triggers)
│   └── regression-checks.ts  (NEW — known bug reproductions)
```

### Test Execution Pattern

All new checks follow the existing pattern:
- Async IIFE as a JS code string
- Execute via `mcp__tauri-mcp__execute_js`
- Result stored in `window.__SMOKE_RESULT`
- Return format: `{ check: string, pass: boolean, error?: string, details?: any, ABORT?: boolean }`

### New Check Suites

#### CRUD Checks (`crud-checks.ts`) — 16 checks

CRUD tests form a **sequential pipeline** — each entity test creates data used by the next. This is intentional because fees require project/company/contact IDs.

**Execution order and data flow:**
1. **crud_company** — Creates test company. Stores ID for contact + fee creation.
2. **crud_contact** — Creates test contact linked to test company. Stores ID for fee creation.
3. **crud_project** — Creates test project. Stores ID for fee creation.
4. **crud_fee** — Creates test fee linked to test project/company/contact.
5. **crud_cleanup** — Deletes all entities in reverse order (fee → project → contact → company), then scans for any orphaned "DELETE ME" entities.

Each entity check includes create → read-back → update → verify update. If any step fails, cleanup still runs.

**Invoke parameter names** (must match Rust command signatures exactly):
- `window.__TAURI_INTERNALS__.invoke('create_company', { company: {...} })`
- `window.__TAURI_INTERNALS__.invoke('create_contact', { contact: {...} })`
- `window.__TAURI_INTERNALS__.invoke('create_project', { project: {...} })`
- `window.__TAURI_INTERNALS__.invoke('create_fee', { fee: {...} })`

**Note:** Use `window.__TAURI_INTERNALS__` (not `window.__TAURI__`) for consistency with Tauri v2.24+ and existing smoke checks.

Test data naming: `"DELETE ME - Test {Entity} {Date.now()}"`

**Entity-specific notes:**
- **Companies**: Standalone, no dependencies. Minimal required fields: `name`, `country`.
- **Contacts**: Requires `company_id` from test company created in step 1. Required: `first_name`, `last_name`, `email`.
- **Projects**: Requires `number` object with `country` (use dial_code 971 = UAE, existing in country table), `year`, `sequence` (use 99 to avoid conflicts with real sequences 01-50).
- **Fees**: Requires `project_id`, `company_id`, `contact_id` from test entities created above. Required: `issue_date` (YYYYMM format).

#### UI State Checks (`ui-state-checks.ts`) — 10 checks

1. **modal_open_close** — Click "New" button, verify modal appears, close it, verify gone
2. **form_validation** — Submit empty form, verify error state
3. **search_filter** — Type in search box, verify list filters
4. **dropdown_filter** — Select status filter, verify filtered results
5. **keyboard_nav_1** — Simulate Cmd+1, verify dashboard route
6. **keyboard_nav_2** — Simulate Cmd+2, verify projects route
7. **keyboard_nav_3** — Simulate Cmd+3, verify companies route
8. **keyboard_nav_4** — Simulate Cmd+4, verify contacts route
9. **keyboard_nav_5** — Simulate Cmd+5, verify proposals route
10. **bulk_select** — Check a checkbox, verify bulk action bar appears

#### Integration Checks (`integration-checks.ts`) — 6 checks

1. **status_transition** — Change a fee status (Draft → Sent), verify UI updates
2. **fee_project_status_mapping** — Accept a fee, verify project status changes to Awarded
3. **connection_indicator** — Verify connection status component shows "connected"
4. **entity_count_consistency** — Compare `get_stats()` counts with actual list lengths
5. **settings_modal** — Open settings, verify DB config displayed
6. **detail_panel** — Click a project card, verify detail panel slides in

#### Regression Checks (`regression-checks.ts`) — 8 checks

Each targets a specific past bug:

1. **recordid_v3_parsing** — Verify RecordId fields parse correctly (not `{tb, id}` format)
2. **fee_no_infinity** — Load all fees, verify no field contains -Infinity or NaN (covers math::max empty array bug)
3. **contact_full_name** — Create contact, verify `full_name` computed correctly
4. **fee_deserialization** — Load fees, verify all fields deserialize (no -Infinity in pricing)
5. **company_id_extraction** — Verify company ID extracted as string from Thing object
6. **fee_status_values** — Verify no legacy statuses (Awarded, Lost) in fee table
7. **project_status_values** — Verify no legacy statuses (Accepted, Rejected) in project table
8. **navigation_order** — Verify sidebar order: Dashboard, Projects, Companies, Contacts, Proposals

### Updated CHECK_ORDER

```typescript
export const CHECK_ORDER = [
  // Phase 1: Safety (abort if production)
  'safety',
  // Phase 2: Infrastructure
  'db_connection', 'data_loaded',
  // Phase 3: Data validation
  'project_statuses', 'fee_statuses', 'entity_counts',
  // Phase 4: Navigation
  'navigate_dashboard', 'navigate_projects', 'navigate_proposals',
  'navigate_companies', 'navigate_contacts',
  // Phase 5: UI State
  'modal_open_close', 'form_validation', 'search_filter', 'dropdown_filter',
  'keyboard_nav_1', 'keyboard_nav_2', 'keyboard_nav_3',
  'keyboard_nav_4', 'keyboard_nav_5', 'bulk_select',
  // Phase 6: CRUD (sequential pipeline — each creates data for the next)
  'crud_company', 'crud_contact', 'crud_project', 'crud_fee', 'crud_cleanup',
  // Phase 7: Integration
  'status_transition', 'fee_project_mapping', 'connection_indicator',
  'entity_count_consistency', 'settings_modal', 'detail_panel',
  // Phase 8: Regression
  'recordid_v3', 'fee_no_infinity', 'contact_full_name', 'fee_deser',
  'company_id_extract', 'fee_status_legacy', 'project_status_legacy', 'nav_order',
];
```

### Loop Mode

The `/smoke-test` skill gets an optional `--loop [interval]` flag:

- `/smoke-test` — single run (existing behavior, expanded suite)
- `/smoke-test --loop` — repeat every 10 minutes (default)
- `/smoke-test --loop 5m` — repeat every 5 minutes

Implementation: Uses the existing `/loop` skill infrastructure. The loop runs `/smoke-test` on the specified interval, reporting only failures after the first run (quiet mode for passing checks).

Prerequisites: App must be running via `npm run tauri:dev`.

**Loop mode and CRUD tests:** In loop mode, CRUD tests run every iteration but cleanup always runs at the end of each cycle. If a cycle crashes mid-CRUD, the cleanup check at the start of the next cycle's CRUD phase scans for and removes orphaned "DELETE ME" entities before creating new ones.

### Test Data Safety

All CRUD tests follow the "DELETE ME" convention:
- Names: `"DELETE ME - Test Company 1711100000000"`
- Emails: `"delete-me-1711100000000@example.com"`
- Cleanup check runs after CRUD phase, removes any orphaned test data
- Safety check at start prevents running against production DB

---

## Scope Boundaries

### In Scope
- Axum route module in `emittiv-container-utils`
- Shared health types extraction
- ConfigManager integration in e-fees-api and e-fees-scope
- `config.yaml` files for both services
- Docker volume mount for config hot-reload
- 41 new smoke test checks (4 modules, 52 total with existing 11)
- Loop mode for `/smoke-test` skill
- Tests for all new code (TDD)

### Out of Scope
- Python/Node.js container-utils updates (separate effort)
- Contract test updates (follow-up after axum routes stabilize)
- Actix route module changes (renamed file, re-export for backwards compat, no logic changes)
- Desktop app (Tauri) config migration (uses its own `.env` pattern)
- CI/CD integration of smoke tests (future — requires app running)

---

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| ConfigManager hot-reload in Docker | Polling fallback (2s mtime check) handles bind-mount inotify limitations |
| CRUD tests leaving orphaned data | Cleanup check + "DELETE ME" pattern + production safety abort |
| Axum version mismatch | Pin to axum 0.8 matching e-fees services |
| Feature flag complexity | Only two flags (actix/axum), both well-bounded |
| Smoke test flakiness | 1s navigation delays, retry on DOM checks, deterministic test data |

---

## Success Criteria

1. `emittiv-container-utils` builds with both `--features actix` and `--features axum`
2. e-fees-api and e-fees-scope use shared health routes from the crate
3. Both services load config from `config.yaml` with hot-reload working
4. Secrets remain in `.env` (not in YAML)
5. All 52 smoke test checks pass against dev DB
6. `/smoke-test --loop 5m` runs continuously without accumulating test data
7. Existing actix consumers unaffected (contract tests still pass)
