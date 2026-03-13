# E-Fees Project Handover

## Current Status
v0.13.6 released. Clause library refined to 20 clauses aligned with InDesign template. Assumptions review documented as ongoing topic.

- **Version**: 0.13.6 (released 2026-03-09)
- **Branch**: main
- **Database**: SurrealDB @ ws://10.0.23.11:8000 (emittiv/projects)
- **Tests**: 633/633 passing (frontend), 62/62 passing (API integration), 24/24 passing (scope)
- **API**: e-fees-api v0.3.1 deployed at 10.0.21.80:3200
- **Scope**: e-fees-scope v0.2.3 deployed at 10.0.21.81:3201

## Last Session
**Date**: 2026-03-13
**Summary**: Built the `/sendit` ship-it skill — full autonomous release pipeline running as a background agent with timed CI checkins, code review subagent, and post-publish verification.

### Accomplished
- Created `.claude/skills/sendit/SKILL.md` — arg parsing, background agent spawn instructions
- Created `.claude/skills/sendit/references/agent-prompt.md` — 12-step pipeline (pre-flight → commit → full code review → version bump → push → tag → CI poll loop → update.json sync → Forgejo release verify → KB observation → report)
- Code review step spawns a sonnet subagent checking bugs, security, structure, red-team concerns — BLOCK verdict halts the pipeline before the tag is pushed
- CI poll loop logs timed checkins every 60s with job-level status, 40min timeout
- Dry-run test passed: all pre-flight checks green, version detection correct, DRY-RUN steps printed cleanly

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 (ns: emittiv, db: projects) |
| Dev DB | ws://surreal-dev.internal:8000 (10.0.23.12) |
| API Container | 10.0.21.80:3200 (br0, e-fees-api:v0.3.1) |
| Scope Container | 10.0.21.81:3201 (br0, e-fees-scope:v0.2.3) |
| Scope API Key | efees-scope-2026-s7k2m9xp |
| Forgejo repo | forge.mms.name/emittiv/fee-prop |
| InDesign MCP repo | /Volumes/base/dev/indesign-uxp-server |
| NC project create script | /mnt/user/appdata/scripts/nc-project-create.sh (on Primary 10.0.20.12) |
| Wiki page | slug: "e-fees" |
| Scope schema | e-fees-scope/schema.surql |
| Corpus PDFs | /mnt/user/appdata/e-fees-scope/rfps/ (51 files on AI server) |
| Template markdown | /tmp/template-full.md (converted from InDesign PDF) |

## Architecture
- **Desktop app** (Tauri): Full CRUD, filesystem ops, multi-currency display
- **Shared core** (`crates/e-fees-core/`): Domain types shared between desktop, API & scope
- **Standalone API** (`e-fees-api/`): Full CRUD HTTP, auto-numbering, OpenAPI/Swagger
- **Scope service** (`e-fees-scope/`): Clause library, corpus ingestion, scope generation with LLM polish
- **InDesign MCP** (local): UXP bridge for Claude Code <-> InDesign DOM
- **Nextcloud sync**: Group folder on Primary, syncs to Windows clients

## Next Steps
1. **Assumptions clause refinement** — ongoing review, cross-reference candidates against existing clauses to find genuine gaps (`docs/plans/assumptions-review.md`)
2. **Automate InDesign text variable population** via MCP — set all 21 variables from fee record
3. **Automate InDesign table population** — map PricingBreakdown to 5 pricing tables
4. **Scope text insertion** into InDesign — pipe scope service output to InDesign text frames
5. **Expose folder creation via API** — API on AI server SSH to Primary to run nc-project-create.sh
6. **Stablecoin research** — e-dirham vs USDT, counterfeiting concerns (deferred to separate session)
8. **Scope service networking** — unreachable from AI server host despite container running (works from Mac)

## Critical Rules
1. **SUPERPOWERS SKILLS MANDATORY**: Invoke relevant skill BEFORE any work. No exceptions.
2. **TDD NON-NEGOTIABLE**: Write failing tests FIRST, then implement. Always.
3. **Screenshots**: Peekaboo MCP with `app_target: "app"` — NEVER browser tools for Tauri
4. **Dev command**: `npm run tauri:dev` (not `npm run dev`)
5. **CSS**: Semantic `.emittiv-*` classes, NOT utility strings > 50 chars
6. **Fixed px values**: Desktop app with OS-level scaling, never use rem
7. **Git**: Push to Forgejo (origin) for daily work. GitHub only for tagged releases.
8. **Releases**: ALWAYS background haiku agent. Never interactive polling.
9. **SurrealDB v3 NULL**: Never bind `json!(None)` — omit optional fields from SET clause entirely.
10. **Fee issue_date**: YYYYMM format (6-digit numeric string per DB ASSERT).
11. **Scope fee queries**: OMIT id, backtick-quote keys, bind Value not String, use FLEXIBLE for nested objects.
12. **Scope integration tests**: Run with `--test-threads=1` (shared DB state).
13. **API/Scope redeploy**: rsync to AI server, docker build, stop/rm/run with same env.

## Key Learnings This Session
- **Business terminology**: emittiv is a specialist sub-consultant to LDCs — "Construction Supervision" not "Construction Administration". Approving variations is outside scope (architect/LDC role).
- **Template structure**: Design Phase Notes, Post Contract Phase Notes, and Assumptions are three completely separate sections (not one conflated section).
- **Basis of Appointment hierarchy**: H1 heading with Limitation of Liability and Next Steps as H3/H4 subsections — not separate clauses.
- **Landscape lighting**: Often in scope — NOT safe to default-exclude in assumptions.
- **Revit/BIM**: Often part of scope when agreed during bidding — NOT safe to default-exclude.
- **Assumptions are sparse**: Only 9 bullets across 51 historical proposals — confirms they're project-specific and often skipped.

---
*Updated: 2026-03-13*
