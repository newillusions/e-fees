# SurrealDB v3.0.4 Upgrade — Dependency Pinning & Cleanup

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pin all SurrealDB Rust SDK dependencies to 3.0.4, clean up unused JS SDK packages, verify builds and tests pass, and fix the KB plugin offline issue.

**Architecture:** The e-fees project has 4 Rust crates that depend on SurrealDB (`e-fees-core`, `src-tauri`, `e-fees-api`, `e-fees-scope`). The frontend communicates with SurrealDB exclusively through Tauri IPC (Rust backend) — the JS SDK packages in `package.json` are unused in production code. The KB plugin's "offline" status is from a stale MCP server process, not a code issue.

**Tech Stack:** Rust (surrealdb crate 3.0.x, surrealdb-types 3.0.x), Node.js (surrealdb.js, surrealdb npm packages), SurrealDB server v3.0.4

---

## Context & Findings

### SurrealDB Server Versions
| Instance | IP | Version | Role |
|----------|-----|---------|------|
| KB | 10.0.21.15:8000 | 3.0.0 | KB observations |
| Primary | 10.0.23.11:8000 | **3.0.4** | E-Fees production data |

### v3.0.4 Changelog — Relevant Fixes
- `search::score()` now returns real BM25 scores (was 0.0 in 3.0.0)
- `RecordIdKeyType::Object` serialization fixed
- `SurrealValue::from_value` compatibility for all JSON variants
- Records not existing return `None` (was confusing error)
- Misleading connection errors replaced with actual query failures
- `UPSERT SET` with `IF` expressions evaluates correctly
- Query planner + executor optimizations

### Still Broken in v3.0.4
- Parameterized MATCHES — keep `escapeSurrealSearch()` workaround
- `math::max([])` still returns -Infinity

### Codebase Scan Results
- **No `search::score()` usage** — safe
- **No `math::max()` usage** — safe
- **No `MATCHES` clause** — safe
- **Schema is v3-compatible** — SCHEMAFULL + FLEXIBLE correct
- **Type system correct** — i64, f64, RecordId, DateTime all proper
- **NULL vs NONE** — already handled correctly
- **JS SDKs unused** — `surrealdb.js` only in `archive/` and `performance/` files

### KB Plugin Offline Issue
- Plugin v3.64.0 is current. "KB_VERSION: offline (missing credentials)" string doesn't exist in any plugin source file.
- Session hook works fine (session created successfully).
- Root cause: stale MCP server process from previous session.
- Fix: restart Claude Code session.

---

## File Map

### Files to Modify
| File | Change | Lines |
|------|--------|-------|
| `crates/e-fees-core/Cargo.toml` | Pin surrealdb + surrealdb-types to 3.0.4 | 14-15 |
| `src-tauri/Cargo.toml` | Pin surrealdb + surrealdb-types to 3.0.4 | 27-28 |
| `e-fees-api/Cargo.toml` | Pin surrealdb to 3.0.4 | 11 |
| `e-fees-scope/Cargo.toml` | Pin surrealdb + surrealdb-types to 3.0.4 | 11-12 |
| `package.json` | Remove unused `surrealdb.js` dependency | 109 |

### Files NOT Changed (Verified Safe)
- All `.surql` schema files — v3-compatible
- All Rust source files — type system already correct
- All TypeScript source files — no JS SDK imports in production code
- `Cargo.lock` — will be auto-updated by `cargo update`

---

## Chunk 1: Dependency Updates

### Task 1: Pin Rust SDK in e-fees-core

**Files:**
- Modify: `crates/e-fees-core/Cargo.toml:14-15`

- [ ] **Step 1: Update version constraints**

Change lines 14-15 from:
```toml
surrealdb = { version = "3.0", default-features = false }
surrealdb-types = "3.0"
```
to:
```toml
surrealdb = { version = "3.0.4", default-features = false }
surrealdb-types = "3.0.4"
```

- [ ] **Step 2: Verify the crate still compiles**

Run: `cargo check -p e-fees-core`
Expected: Compiles without errors

---

### Task 2: Pin Rust SDK in src-tauri (desktop app)

**Files:**
- Modify: `src-tauri/Cargo.toml:27-28`

- [ ] **Step 1: Update version constraints**

Change lines 27-28 from:
```toml
surrealdb = { version = "3.0", features = ["kv-mem", "protocol-ws", "protocol-http", "rustls"] }
surrealdb-types = "3.0"
```
to:
```toml
surrealdb = { version = "3.0.4", features = ["kv-mem", "protocol-ws", "protocol-http", "rustls"] }
surrealdb-types = "3.0.4"
```

- [ ] **Step 2: Verify the crate compiles**

Run: `cargo check -p app_lib`
Expected: Compiles without errors

---

### Task 3: Pin Rust SDK in e-fees-api

**Files:**
- Modify: `e-fees-api/Cargo.toml:11`

- [ ] **Step 1: Update version constraint**

Change line 11 from:
```toml
surrealdb = { version = "3.0", features = ["protocol-ws", "rustls"] }
```
to:
```toml
surrealdb = { version = "3.0.4", features = ["protocol-ws", "rustls"] }
```

Note: e-fees-api intentionally does not depend on surrealdb-types — it uses e-fees-core's re-exported types and raw JSON for API responses. This is a design choice, not an oversight.

- [ ] **Step 2: Verify the crate compiles**

Run: `cargo check -p e-fees-api`
Expected: Compiles without errors

---

### Task 4: Pin Rust SDK in e-fees-scope

**Files:**
- Modify: `e-fees-scope/Cargo.toml:11-12`

- [ ] **Step 1: Update version constraints**

Change lines 11-12 from:
```toml
surrealdb = { version = "3.0", features = ["protocol-ws", "rustls"] }
surrealdb-types = "3.0"
```
to:
```toml
surrealdb = { version = "3.0.4", features = ["protocol-ws", "rustls"] }
surrealdb-types = "3.0.4"
```

- [ ] **Step 2: Verify the crate compiles**

Run: `cargo check -p e-fees-scope`
Expected: Compiles without errors

---

### Task 5: Remove unused JS SDK

**Files:**
- Modify: `package.json:109`

- [ ] **Step 1: Remove surrealdb.js from dependencies**

Remove this line from `package.json` dependencies:
```json
"surrealdb.js": "^1.0.0",
```

Keep `"surrealdb": "^1.3.2"` — it's the newer package name and may be used for future frontend direct-connection features. It's harmless as an installed-but-unused dependency.

- [ ] **Step 2: Run npm install to update lockfile**

Run: `npm install`
Expected: Completes without errors, `package-lock.json` updated

- [ ] **Step 3: Verify no production code depends on surrealdb.js**

Run: `grep -r "from ['\"]surrealdb\.js" src/ --include="*.ts" --include="*.svelte" --include="*.js"`
Expected: No matches (only `archive/` and `performance/` files use it — those are not production code)

---

### Task 6: Update Cargo.lock

- [ ] **Step 1: Update lockfile with pinned versions**

Run: `cargo update surrealdb surrealdb-types`
Expected: Cargo.lock updated to resolve surrealdb 3.0.4 and surrealdb-types 3.0.4

- [ ] **Step 2: Verify resolved versions**

Run: `cargo metadata --format-version=1 | python3 -c "import json,sys; d=json.load(sys.stdin); [print(f'{p[\"name\"]} {p[\"version\"]}') for p in d['packages'] if 'surrealdb' in p['name']]"`
Expected: Shows `surrealdb 3.0.4` and `surrealdb-types 3.0.4`

- [ ] **Step 3: Commit dependency updates**

```bash
git add crates/e-fees-core/Cargo.toml src-tauri/Cargo.toml e-fees-api/Cargo.toml e-fees-scope/Cargo.toml package.json package-lock.json Cargo.lock
git commit -m "chore: pin SurrealDB SDK to v3.0.4 and remove unused surrealdb.js

Pin all Rust crates to surrealdb/surrealdb-types 3.0.4 for reproducible
builds matching production DB version. Remove unused surrealdb.js (v1.0.0)
from package.json — frontend uses Tauri IPC, not JS SDK.

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Chunk 2: Build & Test Verification

### Task 7: Full Rust build

- [ ] **Step 1: Clean build all crates**

Run: `cargo build --workspace`
Expected: All crates compile without errors

- [ ] **Step 2: Run all Rust tests**

Run: `cargo test --workspace`
Expected: All tests pass

---

### Task 8: Frontend build & tests

- [ ] **Step 1: Type check**

Run: `npm run check`
Expected: No type errors

- [ ] **Step 2: Run frontend tests**

Run: `npm test`
Expected: 633/633 tests pass (or current count)

---

### Task 9: API integration tests

- [ ] **Step 1: Run API tests against production DB**

Run: `API_BASE_URL=http://10.0.21.80:3200 API_KEY=efees-api-2026-k8x9m4pq SURREAL_URL=ws://10.0.23.11:8000 cargo test -p e-fees-api --test integration_tests -- --test-threads=1`
Expected: 62/62 tests pass (or current count)

---

### Task 10: Scope service tests

- [ ] **Step 1: Run scope tests**

Run: `cargo test -p e-fees-scope --test integration_tests -- --test-threads=1`
Expected: 24/24 tests pass (or current count)

---

## Chunk 3: KB Plugin Fix & Cleanup

### Task 11: Fix KB plugin offline status

This is not a code change — it's a process restart.

- [ ] **Step 1: Verify the issue**

The "KB_VERSION: offline (missing credentials)" string doesn't exist in plugin v3.64.0 source code. The session hook works (session created). This is a stale MCP server process.

- [ ] **Step 2: Instruct user**

Tell the user: "The KB 'offline' message is from a stale MCP server process. Restart Claude Code (`/exit` then relaunch) to fix it. The DB is accessible and the plugin version (3.64.0) is current."

---

### Task 12: Update HANDOVER.md

- [ ] **Step 1: Update handover with upgrade status**

Update `.claude/HANDOVER.md` to reflect:
- Version: 0.13.7 (unchanged)
- SurrealDB primary DB upgraded to v3.0.4
- All Rust SDKs pinned to 3.0.4
- Unused `surrealdb.js` removed
- KB plugin offline = stale process (restart needed)
- Tests verified passing

---

## Chunk 4: Container Rebuilds (Deferred)

### Task 13: Redeploy API container (MANUAL — requires SSH to AI server)

The API and Scope containers on the AI server still run with the unpinned "3.0" versions from Cargo.lock at build time. They work fine with v3.0.4 server, but should be rebuilt for consistency.

- [ ] **Step 1: Rebuild and redeploy API**

```bash
# On AI server (10.0.20.11)
cd /mnt/user/appdata/e-fees-api/source
git pull
docker build -t e-fees-api:v0.3.2 .
docker stop e-fees-api && docker rm e-fees-api
docker run -d --name e-fees-api --restart unless-stopped \
  --network br0 --ip 10.0.21.80 \
  --env-file /mnt/user/appdata/e-fees-api/.env \
  e-fees-api:v0.3.2
```

- [ ] **Step 2: Verify API health**

Run: `curl -s http://10.0.21.80:3200/docs/ | head -5`
Expected: Swagger UI HTML

---

### Task 14: Redeploy Scope container (MANUAL)

- [ ] **Step 1: Rebuild and redeploy Scope**

```bash
# On AI server (10.0.20.11)
cd /mnt/user/appdata/e-fees-scope/source
git pull
docker build -t e-fees-scope:v0.2.4 .
docker stop e-fees-scope && docker rm e-fees-scope
docker run -d --name e-fees-scope --restart unless-stopped \
  --network br0 --ip 10.0.21.81 \
  --env-file /mnt/user/appdata/e-fees-scope/.env \
  e-fees-scope:v0.2.4
```

- [ ] **Step 2: Verify Scope health**

Run: `curl -s http://10.0.21.81:3201/docs/ | head -5`
Expected: Swagger UI HTML

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Cargo resolves different patch version | Low | Low | Pinning to exact 3.0.4 prevents this |
| Build fails with pinned version | Very Low | Medium | Roll back Cargo.toml changes |
| Tests fail after dependency update | Low | Medium | Run full suite before committing |
| Container rebuild breaks API | Low | Medium | Keep old image, rollback if needed |

## Rollback

If any issues arise after pinning:
```bash
git revert HEAD  # Reverts the dependency pin commit
cargo update     # Resolves back to whatever 3.0.x was in lockfile
```

Container rollback: previous Docker images are still available on AI server.
