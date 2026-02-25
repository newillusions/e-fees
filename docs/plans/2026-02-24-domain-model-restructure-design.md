# Domain Model Restructure — Design Notes

**Date:** 2026-02-24
**Reviewed:** 2026-02-25
**Status:** Approved — Ready for implementation planning
**Approach:** B (Introduce venue entity, refine project as engagement)
**Future path:** Evolve toward C (graph model) using SurrealDB graph features when needed

---

## 1. Problem Statement

The current data model conflates three distinct business concepts into two tables:

1. **The physical place** (venue/site) — persists forever, may be worked on multiple times
2. **An engagement** (opportunity/bid cycle) — each time Emittiv is asked to price or work on a venue
3. **A fee proposal** (pricing document) — the actual document sent to a client, with revisions

### Current Pain Points

- **No venue persistence**: When a completed project comes back as a "v2 redesign", it gets a new project number with no formal link to the original. Currently handled with a "v2" suffix in the name.
- **Status confusion**: The dashboard "Active Fees" pill and the Proposals module "Active" filter use different definitions. Fee statuses (`Draft`, `Active`, `Sent`, `Negotiation`, `Awarded`, `Revised`) mix proposal lifecycle with engagement lifecycle.
- **No cross-engagement data access**: Can't navigate from a new engagement to a previous one's fee data, BOQ, or design docs for the same venue.
- **Unused statuses**: PROD data has zero fees with status `Draft`, `Active`, `Sent`, `Negotiation`, or `On Hold`. Fees are only `Awarded`, `Completed`, `Lost`, or `Revised`.
- **Client relationship ambiguity**: The venue and the client engaging Emittiv are independent — different clients can engage Emittiv for the same venue, and Emittiv can even bid for multiple clients on the same venue simultaneously.

### Current Data Snapshot (2026-02-24)

**Fees (31 total):**
| Status | Count |
|--------|-------|
| Lost | 21 |
| Completed | 4 |
| Awarded | 3 |
| Revised | 3 |

**Projects (61 total):**
| Status | Count |
|--------|-------|
| Lost | 36 |
| Completed | 11 |
| Revised | 4 |
| Cancelled | 3 |
| RFP | 3 |
| Active | 2 |
| Awarded | 2 |

---

## 2. Business Context (from discussion)

### Key Business Rules

1. **Venue independence**: A venue (physical place) exists independently of any client. Different clients can commission work on the same venue. The same client can re-engage for the same venue years later.

2. **Engagement = one bid cycle**: Each time Emittiv is asked to price/work on a venue, that's a separate engagement with its own project number (YY-CCCNN).

3. **Packages are flexible**: A package can be:
   - One or more disciplines (LX, AV, CTL) for an entire venue
   - One or more disciplines for a portion of a venue (zone/area)
   - Any combination of the above
   - Packages are freeform, not a rigid taxonomy
   - The `package` field on fees relates to disciplines covered

4. **Revisions are loosely defined**: A revision is typically the same client on the same project, but scope can change between revisions. A "revision" can even span years if the same client comes back for the same venue.

5. **Cross-engagement access is required**: When working on a v2 redesign, Emittiv needs to access the previous engagement's fee data, pricing, BOQ, and design docs from within E-Fees.

6. **Most projects are simple**: One venue, one client, one engagement, one fee. The complex cases (multi-client, re-engagement, competing bids) need to be supported but should not add friction to the common case.

7. **Delivery has two phases**: Once awarded, work flows through a **Design phase** (CON, SD, DD, CD) and then a **Construction phase** (CA, commissioning, site visits). These are distinct workflow modes with different deliverables and billing patterns.

8. **Nothing is set in stone**: The business is still forming patterns. The data model should be flexible enough to adjust as workflows evolve.

9. **Client relationship lives on the fee**: The fee proposal document is addressed to a specific company/contact. Multiple clients can bring the same project to Emittiv, and this is visible from who receives the fee proposals.

---

## 3. Approaches Considered

### Approach A: Minimal — Add venue field + cross-links
- Add a `venue` text field on projects (freeform)
- Add `related_projects` array for cross-references
- Fix status lists

**Pros:** Smallest change, no migration
**Cons:** Venue is just a string, relationships are loose, doesn't solve cross-engagement data access

**Verdict:** Too shallow. Doesn't address the core conflation problem.

### Approach B: Introduce `venue` entity, refine `project` as engagement ✅ CHOSEN
- New `venue` table as persistent top-level entity
- `project` becomes "engagement" (each bid cycle)
- `fee` becomes specifically a pricing proposal
- Clean status separation per entity type

**Pros:** Clean separation, matches mental model, enables cross-engagement access, proper venue history
**Cons:** Moderate migration effort, UI changes needed

**Verdict:** Right balance of structure and pragmatism.

### Approach C: Full graph model (future evolution)
- Everything as nodes and edges using SurrealDB graph features
- `venue <-worked_on- engagement -submitted-> fee -covers-> package`

**Pros:** Maximum flexibility, powerful querying
**Cons:** Heaviest migration, most complex UI, potentially over-engineered for current needs

**Verdict:** Natural evolution of B when the data/query patterns justify it. SurrealDB supports this natively, so migration path is clear.

---

## 4. Proposed Entity Model (Approach B)

### Entity Hierarchy

```
venue (the physical place — persists forever)
  └── project (an engagement — each time asked to bid/work)
       └── fee (a pricing proposal — addressed to a specific client)
            └── [package scope: disciplines + zones, with delivery stages]
```

### 4.1 `venue` (NEW TABLE)

The physical place. Has no status — it just exists. Status lives on engagements.

| Field | Type | Notes |
|-------|------|-------|
| `id` | record | Auto-generated |
| `name` | string | "Hittin Resort", "WAMI P600 Theater", "Lvl 63" |
| `name_short` | string | Short reference name |
| `location.city` | string | City name |
| `location.country` | string | Country name |
| `location.area` | string | District/area |
| `tags` | array\<string\> | Freeform: "hotel", "theater", "mixed-use", "residential" |
| `notes` | string | General notes about the venue |
| `time.created_at` | datetime | Auto |
| `time.updated_at` | datetime | Auto |

**No status field.** A venue is permanent. Whether work is active is determined by its engagements.

**Indexes:** name (search), location.country, tags

### 4.2 `project` (MODIFIED — now means "engagement")

Each time Emittiv is asked to bid or work on a venue. Keeps YY-CCCNN numbering.

| Field | Change | Type | Notes |
|-------|--------|------|-------|
| `venue_id` | **NEW** | record\<venue\> | Which venue this engagement is for |
| `number` | keep | object | YY-CCCNN project number, unchanged |
| `name` | keep | string | Engagement-specific name |
| `name_short` | keep | string | Short name |
| `status` | **REVISED** | string | New engagement lifecycle (see §5) |
| `folder` | keep | string | File system folder path |
| `city` | keep (deprecated) | string | Migrate to venue.location later |
| `country` | keep (deprecated) | string | Migrate to venue.location later |
| `area` | keep (deprecated) | string | Migrate to venue.location later |
| `time` | keep | object | created_at, updated_at |

**No company_id/contact_id on project.** The client relationship lives on the fee — fee proposals are addressed to specific companies/contacts. Who's involved in a project is derived from its fees.

**Common case optimization:** When creating a new project, the UI auto-creates a venue from the project name/location if no existing venue is selected. One-click for the simple case, explicit venue selection for re-engagements.

### 4.3 `fee` (MODIFIED — now means "pricing proposal")

The actual pricing document sent to a client. Linked to an engagement.

| Field | Change | Type | Notes |
|-------|--------|------|-------|
| `project_id` | keep | record\<projects\> | Links to the engagement |
| `company_id` | **KEEP** | record\<company\> | Who this proposal is addressed to |
| `contact_id` | **KEEP** | record\<contacts\> | Primary contact for this proposal |
| `package` | keep | string | Freeform scope description (disciplines covered) |
| `status` | **REVISED** | string | Proposal-specific lifecycle (see §5) |
| `revisions` | keep | array\<object\> | Revision history |
| `rev` | keep | int (computed) | Latest revision number |
| `name` | keep | string | Fee name |
| `number` | keep | string | Fee number (e.g., 25-97105-FP) |
| `issue_date` | keep | string | YYMMDD format |
| `staff_*` | keep | string | Staff details |
| `strap_line` | keep | string | Tagline |
| `pricing` | keep | object | Pricing calculator data |
| `time` | keep | object | created_at, updated_at |

**company_id and contact_id stay on fee.** The fee is a document addressed to someone — different fees on the same project can go to different clients. The JSON export needs company/contact info on the fee. This is not deprecated.

### 4.4 Package Scope (DEFERRED — field on fee for now)

Currently `package` is a freeform string on `fee` describing the disciplines covered. This is sufficient for now.

**Future evolution:** If package management becomes complex (tracking delivery stages per package, separate invoicing), extract to a `package` table linked to fee:
```
fee -covers-> package { disciplines: ["LX", "AV"], zones: ["Theater", "Lobby"], stages: {...} }
```

This aligns with the path toward Approach C (graph model).

### 4.5 Delivery Stages (DEFERRED — stays in pricing calculator)

Design stages (CON, SD, DD, CD) and post-contract stages currently live in the pricing calculator's `stages` object within the fee. They serve multiple purposes:
- Fee pricing breakdown per stage
- Progress tracking (future)
- Invoicing milestones (future)

**For now:** Stages stay embedded in fee pricing data. The project status (`Design` vs `Construction`) captures the high-level delivery phase. Sub-stage detail (which design stage are we in, what % complete) is deferred for a future iteration once the venue/project/fee restructure is bedded in.

**Revisit when:** Invoicing integration needs stage-level billing, or progress tracking per-stage/per-package becomes a workflow need.

---

## 5. Status Lifecycles

### Decision Tree: What status goes where?

```
Is it about the PHYSICAL PLACE?
  → No status. Venues just exist.

Is it about WHETHER WE'RE ENGAGED to work on it?
  → PROJECT (engagement) status:
    Lead → RFP → Submitted → Awarded → Design → Construction → Completed
    Dead ends: Lost, No Response, Cancelled, On Hold, Superseded

Is it about A SPECIFIC PRICING DOCUMENT we sent?
  → FEE (proposal) status:
    Draft → Sent → Negotiation → Accepted → Rejected → No Response → Superseded

Is it about DELIVERY PROGRESS on awarded work?
  → STAGE tracking (stays in pricing calculator for now)
    Design phase: CON → SD → DD → CD
    Construction phase: CA + post-contract stages
```

### 5.1 Project (Engagement) Statuses

| Status | Meaning | Transitions From | Notes |
|--------|---------|-----------------|-------|
| `Lead` | Heard about opportunity, not yet asked to price | (initial) | Available but opt-in; RFP is default |
| `RFP` | Request for proposal received | Lead | Default status for new projects |
| `Submitted` | Fee proposal(s) sent, awaiting response | RFP | Fee proposals are out |
| `Awarded` | Won the work, not yet started | Submitted | Fee accepted by client |
| `Design` | Design phase active (CON/SD/DD/CD) | Awarded | Producing drawings and specs |
| `Construction` | Construction phase active (CA/commissioning) | Design | Site visits, RFIs, shop drawing review |
| `Completed` | All work delivered and closed out | Construction, Design | Project fully closed |
| `Lost` | Explicitly rejected / went with competitor | Submitted, RFP | Known rejection |
| `No Response` | Went silent, presumed dead | Submitted, RFP | Not formally rejected, just cold |
| `Cancelled` | Project cancelled by client | Any non-terminal | Client pulled the project |
| `On Hold` | Paused, may resume | Any non-terminal | Can resume to previous state |
| `Superseded` | Replaced by newer engagement on same venue | Design, Awarded, Submitted | Design complete but project didn't progress to construction under this engagement |

**Transition rules:**
```
Lead ──→ RFP ──→ Submitted ──→ Awarded ──→ Design ──→ Construction ──→ Completed
  │        │          │           │          │            │
  │        │          ├── Lost    │          │            │
  │        │          ├── No Resp │          │            │
  │        │          │           │          │            │
  └──┬─────┴──────────┴───────────┴──────────┴────────────┘
     │
     ├──→ Cancelled (from any non-terminal)
     ├──→ On Hold (from any non-terminal, can resume)
     └──→ Superseded (from Submitted, Awarded, Design)
```

### 5.2 Fee (Proposal) Statuses

| Status | Meaning | Transitions From | Notes |
|--------|---------|-----------------|-------|
| `Draft` | Being prepared, not yet sent | (initial) | Default for new fees |
| `Sent` | Delivered to client | Draft | Proposal is out |
| `Negotiation` | Discussing terms with client | Sent | Optional — may skip straight to Accepted/Rejected |
| `Accepted` | Client accepted this proposal | Sent, Negotiation | Triggers project → Awarded |
| `Rejected` | Client explicitly declined | Sent, Negotiation | Triggers folder move prompt |
| `No Response` | Gone cold, no reply received | Sent, Negotiation | Triggers folder move prompt |
| `Superseded` | Replaced by a newer revision | Draft, Sent, Negotiation | When new revision created |

**Transition rules:**
```
Draft ──→ Sent ──→ Negotiation (optional)
            │              │
            ├── Accepted ←─┤
            ├── Rejected ←─┤
            ├── No Response ←─┤
            └── Superseded ←─┘
                   ↑
Draft ─── Superseded (can supersede before sending)
```

### 5.3 Fee → Project Status Cascading

| Fee Event | Project Effect | Folder Action |
|-----------|---------------|---------------|
| Any fee → `Accepted` | Project → `Awarded` (if not already further) | — |
| Fee → `Rejected` or `No Response` | If **no other active fees** remain on project: prompt project status change | Prompt "Move folder to lost?" |
| Fee → `Rejected` or `No Response` | If **other active fees** still exist: no project change | No folder move |
| Fee → `Superseded` | No project change | — |
| All fees terminal (`Rejected`/`No Response`) | Project → `Lost` or `No Response` (user confirms) | Folder move to lost |

**"Active fees"** for cascade purposes = fees with status `Draft`, `Sent`, or `Negotiation`.

### 5.4 Dashboard Metrics (Revised)

| Pill | Query | Current Count |
|------|-------|---------------|
| **Active Fees** | fee.status IN [`Draft`, `Sent`, `Negotiation`] | 0 (all existing data is historical) |
| **Active Projects** | project.status IN [`RFP`, `Submitted`, `Awarded`, `Design`, `Construction`] | 5 (3 RFP + 2 Active/Awarded) |
| **Total Fees** | count(fee) | 31 |
| **Total Projects** | count(projects) | 61 |

**Constants cleanup:** Single source of truth in `constants.ts` — `ACTIVE_FEE_STATUSES` and `ACTIVE_PROJECT_STATUSES`. Remove hardcoded lists from `stores.ts` and `operations.rs`.

**Note:** After migration, the "Active Fees" count will be 0 because all existing data is historical. This is correct — there are no proposals currently being drafted or negotiated. The count will grow naturally as new work comes in.

---

## 6. Decision Trees for Common Scenarios

### 6.1 New Opportunity Arrives

```
Have we worked on this venue before?
│
├── NO → Create new venue + new project (engagement)
│        UI auto-creates venue from project name/location
│        Simple one-step process for common case
│
└── YES → Is it the same client?
          │
          ├── YES, same scope → Consider revising existing fee
          │                     (add revision to existing fee record)
          │
          ├── YES, different scope → Create new project linked to same venue
          │                          New project number, new fee(s)
          │
          └── NO, different client → Create new project linked to same venue
                                     Fee addressed to new company/contact
                                     New project number
```

### 6.2 Submitting a Fee Proposal

```
Is this the first proposal for this engagement?
│
├── YES → Create fee with status "Draft"
│         Set company_id/contact_id (who it's addressed to)
│         Set package scope (disciplines)
│         Fill pricing calculator
│         When ready → status = "Sent"
│
└── NO → Is it a revision of an existing proposal?
         │
         ├── YES → Add revision to existing fee record
         │         Previous revision data preserved in revisions[] array
         │         If scope changed significantly, note in revision
         │
         └── NO → Create new fee record on same project
                   Different package/scope = different fee
                   May be addressed to same or different client
                   Numbering: 25-97105-LX-FP, 25-97105-AV-FP, etc.
```

### 6.3 Client Response

```
Client responded to our fee proposal:
│
├── Accepted → Fee status = "Accepted"
│              Project status = "Awarded" (if not already further)
│              Delivery phase begins (project → Design)
│
├── Wants changes → Fee status = "Negotiation"
│                    Create new revision when terms agreed
│                    Old revision auto-set to "Superseded"
│
├── Rejected → Fee status = "Rejected"
│              If no other active fees on project:
│                → Prompt project status change + folder move
│              If other active fees remain:
│                → No project change
│
└── No response (gone cold) → Fee status = "No Response"
                               Same cascade logic as Rejected
```

### 6.4 Re-engagement (v2 Redesign)

```
Previous project exists for this venue (e.g., completed 2 years ago):
│
├── Same client returns
│   → Create new project linked to SAME venue
│   → New project number (YY-CCCNN)
│   → Previous project → "Superseded" (if design didn't reach construction)
│   → Navigate: venue → previous project → access old fee data/BOQ
│   → Name might include "v2" or "Phase 2" or "Redesign"
│
└── Different client for same venue
    → Create new project linked to SAME venue
    → Fee addressed to new company/contact
    → Still have access to previous engagement data via venue
    → No access restrictions (single-user app, no Chinese walls needed)
```

### 6.5 Cascading Status Changes

```
Project status changes to Cancelled:
│
├── All related fees with status Draft/Sent/Negotiation
│   → Set to "Rejected"
│
└── Fees already Accepted/Rejected/No Response/Superseded
    → No change (terminal states)

Project status changes to On Hold:
│
└── Fees freeze in current state (no automatic change)
    → Resume returns fees to their pre-hold state

Fee status changes to Rejected or No Response:
│
├── Other active fees remain on project
│   → No project change, no folder move
│
└── Last active fee on project
    → Prompt: "Move folder to lost?"
    → If yes: project → Lost/No Response + folder move
    → If no: just mark the fee, leave project alone
```

---

## 7. Migration Strategy (High Level)

### Phase 1: Schema changes (non-breaking)
1. Create `venue` table
2. Add `venue_id` field to `projects` (optional initially)
3. Update status ASSERT lists to include all new values

### Phase 2: Data migration
1. Auto-create venue records from unique project names/locations
2. Link projects to venues
3. Map old statuses to new:

**Project status mapping:**
| Old | New | Rationale |
|-----|-----|-----------|
| Draft | Lead | Not yet bidding |
| RFP | RFP | Keep |
| Active | Design | Was in active design work |
| Awarded | Awarded | Keep |
| Completed | Completed | Keep |
| Lost | Lost | Keep (unless should be No Response — manual review) |
| Cancelled | Cancelled | Keep |
| On Hold | On Hold | Keep |
| Revised | Superseded | Confirmed: all 4 records are engagements replaced by newer ones |

**Revised project records (confirmed mapping):**
| Project | Old Status | New Status | Reason |
|---------|-----------|-----------|--------|
| `24-97105` MAF MiCC Kids | Revised | Superseded | → `25-97104` (v2), now in construction |
| `25-97104` MAF Kids FEC | Revised | Superseded | → `25-97106` (v3), scope split into smaller builds |
| `25-97101` Shanghai Tang | Revised | Superseded | → `25-97105` (v2), now in construction |
| `24-97107` Reserve Cut | Revised | Superseded | Now "Lvl 63", on hold, building soon |

**Fee status mapping:**
| Old | New | Rationale |
|-----|-----|-----------|
| Draft | Draft | Keep |
| Sent | Sent | Keep |
| Negotiation | Negotiation | Keep |
| Awarded | Accepted | Fee was accepted by client |
| Completed | Accepted | Fee was accepted (delivery tracked at project level) |
| Lost | No Response | Most "Lost" fees were never formally rejected — manual review recommended |
| Cancelled | Rejected | Never proceeded |
| On Hold | Draft | Paused proposals go back to draft |
| Revised | Superseded | Replaced by newer revision |
| Active | Sent | Was out there, in play |

**Note on Lost → No Response:** The majority of "Lost" fees (21 records) likely went cold rather than being explicitly rejected. Review each record during migration — those with known rejection should stay `Rejected`, those that just went silent should be `No Response`.

### Phase 3: UI updates
1. Add venue management (list, create, link)
2. Update project creation flow (auto-create venue or link existing)
3. Update status dropdowns and filters
4. Update dashboard metrics with constants from `constants.ts`
5. Add venue → engagements → fees navigation
6. Update constants: `ACTIVE_FEE_STATUSES`, `ACTIVE_PROJECT_STATUSES`
7. Remove hardcoded status lists from `stores.ts` and `operations.rs`

### Phase 4: Cleanup
1. Migrate city/country/area from project to venue.location
2. Remove old status values from ASSERT lists
3. Remove deprecated location fields from project schema

---

## 8. Open Questions — RESOLVED (2026-02-25)

### Q1: "Revised" projects ✅
**Answer:** "Revised" = superseded by a newer engagement on the same venue. All 4 records confirmed: MAF MiCC Kids → v2 (in construction), MAF Kids FEC → v3 (scope split), Shanghai Tang → v2 (in construction), Reserve Cut → now "Lvl 63" (on hold).
**Decision:** Map to new `Superseded` status. Venue linkage captures the relationship structurally.

### Q2: Access control for competing bids ✅
**Answer:** No restrictions needed. Single-user app, no Chinese walls. All engagements on a venue are visible.
**Decision:** No access control. Revisit only if Emittiv grows to a multi-person team with client confidentiality requirements.

### Q3: Package as entity vs. field ✅
**Answer:** Keep as freeform string on fee. The `package` field relates to disciplines covered (LX, AV, CTL, etc.), not a separable deliverable unit.
**Decision:** No change. Extract to entity only when delivery stage tracking per-package or invoicing integration requires it.

### Q4: Fee numbering ✅
**Answer:** Current pattern works. Format: `{project_number}-{scope}-FP` (e.g., `25-97105-FP`, `25-97105-LX-FP`). Multiple fees on same project use scope/discipline identifiers. Stacks neatly in folder views.
**Decision:** No change needed.

### Q5: "Lead" status ✅
**Answer:** Include it. Not used today but zero cost to add, provides pipeline visibility as workflow evolves.
**Decision:** `Lead` available but opt-in. `RFP` remains the default status when creating new projects.

### Q6: Delivery stage tracking ✅
**Answer:** Keep stages in pricing calculator. Add `Design` and `Construction` as project statuses to capture the high-level delivery phase. Sub-stage detail (CON/SD/DD/CD progress tracking) deferred.
**Decision:** Stages stay embedded in fee pricing data. Revisit stage model after venue/project/fee restructure is stable and stage/invoicing needs become clearer.

### Q7: Status constants and fee lifecycle ✅
**Answer:** Clean up all hardcoded status lists. Single source of truth in `constants.ts`. Fee statuses: Draft → Sent → Negotiation (optional) → Accepted/Rejected/No Response/Superseded. Fee cascade to project: only when last active fee resolves. Folder move prompt: only when no active fees remain.
**Decision:** `ACTIVE_FEE_STATUSES = ['Draft', 'Sent', 'Negotiation']`, `ACTIVE_PROJECT_STATUSES = ['RFP', 'Submitted', 'Awarded', 'Design', 'Construction']`. Remove hardcoded lists from stores.ts and operations.rs.

### Q8: company_id / contact_id on fees ✅
**Answer:** Keep on fee, NOT deprecated. The fee is a document addressed to a specific company/contact. Different fees on the same project can go to different clients. The JSON export needs this data. No company_id/contact_id on project — client relationship is derived from fees.
**Decision:** No change to fee fields. No new client fields on project.

---

## 9. Files Affected (Preliminary)

### Backend (Rust)
- `src-tauri/src/db/types.rs` — New Venue struct, updated Project status enum
- `src-tauri/src/db/operations.rs` — New venue CRUD, updated queries, status changes, cascade logic
- `src-tauri/src/db/tests.rs` — New tests for venue operations, migration tests
- `src-tauri/src/lib.rs` — Register new Tauri commands

### Frontend (Svelte/TypeScript)
- `src/lib/constants.ts` — Updated status lists, `ACTIVE_FEE_STATUSES`, `ACTIVE_PROJECT_STATUSES`
- `src/lib/stores.ts` — Venue store, updated dashboard metrics using constants
- `src/lib/types/index.ts` — Venue type, updated Project/Fee types
- `src/routes/Proposals.svelte` — Updated filters, status display
- `src/routes/Projects.svelte` — Updated filters, venue linking
- New: `src/routes/Venues.svelte` — Venue list/management
- New: `src/lib/components/VenueModal.svelte` — Venue create/edit

### Database
- Migration DDL for venue table creation
- Data migration script for creating venues from existing projects
- Status mapping script (with manual review step for Lost → No Response)

---

## 10. Design Principles

These principles guided the decisions and should guide implementation:

1. **The tool must reduce friction, not add it.** Every field and status should earn its place by being useful in daily workflow.
2. **Optimize for the common case.** One venue, one client, one fee = one-click creation. Complex cases supported but not forced.
3. **Don't over-engineer.** Keep package as a string, keep stages in pricing, keep access unrestricted — until real workflow needs justify the complexity.
4. **Status tells a story.** Each entity's status captures its own lifecycle. No conflation between "did we win?" (project) and "was this document accepted?" (fee).
5. **Data model matches mental model.** Venue = place, project = engagement, fee = document. Three concepts, three entities.

---

*Design discussion 2026-02-24. Reviewed and approved 2026-02-25. Ready for implementation planning.*
