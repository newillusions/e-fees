# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB backend. **v0.11.0 released** + code review completed + CSS restructure completed + Excel pricing import done + export save dialog added.

- **Version**: 0.11.0 (released 2026-02-13)
- **Branch**: `main` (commit aaaa8d4, pushed to origin)
- **Database**: SurrealDB @ ws://10.0.21.8:8000 (emittiv/projects)
- **Tests**: 86 Rust tests passing, 8 pre-existing svelte-check warnings

## Last Session (2026-02-16)
**Summary**: Excel pricing import (27 fees from 78 Excel files), post-import audit with 10 fixes, CSS visual verification, and Excel export save dialog implementation.

### Excel Pricing Import
- Imported pricing data from 78 Excel files into 27 fee records
- Post-import audit found and fixed 10 data issues in SurrealDB
- Validation report in `docs/IMPORT_VALIDATION_REPORT.md`
- Commit: 7576ced

### Excel Export — Save Dialog Fix
- **Problem**: Export went to macOS temp dir, no save dialog, modal text overflow
- **Fix (commit aaaa8d4)**: Native save dialog via `@tauri-apps/plugin-dialog`
- **Files changed**: export.rs, types.rs, revisions.ts, ProposalDetail.svelte, WarningModal.svelte
- **Serde fix**: `PricingConfig`/`PricingBreakdown` now have `#[serde(default)]` + `Default` derive — fixes deserialization of imported data with partial config fields
- **Status**: Save dialog confirmed working (screenshot verified), but needs **clean rebuild to test end-to-end** — the dev session used a cached binary without the Rust changes. File still went to temp dir during testing because old binary was running.

### MUST VERIFY TOMORROW
- Start fresh `npm run tauri:dev` (ensures Rust changes are compiled)
- Click Export on a proposal with pricing (e.g., 25-96501-FP-1)
- Verify native save dialog appears AND file saves to chosen location
- Verify filename format: `25-96501-FP-1-00 Pricing.xlsx` (no `-FP-` duplication)

### CSS Restructure Visual Verification
- All pages verified via Peekaboo screenshots: Dashboard, Projects, Companies, Contacts, Proposals, Proposal Detail, Pricing Calculator, Dev Mode
- All rendering correctly with semantic CSS classes

## Next Steps (Priority Order)
1. **Verify export save dialog** — Clean rebuild + end-to-end test (see above)
2. **Update window title** — `tauri.conf.json` line 15 still says "v0.10.25"
3. **Multi-currency hover** — AED equivalents on hover when quoting in foreign currency
4. **Version bump + release** — v0.12.0 with all recent improvements

## Key Technical Context

### Build & Release Process
1. Bump version in package.json, tauri.conf.json, Cargo.toml
2. Push main to GitHub + create `v*` tag → triggers GitHub Actions
3. Builds macOS (aarch64 + x86_64) + Windows
4. Artifacts auto-upload to Forgejo release via `GITEA_TOKEN` secret

### CSS Architecture (Post-Restructure)
- `app.css` has 3 layers: `@layer base`, `@layer components`, `@layer utilities`
- ~130 `.emittiv-*` semantic classes in components layer
- ~490 utility lines remaining (layout primitives, spacing, flex)
- 16 CSS custom properties (`--emittiv-*`) for design tokens
- All values in fixed `px` (desktop app, OS handles DPI scaling)

### Critical Rules
1. **Screenshots**: Peekaboo MCP with `app_target: "app"` — NEVER browser tools for Tauri
2. **Dev command**: `npm run tauri:dev` (not `npm run dev`)
3. **CSS**: Semantic `.emittiv-*` classes, NOT utility strings > 50 chars
4. **Fixed px values**: Desktop app with OS-level scaling, never use rem
5. **Process safety**: NEVER pkill without permission
6. **Git**: Push to Forgejo (origin) for daily work. GitHub only for tagged releases.

### Key Files
| Purpose | Location |
|---------|----------|
| CSS Restructure Plan | `docs/CSS_RESTRUCTURE_PLAN.md` |
| Master CSS | `src/styles/app.css` |
| Excel export commands | `src-tauri/src/commands/export.rs` |
| Excel template generator | `src-tauri/src/excel_export.rs` |
| DB types (serde) | `src-tauri/src/db/types.rs` |
| ProposalDetail (export handler) | `src/lib/components/ProposalDetail.svelte` |
| Build workflow | `.github/workflows/build-releases.yml` |

---
*Updated: 2026-02-16*
