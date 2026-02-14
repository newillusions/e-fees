# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB backend. **v0.11.0 released** with fee pricing calculator, multi-currency support, WHT gross-up, and pricing persistence. All platforms built and published to Forgejo.

- **Version**: 0.11.0 (released 2026-02-13)
- **Branch**: `main` (commit 153f649)
- **Database**: SurrealDB @ ws://10.0.21.8:8000 (emittiv/projects)
- **Pricing Module**: Calculator integrated into proposals, persistence working
- **Auto-updater**: update.json on GitHub, artifacts on Forgejo

## Last Session (2026-02-13)
**Summary**: Released v0.11.0. Pushed to GitHub, triggered CI build (macOS aarch64/x86_64 + Windows). Forgejo upload failed due to stale GITEA_TOKEN. Manually uploaded 6 artifacts. Created new Forgejo API token and updated GitHub secret. Updated local credentials.

### Key Changes in v0.11.0
- Fee pricing calculator with multi-currency and WHT tax support
- Live exchange rates from ECB with rate locking
- Discipline codes (LX, VID, AUD, CTL) and stage codes (CON, SD, DD, CD)
- WHT gross-up display with per-cell hover tooltips
- Payment schedule improvements
- Pricing persistence fix (Rust Fee struct + SurrealDB datetime fix)

### Fixes Applied
- **SurrealDB datetime**: Two-statement query with `SET time.updated_at = time::now()` (not string)
- **Rust Fee struct**: Added `Option<serde_json::Value>` for pricing, post_contract_items, reimbursable_costs, payment_schedule
- **Forgejo tokens**: Old Gitea tokens replaced. New `github-actions-releases` token with `repository:read+write` scope. Updated `GITEA_TOKEN` GitHub secret.

## Next Steps
1. **Update window title** — `tauri.conf.json` line 15 still says "v0.10.25"
2. **Clean up feature branch** — `feat/fee-pricing-calculator` merged, can delete
3. **Build Excel export** — Rust xlsx library, match RFPs client-facing template
4. **Multi-currency hover** — Show AED equivalents on hover when quoting in foreign currency
5. **Test pricing workflow** end-to-end with real proposal data

## Key Technical Context

### Build & Release Process
1. Bump version in package.json, tauri.conf.json, Cargo.toml
2. Push main to GitHub + create `v*` tag → triggers GitHub Actions
3. Builds macOS (aarch64 + x86_64) + Windows
4. Artifacts auto-upload to Forgejo release via `GITEA_TOKEN` secret
5. `update.json` generated with signatures, pushed to both remotes
6. Existing installs check `raw.githubusercontent.com/.../update.json`
7. Download URLs point to `forge.mms.name` releases

### Critical Rules
1. **Screenshots**: Use Peekaboo MCP with `app_target: "app"` - NEVER Playwright for Tauri apps
2. **Dev command**: Use `npm run tauri:dev` (not `npm run dev`)
3. **CSS**: All styling through master CSS classes in `app.css` - no inline Tailwind > 50 chars
4. **Fixed px values**: Desktop app with OS-level scaling, never use rem
5. **Process safety**: NEVER pkill without permission

### Key Files
| Purpose | Location |
|---------|----------|
| DB config (Rust) | `src-tauri/src/db/config.rs` |
| Fee entities (Rust) | `src-tauri/src/db/entities.rs` (pricing fields line 123-152) |
| Fee update (Rust) | `src-tauri/src/db/client.rs` (two-statement query line 468-502) |
| Pricing panels | `src/lib/components/pricing/*.svelte` |
| ProposalModal | `src/lib/components/ProposalModal.svelte` (pricing integration) |
| Pricing types | `src/types/database.ts` (Stage, PricingConfig, calculatePricingTotals) |
| WHT docs | `docs/WITHHOLDING_TAX.md` |
| Master CSS | `src/styles/app.css` |
| Build workflow | `.github/workflows/build-releases.yml` |
| Update manifest | `update.json` |

---
*Updated: 2026-02-13*
