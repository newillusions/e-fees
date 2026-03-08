# Deliverable Scope Builder — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace flat clause-based scope generation with a layered deliverable model (generic + discipline + conditional) and drag-and-drop scope builder UI.

**Architecture:** New `deliverable` and `stage_config` tables in SurrealDB, new API routes in e-fees-scope axum service, new assembly engine composing deliverables by layer, and a new Scope Builder view in the Tauri desktop app.

**Tech Stack:** Rust (axum 0.8, surrealdb 3.0), SurrealDB SCHEMAFULL, Svelte 5 (Tauri 2 desktop app), TypeScript

**Design doc:** `docs/plans/2026-03-08-deliverable-scope-builder-design.md`

---

## Task 1: Schema — `stage_config` table

**Files:**
- Modify: `e-fees-scope/schema.surql`

**Step 1: Add stage_config DDL to schema file**

Append after the `proposal_corpus` section in `e-fees-scope/schema.surql`:

```sql
-- ============================================================
-- Stage configuration (canonical stages with aliases)
-- ============================================================
DEFINE TABLE stage_config SCHEMAFULL;

DEFINE FIELD canonical_name ON stage_config TYPE string;
DEFINE FIELD default_label  ON stage_config TYPE string;
DEFINE FIELD aliases        ON stage_config TYPE option<array<string>>;
DEFINE FIELD sort_order     ON stage_config TYPE int;
DEFINE FIELD intro_text     ON stage_config TYPE option<string>;
DEFINE FIELD status         ON stage_config TYPE string DEFAULT "active";

DEFINE INDEX idx_stage_canonical ON stage_config FIELDS canonical_name UNIQUE;
```

**Step 2: Apply schema to dev DB**

Run:
```bash
# Via scope service's SurrealDB connection (same DB: emittiv/projects)
curl -s -X POST "http://10.0.23.11:8000/sql" \
  -u "root:root" \
  -H "surreal-ns: emittiv" \
  -H "surreal-db: projects" \
  -H "Content-Type: text/plain" \
  -d "$(cat e-fees-scope/schema.surql | grep -A20 'stage_config')"
```
Expected: `[{"status":"OK"}]` for each statement

**Step 3: Seed the 7 standard stages**

```bash
curl -s -X POST "http://10.0.23.11:8000/sql" \
  -u "root:root" \
  -H "surreal-ns: emittiv" \
  -H "surreal-db: projects" \
  -H "Content-Type: text/plain" \
  -d "
CREATE stage_config SET canonical_name = 'preliminaries', default_label = 'Preliminaries', sort_order = 10, aliases = ['Briefing'];
CREATE stage_config SET canonical_name = 'concept', default_label = 'Concept Design', sort_order = 20, aliases = ['CD', 'Stage 1'];
CREATE stage_config SET canonical_name = 'schematic', default_label = 'Schematic Design', sort_order = 30, aliases = ['SD', '50% DD', 'Preliminary Design', 'Stage 2'];
CREATE stage_config SET canonical_name = 'detailed', default_label = 'Detailed Design', sort_order = 40, aliases = ['DD', 'Design Development', '100% DD', 'Stage 3'];
CREATE stage_config SET canonical_name = 'ift', default_label = 'Issued for Tender', sort_order = 50, aliases = ['IFT', 'Tender Documentation', 'Stage 4'];
CREATE stage_config SET canonical_name = 'ifc', default_label = 'Issued for Construction', sort_order = 60, aliases = ['IFC', 'Construction Documentation', 'Stage 4a'];
CREATE stage_config SET canonical_name = 'post_contract', default_label = 'Post-Contract Services', sort_order = 70, aliases = ['Site Supervision', 'Construction Support', 'Stage 5'];
CREATE stage_config SET canonical_name = 'handover', default_label = 'Hand Over and Close Out', sort_order = 80, aliases = ['Commissioning', 'Stage 6'];
"
```

**Step 4: Verify seed data**

Run: `curl -s -X POST "http://10.0.23.11:8000/sql" -u "root:root" -H "surreal-ns: emittiv" -H "surreal-db: projects" -H "Content-Type: text/plain" -d "SELECT * FROM stage_config ORDER BY sort_order;"`
Expected: 8 rows (7 stages) with canonical_name, default_label, sort_order

**Step 5: Commit**

```bash
git add e-fees-scope/schema.surql
git commit -m "feat(scope): add stage_config table schema and seed data"
```

---

## Task 2: Schema — `deliverable` table

**Files:**
- Modify: `e-fees-scope/schema.surql`

**Step 1: Add deliverable DDL to schema file**

Append after `stage_config` section:

```sql
-- ============================================================
-- Deliverable library (atomic deliverable items)
-- ============================================================
DEFINE TABLE deliverable SCHEMAFULL;

DEFINE FIELD title            ON deliverable TYPE string;
DEFINE FIELD short_name       ON deliverable TYPE string;
DEFINE FIELD body             ON deliverable TYPE string;
DEFINE FIELD stage            ON deliverable TYPE string;
DEFINE FIELD layer            ON deliverable TYPE string
    ASSERT $value IN ['generic', 'discipline', 'conditional'];
DEFINE FIELD discipline       ON deliverable TYPE option<string>;
DEFINE FIELD condition        ON deliverable TYPE option<object> FLEXIBLE;
DEFINE FIELD replaces         ON deliverable TYPE option<record<deliverable>>;
DEFINE FIELD sort_order       ON deliverable TYPE int;
DEFINE FIELD source_proposals ON deliverable TYPE option<array<string>>;
DEFINE FIELD usage_history    ON deliverable TYPE option<array<object>> FLEXIBLE;
DEFINE FIELD usage_history.*  ON deliverable TYPE object FLEXIBLE;
DEFINE FIELD tags             ON deliverable TYPE option<array<string>>;
DEFINE FIELD status           ON deliverable TYPE string DEFAULT "active";
DEFINE FIELD version          ON deliverable TYPE int DEFAULT 1;
DEFINE FIELD created_at       ON deliverable TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at       ON deliverable TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_deliverable_stage  ON deliverable FIELDS stage;
DEFINE INDEX idx_deliverable_layer  ON deliverable FIELDS layer;
DEFINE INDEX idx_deliverable_status ON deliverable FIELDS status;
```

**Step 2: Apply schema to DB**

Run: Same curl pattern as Task 1, targeting only the new DDL statements.
Expected: All OK

**Step 3: Extend scope_assembly table**

```sql
-- Add new fields to scope_assembly for deliverable tracking
DEFINE FIELD deliverables_used ON scope_assembly TYPE option<array<object>> FLEXIBLE;
DEFINE FIELD deliverables_used.* ON scope_assembly TYPE object FLEXIBLE;
DEFINE FIELD stage_labels ON scope_assembly TYPE option<object> FLEXIBLE;
DEFINE FIELD manual_items ON scope_assembly TYPE option<array<object>> FLEXIBLE;
DEFINE FIELD manual_items.* ON scope_assembly TYPE object FLEXIBLE;
```

**Step 4: Verify schema**

Run: `curl ... -d "INFO FOR TABLE deliverable;"`
Expected: Shows all defined fields and indexes

**Step 5: Commit**

```bash
git add e-fees-scope/schema.surql
git commit -m "feat(scope): add deliverable table and extend scope_assembly schema"
```

---

## Task 3: Models — Rust structs for deliverable and stage_config

**Files:**
- Modify: `e-fees-scope/src/models.rs`

**Step 1: Add StageConfig and Deliverable structs**

Add after the `Clause` struct in `models.rs`:

```rust
// ── Stage Config models ──────────────────────────────────────────

/// A stage configuration record as stored in SurrealDB.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct StageConfig {
    pub id: RecordId,
    pub canonical_name: String,
    pub default_label: String,
    pub aliases: Option<Vec<String>>,
    pub sort_order: i64,
    pub intro_text: Option<String>,
    pub status: String,
}

/// Payload for updating a stage config.
#[derive(Debug, Deserialize)]
pub struct UpdateStageConfig {
    pub default_label: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub sort_order: Option<i64>,
    pub intro_text: Option<String>,
}

// ── Deliverable models ───────────────────────────────────────────

/// A deliverable record as stored in SurrealDB.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Deliverable {
    pub id: RecordId,
    pub title: String,
    pub short_name: String,
    pub body: String,
    pub stage: String,
    pub layer: String,
    pub discipline: Option<String>,
    pub condition: Option<surrealdb_types::Value>,
    pub replaces: Option<RecordId>,
    pub sort_order: i64,
    pub source_proposals: Option<Vec<String>>,
    pub usage_history: Option<surrealdb_types::Value>,
    pub tags: Option<Vec<String>>,
    pub status: String,
    pub version: i64,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

/// Payload for creating a new deliverable.
#[derive(Debug, Deserialize)]
pub struct NewDeliverable {
    pub title: String,
    pub short_name: String,
    pub body: String,
    pub stage: String,
    pub layer: String,
    pub discipline: Option<String>,
    pub condition: Option<serde_json::Value>,
    pub replaces: Option<String>,
    pub sort_order: i64,
    pub source_proposals: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

/// Payload for updating an existing deliverable (all fields optional).
#[derive(Debug, Deserialize)]
pub struct UpdateDeliverable {
    pub title: Option<String>,
    pub short_name: Option<String>,
    pub body: Option<String>,
    pub stage: Option<String>,
    pub layer: Option<String>,
    pub discipline: Option<String>,
    pub condition: Option<serde_json::Value>,
    pub replaces: Option<String>,
    pub sort_order: Option<i64>,
    pub source_proposals: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

/// Request to assemble deliverables for a fee.
#[derive(Debug, Deserialize)]
pub struct AssembleRequest {
    pub fee_id: String,
    /// Disciplines to include (e.g., ["lighting", "av"]).
    pub disciplines: Vec<String>,
    /// Project attributes for conditional matching (e.g., {"tool": "revit"}).
    pub conditions: Option<serde_json::Value>,
    /// Stages to include (canonical names). If empty, includes all active stages.
    pub stages: Option<Vec<String>>,
    /// Stage label overrides (e.g., {"schematic": "50% DD"}).
    pub stage_labels: Option<serde_json::Map<String, serde_json::Value>>,
    /// Whether to run LLM polish on the assembled text.
    #[serde(default)]
    pub polish: bool,
}

/// Request to save the scope builder state.
#[derive(Debug, Deserialize)]
pub struct SaveScopeBuilderRequest {
    pub fee_id: String,
    /// Array of deliverable references with optional wording overrides.
    pub deliverables: Vec<ScopeDeliverableEntry>,
    /// Custom one-off items not from the library.
    pub manual_items: Option<Vec<ManualDeliverableEntry>>,
    /// Stage label overrides for this proposal.
    pub stage_labels: Option<serde_json::Map<String, serde_json::Value>>,
    /// Whether to run LLM polish.
    #[serde(default)]
    pub polish: bool,
}

/// A deliverable entry in a saved scope assembly.
#[derive(Debug, Serialize, Deserialize)]
pub struct ScopeDeliverableEntry {
    pub deliverable_id: String,
    pub stage: String,
    pub sort_order: i64,
    /// If set, this overrides the master wording for this proposal only.
    pub wording_override: Option<String>,
}

/// A manually added deliverable (not from library).
#[derive(Debug, Serialize, Deserialize)]
pub struct ManualDeliverableEntry {
    pub title: String,
    pub short_name: String,
    pub body: String,
    pub stage: String,
    pub sort_order: i64,
}
```

**Step 2: Verify it compiles**

Run: `cd /Volumes/base/dev/app/e-fees && cargo check -p e-fees-scope`
Expected: Compiles with no errors (may have warnings for unused structs — that's fine)

**Step 3: Commit**

```bash
git add e-fees-scope/src/models.rs
git commit -m "feat(scope): add Deliverable and StageConfig model structs"
```

---

## Task 4: API — Stage config endpoints

**Files:**
- Create: `e-fees-scope/src/routes/stages.rs`
- Modify: `e-fees-scope/src/routes/mod.rs`
- Modify: `e-fees-scope/src/main.rs`

**Step 1: Create stages route module**

Create `e-fees-scope/src/routes/stages.rs` with list and update handlers following the same pattern as `clauses.rs`:

- `GET /stages` — list all stage configs ordered by sort_order
- `PUT /stages/{canonical_name}` — update label, aliases, intro_text

Follow the exact same JSON conversion pattern as `clause_to_json` (extract RecordId key, map fields to serde_json::Value). Use `StageConfig` for deserialization from SurrealDB.

**Step 2: Register module**

Add `pub mod stages;` to `e-fees-scope/src/routes/mod.rs`.

**Step 3: Add routes to main.rs**

In the `protected` Router in `main.rs`, add:
```rust
.route("/stages", get(routes::stages::list_stages))
.route("/stages/{canonical_name}", axum::routing::put(routes::stages::update_stage))
```

Add to the `#[openapi(paths(...))]` list.

Add a new tag: `(name = "Stages", description = "Stage configuration")`

**Step 4: Verify it compiles**

Run: `cargo check -p e-fees-scope`
Expected: Compiles

**Step 5: Commit**

```bash
git add e-fees-scope/src/routes/stages.rs e-fees-scope/src/routes/mod.rs e-fees-scope/src/main.rs
git commit -m "feat(scope): add stage config API endpoints"
```

---

## Task 5: API — Deliverable CRUD endpoints

**Files:**
- Create: `e-fees-scope/src/routes/deliverables.rs`
- Modify: `e-fees-scope/src/routes/mod.rs`
- Modify: `e-fees-scope/src/main.rs`

**Step 1: Create deliverables route module**

Create `e-fees-scope/src/routes/deliverables.rs` with these handlers:

- `list_deliverables` — `GET /deliverables` with query params: stage, layer, discipline, status, search (fuzzy text match on title+body)
- `get_deliverable` — `GET /deliverables/{id}`
- `create_deliverable` — `POST /deliverables`
- `update_deliverable` — `PUT /deliverables/{id}` — bumps version, updates updated_at
- `delete_deliverable` — `DELETE /deliverables/{id}` — soft-delete (set status=archived)
- `deliverable_analytics` — `GET /deliverables/analytics` — usage counts grouped by stage, discipline

Follow the same patterns as `clauses.rs`:
- `deliverable_to_json()` helper for serialization
- Dynamic SET clause building for partial updates
- Never bind NULL for option<T> fields — only include in query when Some
- Use `json_to_dbvalue()` for object fields (condition)
- Validate layer is one of `generic`, `discipline`, `conditional`
- If `layer=discipline`, require `discipline` field
- If `replaces` is provided, bind as `type::record('deliverable', $replaces)`

For the `search` query param, use SurrealDB string functions:
```sql
WHERE (string::lowercase(title) CONTAINS string::lowercase($search)
    OR string::lowercase(body) CONTAINS string::lowercase($search)
    OR string::lowercase(short_name) CONTAINS string::lowercase($search))
```

**Step 2: Register module and routes**

Add `pub mod deliverables;` to `mod.rs`.

Add routes to `main.rs`:
```rust
.route("/deliverables", get(routes::deliverables::list_deliverables).post(routes::deliverables::create_deliverable))
.route("/deliverables/analytics", get(routes::deliverables::deliverable_analytics))
.route("/deliverables/{id}", get(routes::deliverables::get_deliverable).put(routes::deliverables::update_deliverable).delete(routes::deliverables::delete_deliverable))
```

Add to OpenApi paths list. Add tag: `(name = "Deliverables", description = "Deliverable library CRUD")`

**Step 3: Verify compilation**

Run: `cargo check -p e-fees-scope`
Expected: Compiles

**Step 4: Commit**

```bash
git add e-fees-scope/src/routes/deliverables.rs e-fees-scope/src/routes/mod.rs e-fees-scope/src/main.rs
git commit -m "feat(scope): add deliverable CRUD API endpoints"
```

---

## Task 6: Assembly engine — auto-populate deliverables for a fee

**Files:**
- Create: `e-fees-scope/src/routes/assembly.rs`
- Modify: `e-fees-scope/src/routes/mod.rs`
- Modify: `e-fees-scope/src/main.rs`

**Step 1: Create assembly route module**

Create `e-fees-scope/src/routes/assembly.rs` with:

**`POST /scope/assemble`** — the core assembly endpoint:

1. Parse `AssembleRequest` (fee_id, disciplines, conditions, stages, stage_labels)
2. Fetch stage configs (filtered by `stages` if provided, otherwise all active)
3. For each stage, query deliverables:
   ```sql
   SELECT * FROM deliverable
   WHERE stage = $stage AND status = 'active'
     AND (layer = 'generic'
       OR (layer = 'discipline' AND discipline IN $disciplines)
       OR (layer = 'conditional'))
   ORDER BY sort_order ASC
   ```
4. Filter conditionals: for each `layer=conditional` deliverable, check if its `condition` object is a subset of the request's `conditions`. Simple key-value matching (e.g., `{"tool": "revit"}` matches if conditions has `tool=revit`).
5. Dedup: if a deliverable has `replaces` set, remove the referenced deliverable from the list.
6. Return the proposed deliverable set grouped by stage — does NOT save to DB. Frontend uses this to populate the scope builder.

Response shape:
```json
{
  "stages": [
    {
      "canonical_name": "concept",
      "label": "Concept Design",
      "intro_text": "...",
      "deliverables": [
        { "id": "...", "short_name": "...", "title": "...", "body": "...", "layer": "generic", "sort_order": 10 }
      ]
    }
  ]
}
```

**`POST /scope/save`** — save scope builder state:

1. Parse `SaveScopeBuilderRequest`
2. For each deliverable entry, snapshot the wording (use override if provided, otherwise fetch master body)
3. Optionally render final text + LLM polish (reuse existing `llm::polish_scope`)
4. Upsert `scope_assembly` with `deliverables_used`, `stage_labels`, `manual_items`
5. Stamp `usage_history` on each deliverable used

**`GET /scope/{fee_id}/deliverables`** — retrieve saved builder state:

1. Fetch scope_assembly by fee_id
2. Return `deliverables_used`, `stage_labels`, `manual_items`

**Step 2: Register routes**

Add `pub mod assembly;` to `mod.rs`.

Routes in `main.rs`:
```rust
.route("/scope/assemble", post(routes::assembly::assemble_deliverables))
.route("/scope/save", post(routes::assembly::save_scope_builder))
.route("/scope/{fee_id}/deliverables", get(routes::assembly::get_scope_deliverables))
```

**Step 3: Verify compilation**

Run: `cargo check -p e-fees-scope`
Expected: Compiles

**Step 4: Commit**

```bash
git add e-fees-scope/src/routes/assembly.rs e-fees-scope/src/routes/mod.rs e-fees-scope/src/main.rs
git commit -m "feat(scope): add assembly engine with layered deliverable composition"
```

---

## Task 7: Integration tests — deliverable and assembly API

**Files:**
- Modify or create: `e-fees-scope/tests/integration_tests.rs` (or whatever test file exists)

**Step 1: Write integration tests**

Test against the running scope service. Tests should cover:

1. **Stage config**: GET /stages returns 8 stages in order
2. **Deliverable CRUD**:
   - POST /deliverables — create generic deliverable (title must include "DELETE ME")
   - GET /deliverables — list with stage filter
   - GET /deliverables/{id} — single fetch
   - PUT /deliverables/{id} — update body, verify version bumped
   - DELETE /deliverables/{id} — verify status=archived
   - GET /deliverables?search=... — fuzzy search
3. **Assembly**:
   - POST /scope/assemble — with disciplines=["lighting"], verify returns grouped by stage
   - POST /scope/assemble — with conditions={"tool":"revit"}, verify conditional deliverables included
   - POST /scope/assemble — verify dedup (create a discipline deliverable with `replaces`, verify generic removed)
4. **Cleanup**: Delete all "DELETE ME" test deliverables

Follow existing test patterns. Use `reqwest` with `X-API-Key` header.

**Step 2: Run tests**

Run: `API_BASE_URL=http://10.0.21.81:3201 API_KEY=efees-scope-2026-s7k2m9xp cargo test -p e-fees-scope -- --test-threads=1`
Expected: All tests pass

**Step 3: Commit**

```bash
git add e-fees-scope/tests/
git commit -m "test(scope): add integration tests for deliverable CRUD and assembly"
```

---

## Task 8: Seed deliverables from corpus (manual curation session)

This is an interactive task — not automated code. The steps are:

**Step 1: Fix corpus extracted_text retrieval**

The list API returns 0 chars for extracted_text. Check if the corpus detail endpoint (`GET /corpus/{id}`) returns the text. If not, investigate the SurrealDB query — the `extracted_text` field may need to be explicitly selected or the list query may be omitting it for size.

**Step 2: Extract deliverable items from corpus**

Write a one-off script (or use curl + jq) to:
1. Fetch all corpus docs with text
2. Parse "Typical deliverables include:" bullet lists
3. Group by stage (identify from section headers like "Stage 1", "Concept Design", etc.)
4. Output as JSON for review

**Step 3: Cluster and curate**

Present the extracted deliverables to the user for curation:
- Side-by-side variants of the same deliverable across proposals
- User picks best wording or writes master version
- Tag each with stage, layer, discipline, source_proposals

**Step 4: Seed via API**

Use POST /deliverables to create each curated deliverable.

**Step 5: Commit any scripts**

```bash
git add e-fees-scope/scripts/
git commit -m "chore(scope): add corpus extraction scripts for deliverable seeding"
```

---

## Task 9: Deploy updated scope service

**Step 1: Build and push Docker image**

```bash
# On dev machine
rsync -avz --delete e-fees-scope/ root@10.0.20.11:/mnt/user/appdata/e-fees-scope/source/
ssh root@10.0.20.11 "cd /mnt/user/appdata/e-fees-scope/source && docker build -t e-fees-scope:v0.3.0 ."
```

**Step 2: Restart container**

```bash
ssh root@10.0.20.11 "docker stop e-fees-scope && docker rm e-fees-scope && docker run -d \
  --name e-fees-scope \
  --network br0 --ip 10.0.21.81 \
  --restart unless-stopped \
  -e SURREAL_URL=ws://10.0.23.11:8000 \
  -e SURREAL_USER=root -e SURREAL_PASS=root \
  -e API_KEY=efees-scope-2026-s7k2m9xp \
  -e OLLAMA_URL=http://10.0.21.20:11434 \
  e-fees-scope:v0.3.0"
```

**Step 3: Verify health**

Run: `curl http://10.0.21.81:3201/health`
Expected: `{"status":"ok",...}`

**Step 4: Verify new endpoints**

Run: `curl -H "X-API-Key: efees-scope-2026-s7k2m9xp" http://10.0.21.81:3201/stages`
Expected: 8 stage configs

---

## Task 10: Frontend — Scope Builder UI (Tauri desktop app)

> **Note:** This is the largest task and may be split into sub-tasks during execution. The frontend is a Svelte 5 component in the Tauri app.

**Files:**
- Create: `src/lib/components/scope/ScopeBuilder.svelte` — main builder view
- Create: `src/lib/components/scope/StageSection.svelte` — collapsible stage with deliverable cards
- Create: `src/lib/components/scope/DeliverableCard.svelte` — draggable card showing short_name
- Create: `src/lib/components/scope/DeliverableLibrary.svelte` — right panel with search
- Create: `src/lib/components/scope/DeliverableDetail.svelte` — inline expanded view / edit modal
- Create: `src/lib/api/scope.ts` — API client for scope service endpoints
- Modify: `src/App.svelte` — add route for scope builder
- Create: `src/lib/components/scope/scope.css` — `.emittiv-scope-*` classes

**Key implementation notes:**
- Use Svelte 5 runes (`$state`, `$derived`, `$effect`) not legacy `$:` syntax
- Use `let { prop } = $props()` not `export let`
- Use existing `.emittiv-*` classes from `app.css`, extend with `.emittiv-scope-*`
- Fixed px values (desktop app with OS scaling)
- Drag and drop: use HTML5 drag-and-drop API (`draggable`, `ondragstart`, `ondrop`)
- Fuzzy search: client-side filter on fetched deliverables (the API also supports server-side search)
- Scope builder route accessed from ProposalModal or a new toolbar button

**The scope builder communicates with the scope service API (10.0.21.81:3201), not with the Tauri backend.** Use `fetch()` directly from the frontend (CORS already configured on the scope service).

This task will be broken into smaller steps during execution based on the implementing agent's judgement.

---

## Implementation Order Summary

| Task | Component | Depends On | Estimated Size |
|------|-----------|-----------|----------------|
| 1 | Schema: stage_config | — | Small |
| 2 | Schema: deliverable + scope_assembly extension | 1 | Small |
| 3 | Models: Rust structs | — | Small |
| 4 | API: stage config endpoints | 1, 3 | Medium |
| 5 | API: deliverable CRUD | 2, 3 | Medium |
| 6 | API: assembly engine | 4, 5 | Large |
| 7 | Integration tests | 4, 5, 6 | Medium |
| 8 | Seed deliverables from corpus | 5 | Interactive |
| 9 | Deploy updated service | 7 | Small |
| 10 | Frontend: Scope Builder UI | 6, 9 | Large |
