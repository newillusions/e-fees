# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB v3 backend.

- **Version**: 0.12.10 (released on main)
- **Active Branch**: `feat/domain-model-restructure` (in worktree `.worktrees/domain-model-restructure`)
- **Database**: SurrealDB v3 @ ws://10.0.23.11:8000 (emittiv/projects)
- **Tests**: 90 Rust tests passing, svelte-check 0 errors, vitest 26/30 passing (4 pre-existing happy-dom failures)

## Domain Model Restructure — READY TO MERGE

### What Was Done
All 13 implementation tasks completed + code review fixes applied. The restructure introduces a **venue/engagement/fee** separation:

- **Venue** = persistent physical place (hotel, resort), no lifecycle status
- **Project** = engagement/bid cycle with lifecycle: Lead → RFP → Submitted → Awarded → Design → Construction → Completed (+ Lost, No Response, Cancelled, On Hold, Superseded)
- **Fee** = pricing document with its own lifecycle: Draft → Sent → Negotiation → Accepted → Rejected → No Response → Superseded

### Branch Commits (12 total on `feat/domain-model-restructure`)
```
e3b619e fix: address code review findings — status values, type alignment, cleanup
d1f0ca7 feat(db): add venue creation migration script from existing projects
01148f2 fix(types): update old status values to match new ProjectStatus and FeeStatus types
9c3f93c test: add venue tests, update status values in Rust backend
e39e44d feat(ui): update project and proposal pages for new status lifecycles
0e04f0b feat(ui): add Venues page with list view and create/edit modal
881d11a feat(frontend): update types, constants, and stores for domain model restructure
958b3d5 feat(backend): add venue CRUD operations and Tauri commands
fcb546b feat(types): add Venue struct and venue_id to Project
d4df196 feat(db): data migration — map statuses to new lifecycle
18c82d0 feat(db): add venue_id to projects, prepare status ASSERT updates
d6bc3ed feat(db): create venue table for domain model restructure
```

### Code Review Status
- Code review completed by `superpowers:code-reviewer`
- All Critical and Important issues fixed (commit `e3b619e`)
- Remaining suggestions (S1-S5) are minor/deferred — activity logging, index types, script comments, keyboard shortcuts

### What's Left Before Merge
1. **Merge to main** — user needs to decide: squash merge vs merge commit
2. **Run DB migration scripts** on production (10.0.23.11):
   - `scripts/migration/001-create-venue-table.surql` — creates venue table
   - `scripts/migration/002-add-venue-id-to-projects.surql` — adds venue_id field + updates status ASSERT
   - `scripts/migration/003-data-migration.surql` — maps old statuses to new ones
   - `scripts/migration/004-create-venues-from-projects.surql` — creates venue records (has commented-out UPDATE statements needing manual verification)
3. **Release** — version bump + `/release` after merge

### Design Documents
- **Design doc**: `docs/plans/2026-02-24-domain-model-restructure-design.md`
- **Implementation plan**: `docs/plans/2026-02-25-domain-model-restructure-plan.md`

## Key Technical Context

### Database Configuration
- **Production DB**: ws://10.0.23.11:8000 (ns: emittiv, db: projects)
- **Installed app config**: `~/Library/Application Support/com.emittiv.e-fees/.env`
- **Dev config**: `src-tauri/.env` (gitignored)
- **App logs**: `~/Library/Logs/com.emittiv.e-fees/E-Fees.log`

### New Status Definitions (Post-Restructure)
- **Project**: Lead, RFP, Submitted, Awarded, Design, Construction, Completed, Lost, No Response, Cancelled, On Hold, Superseded
- **Fee**: Draft, Sent, Negotiation, Accepted, Rejected, No Response, Superseded
- **FeeStage**: Draft, Prepared, Sent, Under Review, Clarification, Negotiation, Accepted, Rejected
- **Venue**: No status (persistent entity)

### Folder Mapping (Post-Restructure)
- `01 RFPs` → Lead, RFP, Submitted
- `11 Current` → Awarded, Design, Construction
- `99 Completed` → Completed
- `00 Inactive` → Lost, No Response, Cancelled, On Hold, Superseded

### SurrealDB v3 Gotchas (Accumulated Knowledge)
1. **`math::max([])` = `-Infinity` (float)**, not NULL. Use IF/ELSE guard.
2. **Binary protocol sends all ints as i64** — `i32` fields fail
3. **`#[serde(default)]` ignored by SurrealValue** — binary protocol skips serde attributes
4. **`serde_json::Value` can't handle native datetime/record** — use `surrealdb_types::Value` (DbValue)
5. **`SurrealValue` derive ignores `#[serde(rename)]`** — use `serde_json::Value` passthrough
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
| Frontend types | `src/types/database.ts` |
| Status constants | `src/lib/constants.ts` |
| Venue page | `src/lib/pages/Venues.svelte` |
| Venue modal | `src/lib/components/VenueModal.svelte` |
| Master CSS | `src/styles/app.css` |
| Excel export | `src-tauri/src/excel_export.rs` |
| Folder management | `src/lib/api/folderManagement.ts` |
| Migration scripts | `scripts/migration/001-004` |

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
*Updated: 2026-02-26*
