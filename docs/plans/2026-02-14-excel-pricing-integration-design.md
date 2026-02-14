# Excel Pricing Integration Design

**Date:** 2026-02-14
**Status:** Draft
**Scope:** Historical import, Excel export, fee versioning

## Overview

Three parallel workstreams to integrate E-Fees with the existing Excel-based pricing workflow:

- **Workstream A**: Import historical pricing data from project folder Excel files into SurrealDB
- **Workstream B**: Export pricing from E-Fees back to the Excel template format in project folders
- **Workstream C**: Fee version management UI (create revisions, view history)

Long-term evolution: Excel templates (now) -> CSV for InDesign merge (next) -> direct PDF generation from E-Fees (final).

## Template Structure Analysis

### Pricing Template (`*-FP-NN Pricing.xlsx`)

Single sheet with a discipline x stage matrix. 90% fixed structure; only stage count (3-4) causes row shifts.

#### Fixed Rows (Never Move)

| Row | Col B | Col C-H | Col I | Purpose |
|-----|-------|---------|-------|---------|
| 1 | "Project Target" | Discipline names | "Total" | Header |
| 2 | Target amount (e.g. 185000) | Allocation % per discipline | =SUM | Multipliers |
| 3 | "Target Price" | =B2*C2 per discipline | =SUM | Calculated targets |
| 6 | "Remaining" | =target-sub per discipline | =SUM | Unallocated budget |
| 8 | "Design Stages" | | | Section header |

#### Fixed Config Section (Cols O-R, Never Move)

| Cell | Content | Purpose |
|------|---------|---------|
| O2/P2 | "Stages" / `=COUNT(A:A)` | Stage count |
| O3/P3 | "VAT" / 0.05 | VAT rate |
| O4/P4 | "Mobilisation" / 0.2-0.3 | Mobilisation % |
| L8/M8 | "Rec Fee" / "%" | Recommended fee labels |
| O8/P8 | "Mobilisation" / `=I{subtotalRow}*P4` | Mob calculation |
| R8 | "Costs" | Cost column header |

#### Dynamic Rows (Shift Based on Stage Count)

```
N = stage count (3 or 4 observed)

Row 9 to 8+N:      Design stage rows (numbered 1-N in col A)
Row 9+N (Sub):      Subtotal per discipline  =SUM(C9:C{8+N})
Row 10+N (VAT):     VAT calculation          =I{9+N}*P3
Row 11+N (Grand):   Grand total              =SUM(I{9+N}:I{10+N})
Row 14+N:           Post-contract header "Post Contract"
Row 15+N onwards:   Post-contract line items (2-5 items)
```

**Example for 4 stages:** Sub=13, VAT=14, Grand=15, Post-contract starts=18
**Example for 3 stages:** Sub=12, VAT=13, Grand=14, Post-contract starts=17

#### Side Columns (L-M, P, R) — Per Stage Row

For each stage row (9 to 8+N):
- L{row}: `=$L$6*M{row}` — Recommended fee for this stage
- M{row}: Stage percentage (e.g. 0.25, 0.30, 0.35, 0.10)
- P{row}: `=I{row}-($P$8/$P$2)` — Net fee after mobilisation
- R{row}: Cost value (typically 0, manually entered)

#### 6 Disciplines (Always Fixed)

| Column | Name | Typical Projects |
|--------|------|-----------------|
| C | Lighting | All projects |
| D | Video | Entertainment/hospitality |
| E | Audio | Entertainment/hospitality |
| F | SFX | Theme parks |
| G | Show Control | Theme parks/entertainment |
| H | Sub | Subcontractor allocation |
| I | Total | =SUM(C:H) always |

#### Post-Contract Items

Located after Grand Total + 3 gap rows. Variable count (2-5 items):

| Column | Content |
|--------|---------|
| B | Item name (e.g. "Submittal Review") |
| C | qty (quantity) |
| D | price (unit price) |
| E | est = D*C (formula) |
| F | Notes |

Standard items: Submittal Review, Construction Supervision, Programming, Handover, DLP.
Some projects have fewer (JOH: 2 items) or custom names.

Post-contract has its own VAT + Grand Total rows after the last item.

#### Stage Name Variations

| Project | Stages | Names |
|---------|--------|-------|
| Standard | 4 | CD, SD, DD, TD |
| MAF FEC v3 | 3 | CD, DD, IFC |
| Shanghai Tang | 3 | 50% DD, 100% DD, TD |
| JOH | 4 | Initial Engineering (x4) |

### Invoice Running List (`00 Inv Running List.xlsx`)

Sheet "Running Total" with columns A-L:

| Row/Cell | Content |
|----------|---------|
| A1/G1 | "Approved Budget" / LPO value excl VAT |
| K1/L1 | "Net" / Payment terms (30 days) |
| A2/G2 | "Total Billed to Date" / =SUM(C6:C101) |
| A3/G3 | "% of LPO" / =G2/G1 |
| Row 5 | Headers: Inv No, Inv Date, Inv Amount, Inv Due, Received, Processing, Remaining Budget |
| Row 6+ | Invoice rows with formulas: Due=Date+terms, Remaining=prev-amount |
| Col J | Stage code per invoice (Mob, CD, SD, DD, TD) |

## Workstream A: Historical Import Agent

### Approach

Standalone task agent using Excel MCP to read files and SurrealDB MCP to write records.
No app dependency. Idempotent (safe to re-run).

### File Discovery

Base path: `/Volumes/svrroot/user/emittiv/nc/__groupfolders/1/01 Projects/`

Scan folders: `00 Inactive/`, `01 RFPs/`, `11 Current/`, `99 Completed/`

#### Pricing Files

Target: `*/02 Proposal/*.xlsx` files containing pricing data.

Filename patterns to match:
1. `yy-cccnn-FP-NN Pricing.xlsx` (standard)
2. `e-yy-cccnn-FP-NN Pricing.xlsx` (older with e- prefix)
3. `Pricing.xlsx` (bare, early projects)
4. `yy-cccnn Pricing.xlsx` or `yy-cccnn Pricing rN.xlsx` (numbered)
5. `e-yy-cccnn-XX-FP-NN Pricing.xlsx` (sub-packages, e.g. AA, EL)

Skip unfilled templates: Detect by checking B2 (project target) AND C2:H2 (discipline multipliers). If all are 0/null/empty, skip.

Skip non-pricing files: Vendor quotes ("Quote N -"), client schedules ("QPMO-"), sub-consultant sheets, deployment/expense files.

#### Invoice Files

Target: `*/03 Contract/*/00 Inv Running List.xlsx`

~16 files across Current and Completed projects.

### Data Mapping

```
Excel Cell/Range          -> E-Fees Field
----------------------------------------------
Folder name (yy-cccnn)   -> project.number (match existing or create)
Folder status (00/01/11/99) -> project.status (inactive/rfp/active/completed)
FP-NN from filename      -> fee.rev (revision number)
XX from filename          -> fee.package (sub-package code, e.g. "AA")

B2                        -> fee.pricing.baseFee (project target)
C1:H1                     -> fee.pricing.disciplines[].name
C2:H2                     -> fee.pricing.disciplines[].multiplier
P3                        -> fee.pricing.taxRate
P4                        -> fee.pricing.mobilisation

Rows 9 to 8+N, Col A     -> stage numbers
Rows 9 to 8+N, Col B     -> fee.pricing.stages[].name
Rows 9 to 8+N, C-H       -> fee.pricing.cells[stage][discipline].quoted
M9:M{8+N}                -> fee.pricing.stages[].percentage
R9:R{8+N}                -> fee.pricing.stages[].costs

Post-contract rows, B     -> fee.postContractItems[].name
Post-contract rows, C     -> fee.postContractItems[].qty
Post-contract rows, D     -> fee.postContractItems[].price
Post-contract rows, E     -> fee.postContractItems[].estimated (verify =C*D)
```

### Source Tracking

Every imported fee record gets an `import_source` field:

```json
{
  "import_source": {
    "file_path": "/Volumes/svrroot/.../25-97109-FP-01 Pricing.xlsx",
    "file_modified": "2025-03-16T10:30:00Z",
    "file_size": 11877,
    "imported_at": "2026-02-14T12:00:00Z",
    "import_version": "1.0",
    "checksum": "sha256:abc123...",
    "project_folder_status": "01 RFPs",
    "filename_pattern": "standard"
  }
}
```

### Idempotency

Match existing records by `project_id + rev` (unique index). If a record already exists:
- Compare checksum. If unchanged, skip.
- If changed, update and log the change.

### Validation Report

After import, generate `docs/import-validation-report.md` with one entry per file:

```
## 25-97109 Mapletree Warehouse (FP-01)
Source: .../25-97109-FP-01 Pricing.xlsx
Status: IMPORTED (fee:xxx)

| Field | Excel Value | DB Value | Match |
|-------|------------|----------|-------|
| Project Target | 185,000 | 185,000 | OK |
| Disciplines | Lighting (100%) | LX (1.00) | OK |
| Stages | CD/SD/DD/TD | CD/SD/DD/TD | OK |
| Stage % | 25/30/35/10 | 25/30/35/10 | OK |
| VAT Rate | 5% | 5% | OK |
| Mobilisation | 20% | 20% | OK |
| Post-Contract | 5 items | 5 items | OK |
| Design Subtotal | 185,000 | 185,000 | OK |
| Grand Total | 194,250 | 194,250 | OK |
```

Files that can't be parsed (non-standard format, YYYYMMDD naming, etc.) are listed separately for manual review.

### Invoice Import

For each `00 Inv Running List.xlsx`:
1. Read G1 (LPO value), L1 (payment terms)
2. Read invoice rows (A6+): number, date, amount, due, received, processing
3. Read J6+ for stage codes per invoice
4. Store as `fee.invoices[]` array on the matching fee record

## Workstream B: Excel Export to Project Folders

### New Rust Function

`generate_fee_template(fee: &Fee, output_path: &Path, stage_count: usize) -> Result<String, String>`

Generates the pricing template format (with formulas), not the branded summary. The exported file is a working spreadsheet.

### Row Calculation

```rust
let subtotal_row = 8 + stage_count + 1;
let vat_row = subtotal_row + 1;
let grand_row = vat_row + 1;
let post_contract_start = grand_row + 3;
```

### Path Resolution

1. Read `project_folder_path` from settings (SMB share base)
2. Find project folder by matching project number in folder names
3. Save to `02 Proposal/{project_number}-FP-{rev:02} Pricing.xlsx`

### New Tauri Command

`export_fee_to_project_folder(fee_id: String) -> Result<ExportResult, String>`

Returns the path where the file was saved.

### Invoice Running List Update

For active/current projects with invoice data, also update/create `03 Contract/03 Invoices/00 Inv Running List.xlsx`.

## Workstream C: Fee Version Management UI

### New Fields

Add to Fee struct:
- `package: Option<String>` — Sub-package code (e.g. "AA", "EL") for projects with multiple pricing scopes

### UI Changes

1. **"New Revision" button** on proposal detail:
   - Clones current fee as FP-(N+1)
   - Copies pricing data
   - Sets new revision date
   - Opens the new revision for editing

2. **Revision dropdown/sidebar** on proposal view:
   - Shows all revisions for the project
   - Current revision highlighted
   - Click to switch between revisions

3. **Proposals list filter**:
   - Default: show latest revision only per project
   - Toggle: "Show all revisions"
   - Visual indicator for projects with multiple revisions

### Database Query

Add filtered query:
```sql
SELECT * FROM fee WHERE rev = (
  SELECT math::max(rev) FROM fee WHERE project_id = $parent.project_id
)
```

Or simpler: fetch all, group by project_id in the frontend, show latest by default.

## Implementation Priority

1. **Workstream A** (import) — Can run immediately, no code changes needed
2. **Workstream C** (versioning UI) — Needed before B (export needs revision selection)
3. **Workstream B** (export) — Depends on C for revision handling

Workstream A is independent and can run in parallel with C.

## File Inventory

- ~60 pricing Excel files to import (after filtering templates and non-pricing files)
- ~16 invoice running lists to import
- 4 project folder statuses: Inactive, RFPs, Current, Completed
- 2 structural variants: 3-stage and 4-stage layouts
- 6 filename patterns to handle

## Risk Factors

1. **Non-standard files**: YYYYMMDD format files, client-issued pricing schedules — flagged for manual review
2. **SMB share availability**: Agent requires network access to `/Volumes/svrroot`
3. **Discipline name mapping**: Excel uses "Lighting", E-Fees uses "LX" — mapping table needed
4. **Sub-packages**: Only Wynn (23-97102) observed — may need more examples
5. **Formula preservation**: Exported Excel must maintain working formulas
