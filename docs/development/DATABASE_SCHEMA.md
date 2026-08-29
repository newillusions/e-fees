# Fee Proposal App - Database Schema Documentation

> **Doc audit note (2026-07-16):** this file's connection details, status enums, and
> "Current Data Status" snapshot were stale (predated at least two status-enum
> revisions and an IP migration) and have been corrected against the live
> source of truth: `e-fees-api/src/validation.rs` (status enums) and
> `CLAUDE.md` (connection info). Table structure (`DEFINE TABLE`/`DEFINE FIELD`)
> blocks below are illustrative of the general shape, not verified against a
> live `INFO FOR TABLE` dump - treat them as a starting reference, not a
> guaranteed-current schema. `fee` and scope-service tables are SCHEMALESS in
> practice (see CLAUDE.md "Critical query patterns"), so the `fee` block's
> `DEFINE FIELD ... ASSERT` lines are historical/aspirational, not enforced.

## Database Connection Details

**SurrealDB Instance (prod)**: `ws://10.0.23.11:8000` (SurrealDB 3.1.2)
**SurrealDB Instance (dev)**: `ws://10.0.23.12:8000` (SurrealDB 3.1.4, ns `emittiv_dev`) - a point release ahead of prod; don't assume schema/behavior parity without checking both directly
**Namespace**: `emittiv` (prod) / `emittiv_dev` (dev)
**Database**: `projects`
**Authentication**: Per-app SurrealDB user via environment variables (`SURREAL_USER`/`SURREAL_PASS` for e-fees-api, see `e-fees-api/.env.example`)

## Database Overview

The database contains 6 main application tables (`projects`, `fee`, `company`, `contacts`, `country`, `currency`) plus the e-fees-scope service's own tables (`clause`, `scope_assembly`, `clause_corpus_stat`, etc. - see `e-fees-scope/schema.surql`, not covered by this document). All tables use auto-managed timestamps and record-link foreign keys.

---

## Table Structures

### 1. `projects` Table

**Purpose**: Project opportunities emittiv is bidding on  
**ID Format**: `projects:⟨YY_CCCNN⟩` (auto-generated)  
**Type**: SCHEMAFULL

```sql
DEFINE TABLE projects SCHEMAFULL;
DEFINE FIELD name ON projects TYPE string ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD name_short ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD status ON projects TYPE string ASSERT $value INSIDE ['Lead', 'RFP', 'Submitted', 'Awarded', 'Design', 'Construction', 'Completed', 'Lost', 'No Response', 'Cancelled', 'On Hold', 'Superseded'] DEFAULT 'Lead';
-- ^ current enum per e-fees-api/src/validation.rs PROJECT_STATUSES (source of truth is src/types/database.ts on the desktop side)
DEFINE FIELD area ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD city ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD country ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD folder ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD number ON projects TYPE object ASSERT $value != NONE;
DEFINE FIELD number.year ON projects TYPE int ASSERT $value >= 20 AND $value <= 50;
DEFINE FIELD number.country ON projects TYPE int ASSERT $value != NONE;
DEFINE FIELD number.seq ON projects TYPE int ASSERT $value >= 1 AND $value <= 999;
DEFINE FIELD number.id ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD time ON projects TYPE object VALUE { created_at: time::now(), updated_at: time::now() };
DEFINE FIELD time.created_at ON projects TYPE datetime VALUE time::now();
DEFINE FIELD time.updated_at ON projects TYPE datetime VALUE time::now();

-- Unique constraint on project number
DEFINE INDEX project_number_unique ON projects FIELDS number.id UNIQUE;
```

**P0 backfill fields (2026-08-19, `scripts/migration/v006_backfill_metadata_fields.surql`):**
net-new, all `option<T>` (absent = `NONE`, never `NULL`) so existing rows and
existing writers are unaffected. Not read by the desktop app or `e-fees-api`
yet - populated by the historical-backfill phases (P1-P4, see
`/Volumes/base/dev/.claude/research/2026-08-08-pricing-ingestion/03-ingestion-plan.md`).

```sql
-- Open-vocabulary project type/category (free text + UI typeahead, no enum -
-- the vocabulary is still being discovered).
DEFINE FIELD project_type ON projects TYPE option<string>;
DEFINE FIELD project_category ON projects TYPE option<string>;

-- Structured discipline entries. "Sub" is a subcontractor slot, not a
-- discipline name - kind=Sub + sub_company must never reach a client-facing
-- export (enforced at the app/export layer, not by this DDL).
DEFINE FIELD disciplines ON projects TYPE option<array<object>>;
DEFINE FIELD disciplines[*].kind ON projects
    TYPE string
    ASSERT $value IN ['Lighting', 'Video', 'Audio', 'SFX', 'Show Control', 'Sub'];
DEFINE FIELD disciplines[*].sub_discipline ON projects TYPE option<string>;
DEFINE FIELD disciplines[*].sub_company ON projects TYPE option<record<company>>;

-- Lifecycle (`stage`) and commercial result (`outcome`) as two separate
-- axes - NOT a replacement for the existing `status` field, which stays
-- untouched and app-workflow-owned. `outcome = NONE` means open/pending;
-- "Open" is never a written value.
DEFINE FIELD stage ON projects
    TYPE option<string>
    ASSERT $value = NONE OR $value IN
        ['RFP', 'Design', 'Construction', 'Completed', 'Superseded', 'Cancelled'];
DEFINE FIELD outcome ON projects
    TYPE option<string>
    ASSERT $value = NONE OR $value IN ['Won', 'Lost', 'No Response', 'Cancelled'];
DEFINE FIELD outcome_changed_at ON projects TYPE option<datetime>;
DEFINE FIELD successor ON projects TYPE option<record<projects>>;
    -- Forward pointer, populated only when stage=Superseded.

-- Project metadata, mined from client documents in a later backfill phase.
DEFINE FIELD gfa_sqm         ON projects TYPE option<float>;
DEFINE FIELD plot_area_sqm   ON projects TYPE option<float>;
DEFINE FIELD scope_summary   ON projects TYPE option<string>;
DEFINE FIELD loss_reason     ON projects TYPE option<string>;
DEFINE FIELD source_document ON projects TYPE option<string>;
DEFINE FIELD data_provenance ON projects TYPE option<object> FLEXIBLE;
```

Note (SurrealDB 3.1.4, dev): defining `disciplines[*].kind` (etc.) implicitly
creates an intermediate wildcard object field SurrealDB echoes in
`INFO FOR TABLE` as `disciplines.*` - rolling back requires explicitly
removing it with the bracket-star form (`REMOVE FIELD disciplines[*] ON
projects;`), not the dot-star form the introspection output displays. See
the rollback script's header comment for the full note. Not yet re-verified
against prod's 3.1.2.

## Migration workflow (schema-guard onboarded 2026-08-19)

Schema migrations for `projects` go through the workspace migration guard,
`~/.claude/scripts/apply-migration.sh` (`obs:q2lbpycedhr8ppggv56u`) - the
same tool `pa-core-rs` uses. It proves, on every single run, that the
migration being applied is reversible (rollback file exists AND round-trips
cleanly, re-tested live every time) and, for anything not provably additive,
that the entire target database is recoverable before it writes anything.

**File convention**: `scripts/migration/vNNN_slug.surql` (forward) +
`scripts/migration/vNNN_slug_rollback.surql` (rollback), starting from
`v006_backfill_metadata_fields.surql`. The pre-guard files
(`001-create-venue-table.surql` … `005-country-normalization.surql`) keep
their original hyphenated names - they are historical and will never be
re-run through the guard; see `000-bootstrap-schema-version-tracking.surql`
for exactly which of them actually reached prod (not all five - `venue` was
an abandoned parallel workstream that never shipped to either environment).

**Version-stamp discipline** (same idiom as `pa-core-rs/schema/README.md`):
every migration from v006 onward ends with a trailing stamp block:

```sql
LET $existing = (SELECT id FROM app_state WHERE key = 'schema_version' LIMIT 1);
IF array::len($existing) > 0 THEN
  (UPDATE $existing[0].id SET value = 'NN', updated_at = time::now())
ELSE
  (CREATE app_state SET key = 'schema_version', value = 'NN', updated_at = time::now())
END;
```

and its rollback ends with the same block stamping `NN - 1`. The guard reads
`app_state.schema_version` before applying anything and refuses unless the
target is at exactly `version - 1` (refuses skipped versions, re-runs, and
wrong-database applies). `app_state` itself (`key`/`value`/`updated_at`,
unique index on `key`) was bootstrapped once, by hand
(`000-bootstrap-schema-version-tracking.surql`, never through the guard -
see that file for why the guard structurally cannot bootstrap its own
tracking table), stamped at `'5'` on both dev and prod.

**Applying a new migration** (`NNN` = next version number):

```bash
~/.claude/scripts/apply-migration.sh \
  --schema-dir scripts/migration --version NNN \
  --host 10.0.23.11:8000 --ns emittiv --db projects \
  --creds EFEES_SURREALDB --dry-run   # then re-run without --dry-run
```

`--dev-host` defaults to `10.0.23.12:8000` (dev) and is where the guard
proves the rollback round-trip in a throwaway namespace before ever touching
the real target - it does not need to be passed explicitly for e-fees. Prod
application is orchestrator-owned per the workspace deploy-ownership
contract, not run by a project instance.

**Business Logic - Project Numbering**:
- Format: `YY-CCCNN` (e.g., `25-97105`)
- YY: 2-digit year (25 = 2025)
- CCC: Country dial code (971=UAE, 966=Saudi)  
- NN: Sequence number (05 = 5th project)
- **Critical**: Sequence stored as integer but represents 2-digit padded values
- UAE projects start from sequence 1 (display as 97101)
- Saudi projects start from sequence 1 (display as 96601)
- Sequence resets annually per country

**Sample Project Record**:
```json
{
  "id": "projects:⟨25_97105⟩",
  "name": "Shanghai Tang Revision",
  "name_short": "Shanghai Tang v2", 
  "status": "Active",
  "area": "Etihad Towers",
  "city": "Abu Dhabi",
  "country": "U.A.E.",
  "folder": "25-97105 Shanghai Tang v2",
  "number": {
    "year": 25,
    "country": 971,
    "seq": 5,
    "id": "25-97105"
  },
  "time": {
    "created_at": "2025-06-14T17:52:39.604Z",
    "updated_at": "2025-06-15T11:09:14.587Z"
  }
}
```

---

### 2. `fee` Table

**Purpose**: Fee proposals created by emittiv staff
**ID Format**: `fee:⟨YY_CCCNN_R⟩` (record key embeds the *submitted* `rev` at
create time - see "Revision Management" below for why that can diverge from
the DB-computed `rev` field on the row if a caller ever passes a `rev` that
doesn't match what `revisions[]` implies)
**Type**: SCHEMALESS (`fee` itself carries no `DEFINE TABLE`; individual
fields below are still defined and enforced regardless of table-level
schema mode - see CLAUDE.md "Critical query patterns")

> **Verified 2026-08-28** against dev (10.0.23.12) via a live `INFO FOR
> TABLE fee` (same audit that produced `scripts/migration/v007_fee_revisions.surql`).
> The block below is the confirmed live shape, not the "illustrative, not
> verified" placeholder this section carried previously - that placeholder's
> `rev` VALUE clause (`math::max($value.revisions[*].revision_number)` with
> no empty-array guard) was WRONG: SurrealDB v3's `math::max([])` returns
> `-Infinity`, not `0`/`NONE` (see the project's pinned kb-critical rule
> "Surrealdb V3 Math Max Empty Array"). The live field already guards this;
> the block below matches it exactly.

```sql
DEFINE FIELD name ON fee TYPE string ASSERT string::len($value) > 0 PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD number ON fee TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD project_id ON fee TYPE record<projects> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD company_id ON fee TYPE record<company> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD contact_id ON fee TYPE record<contacts> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD status ON fee TYPE string DEFAULT 'Draft' ASSERT $value INSIDE ['Draft', 'Sent', 'Negotiation', 'Accepted', 'Rejected', 'No Response', 'Superseded'] PERMISSIONS FULL;
DEFINE FIELD issue_date ON fee TYPE string ASSERT string::len($value) = 6 AND string::is_numeric($value) PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD activity ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD package ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD strap_line ON fee TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD staff_name ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD staff_email ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD staff_phone ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD staff_position ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD revisions ON fee TYPE array<object> DEFAULT [] PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD `revisions.*` ON fee TYPE object PERMISSIONS FULL;
DEFINE FIELD rev ON fee TYPE int
    VALUE (IF array::len(revisions.*.revision_number) > 0
           THEN math::max(revisions.*.revision_number)
           ELSE 0 END)
    PERMISSIONS FOR select WHERE FULL, FOR create, update FULL;
DEFINE FIELD time ON fee TYPE object PERMISSIONS FULL;
DEFINE FIELD time.created_at ON fee TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON fee TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

-- Unique constraint on project + revision
DEFINE INDEX fee_project_rev ON fee FIELDS project_id, rev UNIQUE;
```

**Revision Management** (fixed 2026-08-28, e-fees revision audit - see
`crates/e-fees-core/src/models/fee.rs`'s `Revision::new`):
- `rev` is DB-computed from `revisions[*].revision_number` via the `VALUE`
  clause above - it is **not** settable directly. Any `FeeCreate`/`FeeUpdate`
  that writes `revisions: []` gets `rev = 0` regardless of what `rev` value
  it submits, which is exactly the bug this fix closes: every fee-creation
  call site (`clone_fee_as_revision`, `import_wizard.rs`, `agent_server.rs`)
  now populates `revisions[]` with a real entry via `Revision::new` before
  creating or updating a row.
- Each revision entry: `revision_number`, `revision_date`, `author_email`,
  `author_name`, `notes` - a complete audit trail for all proposal changes.
- `clone_fee_as_revision` seeds the new fee's `revisions[]` from the
  source's full history plus one new entry, and flips the source fee's
  `status` to `Superseded` (per `docs/plans/2026-02-24-domain-model-restructure-design.md`
  L375-376: "Create new revision when terms agreed / Old revision
  auto-set to 'Superseded'").
- **Historical data**: every fee row created before this fix has
  `revisions: []` / `rev: 0` live (confirmed on dev, 100% of rows sampled).
  `crates/e-fees-core/src/bin/backfill_fee_revisions_seed.rs` retroactively seeds a real
  `revisions[]` entry on every such row - see that file's header for the
  provenance rules (rows the P1 historical-backfill loader already
  annotated with `data_provenance.superseded_revisions` get their full
  multi-entry history reconstructed; every other row gets a single
  `revision_number: 1` entry).

---

### 3. `company` Table

**Purpose**: Client companies that issue project opportunities  
**ID Format**: `company:ABBREVIATION` (e.g., `company:CHE`)  
**Type**: SCHEMAFULL

```sql
DEFINE TABLE company SCHEMAFULL;
DEFINE FIELD name ON company TYPE string ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD name_short ON company TYPE string ASSERT $value != NONE;
DEFINE FIELD abbreviation ON company TYPE string ASSERT $value != NONE;
DEFINE FIELD city ON company TYPE string ASSERT $value != NONE;
DEFINE FIELD country ON company TYPE string ASSERT $value != NONE;
DEFINE FIELD reg_no ON company TYPE option<string>;
DEFINE FIELD tax_no ON company TYPE option<string>;
DEFINE FIELD time ON company TYPE object VALUE { created_at: time::now(), updated_at: time::now() };
```

**Sample Company Record**:
```json
{
  "id": "company:CHE",
  "name": "Conrad Hilton",
  "name_short": "Conrad Etihad", 
  "abbreviation": "CHE",
  "city": "Abu Dhabi",
  "country": "U.A.E.",
  "time": {
    "created_at": "2025-06-14T17:51:01.928Z",
    "updated_at": "2025-06-14T17:51:01.928Z"
  }
}
```

---

### 4. `contacts` Table

**Purpose**: Individual contact persons at client companies  
**ID Format**: `contacts:random_id` (auto-generated)  
**Type**: SCHEMAFULL

```sql
DEFINE TABLE contacts SCHEMAFULL;
DEFINE FIELD first_name ON contacts TYPE string ASSERT $value != NONE;
DEFINE FIELD last_name ON contacts TYPE string ASSERT $value != NONE;
DEFINE FIELD email ON contacts TYPE string ASSERT is::email($value);
DEFINE FIELD phone ON contacts TYPE string ASSERT string::contains($value, '+') AND string::len($value) > 0;
DEFINE FIELD position ON contacts TYPE string ASSERT $value != NONE;
DEFINE FIELD company ON contacts TYPE record<company> ASSERT $value != NONE;
DEFINE FIELD full_name ON contacts TYPE string VALUE string::concat($value.first_name, ' ', $value.last_name);
DEFINE FIELD time ON contacts TYPE object VALUE { created_at: time::now(), updated_at: time::now() };

-- Unique email constraint
DEFINE INDEX contact_email_unique ON contacts FIELDS email UNIQUE;
```

---

### 5. `country` Table (Reference Data)

**Purpose**: Pre-populated reference table for countries  
**Type**: SCHEMAFULL, read-only reference data

Contains ~250 countries with fields:
- `name`, `name_formal`, `name_official`
- `code`, `code_alt`, `dial_code` 
- `currency_code`

**Key Countries for Project Numbering**:
- `country:AE` - dial_code: 971 (UAE)
- `country:SA` - dial_code: 966 (Saudi Arabia)

---

### 6. `currency` Table (Reference Data)

**Purpose**: Pre-populated reference table for currencies  
**Type**: SCHEMAFULL, read-only reference data

Contains ~180 currencies with fields:
- `code` (USD, AED, SAR, etc.)
- `name` (full currency name)

---

## Critical Business Rules

### Project Number Generation Algorithm

```typescript
// Generate next project number
function generateNextProjectNumber(countryDialCode: number, year?: number): string {
  const currentYear = year || (new Date().getFullYear() % 100);
  
  // Query database for highest sequence in this year/country
  const query = `SELECT math::max(number.seq) as max_seq FROM projects 
                 WHERE number.year = ${currentYear} AND number.country = ${countryDialCode}`;
  
  const result = await db.query(query);
  const nextSeq = (result.max_seq || 0) + 1;
  
  // Format: YY-CCCNN
  const projectNumber = `${currentYear.toString().padStart(2, '0')}-${countryDialCode}${nextSeq.toString().padStart(2, '0')}`;
  
  return projectNumber; // e.g., "25-97105"
}
```

### Validation Rules

1. **Project Numbers**: Must be unique across all projects
2. **Email Addresses**: Must be unique across all contacts
3. **Phone Numbers**: Must contain '+' character
4. **Issue Dates**: Must be exactly 6 digits (YYMMDD format)
5. **Sequence Numbers**: 1-999 range, but display as 2-digit padded

### Fee Status Workflow

There is no `rfp` table - the workflow below applies to the `fee` table's `status`
field (project-level status is the separate `PROJECT_STATUSES` enum above):

```
Draft → Sent → Negotiation → Accepted/Rejected/No Response
                    ↓
                Superseded (when a new revision replaces this fee)
```

---

## Database Relationships

```
projects ←──── fee ────→ company
                ↓
             contacts ────→ company

country ────→ projects (via country name)
currency ────→ fee (for pricing)
```

---

## Current Data Status

Removed as of this audit (2026-07-16) - the "48 Projects / 37 RFPs / 19 Companies"
snapshot here was dated June 2025 and had drifted from live counts, and there is no
`rfp` table (see "Fee Status Workflow" above - this doc previously conflated `fee`
with a nonexistent `rfp` table throughout). For a current count, query `GET /stats`
on e-fees-api (`http://10.0.21.80:3200/stats`, requires `X-API-Key`) rather than
trusting a static snapshot in this document.

---

## Integration Notes for Claude Code

### Required Database Operations

1. **Project Creation**:
   ```sql
   -- Check next available sequence
   SELECT math::max(number.seq) FROM projects WHERE number.year = ? AND number.country = ?;
   
   -- Create project with generated number
   CREATE projects SET name = ?, number = { year: ?, country: ?, seq: ?, id: ? }, ...;
   ```

2. **Validation Queries**:
   ```sql
   -- Check if project number exists
   SELECT id FROM projects WHERE number.id = ?;
   
   -- Check if email exists
   SELECT id FROM contacts WHERE email = ?;
   ```

3. **Foreign Key Handling**:
   - Use `record<table>` type for foreign keys
   - Example: `project_id: record<projects>`
   - SurrealDB automatically validates relationships

### Error Handling

- **Unique Constraint Violations**: Project numbers, email addresses
- **Foreign Key Violations**: Invalid company/contact references  
- **Validation Failures**: Invalid email format, phone format, date format
- **Business Logic Violations**: Invalid status transitions, sequence ranges

### Performance Considerations

- All tables have optimized indexes for common queries
- Project number lookup is indexed for fast validation
- Foreign key relationships are indexed for JOIN operations

---

This documentation provides Claude Code with everything needed to implement the project creation workflow without requiring direct database access.
