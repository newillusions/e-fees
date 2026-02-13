# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB backend. Database successfully migrated to new server (10.0.21.8). Import wizard and Agent API server built and tested. Fee CRUD endpoints complete. Agent API network-accessible with X-API-Key auth. MCP E2E test suite with 4 test files (48 tests).

- **Version**: 0.10.27
- **Branch**: `feat/fee-pricing-calculator` (commit 43d43eb)
- **Database**: SurrealDB @ ws://10.0.21.8:8000 (emittiv/projects) — **MIGRATION COMPLETE**
- **Import Wizard**: Complete — reads RFPs JSON exports, imports into SurrealDB
- **Agent API**: Complete — HTTP server with full project, fee, company, contact CRUD + X-API-Key auth
- **Pricing Module**: Withholding tax fix complete, stage rounding working
- **Integration**: Phase 1 import wizard + agent API are the key ecosystem bridges

## Last Session (2026-02-13)
**Summary**: Completed SurrealDB migration from 10.0.1.17 to 10.0.21.8. Updated all critical configuration files (.mcp.json, claude.md, HANDOVER.md, MCP config docs, SQL scripts). Documented EFEES_AGENT_API_KEY usage (optional API key for Agent API authentication via X-API-Key header).

### Files Updated This Session
- `.mcp.json` — SurrealDB MCP server URL (http://10.0.21.8:8000)
- `claude.md` — Database Configuration section
- `.claude/HANDOVER.md` — Database URL, migration context, next steps
- `scripts/add-indexes.surql` — CLI command example
- `.claude/MCP-SERVERS.md` — Remote database option
- `.claude/MCP-CONFIG.md` — SurrealDB config table and JSON example

### Agent API Endpoints
| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| GET | `/api/health` | No | Health check (version, DB status, uptime) |
| GET | `/api/stats` | Yes | Entity counts (projects, companies, contacts, fees) |
| GET | `/api/projects` | Yes | List all projects |
| GET | `/api/projects/:id` | Yes | Get project by ID |
| POST | `/api/projects` | Yes | Create project |
| GET | `/api/fees` | Yes | List fees (optional ?project_id= filter) |
| GET | `/api/fees/:id` | Yes | Get fee by ID |
| POST | `/api/fees` | Yes | Create fee |
| POST | `/api/fees/:id/export` | Yes | Export fee as .xlsx |
| GET | `/api/companies` | Yes | List all companies |
| POST | `/api/companies` | Yes | Create company |
| GET | `/api/contacts` | Yes | List contacts (optional ?company_id= filter) |
| GET | `/api/contacts/:id` | Yes | Get contact by ID |
| POST | `/api/contacts` | Yes | Create contact |

### Auth Configuration
- `EFEES_AGENT_BIND`: Bind address (default: `0.0.0.0:3100`)
- `EFEES_AGENT_API_KEY`: Optional env var for X-API-Key authentication
  - If set: All endpoints except /api/health require `X-API-Key` header
  - If not set: All requests allowed (backwards compatible)
  - Used in: `src-tauri/src/agent_server.rs` (middleware), `e2e-mcp/tests/*.mcp.ts` (tests)
  - No default value — user must generate and set if auth desired
- 401 response: `{"error": "Unauthorized: missing or invalid X-API-Key"}`

## Next Steps
1. **Add exchange rate/conversion** to PricingConfig (AED base, configurable factor per proposal)
2. **Build Excel export** (Rust xlsx library, match RFPs client-facing template)
3. **Test pricing workflow** end-to-end
4. **Fix pre-existing project-crud.mcp.ts** — requires Tauri MCP tools (screenshot, SurrealDB MCP) which fail in test environment

## Key Technical Context

### Tech Stack
- **Frontend**: Svelte 5 with TypeScript, TailwindCSS
- **Backend**: Tauri v2 (Rust) + Axum HTTP server
- **Database**: SurrealDB @ ws://10.0.21.8:8000 (emittiv/projects)
- **Agent API**: http://0.0.0.0:3100/api/ (configurable via EFEES_AGENT_BIND, auth via EFEES_AGENT_API_KEY)
- **Design**: Emittiv brand palette (black/orange theme)

### Critical Rules
1. **Screenshots**: Use Peekaboo MCP with `app_target: "app"` - NEVER Playwright for Tauri apps
2. **UI Automation**: NEVER use cliclick/osascript - interrupts user's work
3. **Dev command**: Use `npm run tauri:dev` (not `npm run dev`)
4. **CSS**: All styling through master CSS classes in `app.css` - no inline Tailwind > 50 chars
5. **Fixed px values**: Desktop app with OS-level scaling, never use rem
6. **Process safety**: NEVER pkill without permission

### Withholding Tax
- Withholding does NOT inflate proposal totals -- it's informational only in proposals
- Gross-up happens at invoice time: `invoiced = quoted / (1 - rate)`
- Full reference: `docs/WITHHOLDING_TAX.md`

### DB Migration Context
- **COMPLETED (2026-02-13)**: Migrated to ws://10.0.21.8:8000 (ns:emittiv, db:projects)
- Old IP (10.0.1.17) is now unreachable
- Config is fully env-driven: `DatabaseConfig::from_env()` reads SURREALDB_URL, SURREALDB_NS, etc.
- Also configurable via Settings UI in the app
- Data volume: ~48 projects, ~37 rfps, ~19 companies, ~250 countries, ~180 currencies
- Note: 38+ documentation files still reference old IP, but most are archived/test fixtures (not production-critical)

### Emittiv Ecosystem Context
- **Hub Topic**: emittiv-ecosystem (5 members: ailx, e-fees, lx-specs, rfps, dev)
- **Integration Design**: /Volumes/base/dev/docs/plans/2026-02-07-ecosystem-integration-design.md
- **API Contract**: /Volumes/base/dev/docs/plans/ailx-to-efees-api-contract.md
- **Phase 1 Timeline**: Weeks 1-3 (schema alignment + migration prep)
- **Dependencies**: RFPs JSON export format is now consumed by import wizard
- Acting as E-Fees instance means using `acting_as: "e-fees"` parameter on all KB tools

### Key Files
| Purpose | Location |
|---------|----------|
| DB config (Rust) | `src-tauri/src/db/config.rs` (DatabaseConfig::from_env) |
| Agent server (Rust) | `src-tauri/src/agent_server.rs` |
| Agent client (TS) | `src/lib/api/agent.ts` |
| Agent tests | `src/lib/api/agent.test.ts` |
| MCP E2E config | `vitest.e2e-mcp.config.ts` |
| MCP E2E tests | `e2e-mcp/tests/*.mcp.ts` (4 files, 48 tests) |
| MCP test fixtures | `e2e-mcp/fixtures/test-data-safe.ts` |
| MCP cleanup utils | `e2e-mcp/fixtures/cleanup-utilities.ts` |
| Import wizard (Rust) | `src-tauri/src/commands/import_wizard.rs` |
| Import wizard (API) | `src/lib/api/import.ts` |
| Import wizard (UI) | `src/lib/components/ImportWizard.svelte` |
| Import wizard (tests) | `src/lib/api/import.test.ts` |
| Pricing panels | `src/lib/components/pricing/*.svelte` |
| Stage types & defaults | `src/types/database.ts` (Stage, PricingConfig, DEFAULT_DESIGN_STAGES) |
| Pricing calculation | `src/types/database.ts:calculatePricingTotals()` |
| Withholding tax docs | `docs/WITHHOLDING_TAX.md` |
| Format utilities | `src/lib/utils/format.ts` (includes `roundToIncrement`) |
| Fee API | `src/lib/api/fees.ts` (CRUD + JSON export) |
| Master CSS | `src/styles/app.css` |

---
*Updated: 2026-02-13*
