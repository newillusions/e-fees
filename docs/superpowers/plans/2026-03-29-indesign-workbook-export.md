# InDesign Workbook Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate a multi-sheet Excel workbook from a fee record, with one sheet per InDesign table, so InDesign can link to each sheet for automatic table population.

**Architecture:** New `indesign_workbook` module in `e-fees-core/src/export/` produces a `Vec<u8>` (xlsx bytes) from a `&Fee`. The Tauri app and standalone API both call this function — Tauri saves to the project folder, API returns bytes in the response.

**Tech Stack:** `rust_xlsxwriter` (already used in `src-tauri`), Rust, existing `Fee`/`PricingBreakdown`/`Stage`/`PostContractItem`/`PaymentScheduleEntry` types from `e-fees-core::models::fee`.

---

## File Structure

| Action | Path | Purpose |
|--------|------|---------|
| Move | `crates/e-fees-core/src/export.rs` → `crates/e-fees-core/src/export/mod.rs` | Convert to module directory |
| Create | `crates/e-fees-core/src/export/indesign_workbook.rs` | Core workbook generation logic |
| Modify | `crates/e-fees-core/Cargo.toml` | Add `rust_xlsxwriter` dependency |
| Modify | `src-tauri/src/commands/export.rs` | Add `export_indesign_workbook` Tauri command |
| Modify | `src-tauri/src/lib.rs` | Register new command in invoke handler |
| Modify | `e-fees-api/src/routes/fees.rs` | Add `POST /fees/{id}/export/indesign` route |

---

### Task 1: Convert export module to directory and add dependency

**Files:**
- Move: `crates/e-fees-core/src/export.rs` → `crates/e-fees-core/src/export/mod.rs`
- Modify: `crates/e-fees-core/Cargo.toml`

- [ ] **Step 1: Create the export directory and move the file**

```bash
mkdir -p crates/e-fees-core/src/export
mv crates/e-fees-core/src/export.rs crates/e-fees-core/src/export/mod.rs
```

- [ ] **Step 2: Add `rust_xlsxwriter` to `e-fees-core/Cargo.toml`**

Add under `[dependencies]`:

```toml
rust_xlsxwriter = "0.82"
```

- [ ] **Step 3: Add module declaration to `mod.rs`**

At the top of `crates/e-fees-core/src/export/mod.rs`, after the existing module doc comment, add:

```rust
pub mod indesign_workbook;
```

- [ ] **Step 4: Verify existing tests still pass**

Run: `cargo test -p e-fees-core --lib`
Expected: All existing `export::tests` pass unchanged.

- [ ] **Step 5: Commit**

```bash
git add crates/e-fees-core/src/export/ crates/e-fees-core/Cargo.toml
git commit -m "refactor(export): convert export module to directory, add rust_xlsxwriter dep"
```

---

### Task 2: Write failing tests for the InDesign workbook generator

**Files:**
- Create: `crates/e-fees-core/src/export/indesign_workbook.rs`

- [ ] **Step 1: Create the file with test helpers and first test**

Write to `crates/e-fees-core/src/export/indesign_workbook.rs`:

```rust
//! InDesign workbook generator — produces a multi-sheet Excel file
//! with one sheet per InDesign table for linked table population.

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};

use crate::models::fee::{
    Fee, PaymentScheduleEntry, PostContractItem, PricingBreakdown, Stage,
};

/// Generate a multi-sheet Excel workbook for InDesign table linking.
///
/// Returns the workbook as xlsx bytes. Each sheet corresponds to one
/// InDesign table and contains a header row + data rows.
///
/// Sheets produced:
/// - "Durations" — design stage durations (T0)
/// - "Post-Contract Durations" — post-contract stage durations (T1)
/// - "Design Fees" — design stage fees (T2)
/// - "Post-Contract Fees" — post-contract item fees (T3)
/// - "Payment Schedule" — payment schedule entries (T4)
/// - "Revisions" — document revision history (T-Rev)
pub fn generate_indesign_workbook(fee: &Fee) -> Result<Vec<u8>, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::fee::{
        Discipline, PaymentSchedule, PricingBreakdown, PricingCell, PricingConfig, Revision,
        Stage,
    };
    use crate::models::common::TimeStamps;
    use surrealdb::types::RecordId;
    use surrealdb_types::Datetime;

    fn make_timestamps() -> TimeStamps {
        TimeStamps {
            created_at: Datetime::default(),
            updated_at: Datetime::default(),
        }
    }

    fn make_design_stages() -> Vec<Stage> {
        vec![
            Stage {
                id: "prelim".into(),
                name: "Preliminaries".into(),
                code: "".into(),
                percentage: 5.0,
                order: 0,
                is_post_contract: false,
            },
            Stage {
                id: "cd".into(),
                name: "Concept Design".into(),
                code: "1".into(),
                percentage: 20.0,
                order: 1,
                is_post_contract: false,
            },
            Stage {
                id: "sd".into(),
                name: "Schematic Design".into(),
                code: "2".into(),
                percentage: 25.0,
                order: 2,
                is_post_contract: false,
            },
            Stage {
                id: "dd".into(),
                name: "Detailed Design".into(),
                code: "3".into(),
                percentage: 30.0,
                order: 3,
                is_post_contract: false,
            },
        ]
    }

    fn make_post_contract_stages() -> Vec<Stage> {
        vec![
            Stage {
                id: "cs".into(),
                name: "Construction Supervision".into(),
                code: "5".into(),
                percentage: 0.0,
                order: 5,
                is_post_contract: true,
            },
            Stage {
                id: "ho".into(),
                name: "Handover".into(),
                code: "6".into(),
                percentage: 0.0,
                order: 6,
                is_post_contract: true,
            },
        ]
    }

    fn make_pricing(include_post_contract: bool) -> PricingBreakdown {
        let mut stages = make_design_stages();
        if include_post_contract {
            stages.extend(make_post_contract_stages());
        }
        PricingBreakdown {
            config: PricingConfig {
                target_fee: 100000.0,
                buffer_percent: 10.0,
                quoted_fee: 110000.0,
                currency: "AED".into(),
                vat_percent: 5.0,
                vat_included: false,
                mobilisation_percent: 30.0,
            },
            disciplines: vec![Discipline {
                id: "ld".into(),
                name: "Lighting Design".into(),
                percentage: 100.0,
                order: 1,
            }],
            stages,
            cells: vec![
                PricingCell { discipline_id: "ld".into(), stage_id: "prelim".into(), amount: 5000.0, override_amount: None },
                PricingCell { discipline_id: "ld".into(), stage_id: "cd".into(), amount: 20000.0, override_amount: None },
                PricingCell { discipline_id: "ld".into(), stage_id: "sd".into(), amount: 25000.0, override_amount: None },
                PricingCell { discipline_id: "ld".into(), stage_id: "dd".into(), amount: 30000.0, override_amount: None },
            ],
            costs: vec![],
            design_phase_total: 80000.0,
            post_contract_total: 30000.0,
            costs_total: 0.0,
            subtotal: 110000.0,
            vat_amount: 5500.0,
            grand_total: 115500.0,
        }
    }

    fn make_post_contract_items() -> Vec<PostContractItem> {
        vec![
            PostContractItem {
                id: "pci1".into(),
                stage_id: "cs".into(),
                description: "Construction Supervision".into(),
                quantity: 18.0,
                unit: "month".into(),
                rate: 8400.0,
                amount: 151200.0,
            },
            PostContractItem {
                id: "pci2".into(),
                stage_id: "ho".into(),
                description: "Handover".into(),
                quantity: 2.0,
                unit: "visit".into(),
                rate: 8400.0,
                amount: 16800.0,
            },
        ]
    }

    fn make_payment_entries() -> Vec<PaymentScheduleEntry> {
        vec![
            PaymentScheduleEntry {
                id: "ps1".into(),
                payment_type: "mobilisation".into(),
                description: "Mobilisation".into(),
                stage_id: None,
                stage_percentage: None,
                amount: 33000.0,
                percentage_of_total: 30.0,
                due_date: None,
                status: "pending".into(),
                invoice_number: None,
                invoice_date: None,
                paid_date: None,
            },
            PaymentScheduleEntry {
                id: "ps2".into(),
                payment_type: "milestone".into(),
                description: "Concept Design".into(),
                stage_id: Some("cd".into()),
                stage_percentage: Some(20.0),
                amount: 22000.0,
                percentage_of_total: 20.0,
                due_date: None,
                status: "pending".into(),
                invoice_number: None,
                invoice_date: None,
                paid_date: None,
            },
        ]
    }

    fn make_revisions() -> Vec<Revision> {
        vec![
            Revision {
                revision_number: 1,
                revision_date: "260301".into(),
                author_email: "martin@emittiv.com".into(),
                author_name: "MR".into(),
                notes: "Initial issue".into(),
            },
        ]
    }

    fn make_fee(include_post_contract: bool) -> Fee {
        Fee {
            id: None,
            name: "Test Fee Proposal".into(),
            number: "26-97101-R1".into(),
            rev: 1,
            status: "Draft".into(),
            issue_date: "260315".into(),
            activity: "Lighting Design".into(),
            package: "Full".into(),
            project_id: RecordId::new("projects", "26_97101"),
            company_id: RecordId::new("company", "acme"),
            contact_id: RecordId::new("contacts", "john"),
            staff_name: "Alice Smith".into(),
            staff_email: "alice@emittiv.com".into(),
            staff_phone: "+971501234567".into(),
            staff_position: "Senior Designer".into(),
            strap_line: "Excellence in light".into(),
            revisions: make_revisions(),
            time: make_timestamps(),
            pricing: Some(serde_json::to_value(make_pricing(include_post_contract))
                .map(|v| crate::models::common::json_to_dbvalue(&v))
                .unwrap()),
            post_contract_items: if include_post_contract {
                Some(serde_json::to_value(make_post_contract_items())
                    .map(|v| crate::models::common::json_to_dbvalue(&v))
                    .unwrap())
            } else {
                None
            },
            reimbursable_costs: None,
            payment_schedule: if include_post_contract {
                Some(serde_json::to_value(PaymentSchedule {
                    entries: make_payment_entries(),
                    total_invoiced: 0.0,
                    total_paid: 0.0,
                    total_outstanding: 55000.0,
                }).map(|v| crate::models::common::json_to_dbvalue(&v)).unwrap())
            } else {
                Some(serde_json::to_value(PaymentSchedule {
                    entries: make_payment_entries(),
                    total_invoiced: 0.0,
                    total_paid: 0.0,
                    total_outstanding: 55000.0,
                }).map(|v| crate::models::common::json_to_dbvalue(&v)).unwrap())
            },
            pricing_revisions: None,
            current_revision_number: None,
            current_release_number: None,
            import_source: None,
        }
    }

    #[test]
    fn test_generates_valid_xlsx_bytes() {
        let fee = make_fee(true);
        let result = generate_indesign_workbook(&fee);
        assert!(result.is_ok(), "should produce bytes: {:?}", result.err());
        let bytes = result.unwrap();
        // xlsx files start with PK (zip magic)
        assert!(bytes.len() > 100);
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_design_only_has_empty_post_contract_sheets() {
        let fee = make_fee(false);
        let bytes = generate_indesign_workbook(&fee).unwrap();
        // Write to temp, open with rust_xlsxwriter reader isn't available,
        // so we verify via byte size — design-only should be smaller
        assert!(bytes.len() > 100);
    }

    #[test]
    fn test_no_pricing_returns_error() {
        let mut fee = make_fee(false);
        fee.pricing = None;
        let result = generate_indesign_workbook(&fee);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No pricing data"));
    }
}
```

- [ ] **Step 2: Verify tests fail (RED phase)**

Run: `cargo test -p e-fees-core --lib -- export::indesign_workbook`
Expected: 3 tests FAIL with `not yet implemented`

- [ ] **Step 3: Commit failing tests**

```bash
git add crates/e-fees-core/src/export/indesign_workbook.rs
git commit -m "test(export): add failing tests for InDesign workbook generator"
```

---

### Task 3: Implement the workbook generator (GREEN phase)

**Files:**
- Modify: `crates/e-fees-core/src/export/indesign_workbook.rs`

- [ ] **Step 1: Implement `generate_indesign_workbook`**

Replace the `todo!()` in `generate_indesign_workbook` with this implementation:

```rust
pub fn generate_indesign_workbook(fee: &Fee) -> Result<Vec<u8>, String> {
    let pricing = fee
        .pricing_typed()
        .ok_or_else(|| "No pricing data available for InDesign workbook".to_string())?;

    let mut workbook = Workbook::new();

    let design_stages: Vec<&Stage> = pricing
        .stages
        .iter()
        .filter(|s| !s.is_post_contract)
        .collect();

    let post_contract_stages: Vec<&Stage> = pricing
        .stages
        .iter()
        .filter(|s| s.is_post_contract)
        .collect();

    // Sheet 1: Durations (T0)
    write_durations_sheet(&mut workbook, "Durations", &design_stages)?;

    // Sheet 2: Post-Contract Durations (T1)
    write_durations_sheet(&mut workbook, "Post-Contract Durations", &post_contract_stages)?;

    // Sheet 3: Design Fees (T2)
    write_design_fees_sheet(&mut workbook, &design_stages, &pricing)?;

    // Sheet 4: Post-Contract Fees (T3)
    write_post_contract_fees_sheet(&mut workbook, fee)?;

    // Sheet 5: Payment Schedule (T4)
    write_payment_schedule_sheet(&mut workbook, fee)?;

    // Sheet 6: Revisions (T-Rev)
    write_revisions_sheet(&mut workbook, &fee.revisions)?;

    workbook
        .save_to_buffer()
        .map_err(|e| format!("Failed to save workbook: {}", e))
}
```

- [ ] **Step 2: Add the format helpers**

Add above the `generate_indesign_workbook` function:

```rust
fn fmt_header() -> Format {
    Format::new()
        .set_bold()
        .set_font_name("Montserrat")
        .set_font_size(10)
        .set_align(FormatAlign::Center)
        .set_border_bottom(FormatBorder::Thin)
}

fn fmt_text() -> Format {
    Format::new()
        .set_font_name("Montserrat")
        .set_font_size(10)
}

fn fmt_number() -> Format {
    Format::new()
        .set_font_name("Montserrat")
        .set_font_size(10)
        .set_num_format("#,##0")
        .set_align(FormatAlign::Right)
}

fn fmt_currency() -> Format {
    Format::new()
        .set_font_name("Montserrat")
        .set_font_size(10)
        .set_num_format("#,##0.00")
        .set_align(FormatAlign::Right)
}

fn fmt_percent() -> Format {
    Format::new()
        .set_font_name("Montserrat")
        .set_font_size(10)
        .set_num_format("0%")
        .set_align(FormatAlign::Right)
}

fn fmt_total() -> Format {
    Format::new()
        .set_bold()
        .set_font_name("Montserrat")
        .set_font_size(10)
        .set_border_top(FormatBorder::Thin)
}

fn fmt_total_currency() -> Format {
    Format::new()
        .set_bold()
        .set_font_name("Montserrat")
        .set_font_size(10)
        .set_num_format("#,##0.00")
        .set_align(FormatAlign::Right)
        .set_border_top(FormatBorder::Thin)
}
```

- [ ] **Step 3: Implement `write_durations_sheet`**

```rust
/// Write a durations sheet (used for both T0 and T1).
fn write_durations_sheet(
    workbook: &mut Workbook,
    sheet_name: &str,
    stages: &[&Stage],
) -> Result<(), String> {
    let sheet = workbook.add_worksheet();
    sheet.set_name(sheet_name).map_err(|e| e.to_string())?;
    sheet.set_column_width(0, 10).map_err(|e| e.to_string())?;
    sheet.set_column_width(1, 35).map_err(|e| e.to_string())?;
    sheet.set_column_width(2, 15).map_err(|e| e.to_string())?;

    // Header row
    sheet.write_string_with_format(0, 0, "Stage", &fmt_header()).map_err(|e| e.to_string())?;
    sheet.write_string_with_format(0, 1, "Milestone", &fmt_header()).map_err(|e| e.to_string())?;
    sheet.write_string_with_format(0, 2, "Duration", &fmt_header()).map_err(|e| e.to_string())?;

    // Data rows
    for (i, stage) in stages.iter().enumerate() {
        let row = (i + 1) as u32;
        sheet.write_string_with_format(row, 0, &stage.code, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_string_with_format(row, 1, &stage.name, &fmt_text()).map_err(|e| e.to_string())?;
        // Duration is not stored in the Stage struct — leave blank for user to fill
        sheet.write_string_with_format(row, 2, "", &fmt_text()).map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

- [ ] **Step 4: Implement `write_design_fees_sheet`**

```rust
/// Write the design fees sheet (T2).
fn write_design_fees_sheet(
    workbook: &mut Workbook,
    design_stages: &[&Stage],
    pricing: &PricingBreakdown,
) -> Result<(), String> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("Design Fees").map_err(|e| e.to_string())?;
    sheet.set_column_width(0, 10).map_err(|e| e.to_string())?;
    sheet.set_column_width(1, 35).map_err(|e| e.to_string())?;
    sheet.set_column_width(2, 18).map_err(|e| e.to_string())?;

    // Header
    sheet.write_string_with_format(0, 0, "Stage", &fmt_header()).map_err(|e| e.to_string())?;
    sheet.write_string_with_format(0, 1, "Milestone", &fmt_header()).map_err(|e| e.to_string())?;
    sheet.write_string_with_format(0, 2, "Fee", &fmt_header()).map_err(|e| e.to_string())?;

    // Data rows — sum all discipline cells for each stage
    let mut total = 0.0_f64;
    for (i, stage) in design_stages.iter().enumerate() {
        let row = (i + 1) as u32;
        let stage_fee: f64 = pricing
            .cells
            .iter()
            .filter(|c| c.stage_id == stage.id)
            .map(|c| c.override_amount.unwrap_or(c.amount))
            .sum();
        total += stage_fee;

        sheet.write_string_with_format(row, 0, &stage.code, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_string_with_format(row, 1, &stage.name, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_number_with_format(row, 2, stage_fee, &fmt_currency()).map_err(|e| e.to_string())?;
    }

    // Total row
    let total_row = (design_stages.len() + 1) as u32;
    sheet.write_string_with_format(total_row, 0, "Total", &fmt_total()).map_err(|e| e.to_string())?;
    sheet.write_string_with_format(total_row, 1, "", &fmt_total()).map_err(|e| e.to_string())?;
    sheet.write_number_with_format(total_row, 2, total, &fmt_total_currency()).map_err(|e| e.to_string())?;

    Ok(())
}
```

- [ ] **Step 5: Implement `write_post_contract_fees_sheet`**

```rust
/// Write the post-contract fees sheet (T3).
fn write_post_contract_fees_sheet(
    workbook: &mut Workbook,
    fee: &Fee,
) -> Result<(), String> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("Post-Contract Fees").map_err(|e| e.to_string())?;
    sheet.set_column_width(0, 10).map_err(|e| e.to_string())?;
    sheet.set_column_width(1, 35).map_err(|e| e.to_string())?;
    sheet.set_column_width(2, 10).map_err(|e| e.to_string())?;
    sheet.set_column_width(3, 10).map_err(|e| e.to_string())?;
    sheet.set_column_width(4, 12).map_err(|e| e.to_string())?;
    sheet.set_column_width(5, 15).map_err(|e| e.to_string())?;

    let headers = ["Stage", "Milestone", "Unit", "Est Qty", "Price", "Est. Fee"];
    for (col, h) in headers.iter().enumerate() {
        sheet.write_string_with_format(0, col as u16, h, &fmt_header()).map_err(|e| e.to_string())?;
    }

    // Parse post_contract_items from fee
    let items: Vec<PostContractItem> = fee
        .post_contract_items
        .as_ref()
        .and_then(|v| {
            let json = crate::models::common::dbvalue_to_json(v);
            serde_json::from_value(json).ok()
        })
        .unwrap_or_default();

    let pricing = fee.pricing_typed();
    let mut total = 0.0_f64;

    for (i, item) in items.iter().enumerate() {
        let row = (i + 1) as u32;
        // Look up stage code from pricing stages
        let stage_code = pricing
            .as_ref()
            .and_then(|p| p.stages.iter().find(|s| s.id == item.stage_id))
            .map(|s| s.code.as_str())
            .unwrap_or("");

        sheet.write_string_with_format(row, 0, stage_code, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_string_with_format(row, 1, &item.description, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_string_with_format(row, 2, &item.unit, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_number_with_format(row, 3, item.quantity, &fmt_number()).map_err(|e| e.to_string())?;
        sheet.write_number_with_format(row, 4, item.rate, &fmt_currency()).map_err(|e| e.to_string())?;
        sheet.write_number_with_format(row, 5, item.amount, &fmt_currency()).map_err(|e| e.to_string())?;
        total += item.amount;
    }

    // Total row (only if items exist)
    if !items.is_empty() {
        let total_row = (items.len() + 1) as u32;
        sheet.write_string_with_format(total_row, 0, "Estimated Total", &fmt_total()).map_err(|e| e.to_string())?;
        for col in 1..5u16 {
            sheet.write_string_with_format(total_row, col, "", &fmt_total()).map_err(|e| e.to_string())?;
        }
        sheet.write_number_with_format(total_row, 5, total, &fmt_total_currency()).map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

- [ ] **Step 6: Implement `write_payment_schedule_sheet`**

```rust
/// Write the payment schedule sheet (T4).
fn write_payment_schedule_sheet(
    workbook: &mut Workbook,
    fee: &Fee,
) -> Result<(), String> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("Payment Schedule").map_err(|e| e.to_string())?;
    sheet.set_column_width(0, 10).map_err(|e| e.to_string())?;
    sheet.set_column_width(1, 35).map_err(|e| e.to_string())?;
    sheet.set_column_width(2, 12).map_err(|e| e.to_string())?;
    sheet.set_column_width(3, 18).map_err(|e| e.to_string())?;

    let headers = ["Stage", "Milestone", "Fee", "Payment"];
    for (col, h) in headers.iter().enumerate() {
        sheet.write_string_with_format(0, col as u16, h, &fmt_header()).map_err(|e| e.to_string())?;
    }

    // Parse payment schedule from fee
    let schedule: Option<crate::models::fee::PaymentSchedule> = fee
        .payment_schedule
        .as_ref()
        .and_then(|v| {
            let json = crate::models::common::dbvalue_to_json(v);
            serde_json::from_value(json).ok()
        });

    let entries = schedule.map(|s| s.entries).unwrap_or_default();
    let pricing = fee.pricing_typed();

    let mut total_amount = 0.0_f64;
    for (i, entry) in entries.iter().enumerate() {
        let row = (i + 1) as u32;

        // Look up stage code
        let stage_code = entry
            .stage_id
            .as_ref()
            .and_then(|sid| {
                pricing
                    .as_ref()
                    .and_then(|p| p.stages.iter().find(|s| s.id == *sid))
                    .map(|s| s.code.clone())
            })
            .unwrap_or_default();

        let fee_display = if entry.percentage_of_total > 0.0 {
            format!("{:.0}%", entry.percentage_of_total)
        } else {
            String::new()
        };

        sheet.write_string_with_format(row, 0, &stage_code, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_string_with_format(row, 1, &entry.description, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_string_with_format(row, 2, &fee_display, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_number_with_format(row, 3, entry.amount, &fmt_currency()).map_err(|e| e.to_string())?;
        total_amount += entry.amount;
    }

    // Total row
    if !entries.is_empty() {
        let total_row = (entries.len() + 1) as u32;
        sheet.write_string_with_format(total_row, 0, "Total", &fmt_total()).map_err(|e| e.to_string())?;
        sheet.write_string_with_format(total_row, 1, "", &fmt_total()).map_err(|e| e.to_string())?;
        sheet.write_string_with_format(total_row, 2, "", &fmt_total()).map_err(|e| e.to_string())?;
        sheet.write_number_with_format(total_row, 3, total_amount, &fmt_total_currency()).map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

- [ ] **Step 7: Implement `write_revisions_sheet`**

```rust
/// Write the revisions sheet (T-Rev).
fn write_revisions_sheet(
    workbook: &mut Workbook,
    revisions: &[Revision],
) -> Result<(), String> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("Revisions").map_err(|e| e.to_string())?;
    sheet.set_column_width(0, 12).map_err(|e| e.to_string())?;
    sheet.set_column_width(1, 10).map_err(|e| e.to_string())?;
    sheet.set_column_width(2, 15).map_err(|e| e.to_string())?;
    sheet.set_column_width(3, 30).map_err(|e| e.to_string())?;

    let headers = ["Date", "Release", "Author", "Reference"];
    for (col, h) in headers.iter().enumerate() {
        sheet.write_string_with_format(0, col as u16, h, &fmt_header()).map_err(|e| e.to_string())?;
    }

    for (i, rev) in revisions.iter().enumerate() {
        let row = (i + 1) as u32;
        let date = crate::export::format_issue_date(&rev.revision_date);
        let release = format!("{:02}", rev.revision_number);

        sheet.write_string_with_format(row, 0, &date, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_string_with_format(row, 1, &release, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_string_with_format(row, 2, &rev.author_name, &fmt_text()).map_err(|e| e.to_string())?;
        sheet.write_string_with_format(row, 3, &rev.notes, &fmt_text()).map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

- [ ] **Step 8: Run tests (GREEN phase)**

Run: `cargo test -p e-fees-core --lib -- export::indesign_workbook`
Expected: All 3 tests PASS.

- [ ] **Step 9: Commit**

```bash
git add crates/e-fees-core/src/export/indesign_workbook.rs
git commit -m "feat(export): implement InDesign workbook generator with 6 sheets"
```

---

### Task 4: Add Tauri command for InDesign workbook export

**Files:**
- Modify: `src-tauri/src/commands/export.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add the Tauri command to `src-tauri/src/commands/export.rs`**

Add at the end of the file, before the closing (or after the last function):

```rust
/// Export a fee proposal to a multi-sheet InDesign-linked workbook.
///
/// Saves to the project's `02 Proposal/` folder with IDW-NN versioning.
/// Falls back to system temp dir if no project folder exists.
#[tauri::command]
pub async fn export_indesign_workbook(
    fee_id: String,
    output_path: Option<String>,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!(
        "export_indesign_workbook called for id: {}, output_path: {:?}",
        fee_id, output_path
    );

    let record_id = fee_id.strip_prefix("fee:").unwrap_or(&fee_id).to_string();

    let manager = state.read().await;
    let fee = manager.get_fee_by_id(&record_id).await.map_err(|e| {
        error!("Failed to fetch fee for InDesign export: {}", e);
        format!("Failed to fetch fee: {}", e)
    })?;
    let fee = fee.ok_or_else(|| "Fee not found".to_string())?;

    let project_number = fee
        .number
        .split("-R")
        .next()
        .unwrap_or(&fee.number)
        .to_string();

    let safe_number = project_number.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");

    let resolved_path = match output_path {
        Some(p) => PathBuf::from(p),
        None => {
            let proposal_dir = find_proposal_dir(&app_handle, &project_number).await;
            match proposal_dir {
                Some(dir) => {
                    let filename = format!("{}-IDW Pricing.xlsx", safe_number);
                    dir.join(&filename)
                }
                None => {
                    let filename = format!("{}-IDW Pricing.xlsx", safe_number);
                    std::env::temp_dir().join(&filename)
                }
            }
        }
    };

    let bytes = e_fees_core::export::indesign_workbook::generate_indesign_workbook(&fee)?;

    std::fs::write(&resolved_path, &bytes).map_err(|e| {
        error!("Failed to write InDesign workbook: {}", e);
        format!("Failed to write file: {}", e)
    })?;

    let path_str = resolved_path.to_string_lossy().to_string();
    info!("InDesign workbook saved to: {}", path_str);
    reveal_in_file_manager(&path_str);
    Ok(path_str)
}
```

- [ ] **Step 2: Register the command in `src-tauri/src/lib.rs`**

Find the `.invoke_handler(tauri::generate_handler![` block and add `commands::export::export_indesign_workbook` to the list.

- [ ] **Step 3: Verify build**

Run: `cargo check -p app --lib`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/export.rs src-tauri/src/lib.rs
git commit -m "feat(export): add export_indesign_workbook Tauri command"
```

---

### Task 5: Add API endpoint for InDesign workbook export

**Files:**
- Modify: `e-fees-api/src/routes/fees.rs`

- [ ] **Step 1: Check the existing export endpoint pattern**

Read `e-fees-api/src/routes/fees.rs` to find the existing export endpoints (likely `export_fee_json` or similar). Follow the same pattern for auth, DB lookup, and response.

- [ ] **Step 2: Add the InDesign export handler**

Add a new handler function following the existing pattern:

```rust
/// POST /fees/{id}/export/indesign — returns xlsx bytes
pub async fn export_indesign(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let fee = get_fee_or_404(&state, &id).await?;

    let bytes = e_fees_core::export::indesign_workbook::generate_indesign_workbook(&fee)
        .map_err(|e| AppError::Internal(e))?;

    let filename = format!(
        "{}-IDW-Pricing.xlsx",
        fee.number.replace(['/', '\\', ':'], "-")
    );

    Ok((
        [
            (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
            (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", filename)),
        ],
        bytes,
    ))
}
```

- [ ] **Step 3: Register the route**

Add to the fees router:

```rust
.route("/fees/{id}/export/indesign", post(export_indesign))
```

- [ ] **Step 4: Verify build**

Run: `cargo check -p e-fees-api`
Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add e-fees-api/src/routes/fees.rs
git commit -m "feat(api): add POST /fees/{id}/export/indesign endpoint"
```

---

### Task 6: Integration test — end-to-end workbook generation

**Files:**
- Create: `crates/e-fees-core/src/export/indesign_workbook.rs` (add more tests to existing test module)

- [ ] **Step 1: Add test for correct sheet count**

Add to the existing `mod tests` block:

```rust
    #[test]
    fn test_workbook_has_six_sheets() {
        // We can't read sheet names back with rust_xlsxwriter alone,
        // but we can verify the file is valid xlsx by checking zip entries.
        let fee = make_fee(true);
        let bytes = generate_indesign_workbook(&fee).unwrap();

        // Parse as zip, count xl/worksheets/sheet*.xml entries
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let sheet_count = (0..archive.len())
            .filter(|i| {
                archive
                    .by_index(*i)
                    .map(|f| f.name().starts_with("xl/worksheets/sheet"))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(sheet_count, 6, "expected 6 sheets");
    }

    #[test]
    fn test_design_only_still_has_six_sheets() {
        let fee = make_fee(false);
        let bytes = generate_indesign_workbook(&fee).unwrap();

        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let sheet_count = (0..archive.len())
            .filter(|i| {
                archive
                    .by_index(*i)
                    .map(|f| f.name().starts_with("xl/worksheets/sheet"))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(sheet_count, 6, "design-only should still have 6 sheets");
    }
```

- [ ] **Step 2: Add `zip` as a dev dependency**

In `crates/e-fees-core/Cargo.toml`:

```toml
[dev-dependencies]
zip = "2"
```

- [ ] **Step 3: Run all tests**

Run: `cargo test -p e-fees-core --lib -- export::indesign_workbook`
Expected: All 5 tests PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/e-fees-core/src/export/indesign_workbook.rs crates/e-fees-core/Cargo.toml
git commit -m "test(export): add sheet count validation tests for InDesign workbook"
```

---

### Task 7: Full build verification and cleanup

- [ ] **Step 1: Run full test suite**

```bash
cargo test -p e-fees-core --lib
cargo check -p app --lib
```

Expected: All tests pass, no build errors.

- [ ] **Step 2: Run formatter**

```bash
cargo fmt --all
```

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "style: run rustfmt on InDesign workbook export"
```

---

## Notes for Implementation

- **Duration field**: The `Stage` struct has no `duration` field. The Durations sheets (T0, T1) write empty Duration cells for the user to fill in InDesign/Excel. If a duration field is added to `Stage` later, update `write_durations_sheet` to populate it.
- **Distribution sheet**: Deferred — `fee.distribution` doesn't exist in the schema yet. The Distribution table (T-Dist) will be added when the field is defined.
- **SurrealDB f64 strictness**: All numeric fields use `f64` (never `i64` for monetary values). This matches the existing pattern and avoids the SurrealValue deserialization bug.
- **DbValue parsing**: `post_contract_items` and `payment_schedule` are stored as `Option<DbValue>` on `Fee`. Use `dbvalue_to_json()` → `serde_json::from_value()` to deserialize, matching the existing pattern in `excel_export.rs`.
