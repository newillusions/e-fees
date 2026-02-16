# Excel Pricing Import Agent

**Purpose**: Standalone Claude Code task agent that scans project folders, reads pricing Excel files, and writes historical fee records to SurrealDB.

**Agent Type**: Task agent (spawned via Claude Code Task tool)
**Model Recommendation**: `model="opus"` (complex file parsing + DB operations)
**Estimated Duration**: 30-60 minutes per import batch

---

## Prerequisites

### System Requirements
- SMB share mounted at `/Volumes/svrroot` (Unraid Primary server)
- SurrealDB accessible at `ws://10.0.21.8:8000`
- Excel MCP tools available (read_data_from_excel, validate_excel_range)
- SurrealDB MCP tools available (query, select, create, update)

### Access Credentials
- **SurrealDB**:
  - Endpoint: `ws://10.0.21.8:8000`
  - Namespace: `emittiv`
  - Database: `projects`
  - Username: `martin`
  - Password: `th38ret3ch`

### Target Environment
- Base path: `/Volumes/svrroot/user/emittiv/nc/__groupfolders/1/01 Projects/`
- Scan folders:
  - `00 Inactive/` (archived projects)
  - `01 RFPs/` (active proposals)
  - `11 Current/` (current contracts)
  - `99 Completed/` (delivered projects)

---

## File Discovery Strategy

### Pricing File Locations
Within each project folder, search for Excel files:
```
{project_folder}/02 Proposal/*.xlsx
```

### Filename Pattern Matching (Priority Order)

Priority determines which file to use when multiple pricing files exist for same project:

1. **Standard Pattern**: `yy-cccnn-FP-NN Pricing.xlsx`
   - Example: `25-97109-FP-01 Pricing.xlsx`
   - yy = year (25)
   - ccc = country code (971=UAE, 966=Saudi, etc.)
   - nn = sequence in year (09)
   - FP-NN = fee package number (01-99)

2. **Legacy with e- Prefix**: `e-yy-cccnn-FP-NN Pricing.xlsx`
   - Example: `e-25-97105-FP-01 Pricing.xlsx`
   - Early projects (pre-2025)

3. **Bare Filename**: `Pricing.xlsx`
   - Very early projects
   - Must extract project number from folder name

4. **Numbered Revisions**: `yy-cccnn Pricing.xlsx` or `yy-cccnn Pricing rN.xlsx`
   - Example: `25-97110 Pricing.xlsx` or `25-97110 Pricing r2.xlsx`
   - No FP number implies FP-01 (default fee package)

5. **Sub-Package Pattern**: `e-yy-cccnn-XX-FP-NN Pricing.xlsx`
   - Example: `e-25-97115-AA-FP-01 Pricing.xlsx`
   - XX = package code (AA, EL, SYS, etc.)
   - Maps to `fee.package` field

### Skip Rules

Skip files that match ANY of:
- B2 is 0/null/empty AND all of C2:H2 are 0/null/empty
  - Indicates unfilled template, not actual pricing
- Filename contains "Quote N -" (vendor quote sheets)
- Filename contains "QPMO-" (client schedules)
- Sheet name contains "Sub-consultant", "Deployment", "Expenses"
- File is read-only or corrupted
- **Unfilled templates**: Filename matches `e-yy-nnnn-FP-*` or `yy-cccnn-FP-*` literally
  (these are blank template files, not real pricing)
- **Template folder**: Path contains `_yy-cccnn Project Name/` (the template project folder)
- **Non-pricing Excel**: Filename contains "Sub Consultants Pricing Sheet" or "Areas + Pricing Sheet"
  (these are vendor/client sheets, not emittiv pricing)
- **Archive duplicates**: File is in a `00 Archive/` subdirectory AND a newer version
  of the same project+FP exists in the parent `02 Proposal/` directory.
  If the archive has FP-01 and parent has FP-02, both are DIFFERENT packages — import both.
  Only skip archive when the SAME FP number exists in both locations.

---

## Excel Template Structure

### Header (Row 1)
```
B1: "Project Target"
C1-H1: Discipline names (Lighting, Video, Audio, SFX, Show Control, Sub)
I1: "Total"
```

### Target Row (Row 2)
```
B2: Target amount (e.g., 500000) - PRIMARY KEY for extraction
C2-H2: Allocation multipliers/percentages
I2: SUM of C2:H2
```

### Calculated Row (Row 3)
```
B3: "Target Price"
C3-H3: =B2*C2, =B2*D2, etc. (calculated prices per discipline)
I3: SUM (total target price)
```

### Remaining Row (Row 6)
```
B6: "Remaining"
C6-H6: Target - subtotal per discipline
```

### Design Stages Header (Row 8)
```
B8: "Design Stages"
```

### Configuration Section (Columns O-R)
```
O2: "Stages"          P2: Stage count (3 or 4)
O3: "VAT"             P3: VAT rate (e.g., 0.05 for 5%)
O4: "Mobilisation"    P4: Mobilisation percentage (e.g., 0.2 for 20%)
```

### Design Stages (Rows 9 to 8+N, where N = stage count)
```
A9: Stage number (1, 2, 3, 4)
B9-B{8+N}: Stage names (e.g., "Concept", "Schematic", "Design Development", "Contract Documents")
C9-H{8+N}: Stage fee amounts per discipline
M9-M{8+N}: Stage percentage of total (as decimal, e.g., 0.25 = 25%)
R9-R{8+N}: Stage costs/mobilisation
```

### Subtotals (Row 9+N)
```
B{9+N}: "Subtotal"
C{9+N}-H{9+N}: SUM(C9:C{8+N}), SUM(D9:D{8+N}), etc.
I{9+N}: Total subtotal
```

### VAT Row (Row 10+N)
```
B{10+N}: "VAT"
C{10+N}-H{10+N}: Subtotal * VAT rate per discipline
I{10+N}: Total VAT
```

### Grand Total Row (Row 11+N)
```
B{11+N}: "Grand Total"
C{11+N}-H{11+N}: Subtotal + VAT per discipline
I{11+N}: Total grand total
```

### Post-Contract Header (Row 14+N)
```
B{14+N}: "Post-Contract Services" (or similar)
```

### Post-Contract Items (Rows 15+N onward)
```
B: Service name
C: Quantity
D: Unit price
E: C * D (total)
```

---

## Six Disciplines (Fixed)

**Always use these mappings - do NOT infer from Excel sheet names:**

| Column | Excel Name | Fee Code | Description |
|--------|-----------|----------|-------------|
| C | Lighting | LX | Lighting design |
| D | Video | VID | Video/projection |
| E | Audio | AUD | Audio/sound |
| F | SFX | SFX | Special effects/rigging |
| G | Show Control | CTL | Show control/automation |
| H | Sub | SUB | Sub-disciplines/other |

---

## Data Extraction & Mapping

### Step 1: File Validation

For each discovered Excel file:

```
1. Extract project number from filename or folder name
   - Pattern: yy-cccnn (e.g., 25-97102)
   - Regex: /(\d{2}-\d{3,5})/

2. Verify project exists in SurrealDB
   Convert dashes to underscores for ID lookup:
   SELECT * FROM projects:{code_with_underscores};
   If not found: skip with warning "Project not found: 25-97109"

3. Extract fee package number (FP-NN) from filename
   - FP-01 → rev: 1 (integer)
   - FP-02, FP-03 → rev: 2, 3
   - If no FP in filename: default to rev 1
   - Fee record ID: fee:{project_code_underscored}_{rev}

4. Extract sub-package code if present (XX from e-yy-cccnn-XX-FP-NN pattern)
   - Optional: some projects have AA, EL, SYS, etc.
   - Store in fee.package field

5. Read and validate template structure
   - Check B2, C2:H2 (target row)
   - Check O2:R4 (configuration)
   - Check row 9-8+N range exists (design stages)
```

### Step 2: Target Pricing Extraction

```
B2 value = pricing.config.target_fee
  Example: 500000

C2:H2 = discipline percentages (as %)
  Example: C2=30, D2=20, E2=15, F2=15, G2=15, H2=5

  Map to pricing.disciplines[] (ACTUAL SCHEMA):
  [
    { id: "d1", name: "Lighting",     order: 0, percentage: 30.0 },
    { id: "d2", name: "Video",        order: 1, percentage: 20.0 },
    { id: "d3", name: "Audio",        order: 2, percentage: 15.0 },
    { id: "d4", name: "SFX",          order: 3, percentage: 15.0 },
    { id: "d5", name: "Show Control", order: 4, percentage: 15.0 },
    { id: "d6", name: "Sub",          order: 5, percentage: 5.0 }
  ]

  NOTE: Only include disciplines where percentage > 0.
  Many projects are lighting-only (C2=100, D2:H2=0) → single discipline.
```

### Step 3: Configuration Extraction

```
P3 = pricing.config.vat_percent (as percentage)
  Example: 5.0 (for 5% VAT)
  If P3 is a decimal (0.05), multiply by 100 to get percentage.

P4 = pricing.config.mobilisation_percent (as percentage)
  Example: 20.0 (for 20% mobilisation)
  If P4 is a decimal (0.2), multiply by 100 to get percentage.

P2 = stage count (3 or 4, sometimes up to 6)
  Used to determine row range for design stages
```

### Step 4: Design Stages Extraction

For each row from 9 to (8+N), extract:

```
A{row} = stage number (1, 2, 3, 4)
B{row} = stage name (extract and clean)

Stage code mapping (derive from name):
  "Concept" or "Concept Design"      → "CON"
  "Schematic" or "Schematic Design"   → "SD"
  "Design Development" or "Detailed"  → "DD"
  "Contract Documents" or "Tender"    → "CD"
  "Mobilisation"                      → "MOB"
  "IFC"                               → "IFC"
  Other/custom                        → first 3 chars uppercase

pricing.stages[stage_index] (ACTUAL SCHEMA):
{
  id: "s0",                    // "s" + zero-based index
  code: "CON",                 // derived from name
  name: "Concept",             // from B column
  order: 0,                    // zero-based
  percentage: 25.0,            // from column M (as %)
  is_post_contract: false      // true for post-contract stages
}

pricing.cells[] — FLAT ARRAY (one entry per discipline×stage):
[
  { discipline_id: "d1", stage_id: "s0", amount: 75000 },   // C9
  { discipline_id: "d2", stage_id: "s0", amount: 50000 },   // D9
  { discipline_id: "d3", stage_id: "s0", amount: 37500 },   // E9
  { discipline_id: "d4", stage_id: "s0", amount: 37500 },   // F9
  { discipline_id: "d5", stage_id: "s0", amount: 37500 },   // G9
  { discipline_id: "d6", stage_id: "s0", amount: 12500 },   // H9
  { discipline_id: "d1", stage_id: "s1", amount: ... },     // C10
  // ... etc for all stage rows
]

Only include cells where amount > 0.
```

### Step 5: Post-Contract Items Extraction

Scan rows starting from (15+N) until empty row:

```
pricing.post_contract_items[]:
[
  {
    name: "Site supervision",
    quantity: 10,
    unit_price: 2500,
    total: 25000,
    discipline: null  // infer if possible from context
  },
  ...
]
```

### Step 6: Validate Calculations

Before writing to DB:

```
✓ C2:H2 sum ≈ 100% (allow ±0.5% rounding)
✓ Stage percentages (M9:M{8+N}) sum ≈ 100%
✓ Design stages subtotal = Σ(stage cells) per discipline
✓ Grand total = subtotal + VAT
✓ Post-contract items all have positive amounts
```

---

## SurrealDB Schema (Actual)

### Project Matching

Projects use record IDs with underscores: `projects:22_97111`
The `number.id` field stores dash-separated codes: `"22-97111"`

To look up a project by code extracted from filename:
```sql
-- Convert dash code to underscore ID directly
SELECT * FROM projects:22_97111;
-- Or query by number.id field
SELECT * FROM projects WHERE number.id == "22-97111";
```

62 projects exist in DB. The record ID is the code with dashes replaced by underscores.

### Fee Record Structure

Fee records use composite IDs: `fee:{project_code}_{rev_number}` (e.g., `fee:22_97111_1`)

**Defined fields** (from `INFO FOR TABLE fee`):
```
activity:      string
company_id:    record<company>
contact_id:    record<contacts>
issue_date:    string (6 chars, numeric: "220729")
name:          string
number:        string (e.g., "22-97111-FP")
package:       string
project_id:    record<projects>
rev:           int (computed: max of revisions[*].revision_number, default 0)
revisions:     array<object> (default [])
staff_email:   string
staff_name:    string
staff_phone:   string
staff_position: string
status:        string (Draft|Sent|Negotiation|Awarded|Completed|Lost|Cancelled|On Hold|Revised)
strap_line:    string
time.created_at: datetime
time.updated_at: datetime
```

**Pricing sub-fields** (defined on fee table):
```
pricing.config.client_currency:  option<string>
pricing.config.exchange_rate:    option<number>
pricing.config.quote_currency:   option<string>
pricing.config.rate_locked_at:   option<datetime>
```

**NOTE**: The `pricing` object itself is schemaless — additional nested fields (disciplines, stages, cells, config.target_fee, etc.) are stored as flexible data. The only typed sub-fields are the 4 currency config fields above.

### Pricing Object Structure (from live data)

```typescript
// Actual pricing structure used by the app calculator
{
  pricing: {
    config: {
      // Currency fields (typed):
      client_currency: "AED",
      exchange_rate: 1.0,
      quote_currency: "AED",
      rate_locked_at: null,
      // Additional flexible fields for import:
      target_fee: 500000,
      vat_percent: 5.0,
      mobilisation_percent: 20.0
    },

    // Discipline definitions — flat array
    disciplines: [
      { id: "d1", name: "Lighting", order: 0, percentage: 30.0 },
      { id: "d2", name: "Video", order: 1, percentage: 20.0 },
      { id: "d3", name: "Audio", order: 2, percentage: 15.0 },
      { id: "d4", name: "SFX", order: 3, percentage: 15.0 },
      { id: "d5", name: "Show Control", order: 4, percentage: 15.0 },
      { id: "d6", name: "Sub", order: 5, percentage: 5.0 }
    ],

    // Stage definitions — flat array
    stages: [
      {
        id: "s0", code: "CON", name: "Concept",
        order: 0, percentage: 25.0, is_post_contract: false
      },
      {
        id: "s1", code: "SD", name: "Schematic Design",
        order: 1, percentage: 25.0, is_post_contract: false
      },
      {
        id: "s2", code: "DD", name: "Design Development",
        order: 2, percentage: 30.0, is_post_contract: false
      },
      {
        id: "s3", code: "CD", name: "Contract Documents",
        order: 3, percentage: 20.0, is_post_contract: false
      }
    ],

    // Cell values — FLAT array (cross-reference by discipline_id + stage_id)
    cells: [
      { discipline_id: "d1", stage_id: "s0", amount: 37500 },
      { discipline_id: "d1", stage_id: "s1", amount: 37500 },
      { discipline_id: "d2", stage_id: "s0", amount: 25000 },
      // ... one entry per discipline×stage combination
    ],

    // Post-contract items (optional)
    post_contract_items: [
      {
        name: "Site supervision",
        quantity: 10,
        unit_price: 2500,
        total: 25000
      }
    ]
  }
}
```

### Import Metadata

Store import metadata as a flexible field on the fee record:

```typescript
{
  import_source: {
    file_path: "/Volumes/svrroot/.../25-97102/02 Proposal/25-97102-FP-01 Pricing.xlsx",
    imported_at: "2026-02-16T12:00:00Z",
    import_version: "2.0",
    checksum: "sha256:abc123def456...",
    project_folder_status: "01 RFPs",
    filename_pattern: "standard"
  }
}
```

---

## Idempotency & Update Strategy

### Match Existing Records

Fee records have a UNIQUE index on `(project_id, rev)`.
The record ID convention is `fee:{project_code_underscored}_{rev}`.

```sql
-- Direct ID lookup (fastest)
SELECT * FROM fee:25_97102_1;

-- Or query by project + rev
SELECT * FROM fee
WHERE project_id == projects:25_97102
AND rev == 1;
```

**If fee record already exists:**
- Check if `pricing` field is already populated
- If `pricing` exists AND `import_source.checksum` matches: **SKIP** (already imported)
- If `pricing` exists AND checksum differs: **UPDATE** pricing + import_source
- If `pricing` is NONE/missing: **UPDATE** to add pricing data (common case — 29 existing records have no pricing)

**If no fee record exists for this project+FP:**
- **SKIP** with warning — do NOT create fee records (they have required fields like company_id, contact_id, staff info that can't be derived from Excel)
- Log: "No fee record found for {project_code} FP-{nn}. Create the fee proposal in the app first."

### Checksum Calculation

```typescript
// Combine these fields into SHA256
checksum = sha256(JSON.stringify({
  target_fee: 500000,
  disciplines: [...],
  stages: [...],
  post_contract_items: [...],
  vat_percent: 5.0,
  mobilisation_percent: 20.0
}))
```

---

## Invoice Import (Secondary Task)

### Invoice File Discovery

Scan for running lists:
```
{project_folder}/03 Contract/*/00 Inv Running List.xlsx
```

Expected: ~16-20 files across all projects

### Invoice Data Extraction

```
G1: LPO value (original purchase order amount)
L1: Payment terms (e.g., "30 days", "Net 30")

Rows 6+: Invoice entries
  Column A: Invoice number
  Column B: Invoice date
  Column C: Amount
  Column D: Due date
  Column E: Received date (payment)
  Column F: Processing status
  Column J: Stage code (LX, VID, AUD, etc.)
```

### Invoice Record Structure

```typescript
{
  id: "invoice:25-97109-001",
  project_id: "project:25-97109",

  // Invoice basics
  invoice_number: "001",
  date: "2025-04-15",
  amount: 125000,

  // Payment tracking
  due_date: "2025-05-15",
  received_date: "2025-05-20",

  // Context from running list
  lpo_value: 500000,
  payment_terms: "30 days",
  stage_code: "LX",

  // Metadata
  source_file: "/Volumes/svrroot/.../25-97109/03 Contract/00 Inv Running List.xlsx",
  imported_at: "2026-02-14T12:30:00Z"
}
```

---

## Validation Report

After completing import, generate: `docs/import-validation-report.md`

### Report Structure

```markdown
# Excel Import Validation Report
**Generated**: 2026-02-14T12:45:00Z

## Import Summary
- Files scanned: 47
- Files skipped: 5 (unfilled templates)
- Files imported: 42
- Records created: 38
- Records updated: 4
- Errors: 0

## By Folder
- 00 Inactive: 8 files → 8 records
- 01 RFPs: 22 files → 20 records
- 11 Current: 12 files → 10 records
- 99 Completed: 5 files → 4 records

## Per-File Validation

### ✅ 25-97109-FP-01 Pricing.xlsx
- **Location**: 01 RFPs/25-97109/02 Proposal/
- **Status**: CREATED (new)
- **Project**: 25-97109 (FAJAB Resort - Lighting Concept)
- **Excel values**:
  - Target Fee: AED 500,000
  - VAT: 5%
  - Mobilisation: 20%
  - Disciplines: LX(30%) VID(20%) AUD(15%) SFX(15%) CTL(15%) SUB(5%)
  - Stages: 4 (Concept 25%, Schematic 25%, Design Dev 30%, Contract 20%)
  - Design Subtotal: AED 500,000
  - Grand Total: AED 525,000
- **DB values**: ✓ All match
- **Checksum**: sha256:f2a8b...
- **Notes**: Pricing locked at import time

### ⚠️ 25-97115-FP-02 Pricing.xlsx
- **Location**: 01 RFPs/25-97115/02 Proposal/
- **Status**: UPDATED (checksum changed)
- **Changes**: Stage 1 percentage changed from 25% to 30%
- **Previous checksum**: sha256:a1b2c...
- **New checksum**: sha256:d4e5f...
- **Timestamp**: 2026-02-14T12:31:00Z

### 🚫 25-97120 Pricing.xlsx
- **Location**: 01 RFPs/25-97120/02 Proposal/
- **Status**: SKIPPED (unfilled template)
- **Reason**: B2=0, C2:H2 all empty
- **Action**: Manual review recommended

## Discipline Summary
```

---

## Step-by-Step Implementation

### Phase 1: Discovery & Validation

1. **List all project folders**
   ```
   ls -R /Volumes/svrroot/user/emittiv/nc/__groupfolders/1/01\ Projects/
   ```

2. **For each project folder**, scan for `02 Proposal/*.xlsx` files

3. **Apply filename patterns** to identify pricing files (priority order)

4. **Read B2:B2 + C2:H2** to check skip conditions

5. **Build file inventory** with metadata:
   ```
   - file_path
   - folder_status (00/01/11/99)
   - filename_pattern (standard/legacy/bare/numbered/subpackage)
   - project_code
   - fee_package
   - skip_reason (if any)
   ```

### Phase 2: Data Extraction

For each **non-skipped file**:

1. **Verify project in DB**
   ```sql
   -- Direct ID lookup (convert 25-97102 → 25_97102)
   SELECT id, name FROM projects:25_97102;
   ```

2. **Read template range** B1:R{15+stage_count}

3. **Extract configuration** (O2:R4)

4. **Extract disciplines** (C2:H2)

5. **Extract design stages** (rows 9 to 8+N)

6. **Extract post-contract** (rows 15+N onward)

7. **Validate calculations**

8. **Calculate checksum** (SHA256 of pricing data)

### Phase 3: Idempotency Check

For each extracted pricing data:

1. **Query existing fee record**
   ```sql
   -- Direct ID lookup: fee:{project_underscored}_{rev}
   SELECT id, pricing, import_source FROM fee:25_97102_1;
   ```

2. **If fee record exists WITH pricing**:
   - Compare `import_source.checksum` with new checksum
   - Skip if identical
   - Update pricing + import_source if different

3. **If fee record exists WITHOUT pricing** (common — 29 records):
   - UPDATE to add pricing data

4. **If no fee record exists**:
   - SKIP with warning (do not create — fee records need company/contact/staff data)

### Phase 4: Database Write

**For existing fee records** (UPDATE pricing on existing record):
```sql
UPDATE fee:25_97102_1 SET
  pricing = {
    config: { target_fee: 500000, vat_percent: 5.0, mobilisation_percent: 20.0 },
    disciplines: [ ... ],
    stages: [ ... ],
    cells: [ ... ],
    post_contract_items: [ ... ]
  },
  import_source = {
    file_path: "/Volumes/svrroot/...",
    imported_at: time::now(),
    import_version: "2.0",
    checksum: "sha256:...",
    project_folder_status: "01 RFPs",
    filename_pattern: "standard"
  };
```

**IMPORTANT**: Use SurrealDB MCP `query` tool with the UPDATE statement above.
Do NOT use `create` (would fail on existing records) or overwrite other fields.
The UPDATE only touches `pricing` and `import_source` — all other fields (company_id,
contact_id, staff info, status, etc.) are preserved.

**For projects with no fee record**: SKIP — do not create. Log warning.

### Phase 5: Invoice Import (Optional)

Repeat discovery/extraction for `*/03 Contract/*/00 Inv Running List.xlsx` files.

### Phase 6: Validation Report

Generate markdown report comparing Excel values vs DB values for all imported records.

---

## Error Handling

### File Read Errors

```
If Excel file cannot be read:
- Log error with file path
- Skip file
- Continue to next file
- Report in validation report
```

### Validation Errors

```
If calculations don't validate:
- Log discrepancy details
- Mark as "VALIDATION_WARNING" in report
- Still create/update record (data is source of truth)
- Flag for manual review
```

### Database Errors

```
If SurrealDB write fails:
- Log full error message
- Skip that fee record
- Continue to next file
- Report failed count in summary
```

### Project Lookup Failures

```
If project_code not found in DB:
- Log warning: "Project not found: 25-97109"
- Skip file
- Continue to next file
```

---

## Success Criteria

✅ **Import complete when**:
1. All project folders scanned
2. All non-skipped Excel files processed
3. Fee records created/updated in SurrealDB
4. All checksums calculated and stored
5. Validation report generated
6. No data loss or corruption

✅ **Records accurate when**:
1. Target fee matches B2
2. Discipline percentages match C2:H2
3. Stage names match B9:B{8+N}
4. Stage amounts match C9:H{8+N}
5. Stage percentages match M9:M{8+N}
6. VAT and mobilisation percentages match P3:P4

---

## Related Files

- **Excel Template Reference**: See shared project folder templates
- **Database Schema**: `DATABASE_SCHEMA.md`
- **Fee Pricing Patterns**: `.claude/rules/development-workflow.md`
- **SurrealDB Queries**: `src-tauri/src/db/mod.rs`

---

## Notes for Agent

- **Token Budget**: Plan for 150K+ tokens for large batch imports
- **Performance**: Process files sequentially; SurrealDB has single writer lock
- **Data Integrity**: Always verify checksums match before updating
- **Backups**: Consider exporting existing fees before large imports
- **Testing**: Test with 2-3 files first before full batch
- **Logging**: Include detailed logs for every skip, create, update operation

### Current State (2026-02-16)

- **DB connection**: `http://10.0.21.8:8000` (namespace: emittiv, database: projects, user: martin, password: th38ret3ch)
- **62 projects** in DB, **29 fee records** (none have pricing data from import, 2 have app-created pricing)
- **77 Excel files** found, ~59 processable after template/archive filtering
- **Only `25-97109`** (Mapletree Warehouse) has no matching project in DB
- **SurrealDB MCP** is already connected — use `mcp__SurrealDB__query` tool
- **Excel MCP** available — use `mcp__excel__read_data_from_excel` tool
- **File paths** use SMB mount at `/Volumes/svrroot/...` (spaces in path names — always quote)

### Critical: Excel MCP Behavior

The `mcp__excel__read_data_from_excel` tool returns:
- **Literal values** as numbers (75400, 188500)
- **Formula cells** as formula strings ("=SUM(C9:H9)", "=$B$2*C2")
- **Empty cells** as null

**When reading cell values:**
- If value is a number → use it directly
- If value is a string starting with "=" → it's a formula, treat as null/skip
- If value is null → cell is empty

**Percentage format detection:**
- Discipline percentages (C2:H2) may be decimals (0.35) or integers (35 or 100)
- Rule: If ALL non-zero values in C2:H2 are ≤ 1.0, they are decimals → multiply by 100
- Stage percentages (M column) follow the same rule
- VAT (P3) and Mobilisation (P4) follow the same rule

**Stage numbering:**
- Stages don't always start at 1. Some projects skip Concept (start at stage 2 SD).
- Use A column value for informational purposes only. The `order` field should be zero-based sequential.

### Discipline Column Mapping (Fixed)

| Column | Excel Header | Discipline ID | Name |
|--------|-------------|---------------|------|
| C | Lighting | d1 | Lighting |
| D | Video | d2 | Video |
| E | Audio | d3 | Audio |
| F | SFX | d4 | SFX |
| G | Show Control | d5 | Show Control |
| H | Sub | d6 | Sub |

### Stage Code Derivation

| Stage Name Pattern | Code |
|-------------------|------|
| Mobilisation | MOB |
| Concept* | CON |
| Schematic* | SD |
| Design Development* / Detailed* | DD |
| Contract Documents* / Tender* | CD |
| IFC | IFC |
| Other | First 3 chars uppercase |

---

**Last Updated**: 2026-02-16
**Version**: 2.0
**Status**: Schema verified against live DB, ready for implementation
