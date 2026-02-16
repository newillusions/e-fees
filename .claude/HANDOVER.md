# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB backend. **v0.11.0 released** + code review completed + CSS restructure completed.

- **Version**: 0.11.0 (released 2026-02-13)
- **Branch**: `main` (commit cc7e47d, pushed to origin)
- **Database**: SurrealDB @ ws://10.0.21.8:8000 (emittiv/projects)
- **Tests**: 86 Rust tests passing, 8 pre-existing svelte-check warnings

## Last Session (2026-02-16)
**Summary**: Completed full CSS restructure (phases 2-7). Migrated 30+ components from utility class strings to semantic `.emittiv-*` classes. Pruned 161 unused utility definitions. Fixed build-breaking duplicate variant attributes.

### CSS Restructure Results
- **37 files changed**, 765 insertions, 457 deletions
- **CSS**: 85.16 KB → 82.04 KB (-3.7%)
- **JS**: 497.47 KB → 487.67 KB (-2.0%)
- Added 15 new semantic classes: empty-state__icon, card-title, card-meta, chip, alert variants, badge variants, detail-panel, etc.
- Pruned 161 unused utility class definitions from `@layer utilities`
- Fixed duplicate `variant` attributes in ProposalModal, CrudModal, ProjectModal
- Removed 17 redundant `className` overrides on Button components

## Next Steps (Priority Order)
1. **Run the import agent** — `docs/agents/excel-import-agent.md` to import ~60 Excel pricing files
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
| Code Review Findings | `docs/code-review/CODE_REVIEW_FINDINGS_2026-01.md` |
| Excel template export | `src-tauri/src/excel_export.rs` |
| DB operations | `src-tauri/src/db/operations.rs` |
| ProposalModal (largest) | `src/lib/components/ProposalModal.svelte` |
| Build workflow | `.github/workflows/build-releases.yml` |

---
*Updated: 2026-02-16*
