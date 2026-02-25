# Domain Model Restructure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Introduce venue entity, refine project as engagement with Design/Construction phases, clean up fee/project status lifecycles, and update all frontend/backend code to match.

**Architecture:** New `venue` table in SurrealDB linked to projects via `venue_id`. Updated status ASSERT lists on both `projects` and `fee` tables. Frontend gets new Venue route/store/modal, updated constants, and status-aware cascade logic. Backend gets new Venue struct + CRUD operations. Data migration maps existing records to new schema.

**Tech Stack:** Rust (Tauri v2), Svelte 5, SurrealDB v3, TypeScript

**Design Doc:** `docs/plans/2026-02-24-domain-model-restructure-design.md`

---

## Task 1: Database Schema — Create Venue Table

**Files:**
- Create: `scripts/migration/001-create-venue-table.surql`

**Step 1: Write the venue table DDL**

```sql
-- 001-create-venue-table.surql
-- Domain Model Restructure: Phase 1 — Create venue table

DEFINE TABLE venue SCHEMAFULL;

DEFINE FIELD name ON venue TYPE string ASSERT string::len($value) > 0;
DEFINE FIELD name_short ON venue TYPE string DEFAULT '';
DEFINE FIELD location ON venue TYPE object DEFAULT {};
DEFINE FIELD location.city ON venue TYPE string DEFAULT '';
DEFINE FIELD location.country ON venue TYPE string DEFAULT '';
DEFINE FIELD location.area ON venue TYPE string DEFAULT '';
DEFINE FIELD tags ON venue TYPE array<string> DEFAULT [];
DEFINE FIELD notes ON venue TYPE string DEFAULT '';
DEFINE FIELD time ON venue TYPE object VALUE { created_at: time::now(), updated_at: time::now() };
DEFINE FIELD time.created_at ON venue TYPE datetime VALUE $before OR time::now();
DEFINE FIELD time.updated_at ON venue TYPE datetime VALUE time::now();

DEFINE INDEX venue_name ON venue FIELDS name SEARCH ANALYZER SIMPLE BM25;
DEFINE INDEX venue_country ON venue FIELDS location.country;
```

**Step 2: Run DDL against dev DB**

Run: `curl -s -X POST "http://10.0.23.11:8000/sql" -u "martin:th38ret3ch" -H "surreal-ns: emittiv" -H "surreal-db: projects" -H "Accept: application/json" -H "Content-Type: text/plain" --data-binary @scripts/migration/001-create-venue-table.surql`

Expected: All statements return `"status": "OK"`

**Step 3: Verify table exists**

Run: `curl -s -X POST "http://10.0.23.11:8000/sql" -u "martin:th38ret3ch" -H "surreal-ns: emittiv" -H "surreal-db: projects" -H "Accept: application/json" -H "Content-Type: text/plain" -d "INFO FOR TABLE venue;"`

Expected: Shows field definitions matching the DDL

**Step 4: Commit**

```bash
git add scripts/migration/001-create-venue-table.surql
git commit -m "feat(db): create venue table for domain model restructure"
```

---

## Task 2: Database Schema — Add venue_id to Projects + Update Status Lists

**Files:**
- Create: `scripts/migration/002-update-project-fee-schemas.surql`

**Step 1: Write the schema update DDL**

```sql
-- 002-update-project-fee-schemas.surql
-- Domain Model Restructure: Phase 1 — Update project and fee schemas

-- Add venue_id to projects (optional for backward compat during migration)
DEFINE FIELD venue_id ON projects TYPE option<record<venue>>;

-- Update project status ASSERT to new lifecycle
DEFINE FIELD status ON projects TYPE string ASSERT $value INSIDE [
  'Lead', 'RFP', 'Submitted', 'Awarded', 'Design', 'Construction', 'Completed',
  'Lost', 'No Response', 'Cancelled', 'On Hold', 'Superseded'
] PERMISSIONS FOR select, create, update WHERE FULL;

-- Update fee status ASSERT to new lifecycle
DEFINE FIELD status ON fee TYPE string DEFAULT 'Draft' ASSERT $value INSIDE [
  'Draft', 'Sent', 'Negotiation', 'Accepted', 'Rejected', 'No Response', 'Superseded'
] PERMISSIONS FULL;
```

**Step 2: Run DDL against dev DB**

Run: `curl -s -X POST "http://10.0.23.11:8000/sql" -u "martin:th38ret3ch" -H "surreal-ns: emittiv" -H "surreal-db: projects" -H "Accept: application/json" -H "Content-Type: text/plain" --data-binary @scripts/migration/002-update-project-fee-schemas.surql`

Expected: All statements return `"status": "OK"`

**Step 3: Verify both schemas updated**

Run: `curl -s -X POST "http://10.0.23.11:8000/sql" -u "martin:th38ret3ch" -H "surreal-ns: emittiv" -H "surreal-db: projects" -H "Accept: application/json" -H "Content-Type: text/plain" -d "INFO FOR TABLE projects; INFO FOR TABLE fee;" | python3 -c "import sys,json; [print(r['result'].get('fields',{}).get('status','')) for r in json.load(sys.stdin)]"`

Expected: Both show the new ASSERT lists

**Step 4: Commit**

```bash
git add scripts/migration/002-update-project-fee-schemas.surql
git commit -m "feat(db): add venue_id to projects, update status ASSERT lists"
```

---

## Task 3: Database — Data Migration Script

**Files:**
- Create: `scripts/migration/003-data-migration.surql`

**Step 1: Write the data migration**

```sql
-- 003-data-migration.surql
-- Domain Model Restructure: Phase 2 — Migrate existing data

-- Step 1: Map project statuses
-- Draft → Lead
UPDATE projects SET status = 'Lead' WHERE status = 'Draft';
-- Active → Design (these are projects in active design work)
UPDATE projects SET status = 'Design' WHERE status = 'Active';
-- Revised → Superseded (confirmed: all 4 records are superseded engagements)
UPDATE projects SET status = 'Superseded' WHERE status = 'Revised';

-- Step 2: Map fee statuses
-- Awarded → Accepted
UPDATE fee SET status = 'Accepted' WHERE status = 'Awarded';
-- Completed → Accepted (fee was accepted; delivery tracked at project level)
UPDATE fee SET status = 'Accepted' WHERE status = 'Completed';
-- Revised → Superseded
UPDATE fee SET status = 'Superseded' WHERE status = 'Revised';
-- Active → Sent
UPDATE fee SET status = 'Sent' WHERE status = 'Active';
-- On Hold → Draft
UPDATE fee SET status = 'Draft' WHERE status = 'On Hold';
-- Cancelled → Rejected
UPDATE fee SET status = 'Rejected' WHERE status = 'Cancelled';
-- Lost → No Response (majority went cold; review individually post-migration)
UPDATE fee SET status = 'No Response' WHERE status = 'Lost';
```

**Step 2: IMPORTANT — Review before running**

Before executing, verify the counts match expectations:
```sql
SELECT status, count() as cnt FROM projects GROUP BY status;
SELECT status, count() as cnt FROM fee GROUP BY status;
```

Expected project counts: Lost=36, Completed=11, Revised=4, Cancelled=3, RFP=3, Active=2, Awarded=2
Expected fee counts: Lost=21, Completed=4, Awarded=3, Revised=3

**Step 3: Run migration**

Run: `curl -s -X POST "http://10.0.23.11:8000/sql" -u "martin:th38ret3ch" -H "surreal-ns: emittiv" -H "surreal-db: projects" -H "Accept: application/json" -H "Content-Type: text/plain" --data-binary @scripts/migration/003-data-migration.surql`

**Step 4: Verify new status distribution**

```sql
SELECT status, count() as cnt FROM projects GROUP BY status;
SELECT status, count() as cnt FROM fee GROUP BY status;
```

Expected projects: Lost=36, Completed=11, Superseded=4, Cancelled=3, RFP=3, Design=2, Awarded=2
Expected fees: No Response=21, Accepted=7, Superseded=3

**Step 5: Commit**

```bash
git add scripts/migration/003-data-migration.surql
git commit -m "feat(db): data migration — map statuses to new lifecycle"
```

---

## Task 4: Backend — Venue Struct and Types

**Files:**
- Modify: `src-tauri/src/db/types.rs`

**Step 1: Add Venue struct after the Project structs (after line 169)**

Add these types to `src-tauri/src/db/types.rs` after the `NewProject` struct:

```rust
/// Venue location information.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue, Default)]
pub struct VenueLocation {
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub area: String,
}

/// Venue entity representing a physical place.
/// Venues have no status — they persist forever.
/// Whether work is active is determined by their linked engagements (projects).
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Venue {
    pub id: Option<RecordId>,
    pub name: String,
    #[serde(default)]
    pub name_short: String,
    #[serde(default)]
    pub location: VenueLocation,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: String,
    pub time: TimeStamps,
}

/// Venue creation struct without auto-managed fields.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct VenueCreate {
    pub name: String,
    #[serde(default)]
    pub name_short: String,
    #[serde(default)]
    pub location: VenueLocation,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: String,
}
```

**Step 2: Add venue_id to Project struct**

In the `Project` struct (line 128), add after `folder`:

```rust
    #[serde(default)]
    pub venue_id: Option<RecordId>,
```

In the `NewProject` struct (line 159), add after `folder`:

```rust
    #[serde(default)]
    pub venue_id: Option<String>,
```

**Step 3: Run tests to verify compilation**

Run: `cd /Volumes/base/dev/app/e-fees && cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5`

Expected: All existing tests pass (types are additive, no breaking changes)

**Step 4: Commit**

```bash
git add src-tauri/src/db/types.rs
git commit -m "feat(types): add Venue struct and venue_id to Project"
```

---

## Task 5: Backend — Venue CRUD Operations

**Files:**
- Modify: `src-tauri/src/db/operations.rs`

**Step 1: Add Venue import to the use block (line 12-16)**

Update the imports:

```rust
use super::{
    DatabaseManager, PaginatedResponse,
    Project, NewProject, Company, CompanyCreate,
    Contact, ContactCreate, Fee, FeeCreate, FeeUpdate, PricingUpdate,
    EntityCounts, ActivityLog, ActivityLogCreate,
    Venue, VenueCreate,
};
```

**Step 2: Add Venue operations block after Project operations (after line 66)**

```rust
// ==================== Venue Operations ====================

impl DatabaseManager {
    pub async fn get_venues(&self) -> Result<Vec<Venue>, Error> {
        let client = self.get_client()?;
        let mut response = client.query("SELECT * FROM venue ORDER BY name ASC").await?;
        let venues: Vec<Venue> = response.take(0)?;
        info!("Fetched {} venues", venues.len());
        Ok(venues)
    }

    pub async fn get_venues_page(&self, page: usize, page_size: usize) -> Result<PaginatedResponse<Venue>, Error> {
        self.paginate("venue", page, page_size).await
    }

    pub async fn get_venue_by_id(&self, id: &str) -> Result<Option<Venue>, Error> {
        self.get_by_id("venue", id).await
    }

    pub async fn create_venue(&self, venue: VenueCreate) -> Result<Venue, Error> {
        let client = self.get_client()?;
        info!("Creating venue: {}", venue.name);

        let sql = r#"
            CREATE venue CONTENT {
                name: $name,
                name_short: $name_short,
                location: {
                    city: $city,
                    country: $country,
                    area: $area
                },
                tags: $tags,
                notes: $notes
            };
        "#;

        let mut response = client.query(sql)
            .bind(("name", &venue.name))
            .bind(("name_short", &venue.name_short))
            .bind(("city", &venue.location.city))
            .bind(("country", &venue.location.country))
            .bind(("area", &venue.location.area))
            .bind(("tags", &venue.tags))
            .bind(("notes", &venue.notes))
            .await?;

        let created: Option<Venue> = response.take(0)?;
        created.ok_or_else(|| self.not_found_error("create venue"))
    }

    pub async fn update_venue(&self, id: &str, name: String, name_short: String, city: String, country: String, area: String, tags: Vec<String>, notes: String) -> Result<Venue, Error> {
        let client = self.get_client()?;
        info!("Updating venue: {}", id);

        let sql = r#"
            UPDATE type::thing($table, $id) SET
                name = $name,
                name_short = $name_short,
                location.city = $city,
                location.country = $country,
                location.area = $area,
                tags = $tags,
                notes = $notes;
        "#;

        let mut response = client.query(sql)
            .bind(("table", "venue"))
            .bind(("id", id))
            .bind(("name", &name))
            .bind(("name_short", &name_short))
            .bind(("city", &city))
            .bind(("country", &country))
            .bind(("area", &area))
            .bind(("tags", &tags))
            .bind(("notes", &notes))
            .await?;

        let updated: Option<Venue> = response.take(0)?;
        updated.ok_or_else(|| self.not_found_error("update venue"))
    }

    pub async fn delete_venue(&self, id: &str) -> Result<Venue, Error> {
        let client = self.get_client()?;
        info!("Deleting venue: {}", id);

        // Check for linked projects first
        let check_sql = "SELECT count() as count FROM projects WHERE venue_id = type::thing('venue', $id) GROUP ALL;";
        let mut check_response = client.query(check_sql).bind(("id", id)).await?;
        let count_result: Option<serde_json::Value> = check_response.take(0)?;
        let linked_count = count_result.and_then(|v| v.get("count").and_then(|c| c.as_u64())).unwrap_or(0);

        if linked_count > 0 {
            return Err(Error::Api(surrealdb::error::Api::Query(
                format!("Cannot delete venue: {} linked project(s) exist", linked_count)
            )));
        }

        let sql = "DELETE type::thing($table, $id) RETURN BEFORE;";
        let mut response = client.query(sql)
            .bind(("table", "venue"))
            .bind(("id", id))
            .await?;

        let deleted: Option<Venue> = response.take(0)?;
        deleted.ok_or_else(|| self.not_found_error("delete venue"))
    }

    pub async fn get_projects_for_venue(&self, venue_id: &str) -> Result<Vec<Project>, Error> {
        let client = self.get_client()?;
        let sql = "SELECT * FROM projects WHERE venue_id = type::thing('venue', $id) ORDER BY time.created_at DESC;";
        let mut response = client.query(sql).bind(("id", venue_id)).await?;
        let projects: Vec<Project> = response.take(0)?;
        Ok(projects)
    }
}
```

**Step 3: Update get_entity_counts to use new status lists (line 515-520)**

Replace the active fees query:
```rust
        let count_query = r#"
            SELECT count() as count FROM projects GROUP ALL;
            SELECT count() as count FROM company GROUP ALL;
            SELECT count() as count FROM contacts GROUP ALL;
            SELECT count() as count FROM fee GROUP ALL;
            SELECT count() as count FROM fee WHERE status IN ['Draft', 'Sent', 'Negotiation'] GROUP ALL;
        "#;
```

**Step 4: Run tests**

Run: `cd /Volumes/base/dev/app/e-fees && cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5`

Expected: All tests pass

**Step 5: Commit**

```bash
git add src-tauri/src/db/operations.rs
git commit -m "feat(db): add venue CRUD operations, update active fee status query"
```

---

## Task 6: Backend — Tauri Commands for Venues

**Files:**
- Create: `src-tauri/src/commands/venues.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create venue commands**

Create `src-tauri/src/commands/venues.rs`:

```rust
//! Venue management commands.

use tauri::State;
use crate::db::DatabaseManager;
use crate::db::types::{Venue, VenueCreate, VenueLocation, PaginatedResponse};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn get_venues(
    db: State<'_, Arc<RwLock<DatabaseManager>>>,
) -> Result<Vec<Venue>, String> {
    let manager = db.read().await;
    manager.get_venues().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_venues_page(
    page: usize,
    page_size: usize,
    db: State<'_, Arc<RwLock<DatabaseManager>>>,
) -> Result<PaginatedResponse<Venue>, String> {
    let manager = db.read().await;
    manager.get_venues_page(page, page_size).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_venue_by_id(
    id: String,
    db: State<'_, Arc<RwLock<DatabaseManager>>>,
) -> Result<Option<Venue>, String> {
    let manager = db.read().await;
    manager.get_venue_by_id(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_venue(
    name: String,
    name_short: Option<String>,
    city: Option<String>,
    country: Option<String>,
    area: Option<String>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
    db: State<'_, Arc<RwLock<DatabaseManager>>>,
) -> Result<Venue, String> {
    let manager = db.read().await;
    let venue = VenueCreate {
        name,
        name_short: name_short.unwrap_or_default(),
        location: VenueLocation {
            city: city.unwrap_or_default(),
            country: country.unwrap_or_default(),
            area: area.unwrap_or_default(),
        },
        tags: tags.unwrap_or_default(),
        notes: notes.unwrap_or_default(),
    };
    manager.create_venue(venue).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_venue(
    id: String,
    name: String,
    name_short: Option<String>,
    city: Option<String>,
    country: Option<String>,
    area: Option<String>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
    db: State<'_, Arc<RwLock<DatabaseManager>>>,
) -> Result<Venue, String> {
    let manager = db.read().await;
    manager.update_venue(
        &id,
        name,
        name_short.unwrap_or_default(),
        city.unwrap_or_default(),
        country.unwrap_or_default(),
        area.unwrap_or_default(),
        tags.unwrap_or_default(),
        notes.unwrap_or_default(),
    ).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_venue(
    id: String,
    db: State<'_, Arc<RwLock<DatabaseManager>>>,
) -> Result<Venue, String> {
    let manager = db.read().await;
    manager.delete_venue(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_projects_for_venue(
    venue_id: String,
    db: State<'_, Arc<RwLock<DatabaseManager>>>,
) -> Result<Vec<crate::db::types::Project>, String> {
    let manager = db.read().await;
    manager.get_projects_for_venue(&venue_id).await.map_err(|e| e.to_string())
}
```

**Step 2: Register venue module in commands/mod.rs**

Add `pub mod venues;` and re-export all venue commands.

**Step 3: Register venue commands in lib.rs invoke_handler**

Add to the `tauri::generate_handler![]` list:
```rust
commands::venues::get_venues,
commands::venues::get_venues_page,
commands::venues::get_venue_by_id,
commands::venues::create_venue,
commands::venues::update_venue,
commands::venues::delete_venue,
commands::venues::get_projects_for_venue,
```

**Step 4: Run tests + verify compilation**

Run: `cd /Volumes/base/dev/app/e-fees && cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5`

Expected: All tests pass

**Step 5: Commit**

```bash
git add src-tauri/src/commands/venues.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(commands): add Tauri venue CRUD commands"
```

---

## Task 7: Frontend — Update TypeScript Types and Constants

**Files:**
- Modify: `src/types/database.ts`
- Modify: `src/lib/constants.ts`

**Step 1: Add Venue types to database.ts**

After `ProjectNumber` interface (~line 65), add:

```typescript
export interface VenueLocation {
  city: string;
  country: string;
  area: string;
}

export interface Venue {
  id?: string;
  name: string;
  name_short?: string;
  location?: VenueLocation;
  tags?: string[];
  notes?: string;
  time?: TimeInfo;
}

export type VenueCreate = Omit<Venue, 'id' | 'time'>;
export type VenueUpdate = Partial<VenueCreate>;
```

**Step 2: Update ProjectStatus type**

Replace the `ProjectStatus` type:

```typescript
export type ProjectStatus =
  | 'Lead'
  | 'RFP'
  | 'Submitted'
  | 'Awarded'
  | 'Design'
  | 'Construction'
  | 'Completed'
  | 'Lost'
  | 'No Response'
  | 'Cancelled'
  | 'On Hold'
  | 'Superseded';
```

**Step 3: Update FeeStatus type**

Replace the `FeeStatus` type:

```typescript
export type FeeStatus =
  | 'Draft'
  | 'Sent'
  | 'Negotiation'
  | 'Accepted'
  | 'Rejected'
  | 'No Response'
  | 'Superseded';
```

**Step 4: Add venue_id to Project interface**

In the `Project` interface, add after `folder?`:
```typescript
  venue_id?: string;
```

**Step 5: Update constants.ts**

Replace `PROPOSAL_STATUSES`:
```typescript
export const PROPOSAL_STATUSES = [
  'Draft',
  'Sent',
  'Negotiation',
  'Accepted',
  'Rejected',
  'No Response',
  'Superseded'
] as const;
```

Replace `ACTIVE_PROPOSAL_STATUSES`:
```typescript
export const ACTIVE_PROPOSAL_STATUSES: ProposalStatus[] = [
  'Draft',
  'Sent',
  'Negotiation'
];
```

Replace `PROJECT_STATUSES`:
```typescript
export const PROJECT_STATUSES = [
  'Lead',
  'RFP',
  'Submitted',
  'Awarded',
  'Design',
  'Construction',
  'Completed',
  'Lost',
  'No Response',
  'Cancelled',
  'On Hold',
  'Superseded'
] as const;
```

Add new constant:
```typescript
export const ACTIVE_PROJECT_STATUSES: ProjectStatus[] = [
  'RFP',
  'Submitted',
  'Awarded',
  'Design',
  'Construction'
];
```

Update `STATUS_COLORS` to cover new statuses:
```typescript
export const STATUS_COLORS: Record<string, string> = {
  'Lead': 'text-gray-400 bg-gray-400/10',
  'Draft': 'text-yellow-400 bg-yellow-400/10',
  'RFP': 'text-blue-400 bg-blue-400/10',
  'Submitted': 'text-purple-400 bg-purple-400/10',
  'Sent': 'text-purple-400 bg-purple-400/10',
  'Negotiation': 'text-blue-400 bg-blue-400/10',
  'Awarded': 'text-green-400 bg-green-400/10',
  'Accepted': 'text-green-400 bg-green-400/10',
  'Design': 'text-blue-400 bg-blue-400/10',
  'Construction': 'text-teal-400 bg-teal-400/10',
  'Completed': 'text-green-500 bg-green-500/10',
  'Lost': 'text-red-400 bg-red-400/10',
  'Rejected': 'text-red-400 bg-red-400/10',
  'No Response': 'text-gray-500 bg-gray-500/10',
  'Cancelled': 'text-gray-400 bg-gray-400/10',
  'On Hold': 'text-orange-400 bg-orange-400/10',
  'Superseded': 'text-cyan-400 bg-cyan-400/10',
};
```

**Step 6: Verify TypeScript compilation**

Run: `cd /Volumes/base/dev/app/e-fees && npx tsc --noEmit 2>&1 | head -20`

Expected: Type errors in stores.ts (references old status values) — these get fixed in next task

**Step 7: Commit**

```bash
git add src/types/database.ts src/lib/constants.ts
git commit -m "feat(types): update status types and constants for domain model restructure"
```

---

## Task 8: Frontend — Update Stores and Dashboard Metrics

**Files:**
- Modify: `src/lib/stores.ts`

**Step 1: Import ACTIVE_PROPOSAL_STATUSES from constants**

Add at top of file:
```typescript
import { ACTIVE_PROPOSAL_STATUSES } from '$lib/constants';
```

**Step 2: Update statisticsStore derived (line 109-119)**

Replace the hardcoded status list:
```typescript
export const statisticsStore = derived(
  [projectsStore, feesStore, companiesStore, contactsStore],
  ([projects, fees, companies, contacts]) => ({
    totalProjects: projects.length,
    activeFees: fees.filter(f => ACTIVE_PROPOSAL_STATUSES.includes(f.status as any)).length,
    totalCompanies: companies.length,
    totalContacts: contacts.length,
    totalFees: fees.length,
  })
);
```

**Step 3: Add venue store**

After the fee store exports (~line 96), add:

```typescript
// Venue store (follows same factory pattern)
import { venuesApi } from './stores/adapters';
const venuesInternal = createSyncedEntityStore(venuesApi, 'Venue');
export const venuesStore = venuesInternal.itemsStore;
export const venuesLoading = venuesInternal.loadingStore;
export const venuesError = venuesInternal.errorStore;
const venuesActionsInternal = venuesInternal.internalActions;
```

**Step 4: Export venue actions in the actions object**

Find the existing actions export pattern and add venue actions following the same pattern as projects/companies/contacts/fees.

**Step 5: Create venue API adapter**

In `src/lib/stores/adapters.ts`, add the venuesApi adapter following the pattern of projectsApi. Check this file first to understand the pattern.

**Step 6: Verify TypeScript compilation**

Run: `cd /Volumes/base/dev/app/e-fees && npx tsc --noEmit 2>&1 | head -20`

Expected: May still have errors in route components — those get fixed in subsequent tasks

**Step 7: Commit**

```bash
git add src/lib/stores.ts src/lib/stores/adapters.ts
git commit -m "feat(stores): add venue store, update active fees to use constants"
```

---

## Task 9: Frontend — Venue Route and Modal

**Files:**
- Create: `src/routes/Venues.svelte`
- Create: `src/lib/components/VenueModal.svelte`
- Modify: `src/App.svelte` (add route + nav item)

**Step 1: Create VenueModal.svelte**

Follow the pattern of existing modals (e.g., CompanyModal, ContactModal). The modal should:
- Name (required), Name Short, City, Country, Area, Tags (comma-separated), Notes
- Use existing `.emittiv-input`, `.emittiv-btn` classes
- Create and Edit modes

**Step 2: Create Venues.svelte route**

Follow the pattern of Companies.svelte. The page should:
- List venues with name, location, tags, number of linked projects
- Search/filter bar
- Create button → VenueModal
- Click row → show linked projects (or navigate to detail view later)

**Step 3: Add route to App.svelte**

In the routes object:
```typescript
'/venues': Venues,
```

Add navigation item in the sidebar. Update keyboard shortcut mappings (Cmd+1-6).

**Step 4: Verify the app compiles and venue page renders**

Run: `cd /Volumes/base/dev/app/e-fees && npm run check 2>&1 | tail -10`

**Step 5: Commit**

```bash
git add src/routes/Venues.svelte src/lib/components/VenueModal.svelte src/App.svelte
git commit -m "feat(ui): add Venues page with list view and create/edit modal"
```

---

## Task 10: Frontend — Update Project and Proposal Status UI

**Files:**
- Modify: `src/routes/Projects.svelte`
- Modify: `src/routes/Proposals.svelte`
- Modify: `src/lib/components/StatusChangeModal.svelte` (if it exists and references old statuses)

**Step 1: Update Projects.svelte status filters**

Replace any hardcoded status arrays with imports from `constants.ts`. Update filter dropdowns to use `PROJECT_STATUSES`.

**Step 2: Update Proposals.svelte status filters**

Replace any hardcoded status arrays with imports from `constants.ts`. Update filter dropdowns to use `PROPOSAL_STATUSES`.

**Step 3: Update StatusChangeModal if needed**

Check for references to old statuses ('Active', 'Revised', 'Bidding') and update to new values.

**Step 4: Verify compilation**

Run: `cd /Volumes/base/dev/app/e-fees && npm run check 2>&1 | tail -10`

**Step 5: Commit**

```bash
git add src/routes/Projects.svelte src/routes/Proposals.svelte src/lib/components/StatusChangeModal.svelte
git commit -m "feat(ui): update project and proposal pages for new status lifecycles"
```

---

## Task 11: Backend — Update Rust Tests

**Files:**
- Modify: `src-tauri/src/db/tests.rs`

**Step 1: Add venue-related tests**

Add tests for:
- Venue struct serialization/deserialization
- VenueCreate validation
- VenueLocation defaults

**Step 2: Update any test referencing old status values**

Search tests.rs for `'Active'`, `'Revised'`, `'Awarded'` (as fee status), `'Bidding'` and update to new status names where needed.

**Step 3: Run full test suite**

Run: `cd /Volumes/base/dev/app/e-fees && cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -20`

Expected: All tests pass

**Step 4: Commit**

```bash
git add src-tauri/src/db/tests.rs
git commit -m "test: add venue tests, update status values in existing tests"
```

---

## Task 12: Integration Test — Full Roundtrip

**Step 1: Run full backend test suite**

Run: `cd /Volumes/base/dev/app/e-fees && cargo test --manifest-path src-tauri/Cargo.toml 2>&1`

Expected: All tests pass

**Step 2: Run frontend type check**

Run: `cd /Volumes/base/dev/app/e-fees && npm run check 2>&1`

Expected: No errors

**Step 3: Start dev server and verify**

Run: `cd /Volumes/base/dev/app/e-fees && npm run tauri:dev`

Manually verify:
- Dashboard loads with correct counts
- Projects page shows new statuses
- Proposals page shows new statuses
- Venues page loads (empty, since no venues created yet via migration)
- Navigation works (all routes accessible)

**Step 4: Commit any fixes**

```bash
git add -A
git commit -m "fix: integration fixes for domain model restructure"
```

---

## Task 13: Data Migration — Create Venues from Existing Projects

**Files:**
- Create: `scripts/migration/004-create-venues-from-projects.surql`

**Step 1: Write venue creation script**

This is a semi-manual step. Review the distinct venue names from projects and create venue records:

```sql
-- 004-create-venues-from-projects.surql
-- Create venues from existing project data and link projects

-- MAF complex (multiple engagements)
CREATE venue:maf_micc CONTENT {
    name: "MAF MiCC",
    name_short: "MiCC",
    location: { city: "Dubai", country: "UAE", area: "" },
    tags: ["entertainment", "retail"],
    notes: "Majid Al Futtaim kids entertainment centre"
};
UPDATE projects:`24_97105` SET venue_id = venue:maf_micc;
UPDATE projects:`25_97104` SET venue_id = venue:maf_micc;
UPDATE projects:`25_97106` SET venue_id = venue:maf_micc;

-- Shanghai Tang
CREATE venue:shanghai_tang CONTENT {
    name: "Shanghai Tang",
    name_short: "Shanghai Tang",
    location: { city: "Dubai", country: "UAE", area: "" },
    tags: ["retail"],
    notes: ""
};
UPDATE projects:`25_97101` SET venue_id = venue:shanghai_tang;
UPDATE projects:`25_97105` SET venue_id = venue:shanghai_tang;

-- Lvl 63 (formerly Reserve Cut)
CREATE venue:lvl_63 CONTENT {
    name: "Lvl 63",
    name_short: "Lvl 63",
    location: { city: "Dubai", country: "UAE", area: "" },
    tags: ["restaurant"],
    notes: "Formerly known as Reserve Cut"
};
UPDATE projects:`24_97107` SET venue_id = venue:lvl_63;

-- NOTE: Remaining projects need manual venue assignment.
-- Run this query to find unlinked projects:
-- SELECT number.id, name, status FROM projects WHERE venue_id IS NONE;
```

**Step 2: Review and customize the script**

This script needs manual review — the venue names, locations, and tags should be verified. Add more venues as needed for the remaining 55+ projects.

**Step 3: Run the script**

Run against dev DB after review.

**Step 4: Commit**

```bash
git add scripts/migration/004-create-venues-from-projects.surql
git commit -m "feat(db): create initial venues from existing project data"
```

---

## Summary

| Task | Component | Description |
|------|-----------|-------------|
| 1 | DB Schema | Create venue table |
| 2 | DB Schema | Add venue_id to projects, update status ASSERT lists |
| 3 | DB Data | Migrate existing statuses to new lifecycle |
| 4 | Backend | Venue struct and types in Rust |
| 5 | Backend | Venue CRUD operations |
| 6 | Backend | Tauri commands for venue API |
| 7 | Frontend | TypeScript types and constants update |
| 8 | Frontend | Stores and dashboard metrics |
| 9 | Frontend | Venue route and modal |
| 10 | Frontend | Update project/proposal status UI |
| 11 | Backend | Update Rust tests |
| 12 | Integration | Full roundtrip verification |
| 13 | DB Data | Create venues from existing projects |

**Estimated commits:** 13
**Risk areas:** Task 3 (data migration — irreversible on PROD, test on dev first), Task 13 (manual venue assignment needs business knowledge)

---

*Implementation plan for domain model restructure. Design doc: `docs/plans/2026-02-24-domain-model-restructure-design.md`*
