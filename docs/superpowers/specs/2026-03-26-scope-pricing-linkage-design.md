# Scope-Pricing Stage Linkage — Design Spec

**Date**: 2026-03-26
**Status**: Reviewed (v2 — addresses staff review)
**Author**: Martin / Claude
**Relates to**: Scope UI integration (2026-03-25), Pricing module

## Problem

The pricing and scope modules operate as completely separate systems. Pricing defines stages (Schematic Design, Design Development, etc.) with fee percentages. Scope generates deliverable text organised by categories. There is no linkage between the two — stage names, structure, and lifecycle are independent, leading to manual reconciliation and risk of mismatches in proposals.

## Goals

1. **Single source of truth** for stages per proposal — both modules read from the same list
2. **Bidirectional sync** — adding a stage in either module makes it available in the other
3. **Stage name templating** — scope text uses actual stage names, not generic placeholders
4. **Per-fee scope permanence** — each proposal's scope is an independent, auditable document
5. **Revision history** — every scope save is versioned and exported to the project folder

## Non-Goals

- Real-time collaborative editing between pricing and scope
- Automatic fee recalculation when scope changes
- Locking stages to prevent concurrent editing

---

## Architecture

### Stage Data Flow

```
stage_config (global autocomplete pool, extended)
        | fuzzy match on input
        v
  fee.pricing.stages (per-proposal source of truth)
        |
        | shared $state in ProposalModal
        v
  +----------+----------+
  |                     |
pricing UI         scope viewer
(fee allocation)   (reads stages, passes to scope service in request)
                        |
                        v
                   scope service
                   (receives stages as input, generates text)
```

### Source of Truth: `fee.pricing.stages`

The existing `Stage` struct on `fee.pricing.stages` is the authoritative list for each proposal:

```rust
Stage {
    id: String,              // e.g., "design-01"
    name: String,            // e.g., "Schematic Design"
    code: String,            // e.g., "SD"
    percentage: f64,         // Fee percentage
    order: i64,              // Sort order
    is_post_contract: bool,  // Design vs post-contract
}
```

Both pricing and scope modules read from and write to this same array. No new per-fee stage table is needed.

#### `DbValue` serialization path

`fee.pricing` is stored as `Option<DbValue>` — an opaque blob. To read/write stages:

1. **Read path**: `fee.pricing_typed()` deserializes `DbValue` → `PricingBreakdown` via `serde_json::from_value`. Returns `Option<PricingBreakdown>`.
2. **Write path (new)**: Add `fee.set_pricing_stages(stages: Vec<Stage>)` to `e-fees-core`. This reads the existing `PricingBreakdown` (or creates a default if `None`), replaces the `stages` field, and serializes back to `DbValue`.
3. **`pricing: None` case**: If a fee has no pricing data yet, `add_stage_to_fee` creates a minimal `PricingBreakdown` with only the new stage and sensible defaults (0 totals, empty disciplines/cells/costs).

The scope service does **not** read `fee.pricing.stages` from DB directly. Instead, the **frontend passes stages** in the `GenerateScopeRequest` (see API Changes). The scope service is a stateless text generator — it receives stages as input, not by fetching them.

#### Frontend stage state management

Both pricing and scope tabs in `ProposalModal` share a single reactive `stages` variable derived from the loaded fee's pricing data. This prevents the lost-update race condition:

- `ProposalModal` loads the fee, extracts `pricing.stages` into a shared `$state` variable
- Pricing tab reads/writes this shared state
- Scope tab reads this shared state; `add_stage_to_fee` (reverse flow) updates both the DB and the shared state
- On fee save, the shared state is serialized back to `fee.pricing`
- No separate DB writes from individual tabs — all changes go through the shared state → single save

### Global Stage Dictionary

Extend the **existing `stage_config` table** rather than creating a new table. Add fields for autocomplete and usage tracking:

```sql
-- Extend existing stage_config
DEFINE FIELD code            ON stage_config TYPE option<string>;
DEFINE FIELD usage_count     ON stage_config TYPE int DEFAULT 0;
DEFINE FIELD is_post_contract ON stage_config TYPE bool DEFAULT false;
```

The existing `canonical_name`, `default_label`, `aliases`, `sort_order`, and `status` fields already serve the dictionary purpose. `aliases` provides fuzzy matching (multiple names for the same stage).

**Population**:
- Existing `stage_config` records are already seeded with common stage names
- When a user types a new stage name that doesn't match any entry, it's added automatically after confirmation
- `usage_count` increments each time a stage name is used in a new fee, improving autocomplete ranking

**Fuzzy matching**: Frontend uses substring/prefix match on `canonical_name` and `aliases[]`. The table is small (<100 entries).

**Cleanup**: An admin can delete typos via the stage config management (future). No auto-cleanup — the table stays small enough that manual curation is fine.

---

## Bidirectional Stage Sync

### Adding a stage from pricing (normal flow)

1. User adds "BIM Coordination" in the pricing stage list
2. Stage is written to `fee.pricing.stages` with user-specified percentage and order
3. Scope module sees it immediately on next load (reads from `fee.pricing.stages`)
4. Scope shows informational banner: *"BIM Coordination: no scope deliverables"*

### Adding a stage from scope (reverse flow)

1. User adds a stage via scope's advanced controls (selecting from `stage_config` dictionary or typing a new name)
2. System creates a `Stage` with auto-generated fields:
   - `id`: slugified name (e.g., "bim-coordination")
   - `code`: first letters of each word (e.g., "BC")
   - `percentage: 0`
   - `order`: max existing order + 1
   - `is_post_contract`: from `stage_config` if matched, else `false`
3. Stage is added to the shared frontend state (and saved to DB on next fee save)
4. Pricing shows informational banner: *"BIM Coordination: 0% fee allocated"*

### Renaming a stage

1. Rename in either module updates `fee.pricing.stages[].name`
2. The other module sees the new name immediately on next load
3. If scope text was generated with the old name, a banner appears: *"Stage renamed since last generation — regenerate or manually update scope text"*
4. The rename does NOT auto-rewrite generated scope text (protects manual edits)

### Removing a stage

- **From pricing**: Warn if scope has deliverables under this stage before removing. On confirm, the stage is removed from `fee.pricing.stages`. Scope loses that stage's grouping.
- **From scope**: Does not remove from pricing. Scope simply stops showing deliverables for that stage. The stage remains in `fee.pricing.stages` with its fee allocation intact.

### Informational banners

All sync indicators are non-blocking banners:

| Location | Condition | Message |
|----------|-----------|---------|
| Scope viewer | Stage has no deliverables | *"[Stage]: no scope deliverables"* |
| Pricing panel | Stage has 0% allocation | *"[Stage]: 0% fee allocated"* |
| Scope viewer | Stage renamed after generation | *"[Stage] was renamed — regenerate or update manually"* |
| Scope viewer | Stage removed from pricing | *"[Stage] removed from pricing — review scope"* |

---

## Scope Text Templating

### Clause library (generic templates)

Clause bodies in the `clause` table remain generic and reusable across all proposals. They use placeholder tokens:

- `[Phase]` — resolved to the relevant stage name
- `[Company Name]` — from fee's company link
- `[Location]` — from fee's project location
- `[Contact Name]` — from fee's contact link

Existing placeholders like `[Company Name]` and `[Location]` are already partially handled by the LLM polish step.

### Resolution at generation time

Placeholder resolution is handled by the **LLM polish step**, not by regex substitution. The stage names are passed as context to the LLM prompt, which naturally incorporates them into the prose.

When `generateScope` is called:

1. The frontend passes the fee's stage list in the request body
2. Clauses are grouped by category and auto-numbered as today
3. The LLM polish step receives the stage list as structured context:
   - Design stages: "Schematic Design (SD), Design Development (DD), Construction Documentation (CD)"
   - Post-contract stages: "Construction Administration (CA), Defects Liability (DLP)"
4. The LLM uses stage names contextually — e.g., replacing generic `[Phase]` tokens with real stage names where they appear, and referencing specific stages in service descriptions
5. The resolved text is stored in `scope_assembly` — it's now a concrete, per-fee document

**Why LLM, not regex**: Clauses are grouped by category (Administrative, Commercial, Services), not by stage. A clause like "Contract Details" doesn't map 1:1 to a stage — it references multiple stages in its body. The LLM handles this naturally using the stage list as context, rather than requiring a rigid clause-to-stage mapping in the data model.

**Fallback**: If `polish: false` (or LLM unavailable), `[Phase]` placeholders remain as-is in the raw text. The user can manually replace them.

### After generation

- The scope text is fully editable — the user can modify any clause text
- Template placeholders are gone; the text has real stage names
- The clause library is unaffected — it keeps its generic `[Phase]` tokens

---

## Per-Fee Scope Storage & Auditability

### Working version

`scope_assembly` (one record per fee) holds the current working version, as today. No structural change.

### Revision history

Stored in a **separate table** to avoid bloating the working `scope_assembly` record:

```sql
DEFINE TABLE scope_revision SCHEMAFULL;
DEFINE FIELD fee_id         ON scope_revision TYPE record<fee>;
DEFINE FIELD revision       ON scope_revision TYPE int;
DEFINE FIELD clauses        ON scope_revision TYPE array<object> FLEXIBLE;
DEFINE FIELD clauses.*      ON scope_revision TYPE object FLEXIBLE;
DEFINE FIELD clauses.*.clauses   ON scope_revision TYPE option<array<object>> FLEXIBLE;
DEFINE FIELD clauses.*.clauses.* ON scope_revision TYPE object FLEXIBLE;
DEFINE FIELD generated_text ON scope_revision TYPE string;
DEFINE FIELD stages_at_time ON scope_revision TYPE array<string>;
DEFINE FIELD saved_at       ON scope_revision TYPE datetime DEFAULT time::now();
DEFINE FIELD trigger        ON scope_revision TYPE string;
DEFINE INDEX idx_scope_rev_fee ON scope_revision FIELDS fee_id, revision;
```

Each save creates a new `scope_revision` record with the pre-save state.

**Trigger values**: `initial_generation`, `manual_edit`, `regeneration`, `stage_change`

Add `current_revision` (int) and `stages_snapshot` (array of stage names at last generation) to `scope_assembly` to track state without reading all revisions:

```sql
DEFINE FIELD current_revision ON scope_assembly TYPE int DEFAULT 0;
DEFINE FIELD stages_snapshot  ON scope_assembly TYPE option<array<string>>;
```

`stages_snapshot` enables the "stage renamed/removed since last generation" banner by comparing against current `fee.pricing.stages`.

### Markdown file export

On each scope save, export to the project's proposal folder:

- **Revision file**: `<project_folder>/scope/scope-rev-01.md` (incrementing)
- **Current file**: `<project_folder>/scope/scope-current.md` (overwritten)

**Markdown format**:

```markdown
---
fee: 25-97105-1
project: Project Name
revision: 1
date: 2026-03-26
stages: [Schematic Design, Design Development, Construction Documentation]
---

# Scope of Services

## 1.0 Administrative

### 1.1 Prepared For / Contact Details
...

## 2.0 Commercial

### 2.1 Contract Details / Site Attendance
...
```

**Export runs from the Tauri desktop app**, not the scope service. After the scope service returns the saved data, the frontend calls a Tauri command (`export_scope_markdown`) which writes to the local filesystem. The desktop app has access to the project folder via the mounted Nextcloud path (or direct local path).

**Failure handling**: If the project folder doesn't exist or isn't accessible, the DB save still succeeds. A toast notification says *"Scope saved. Folder export skipped — project folder not found."*

---

## API Changes

### Scope service (`e-fees-scope`)

**`POST /scope/generate`** — extend `GenerateScopeRequest`:

```rust
pub struct GenerateScopeRequest {
    pub fee_id: String,
    pub polish: bool,
    pub stages: Option<Vec<StageInput>>,  // NEW: fee's stage list for templating
}

pub struct StageInput {
    pub name: String,
    pub code: String,
    pub is_post_contract: bool,
    pub order: i64,
}
```

If `stages` is provided, the service uses them for `[Phase]` resolution and LLM context. If omitted, falls back to current behaviour (generic text).

### Desktop app (Tauri)

**New command**: `add_stage_to_fee` — adds a stage to `fee.pricing.stages` (used by scope module for reverse sync).

**New command**: `get_fee_stages` — returns `fee.pricing.stages` for a given fee ID (used by scope viewer on load).

**Existing command**: `update_fee` — already handles pricing updates; no change needed for pricing-side stage additions.

### Stage dictionary

**New commands**: `search_stage_dictionary(query: String)` and auto-insert on new stage creation. Simple CRUD, no complex logic.

---

## Frontend Changes

### Pricing module

- Stage name input gets autocomplete from `stage_dictionary`
- Informational banner for stages with 0% allocation that exist in scope

### Scope module

- `ScopeViewer` reads `fee.pricing.stages` on load to know available stages
- `ScopeAdvancedControls` shows fee stages (not global `stage_config`) for filtering
- Banners for stages with no deliverables or post-rename state
- Save action triggers revision history append + markdown export

### ProposalModal

- Both pricing and scope tabs read from the same `fee.pricing.stages`
- No new UI for stage management — stages are managed inline in pricing (primary) or added via scope (secondary)

---

## Migration

### Stage dictionary seeding

Pre-seed `stage_dictionary` with common names extracted from existing fees:

```sql
-- Extract unique stage names from all existing fee pricing data
-- Insert into stage_dictionary with usage_count based on frequency
```

### Existing scope assemblies

No migration needed — existing `scope_assembly` records continue working. `revision_history` starts empty; the first save after the update creates revision 1.

### `stage_config` table

Extended with `code`, `usage_count`, `is_post_contract` fields. Existing data preserved. The table continues to serve as the global stage dictionary and scope assembly fallback.

---

## Testing Strategy

| Test | Type | Description |
|------|------|-------------|
| Stage dictionary CRUD | Unit | Create, search, auto-insert on new stage |
| Stage sync: pricing → scope | Integration | Add stage in pricing, verify scope sees it |
| Stage sync: scope → pricing | Integration | Add stage from scope, verify pricing sees it with 0% |
| Stage rename propagation | Integration | Rename in pricing, verify scope banner appears |
| Template resolution | Unit | `[Phase]` replaced with stage name at generation |
| Revision history append | Unit | Save creates revision entry with correct data |
| Markdown export | Integration | Save produces correctly formatted .md file |
| Export failure resilience | Unit | Missing folder doesn't break DB save |
| Banner conditions | Frontend | Correct banners for 0% / no deliverables / renamed |

---

## File Impact

| File | Change |
|------|--------|
| `e-fees-scope/schema.surql` | Extend `stage_config`, add `scope_revision` table, `stages_snapshot` + `current_revision` fields |
| `e-fees-scope/src/routes/scope.rs` | Accept stages in generate request, template resolution |
| `e-fees-scope/src/models.rs` | `StageInput` struct |
| `crates/e-fees-core/src/models/fee.rs` | No change (Stage struct already exists) |
| `src-tauri/src/commands/` | `stage_dictionary.rs` (new), `scope_export.rs` (new), `fee_stages.rs` (new) |
| `src/lib/api/scope.ts` | Pass stages to generate, new stage dictionary API |
| `src/lib/components/scope/ScopeViewer.svelte` | Load fee stages, banners, revision save |
| `src/lib/components/scope/ScopeAdvancedControls.svelte` | Use fee stages instead of global config |
| `src/lib/components/pricing/PricingCalculatorPanel.svelte` | Stage autocomplete, sync banners |
| `src/lib/types/scope.ts` | StageInput type, revision history types |
