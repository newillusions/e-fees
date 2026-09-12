# E-Fees - Digital Fee Proposal Management System

Premium desktop application (Tauri v2 + Svelte 5) for managing fee proposals, projects, companies, and contacts, plus a standalone REST API and scope microservice. Emittiv brand design system, SurrealDB backend.

## 🚨 Critical Directives
- **ALL E2E testing MUST use the Tauri MCP server - NEVER browser-based tools.** Playwright/Puppeteer/Selenium do not work for Tauri apps. See `docs/testing/CRITICAL_DIRECTIVE_TAURI_MCP_ONLY.md`.
- **TDD is mandatory (Tier 1 project).** RED (failing test) → GREEN (minimum code) → REFACTOR. Applies to new features, bug fixes (reproduce first), refactors, API changes. Test before committing: `cargo test -p app --lib` (Rust), `npm test` (frontend).
- **Always ask before committing.** Daily work pushes ONLY to Forgejo (`origin` = forge.mms.name/emittiv/fee-prop). GitHub is CI-only - push there only for tagged releases.

## Architecture
| Component | What | Where |
|---|---|---|
| Desktop app | Tauri v2 + Svelte 5 UI, filesystem ops (folders, sync) | repo root + `src-tauri/` |
| e-fees-api | axum REST API - full CRUD, exports, OpenAPI | `e-fees-api/` → 10.0.21.80:3200 |
| e-fees-scope | clause library, scope generation, corpus | `e-fees-scope/` → 10.0.21.81:3201 |
| Shared domain models | single source of types | `crates/e-fees-core/` |
| Agent API | desktop-local axum, port 3100 | `src-tauri/src/agent_server.rs` |

Deploys: build on AI server via `ssh unraid-ai`, update via Unraid MCP `update_container` - never raw docker (see `/deploy-containers` skill). `.forgejo/workflows/build-containers.yml` builds e-fees-api and e-fees-scope on every push (matrix job, DinD runner); pushes `:latest` only on `main` - PRs build-only. e-fees-scope has its own `API_KEY` env, distinct from e-fees-api's `EFEES_API_KEY`.

## Tech Stack
- **Frontend**: Svelte 5 (TypeScript, `mount()` API), svelte-spa-router, Vite with HMR
- **Desktop**: Tauri v2 (Rust)
- **Database**: SurrealDB 3.1.2 server (WebSocket), Rust SDK pinned `=3.0.4`
- **Styling**: CSS custom properties (design tokens) + semantic `.emittiv-*` classes. **Tailwind was removed 2026-02 - do not reintroduce.** Desktop app uses fixed `px` values (OS handles DPI scaling at 150-200%), never `rem`.

## Emittiv Design System
```css
--black: #000     /* Primary bg */
--darker: #333    /* Secondary bg */
--dark: #666      /* Tertiary bg */
--light: #999     /* Light text */
--lighter: #ccc   /* Secondary text */
--white: #fff     /* Primary text */
--splash: #f90    /* Orange accent */
```
- **Fonts**: Ubuntu (headings), Montserrat (body)
- **Transitions**: 300ms cubic-bezier(0.4, 0, 0.2, 1)
- Use existing `.emittiv-input`, `.emittiv-select`, `.emittiv-btn` and friends from `app.css` before writing new styles.

## Database
- **Prod**: ws://10.0.23.11:8000 v3.1.2 (ns `emittiv`, db `projects`) | **Dev**: ws://10.0.23.12:8000 v3.1.4 (ns `emittiv_dev`, db `projects`). Dev runs a newer SurrealDB point release than prod - don't assume schema/behavior parity between the two without checking both directly.
- **Schema reference**: [DATABASE_SCHEMA.md](./docs/development/DATABASE_SCHEMA.md). Key tables: `projects`, `fee`, `company`, `contacts`, `country`, `currency`.
- **Project numbers**: YY-CCCNN (e.g. 26-97104 = 2026, UAE dial 971, sequence 04); sequence auto-increments per country/year; country resolved via `fn::resolve_country` (dial_code field).
- **Record keys** use underscore form (`26_97104`); display numbers use hyphens. API route `{id}` = the record key.

### Critical query patterns
- `type::record('table', $key)` for record-id parameterization - never `table:$key` inline.
- `query_bind_map()` for multi-param CREATE/UPDATE in the desktop client; raw `.query().bind()` in axum services.
- `option<T>` fields take `NONE`, never `NULL` - omit optional fields from SET when empty.
- SCHEMAFULL tables (`company`, `contacts`, `projects`, `country`, `currency`, `activity_log`, `scope_revision`) hard-error on undefined-field writes (3.1.2). `fee` + scope tables are SCHEMALESS.
- Export paths use `project.name_short` (canonical folder `{number} {name_short}`), never `project.name`.
- RecordId key extraction: `record_key_string(&id.key)`, not `.key()`.

## Development
```bash
npm install          # Install deps
npm run dev          # Frontend dev
npm run tauri:dev    # Full app dev
npm run tauri:build  # Production build
npm run check        # Type checking
```
- Svelte 5: `mount(App, { target })` - never `new App()`
- SVG assets: import with `?url` suffix; no spaces in filenames
- Docker image builds must `COPY Cargo.lock` (Dockerfile) or the diskann dep breaks the build

## Release
`/release [patch|minor|major]` runs the full pipeline (version bump, tag, CI build, manifest) via a **background haiku agent** - never interactively (CI takes 15-25 min; haiku polls cheaply). Intervene only on reported build failure. Update the `tauri.conf.json` `"title"` field with each release.
Key files: `.claude/commands/release.md` (pipeline), `scripts/sync-version.cjs`, `.github/workflows/build-releases.yml`.

## Conventional Commits
`<type>(<scope>): <description>` - types: feat, fix, docs, style, refactor, test, chore, perf.
Every commit ends with: `Co-Authored-By: Claude <model> <noreply@anthropic.com>`

## Session Workflow
1. `/lamp-on` - load KB context, sync, check hub messages
2. Work - capture key findings via `kb_observe`; keep the `e-fees` wiki page current when modules/architecture change
3. `/lamp-off` - save context, update `.claude/HANDOVER.md`, close KB session

## Standards
Follow [WORKSPACE_STANDARDS.md](/Volumes/base/dev/.claude/WORKSPACE_STANDARDS.md). Credentials live in `~/.claude/.credentials.env`; MCP configs use the `_FROM` wrapper pattern. Code is auto-formatted by PostToolUse hook (Prettier, rustfmt).

## Key References
| What | Where |
|---|---|
| Session handover | [.claude/HANDOVER.md](./.claude/HANDOVER.md) |
| Project rules | `.claude/rules/development-workflow.md` |
| Tacit judgment (incidents, gotchas) | `.claude/rules/judgment.md` |
| Release guide | [RELEASE_PROCESS.md](./RELEASE_PROCESS.md) |
| Known issues | [KNOWN_ISSUES.md](./KNOWN_ISSUES.md) |
| DB schema | [DATABASE_SCHEMA.md](./docs/development/DATABASE_SCHEMA.md) |
| Smoke tests | `/smoke-test` (Tauri MCP, app must be running) |
| Ship pipeline | `/sendit` (review → test → bump → tag → CI → verify) |

## Proposal rendering (fp-docbuilder / fp-template)

A fee record becomes a proposal PDF through one of two backends, chosen once
by `export::fp_render::select_proposal_backend`:

| Variable | Default | Meaning |
|---|---|---|
| `DOCBUILDER_URL` | none | Set = **production path**: POST the manifest to the remote fp-docbuilder service, which owns fill, gate, and render end to end. No `FP_TEMPLATE_ROOT`, python3, or browser needed on this host. Value once deployed: `http://10.0.21.85:8080`. |
| `DOCBUILDER_TOKEN` | none | Bearer token sent with every docbuilder request, when set. |
| `FP_TEMPLATE_ROOT` | none | **Local pipeline only** (ignored when `DOCBUILDER_URL` is set). fp-template checkout; `fill.py` and the three gates live there. Both unset gives 503. |
| `GOTENBERG_URL` | none | **Local pipeline only.** Set = render through gotenberg over HTTP. Unset = local `render.sh` and a local headless Chrome. No host default. Value: `http://10.0.21.83:3000`. |
| `GOTENBERG_TIMEOUT_SECS` | `60` | Per-request budget. |
| `GOTENBERG_TAGGED_PDF` | `true` | Send `generateTaggedPdf`, matching local Chromium. |

`POST /fees/{id}/fp-proposal` answers 422 when the document itself cannot be
issued or rendered as written (a blocked gate, a manifest failure, a missing or
colliding asset, a bundle the backend rejects) and 503 when the backend is
unconfigured or unavailable. Gating always happens before rendering, so a
blocked document never becomes a PDF.

As of 2026-09-06 the `e-fees-api` image carries no python3, no vendored
fp-template checkout, and no browser — the local pipeline (`FP_TEMPLATE_ROOT`,
`GOTENBERG_URL`) is code-complete and tested but development-only now.
`scripts/vendor-fp-template.sh` / `e-fees-api/fp-template.ref` still work for
that dev path; they are no longer part of the image build. Full detail,
including the docbuilder contract and the browser the local pipeline's FILL
step still needs:
[FP-TEMPLATE-INTEGRATION.md](./docs/development/FP-TEMPLATE-INTEGRATION.md).

## PUID/PGID (2026-09-12)

Both `e-fees-api` and `e-fees-scope` images follow Unraid's own convention
(docs.unraid.net "Managing and Customizing Containers") - same shape as
linuxserver.io images, `martin/pa` PR #275, and `martin/anchor` PR #16. Each
container starts as root, `docker-entrypoint.sh` remaps its baked-in
`efeesapi`/`efeesscope` user/group to the `PUID`/`PGID` env vars (default
`99:100`, Unraid's `nobody:users`), chowns the writable mount, then drops
privileges via `gosu` before running the app. Both templates ship both
variables with those defaults - leave them as-is unless the appdata path is
meant to be owned by a different uid/gid on the host.

| Image | Chowned (writable) | Skipped (read-only, never chowned) |
|---|---|---|
| e-fees-api | `/config` (excl. nested `/config/app.env`) | `/config/app.env`, `/root/.ssh`, `/app/source` |
| e-fees-scope | `/config` (excl. nested `/config/app.env`) | `/config/app.env`, `/data/rfps`, `/app/source` |

**Nextcloud SSH key (e-fees-api only):** the optional "SSH Keys" mount
(`/root/.ssh`, typically the host's own `/root/.ssh`) is READ-ONLY and
host-root-owned, so it can never be chowned to the runtime PUID/PGID - once
privileges drop, the folder-export pipeline's `ssh` subprocess (`src/ssh.rs`)
could no longer open a uid-0-owned mode-600 key. Fix, same pattern as
`martin/pa` PR #275: `docker-entrypoint.sh` copies `id_ed25519` (+
`known_hosts` if present) out to a PUID/PGID-owned scratch dir at
`/run/efees-ssh` before the `gosu` exec (see `e-fees-api/copy-ssh-key.sh` /
`copy-ssh-key.test.sh`). `FolderConfig::ssh_key`'s default (`src/config.rs`)
now points at `/run/efees-ssh/id_ed25519` instead of the raw mount - an
explicit `NC_SSH_KEY` env var override is still honoured.

**Pre-existing, unrelated to this change:** `templates/e-fees-api.xml`
defines the template variable as `NC_SSH_KEY_PATH`, but `src/config.rs` reads
`NC_SSH_KEY` - the two names don't match, so the template's SSH-key-path
field has never actually overridden the code's hardcoded default. Not fixed
here (out of scope for PUID/PGID); flagged as a follow-up.

There is no `HEALTHCHECK` instruction in either Dockerfile today - the
Unraid template's WebUI probe is external, so this doesn't interact with the
PUID/PGID change (a future `HEALTHCHECK` would run as the image `USER`, i.e.
root, since neither Dockerfile sets a `USER` line).

## Known Scalability Limits
1. **Database mutex** - `src-tauri/src/db/mod.rs` serializes DB operations (RwLock today; revisit if concurrency grows).
2. **Client-side joins** - `src/lib/stores.ts` joins company names in the frontend (O(1) Map lookups today; revisit at larger datasets).

---
**Last Updated**: 2026-09-06 (docbuilder client - production proposal rendering moved to the remote fp-docbuilder service via DOCBUILDER_URL; local fill+gate+render pipeline kept as a dev-only fallback; e-fees-api image no longer bakes in python3/fp-template)
**Version**: 0.18.1
