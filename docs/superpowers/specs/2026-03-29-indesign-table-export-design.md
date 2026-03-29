# InDesign Table Export — Design Spec

## Problem

Fee proposals need to be produced as InDesign documents with populated pricing tables. Currently, table data exists in the database and can be exported to Excel, but InDesign documents require manual data entry. This is error-prone and time-consuming.

## Solution

Extend the existing Excel export to produce a comprehensive pricing workbook with dedicated sheets that InDesign can link to. InDesign's native "Place from Excel" / "Update Links" feature handles table population and layout reflow.

## Architecture

### Layer 1: Enhanced Excel Export (this deliverable)

A new export function in `e-fees-core` that produces a multi-sheet Excel workbook optimized for InDesign table linking.

**Sheets:**

| Sheet Name | Columns | Source | Maps to InDesign Table |
|------------|---------|--------|----------------------|
| `Durations` | Stage / Milestone / Duration | `pricing.stages` where `is_post_contract=false` | T0 (7×3 in template) |
| `Post-Contract Durations` | Stage / Milestone / Duration | `pricing.stages` where `is_post_contract=true` | T1 (5×3 in template) |
| `Design Fees` | Stage / Milestone / Fee | Design stages with fee amounts | T2 (8×3 in template) |
| `Post-Contract Fees` | Stage / Milestone / Unit / Est Qty / Price / Est. Fee | `fee.post_contract_items[]` | T3 (6×6 in template) |
| `Payment Schedule` | Stage / Milestone / Fee / Payment | Design stages + mobilisation row | T4 (9×4 in template) |
| `Revisions` | Date / Release / Author / Reference | `fee.revisions[]` | T-Rev (4×4 in template) |
| `Distribution` | Date / Release / Distribution | `fee.distribution[]` | T-Dist (4×3 in template) |

**Each sheet includes:**
- A header row matching InDesign table column headers exactly
- Data rows matching the fee record's actual stages (variable count)
- A total/summary row where applicable (T2, T3, T4)
- Post-contract sheets are empty (no data rows) when fee has no post-contract items

### Layer 2: InDesign Template Linking (manual, one-time setup)

The .indd template is manually configured to link each table to its corresponding Excel sheet/range. This is done once in InDesign using Place (Cmd+D) with "Create Static Captions" off and "Replace Selected Item" on.

### Layer 3: Future Automation (deferred)

- UXP scripting for automatic link updates, post-contract section removal, Save As
- Full text/scope content population
- PDF export automation

## Data Flow

```
Fee record (DB)
    ↓
generate_fee_indesign_workbook() — new function in e-fees-core
    ↓
<project>/02 Proposal/<number>-pricing.xlsx  (multi-sheet workbook)
    ↓
InDesign: Update Links → tables populate with current data
    ↓
User reviews, adjusts, exports PDF
    ↓
Revision logged to DB (date, rev, author, ref)
```

## Key Decisions

1. **Excel-linked tables over IDML manipulation** — InDesign handles reflow, formatting, and page layout natively. IDML surgery risks breaking surrounding text and requires understanding InDesign's XML schema deeply.

2. **Separate sheets per table** — Each InDesign table links to one sheet. Clean separation, easy to understand, predictable cell ranges.

3. **Post-contract sheets empty when design-only** — Rather than omitting sheets (which would break InDesign links), sheets exist but contain only the header row. User deletes the linked tables and surrounding text manually in InDesign for design-only proposals.

4. **Reuse existing data logic** — The current `excel_export.rs` already computes all pricing values (stage totals, overrides, post-contract amounts). The new function repackages the same data into an InDesign-friendly structure.

## Implementation Scope

### In Scope
- New `generate_fee_indesign_workbook()` function in `e-fees-core/src/export/`
- Multi-sheet Excel output with all 7 tables
- Tauri command `export_indesign_workbook` (saves to project folder)
- API endpoint `POST /fees/{id}/export/indesign` (returns bytes)
- Revision record creation in DB on each export
- Unit tests for sheet generation with various stage configurations

### Out of Scope (deferred)
- InDesign template linking setup (manual, one-time)
- UXP automation for link updates
- Post-contract section removal automation
- Scope/deliverables text population
- PDF export automation
- Distribution list population (needs schema addition — `fee.distribution` field doesn't exist yet)

## File Locations

- Export function: `crates/e-fees-core/src/export/indesign_export.rs`
- Tauri command: `src-tauri/src/commands/export.rs` (extend existing)
- API route: `e-fees-api/src/routes/fees.rs` (extend existing)
- Excel template reference: `src-tauri/resources/template.idml` (for structure reference)
- InDesign template: project's `02 Proposal/` folder

## Schema Notes

### Existing fields used
- `fee.pricing_typed()` → `PricingBreakdown` (stages, disciplines, cells, config)
- `fee.post_contract_items` → `Vec<PostContractItem>`
- `fee.payment_schedule` → `Vec<PaymentScheduleItem>` (exists in model, not yet exported)
- `fee.revisions` → needs to be added or confirmed in schema
- `fee.number`, `fee.rev`, `fee.status`, `fee.issue_date`

### Fields to confirm/add
- `fee.revisions: Vec<Revision>` — may need schema addition
- `fee.distribution: Vec<DistributionEntry>` — needs schema addition (deferred)

## Testing Strategy

- Unit tests: generate workbook for design-only fee (no post-contract), full fee (all tables), minimal fee (2 stages), maximum fee (10+ stages)
- Verify sheet count, row counts, header accuracy, cell values
- Verify total rows compute correctly
- Integration test: export via Tauri command, verify file exists and is valid xlsx

---
*Created: 2026-03-29*
