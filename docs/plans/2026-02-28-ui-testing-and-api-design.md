# E-Fees UI Testing & Standalone API — Design Document

**Date:** 2026-02-28
**Status:** Approved
**Version:** 0.1

## Problem Statement

Two gaps in the E-Fees project:

1. **No UI validation.** Backend tests (87 Rust tests, svelte-check) pass but don't verify the frontend renders correctly. Breaking changes to components, status values, or modals go undetected until manual use.

2. **No programmatic data access.** E-Fees project/fee data is locked inside the desktop app. Other systems (AILX, n8n, Claude instances) cannot query it. As the ecosystem grows, this becomes a bottleneck.

## Architecture Overview

Two independent workstreams sharing a core library:

```
Workstream A: UI Validation (Tauri MCP)
  Claude ──► Tauri MCP tools ──► Running desktop app
  Purpose: Verify UI renders correctly after code changes
  Runs: During development, locally

Workstream B: E-Fees API (Rust/axum standalone)
  AILX, n8n, Claude ──► e-fees-api (HTTP) ──► SurrealDB
  Purpose: Programmatic access to E-Fees data
  Runs: 24/7 as Docker container on Unraid
```

### Why the desktop app exists

The desktop app's core value proposition is **filesystem operations** — creating project folder structures, copying templates, writing JSON files — with immediate Nextcloud/OpenCloud sync. Everything else (CRUD, querying, status management) is API-shaped work that can live in a standalone service.

Future: If OpenCloud provides direct filesystem API access, even folder operations could migrate to the API, making the desktop app a thin client or potentially unnecessary.

## Repository Structure

```
e-fees/
├── crates/
│   └── e-fees-core/            ← Shared types, validation, query builders
│       ├── src/
│       │   ├── models/         ← Project, Fee, Company, Contact structs
│       │   ├── queries/        ← SurrealQL query strings/builders
│       │   └── lib.rs
│       └── Cargo.toml
├── src-tauri/                   ← Desktop app (depends on e-fees-core)
├── e-fees-api/                  ← Standalone API (depends on e-fees-core)
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs           ← Env-based config
│   │   ├── auth.rs             ← API key middleware
│   │   ├── error.rs            ← Unified error responses
│   │   └── routes/
│   │       ├── mod.rs
│   │       ├── projects.rs
│   │       ├── fees.rs
│   │       ├── companies.rs
│   │       ├── contacts.rs
│   │       ├── stats.rs
│   │       └── health.rs
│   ├── tests/integration/
│   ├── Dockerfile
│   └── Cargo.toml
├── e2e-mcp/                     ← UI test scripts (Tauri MCP)
│   ├── suites/
│   │   ├── smoke.ts            ← Quick: launch, connect, routes render
│   │   ├── projects.ts         ← Project list, filters, status values
│   │   ├── proposals.ts        ← Fee list, status filters, pricing
│   │   ├── companies.ts        ← Company list, CRUD modal
│   │   └── contacts.ts         ← Contact list, company linking
│   ├── helpers/
│   │   ├── mcp-client.ts       ← Wrapper (cleaned up from existing)
│   │   ├── assertions.ts       ← DOM element checks, store checks
│   │   └── navigation.ts       ← Route helpers
│   ├── fixtures/
│   │   └── test-data-safe.ts   ← Existing DELETE ME generators
│   └── run-smoke.sh
└── src/                         ← Svelte 5 frontend
```

## Shared Core Library (`e-fees-core`)

Extracted from the existing Tauri app — not written from scratch.

### What moves into `e-fees-core`

| Category | Examples | Currently in |
|----------|----------|-------------|
| Domain models | `Project`, `Fee`, `Company`, `Contact`, `Currency`, `Country` | `src-tauri/src/db/mod.rs` |
| Enums | `ProjectStatus`, `FeeStatus`, `FolderCategory` | `src-tauri/src/db/mod.rs` |
| Validation | Phone format, project number format, field constraints | Scattered in db/mod.rs |
| Query builders | SurrealQL strings for CRUD operations | Inline in command handlers |

### What stays in each consumer

| `src-tauri` (desktop) | `e-fees-api` (standalone) |
|------------------------|--------------------------|
| `#[tauri::command]` handlers | axum route handlers |
| Window management, file explorer | Auth middleware (API keys) |
| MCP plugin, Agent API server | CORS, rate limiting |
| Frontend assets, Vite/Svelte | Docker deployment config |

### Dependencies for `e-fees-core`

- `serde` / `serde_json` — serialization
- `surrealdb` — types (`RecordId`, `Datetime`, etc.)
- No Tauri dependency, no axum dependency — pure data layer

## API Service Details

### Runtime configuration

```
Host:     Docker container on Unraid (br0 network, 10.0.21.x range)
Port:     3200
DB:       ws://10.0.23.11:8000 (same SurrealDB as desktop app)
Auth:     API key via X-API-Key header
```

### Environment file

Standalone `.env` file for the container (not embedded in credential system, but initially populated from it):

```env
SURREAL_URL=ws://10.0.23.11:8000
SURREAL_NS=emittiv
SURREAL_DB=projects
SURREAL_USER=martin
SURREAL_PASS=<from credentials>
API_KEY=<generated>
API_PORT=3200
```

### API Surface

**v0.1 — Read-only:**

```
GET  /health                — service health + DB connectivity
GET  /stats                 — dashboard counts
GET  /projects              — list with filtering (status, country, year)
GET  /projects/:id          — single project with fees
GET  /fees                  — list with filtering
GET  /fees/:id              — single fee detail
GET  /companies             — list
GET  /companies/:id         — company with contacts
GET  /contacts              — list
```

**v0.2 — Write operations:**

```
POST/PUT/DELETE on all entities
POST /projects/:id/status   — status transitions with validation
POST /fees/:id/clone         — fee revision cloning
```

### Response format

Success:
```json
{
  "data": [...],
  "count": 48,
  "page": 1,
  "per_page": 50
}
```

Error:
```json
{
  "error": "not_found",
  "message": "Project 25-97199 not found"
}
```

### What stays desktop-only

- Folder creation/moving/renaming
- File explorer integration
- Template copying
- var.json writes
- Window management

## Environment Isolation

Two environments, strict separation:

| Environment | SurrealDB | Database | Purpose |
|---|---|---|---|
| **Production** | 10.0.23.11:8000 | emittiv/projects | Real client data — never touched by tests |
| **Dev** | 10.0.23.12:8000 | emittiv/projects | Development, testing, experiments — safe to mutate |

### Production safety guard

Every test suite and integration test checks the database before running:

```typescript
// UI tests (TypeScript)
const dbInfo = await executeJs("window.__TAURI__.invoke('get_db_info')");
if (dbInfo.database === 'projects' && dbInfo.url.includes('10.0.23.11')) {
  throw new Error('REFUSING TO RUN: App is connected to PRODUCTION database');
}
```

```rust
// API integration tests (Rust)
assert!(
    config.database != "projects" || !config.url.contains("10.0.23.11"),
    "REFUSING TO RUN: Tests connected to production database"
);
```

If destructive test isolation is ever needed, a separate database within the dev container (`emittiv/projects_test`) can be created at that point.

## UI Validation Details

### What we validate

| Check | Method | Tauri MCP Tool |
|---|---|---|
| App launches, connects to DB | `execute_js` → check connection status store | `execute_js` |
| All 5 routes render | `execute_js` → navigate, then `get_dom` | `get_dom` |
| Data loads in lists | `execute_js` → read Svelte store counts | `execute_js` |
| Status filters show correct values | `get_dom` → check dropdown options | `get_dom` |
| Modals open and display fields | `execute_js` → trigger modal, `get_dom` → verify | Both |
| No console errors | `execute_js` → check captured errors | `execute_js` |
| Visual regression | `take_screenshot` → review | `take_screenshot` |

### Usage

```bash
# After making changes, with app running via `npm run tauri:dev`:
npm run test:e2e:smoke      # 30-second smoke test
npm run test:e2e:full       # Full suite (~2-3 min)
```

Tests require the desktop app to be running locally. The API has its own integration tests that run independently.

## Implementation Phases

### Phase 1 — UI Smoke Tests (immediate value)

1. Clean up existing `e2e-mcp/` scaffolding
2. Build smoke suite: app connects, routes render, data loads, status values correct
3. Add production safety guard
4. Wire up `npm run test:e2e:smoke`
5. Run against v0.13.1 to validate

### Phase 2 — Core Library Extraction

1. Create `crates/e-fees-core/`
2. Move domain models, enums, validation out of `src-tauri/src/db/mod.rs`
3. Refactor Tauri app to depend on `e-fees-core`
4. Verify all 87 Rust tests still pass

### Phase 3 — API Service

1. Create `e-fees-api/` with axum skeleton
2. Read-only routes (projects, fees, companies, contacts, stats, health)
3. API key auth, env-based config with standalone `.env`
4. Integration tests against dev DB
5. Dockerfile, deploy to Unraid

### Phase 4 — Write Operations & Integration

1. Add POST/PUT/DELETE to API
2. Status transition logic
3. Fee cloning
4. AILX/n8n integration points

Phase 1 is standalone — immediate value. Phases 2-3 can proceed together since core extraction feeds directly into the API build.

## Key Decisions

| Decision | Rationale |
|---|---|
| Rust/axum for API | Shares types with Tauri app via `e-fees-core` crate |
| Standalone service, not inside Tauri | Available 24/7, doesn't require desktop app running |
| Separate `.env` for API container | Easy to share, deploy, configure independently |
| Read-only first | Lower risk, proves the architecture before writes |
| Tauri MCP for UI testing | Only valid approach for Tauri apps (no browser testing) |
| Two environments only (prod/dev) | Dev already serves as test DB — no need for a third |
| Production safety guard | Hard-coded refusal to run tests against prod IP |
