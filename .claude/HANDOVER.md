# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB v3 backend. **v0.12.10** released.

- **Version**: 0.12.10 (released, tag v0.12.10, Forgejo + GitHub)
- **Branch**: `main`
- **Database**: SurrealDB v3 @ ws://10.0.23.11:8000 (emittiv/projects)
- **Tests**: 87 Rust tests passing, 2 ignored (integration tests requiring `--ignored` flag)

## Last Session (2026-02-24)
**Summary**: Released v0.12.10, then deep-dived into domain model restructure. Active Fees pill shows 6 but the status definitions are inconsistent — led to a full entity model review.

### Domain Model Restructure (IN PROGRESS — Design Phase)
- **Design doc**: `docs/plans/2026-02-24-domain-model-restructure-design.md`
- **Approach B chosen**: Introduce `venue` entity, refine `project` as "engagement", clean up fee statuses
- **Key insight**: Three concepts conflated — venue (physical place), engagement (bid cycle), fee (pricing doc)
- **Venue** = persistent place, no status. **Project** = engagement with lifecycle. **Fee** = proposal with its own lifecycle.
- **New project statuses**: Lead → RFP → Bidding → Awarded → Active → Completed (+ Lost, Cancelled, On Hold)
- **New fee statuses**: Draft → Sent → Negotiation → Accepted → Rejected → Superseded
- **8 open questions** in the design doc for morning review
- **Future path**: Evolve toward graph model (Approach C) using SurrealDB graph features when justified

### Release v0.12.10
- Background haiku agent ran full pipeline (~29 min)
- 8 assets on Forgejo release
- Dashboard shows 6 active fees (3 Awarded + 3 Revised) — but this exposed the status definition problem that led to the restructure discussion

## Next Steps (Priority Order)
1. **Review domain model design** — `docs/plans/2026-02-24-domain-model-restructure-design.md` — answer 8 open questions
2. **Write implementation plan** — Once design approved, use writing-plans skill
3. **Multi-currency hover** — AED equivalents on hover when quoting in foreign currency
4. **Verify Windows install** — Test NSIS hook cleanup of old per-user install

## Key Technical Context

### Database Configuration
- **Production DB**: ws://10.0.23.11:8000 (ns: emittiv, db: projects)
- **Installed app config**: `~/Library/Application Support/com.emittiv.e-fees/.env`
- **Dev config**: `src-tauri/.env` (gitignored)
- **App logs**: `~/Library/Logs/com.emittiv.e-fees/E-Fees.log`

### Fee Status Values (PROD)
- **Active (in-play)**: Draft, Active, Sent, Negotiation, Awarded, Revised
- **Inactive**: Completed, Lost, On Hold, Cancelled
- **Current PROD distribution**: Awarded=3, Completed=4, Lost=21, Revised=3

### SurrealDB v3 Gotchas (Accumulated Knowledge)
1. **`math::max([])` = `-Infinity` (float)**, not NULL. Use IF/ELSE guard.
2. **Binary protocol sends all ints as i64** — `i32` fields fail with "Expected number, got number"
3. **`#[serde(default)]` ignored by SurrealValue** — binary protocol skips all serde attributes
4. **`serde_json::Value` can't handle native datetime/record** — use `surrealdb_types::Value` (DbValue)
5. **`SurrealValue` derive ignores `#[serde(rename)]`** — use `serde_json::Value` passthrough for renamed fields
6. **Can't UPDATE a record if existing value fails type coercion** — must REMOVE FIELD, fix data, re-DEFINE

### Release Process (MANDATORY)
Always use `/release` via a **background haiku agent**. Never run interactively.

### Key Files
| Purpose | Location |
|---------|----------|
| DB entity types | `src-tauri/src/db/types.rs` |
| DB operations | `src-tauri/src/db/operations.rs` |
| Integration tests | `src-tauri/src/db/tests.rs` |
| Frontend stores | `src/lib/stores.ts` |
| Master CSS | `src/styles/app.css` |
| Excel export | `src-tauri/src/excel_export.rs` |
| Release command | `.claude/commands/release.md` |
| Window config | `src-tauri/tauri.conf.json` |

### Critical Rules
1. **Screenshots**: Peekaboo MCP with `app_target: "app"` — NEVER browser tools for Tauri
2. **Dev command**: `npm run tauri:dev` (not `npm run dev`)
3. **CSS**: Semantic `.emittiv-*` classes, NOT utility strings > 50 chars
4. **Fixed px values**: Desktop app with OS-level scaling, never use rem
5. **Process safety**: NEVER pkill without permission
6. **Git**: Push to Forgejo (origin) for daily work. GitHub only for tagged releases.
7. **Releases**: ALWAYS background haiku agent. Never interactive polling.
8. **Test DB**: Must match the installed app's DB (10.0.23.11), not the old IP.

---
*Updated: 2026-02-24*
