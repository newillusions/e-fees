# Domain Model Restructure — Design Notes

**Date:** 2026-02-24
**Status:** Draft — In Discussion (to be reviewed morning of 2026-02-25)
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

4. **Revisions are loosely defined**: A revision is typically the same client on the same project, but scope can change between revisions. A "revision" can even span years if the same client comes back for the same venue.

5. **Cross-engagement access is required**: When working on a v2 redesign, Emittiv needs to access the previous engagement's fee data, pricing, BOQ, and design docs from within E-Fees.

6. **Most projects are simple**: One venue, one client, one engagement, one fee. The complex cases (multi-client, re-engagement, competing bids) need to be supported but should not add friction to the common case.

7. **Delivery stages matter for awarded work**: Once a package is won, design stages (CON, SD, DD, CD) and post-contract stages need tracking for progress and invoicing.

8. **Nothing is set in stone**: The business is still forming patterns. The data model should be flexible enough to adjust as workflows evolve.

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
       └── fee (a pricing proposal — may have revisions)
            └── [package scope: disciplines + zones, with delivery stages]
```

### 4.1 `venue` (NEW TABLE)

The physical place. Has no status — it just exists. Status lives on engagements.

| Field | Type | Notes |
|-------|------|-------|
| `id` | record | Auto-generated |
| `name` | string | "Hittin Resort", "WAMI P600 Theater" |
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
| `company_id` | **NEW** | record\<company\> | Who's engaging Emittiv (move from fee) |
| `contact_id` | **NEW** | record\<contacts\> | Primary contact for this engagement |
| `number` | keep | object | YY-CCCNN project number, unchanged |
| `name` | keep | string | Engagement-specific name |
| `name_short` | keep | string | Short name |
| `status` | **REVISED** | string | New engagement lifecycle (see §5) |
| `folder` | keep | string | File system folder path |
| `city` | keep (deprecated) | string | Migrate to venue.location later |
| `country` | keep (deprecated) | string | Migrate to venue.location later |
| `area` | keep (deprecated) | string | Migrate to venue.location later |
| `time` | keep | object | created_at, updated_at |

**Key change:** `company_id` and `contact_id` move here from `fee`. The client relationship is with the engagement, not the individual proposal. Fees inherit the engagement's client unless overridden.

**Common case optimization:** When creating a new project, the UI auto-creates a venue from the project name/location if no existing venue is selected. One-click for the simple case, explicit venue selection for re-engagements.

### 4.3 `fee` (MODIFIED — now means "pricing proposal")

The actual pricing document sent to a client. Linked to an engagement.

| Field | Change | Type | Notes |
|-------|--------|------|-------|
| `project_id` | keep | record\<projects\> | Links to the engagement |
| `package` | keep | string | Freeform scope description |
| `status` | **REVISED** | string | Proposal-specific lifecycle (see §5) |
| `company_id` | **DEPRECATED** | record\<company\> | Inherited from project; keep for backward compat during migration |
| `contact_id` | **DEPRECATED** | record\<contacts\> | Inherited from project; keep for backward compat during migration |
| `revisions` | keep | array\<object\> | Revision history |
| `rev` | keep | int (computed) | Latest revision number |
| `name` | keep | string | Fee name |
| `number` | keep | string | Fee number |
| `issue_date` | keep | string | YYMMDD format |
| `staff_*` | keep | string | Staff details |
| `strap_line` | keep | string | Tagline |
| `pricing` | keep | object | Pricing calculator data |
| `time` | keep | object | created_at, updated_at |

### 4.4 Package Scope (DEFERRED — field on fee for now)

Currently `package` is a freeform string on `fee`. This is sufficient for now.

**Future evolution:** If package management becomes complex (tracking delivery stages per package, separate invoicing), extract to a `package` table linked to fee:
```
fee -covers-> package { disciplines: ["LX", "AV"], zones: ["Theater", "Lobby"], stages: {...} }
```

This aligns with the path toward Approach C (graph model).

---

## 5. Status Lifecycles

### Decision Tree: What status goes where?

```
Is it about the PHYSICAL PLACE?
  → No status. Venues just exist.

Is it about WHETHER WE'RE ENGAGED to work on it?
  → PROJECT (engagement) status:
    Lead → RFP → Bidding → Awarded → Active → Completed
    Dead ends: Lost, Cancelled, On Hold

Is it about A SPECIFIC PRICING DOCUMENT we sent?
  → FEE (proposal) status:
    Draft → Sent → Negotiation → Accepted → Rejected → Superseded

Is it about DELIVERY PROGRESS on awarded work?
  → STAGE tracking (already exists in pricing calculator)
    CON → SD → DD → CD (design stages)
    Post-contract stages as configured
```

### 5.1 Project (Engagement) Statuses

| Status | Meaning | Transitions From | Notes |
|--------|---------|-----------------|-------|
| `Lead` | Heard about opportunity, not yet asked to price | (initial) | New status |
| `RFP` | Request for proposal received | Lead | Existing status |
| `Bidding` | Actively pricing / fee proposals being prepared | RFP | Replaces "Active" in current model |
| `Awarded` | Won the work, not yet started | Bidding | Keep |
| `Active` | Design/delivery work in progress | Awarded | Keep |
| `Completed` | All work delivered and closed out | Active | Keep |
| `Lost` | Didn't win the bid | Bidding, RFP | Keep |
| `Cancelled` | Project cancelled by client | Any except Completed | Keep |
| `On Hold` | Paused, may resume | Any except Completed, Lost | Keep |

**Transition rules:**
```
Lead ──→ RFP ──→ Bidding ──→ Awarded ──→ Active ──→ Completed
  │        │        │           │          │
  │        │        ├──→ Lost   │          │
  │        │        │           │          │
  └──┬─────┴────────┴───────────┴──────────┘
     │
     ├──→ Cancelled (from any non-terminal)
     └──→ On Hold (from any non-terminal, can resume)
```

### 5.2 Fee (Proposal) Statuses

| Status | Meaning | Transitions From | Notes |
|--------|---------|-----------------|-------|
| `Draft` | Being prepared, not yet sent | (initial) | Keep |
| `Sent` | Delivered to client | Draft | Keep |
| `Negotiation` | Client responded, discussing terms | Sent | Keep |
| `Accepted` | Client accepted this proposal | Sent, Negotiation | Replaces "Awarded" on fees |
| `Rejected` | Client rejected this proposal | Sent, Negotiation | More specific than "Lost" |
| `Superseded` | Replaced by a newer revision | Any | Replaces "Revised" |

**Transition rules:**
```
Draft ──→ Sent ──→ Negotiation ──→ Accepted
                      │
                      ├──→ Rejected
                      │
           └──────────┴──→ Superseded (when new revision created)
```

### 5.3 Dashboard Metrics (Revised)

| Pill | Query | Current Count |
|------|-------|---------------|
| **Active Fees** | fee.status IN [`Draft`, `Sent`, `Negotiation`] | 0 (all existing data is historical) |
| **Active Projects** | project.status IN [`RFP`, `Bidding`, `Awarded`, `Active`] | 5 (3 RFP + 2 Active/Awarded) |
| **Total Fees** | count(fee) | 31 |
| **Total Projects** | count(projects) | 61 |

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
                                     Different company_id, new project number
```

### 6.2 Submitting a Fee Proposal

```
Is this the first proposal for this engagement?
│
├── YES → Create fee with status "Draft"
│         Set package scope (disciplines + zones)
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
                   (e.g., separate fee for AV vs. lighting)
```

### 6.3 Client Response

```
Client responded to our fee proposal:
│
├── Accepted → Fee status = "Accepted"
│              Project status = "Awarded" (if not already)
│              Delivery stage tracking begins
│
├── Wants changes → Fee status = "Negotiation"
│                    Create new revision when terms agreed
│                    Old revision auto-set to "Superseded"
│
├── Rejected → Fee status = "Rejected"
│              If all fees on project rejected → Project status = "Lost"
│
└── No response / project died → Project status = "Lost" or "Cancelled"
                                  Fee statuses follow project
```

### 6.4 Re-engagement (v2 Redesign)

```
Previous project exists for this venue (e.g., completed 2 years ago):
│
├── Same client returns
│   → Create new project linked to SAME venue
│   → New project number (YY-CCCNN)
│   → Navigate: venue → previous project → access old fee data/BOQ
│   → Name might include "v2" or "Phase 2" or "Redesign"
│
└── Different client for same venue
    → Create new project linked to SAME venue
    → Different company_id
    → Still have access to previous engagement data via venue
    → (Access control TBD — may want to restrict visibility
       of competitor-commissioned work)
```

### 6.5 Cascading Status Changes

```
Project status changes to Cancelled or On Hold:
│
├── All related fees with status Draft/Sent/Negotiation
│   → Automatically set to "Rejected" (Cancelled)
│   → Or freeze in current state (On Hold)
│
└── Fees already Accepted/Rejected/Superseded
    → No change (terminal states)

Project status changes to Lost:
│
├── Fees in Sent/Negotiation → "Rejected"
├── Fees in Draft → "Rejected" (never sent)
└── Fees already terminal → No change
```

---

## 7. Migration Strategy (High Level)

### Phase 1: Schema changes (non-breaking)
1. Create `venue` table
2. Add `venue_id` field to `projects` (optional initially)
3. Add `company_id`, `contact_id` to `projects` (optional initially)
4. Add new status values to ASSERT lists

### Phase 2: Data migration
1. Auto-create venue records from unique project names/locations
2. Link projects to venues
3. Copy company_id/contact_id from fees to projects
4. Map old statuses to new:

**Project status mapping:**
| Old | New | Rationale |
|-----|-----|-----------|
| Draft | Lead | Not yet bidding |
| RFP | RFP | Keep |
| Active | Active | Keep |
| Awarded | Awarded | Keep |
| Completed | Completed | Keep |
| Lost | Lost | Keep |
| Cancelled | Cancelled | Keep |
| On Hold | On Hold | Keep |
| Revised | *(per-record)* | Needs manual review — may mean "superseded by v2" |

**Fee status mapping:**
| Old | New | Rationale |
|-----|-----|-----------|
| Draft | Draft | Keep |
| Sent | Sent | Keep |
| Negotiation | Negotiation | Keep |
| Awarded | Accepted | Fee was accepted by client |
| Completed | Accepted | Fee was accepted (delivery tracked at project level) |
| Lost | Rejected | Client rejected |
| Cancelled | Rejected | Never proceeded |
| On Hold | Draft | Paused proposals go back to draft |
| Revised | Superseded | Replaced by newer revision |
| Active | Sent | Was out there, in play |

### Phase 3: UI updates
1. Add venue management (list, create, link)
2. Update project creation flow (auto-create venue or link existing)
3. Update status dropdowns and filters
4. Update dashboard metrics
5. Add venue → engagements → fees navigation

### Phase 4: Cleanup
1. Remove deprecated fields (company_id, contact_id from fee)
2. Migrate city/country/area from project to venue
3. Remove old status values from ASSERT lists

---

## 8. Open Questions (For Morning Review)

1. **"Revised" projects**: The 4 projects with status "Revised" — do these mean "this project was superseded by a newer engagement on the same venue"? Or something else? Need to review each record.

2. **Access control for competing bids**: If two different clients engage Emittiv for the same venue, should each client's fee data be visible when working on the other's engagement? Or should there be visibility boundaries?

3. **Package as entity vs. field**: Keep `package` as a freeform string on fee for now? Or introduce a structured `package` field/table from the start? Current thinking: keep it simple, extract later.

4. **Fee numbering**: Currently fees use the format `{project_number}_{revision}` (e.g., `25_97102_1`). Should this change? Or is this still the right pattern?

5. **"Lead" status**: Is this useful? Or do projects only enter the system when an RFP is received? If Emittiv tracks opportunities before being formally asked to bid, Lead is valuable. Otherwise, RFP is the entry point.

6. **Delivery stage tracking**: Currently in the pricing calculator. Does this need to be promoted to a first-class entity for tracking progress per-package? Or is the current approach (stages embedded in fee pricing data) sufficient?

7. **"Active" status on fees**: The current code defines `ACTIVE_PROPOSAL_STATUSES` as `['Draft', 'Sent', 'Negotiation']` in `constants.ts` but the dashboard query uses a broader set. After migration, should the constant be updated to just these three, with everything else being separate lifecycle states?

8. **Backward compatibility**: How long do we keep the deprecated fields (company_id/contact_id on fee)? Immediate removal after migration, or a transition period?

---

## 9. Files Affected (Preliminary)

### Backend (Rust)
- `src-tauri/src/db/types.rs` — New Venue struct, updated Project/Fee structs
- `src-tauri/src/db/operations.rs` — New venue CRUD, updated queries, status changes
- `src-tauri/src/db/tests.rs` — New tests for venue operations, migration tests
- `src-tauri/src/lib.rs` — Register new Tauri commands

### Frontend (Svelte/TypeScript)
- `src/lib/constants.ts` — Updated status lists, new venue types
- `src/lib/stores.ts` — Venue store, updated dashboard metrics
- `src/lib/types/index.ts` — Venue type, updated Project/Fee types
- `src/routes/Proposals.svelte` — Updated filters, status display
- `src/routes/Projects.svelte` — Updated filters, venue linking
- New: `src/routes/Venues.svelte` — Venue list/management
- New: `src/lib/components/VenueModal.svelte` — Venue create/edit

### Database
- Migration DDL for venue table creation
- Data migration script for creating venues from existing projects
- Status mapping script

---

*This document captures the design discussion of 2026-02-24. To be reviewed and refined on 2026-02-25 before implementation planning begins.*
