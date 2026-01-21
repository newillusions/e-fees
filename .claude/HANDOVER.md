# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB backend.
- **Version**: 0.10.24 (Production-ready)
- **Auto-Updater**: Working on macOS
- **KB Integration**: Active (v3.0.0)

## Last Session (2026-01-18)
- Fixed network/DNS configuration for dev database access
- Added `surreal-dev.internal` DNS override in OPNsense Unbound
- Dev database now accessible via hostname
- Mac WiFi moved to trusted SSID ("mrandmrs") for proper routing

## Known Issues

### macOS Dev Mode Subnet Access
The Tauri app in dev mode has issues accessing IP addresses outside the app's subnet. This affects SurrealDB connections when the database is on a different VLAN (10.0.23.x) than the dev machine.

**Workarounds:**
1. Use hostname instead of IP if DNS resolves correctly
2. Test in release builds which may have different network permissions
3. Consider running SurrealDB locally for development

## Key Technical Context

### Tech Stack
- **Frontend**: Svelte 5 with TypeScript, TailwindCSS
- **Backend**: Tauri v2 (Rust)
- **Database**: SurrealDB @ http://surreal-dev.internal:8000 (emittiv/projects)
- **KB Database**: SurrealDB @ http://10.0.23.11:8000 (kb/knowledge)
- **Design**: Emittiv brand palette (black/orange theme)

### Critical Rules
1. **Testing**: MUST use Tauri MCP server - browser tools (Playwright/Puppeteer) don't work
2. **Git**: Dual remotes - push to both `origin` (Gitea) and `github` (GitHub)
3. **Releases**: Use `./scripts/publish-release.sh VERSION`

### Database Tables
- `projects` (48 records) - Project opportunities
- `fee` (37 records) - Fee proposals (renamed from `rfp`)
- `company` (19 records) - Client companies
- `contacts` - Contact persons linked to companies
- `country`, `currency` - Reference data

### Known Scalability Issues (from SITREP)
1. **Database Mutex** - Single connection bottleneck in `db/mod.rs`
2. **No Pagination** - All data loaded at once in `stores.ts`
3. **Client-side Joins** - N+1 query pattern with derived stores

## Recent Features (v0.10.x)
- Unified search with normalized matching
- Activity logging system
- Folder sync feature (in progress)
- Project folder links with status-based subfolder paths

## Next Steps
1. Complete folder sync testing and commit
2. Implement pagination (scalability priority)
3. Add database connection pooling
4. Continue feature development per SITREP roadmap

## Key Files
| Purpose | Location |
|---------|----------|
| Main instructions | `CLAUDE.md` |
| Architecture analysis | `docs/planning/SITREP_2025-12-06.md` |
| Release workflow | `RELEASE_PROCESS.md` |
| Known issues | `KNOWN_ISSUES.md` |
| DB commands | `src-tauri/src/commands/mod.rs` |
| Frontend stores | `src/lib/stores.ts` |
| API layer | `src/lib/api.ts` |

## KB Commands
- `/lamp-on` - Load context at session start
- `/lamp-off` - Save learnings at session end
- `/kb <query>` - Search knowledge base
- `/kb-save` - Save specific observation
- `/kb-pause` - Pause KB capture temporarily
- `/handoff` - Generate handoff notes for another session

---
*Updated: 2026-01-18*
