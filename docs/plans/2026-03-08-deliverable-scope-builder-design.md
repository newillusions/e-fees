# Deliverable Scope Builder — Design Document

**Date**: 2026-03-08
**Status**: Approved
**Scope**: e-fees-scope service + desktop app frontend

## Problem

The current clause library has 13 flat, generic clauses that don't reflect how emittiv actually builds scope/deliverables sections in fee proposals. Real proposals have:

- Stage-specific deliverable lists (Concept → Schematic → Detailed → IFT → IFC → Post-Contract → Handover)
- A generic base layer that appears on all projects
- Discipline-specific additions (lighting, AV, sound, controls)
- Conditional deliverables that swap based on project attributes (Revit vs CAD, BIM requirements)
- Flexible stage naming (e.g., "Schematic Design" → "50% DD")
- Deliverable wording refined over 51 real proposals

The system needs to model this layered composition and provide a drag-and-drop editor for per-proposal customisation.

## Data Model

### `deliverable` table (new, replaces clause for scope deliverables)

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Full deliverable name |
| `short_name` | string | 3-5 word card label for the UI |
| `body` | string | Full professional wording |
| `stage` | string | Canonical stage name (preliminaries, concept, schematic, detailed, ift, ifc, post_contract, handover) |
| `layer` | string | `generic` \| `discipline` \| `conditional` |
| `discipline` | option\<string\> | null for generic; `lighting`, `av`, `sound`, `controls` etc. |
| `condition` | option\<object\> FLEXIBLE | null unless conditional; e.g. `{"tool": "revit", "lod": 300}` |
| `replaces` | option\<record\<deliverable\>\> | Points to a generic deliverable this one supersedes |
| `sort_order` | int | Ordering within stage |
| `source_proposals` | array\<string\> | Corpus doc references where wording was found/derived |
| `usage_history` | array\<object\> FLEXIBLE | `[{fee_id, date}]` tracking which proposals used this |
| `tags` | array\<string\> | Freeform tags for filtering/search |
| `status` | string | `active` \| `draft` \| `archived` |
| `version` | int | Incremented on edits |
| `created_at` | datetime | |
| `updated_at` | datetime | |

### `stage_config` table (new)

| Field | Type | Description |
|-------|------|-------------|
| `canonical_name` | string | Internal key (preliminaries, concept, schematic, etc.) |
| `default_label` | string | Display name ("Concept Design") |
| `aliases` | array\<string\> | Alternative names (["50% DD", "Preliminary Design"]) |
| `sort_order` | int | Stage sequence |
| `intro_text` | option\<string\> | Optional preamble prose for the stage section |
| `status` | string | `active` \| `inactive` |

### `scope_assembly` table (extended)

Existing table gets additional fields:

| Field | Type | Description |
|-------|------|-------------|
| `deliverables_used` | array\<object\> FLEXIBLE | `[{deliverable_id, stage, short_name, wording_snapshot}]` |
| `stage_labels` | option\<object\> FLEXIBLE | `{canonical_name → display_label}` overrides for this proposal |
| `manual_edits` | array\<object\> FLEXIBLE | Custom/one-off deliverables added for this proposal only |

## Assembly Logic

### Input
Fee record containing: project type, packages (disciplines), tool preference (Revit/CAD), selected stages.

### Steps

1. **Resolve stages** — look up `stage_config` for included stages, apply any label overrides from fee/project data.

2. **Gather deliverables per stage**:
   - All `layer=generic` + `status=active` for the stage
   - All `layer=discipline` matching the proposal's packages
   - All `layer=conditional` where `condition` matches project attributes

3. **Dedup/blend** — if a discipline deliverable has `replaces` pointing to a generic, the generic is dropped and the discipline version takes its slot. Otherwise both appear (additive).

4. **Sort** by `sort_order` within each stage.

5. **Render** — compose stage intro text + deliverable list. Substitute placeholders (`[Company Name]`, `[LOD Level]`, etc.) from fee/project data.

6. **Optional LLM polish** — smooth transitions only. The LLM is a copy-editor, not an author. Preserves curated deliverable wording.

7. **Record usage** — stamp each deliverable's `usage_history` with `{fee_id, date}`.

## Frontend: Scope Builder UI

### Location
New route in the Tauri desktop app, accessible from the proposal detail view.

### Layout
Vertical stage-by-stage sections. Each stage is collapsible. A stage nav sidebar or breadcrumb for jumping between stages.

### Each Stage Section — Two Zones

**Left: Active deliverables** (in this proposal)
- Cards show `short_name` only (compact, draggable)
- Single click → select/deselect for bulk ops
- Double-click → inline expand showing full wording, editable
- Drag to reorder within stage
- Drag to move to a different stage (drop on stage nav)
- Context menu: Duplicate, Save as New, Remove, Edit
- Visual indicators:
  - Layer badge: `G` (generic) / `L` (lighting) / `C` (conditional)
  - Edit indicator if wording was modified from master
  - Source link: which proposals this wording came from
  - Usage count: how many proposals have used this deliverable

**Right: Library panel** (available deliverables not yet added)
- Fuzzy search box at top — searches titles, body text, tags
- Results show short name + preview snippet
- Hover → full text popover (quick preview)
- Click `+` or drag left to add to active list
- Filtered to relevant items but searchable across everything

### Card Actions
- **Duplicate** — copies deliverable into this proposal as independent instance (edit without affecting master)
- **Save as New** — takes edited wording and creates a new master deliverable in the library (with source traceability to original)
- **Remove** — removes from this proposal only (master unaffected)

### Stage Header
- Editable label (click to rename for this proposal, e.g., "Schematic Design" → "50% DD")
- Collapse/expand toggle
- Count badge (number of active deliverables)
- Stage intro text (editable)

### Navigation
Vertical scroll with sticky stage headers. Optional step-through mode (Next/Prev) for focused editing. User preference toggle.

### Toolbar Actions
- **Generate** — render assembled deliverables into final scope prose
- **Polish** — run LLM copy-editing pass
- **Export** — send to InDesign / JSON
- **Reset** — re-run assembly logic (discards manual changes)

## Corpus Extraction Workflow

To seed the deliverable library from 51 ingested proposals:

1. **Fix corpus retrieval** — the list API currently returns 0 chars for extracted_text. Fix serialization or use detail endpoint.

2. **Extract deliverable items** — parse "Typical deliverables include:" bullet lists per stage from each corpus document.

3. **Cluster similar items** — group variants of the same deliverable across proposals (e.g., "Prepare 3D layouts to LOD 300" vs "Prepare package specific 3D layouts to LOD 400").

4. **Curate** — present variants side by side. User chooses best wording or writes the master version.

5. **Tag** — assign layer, discipline, conditions. Link back to source proposals.

6. **Build stage templates** — from the most commonly appearing deliverable sets per stage.

### Edge Cases
- Jeddah Opera House, WAMI Review, Trojena Observatory are technical management engagements — different stage structure. These should be tagged as `project_type: technical_management` and handled as a separate template set (future work).

## Relationship to Existing Clause Library

The current 13 clauses in the `clause` table serve a different purpose — they're contractual/commercial sections (Fees, Payment Terms, Assumptions, etc.), not stage deliverables.

**Decision**: Keep the `clause` table for contractual prose. The new `deliverable` table handles stage-specific scope content. Both feed into `scope_assembly` but are distinct concerns.

Future work: apply the same layered composition model to the clause library (contractual sections vary by project type too).

## API Endpoints (scope service)

### Deliverable CRUD
- `GET /deliverables` — list with filters (stage, layer, discipline, status, search)
- `POST /deliverables` — create
- `GET /deliverables/{id}` — detail
- `PUT /deliverables/{id}` — update
- `DELETE /deliverables/{id}` — soft-delete

### Stage Config
- `GET /stages` — list all stage configs
- `PUT /stages/{canonical_name}` — update labels/aliases/intro

### Scope Assembly (extended)
- `POST /scope/assemble` — auto-populate deliverables for a fee (returns proposed set, doesn't save)
- `POST /scope/save` — save the current scope builder state
- `GET /scope/{fee_id}/deliverables` — get saved deliverable set for a fee

### Deliverable Analytics
- `GET /deliverables/analytics` — usage counts, discipline breakdown, most/least used
- `GET /deliverables/{id}/usage` — which proposals used this deliverable, when

## Implementation Order

1. Schema: `deliverable` + `stage_config` tables in SurrealDB
2. API: CRUD endpoints for deliverables and stage config
3. Corpus extraction: fix retrieval, parse, cluster, present for curation
4. Assembly logic: auto-populate from layers + dedup
5. Frontend: Scope Builder UI in desktop app
6. Integration: connect to existing scope generation + InDesign export pipeline
