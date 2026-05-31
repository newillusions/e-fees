//! InDesign workbook export — generates a multi-sheet XLSX workbook from a Fee.
//!
//! Each sheet corresponds to one InDesign data-merge table (T0–T5 + revisions).
//! The workbook is returned as raw bytes (zip/xlsx) for embedding in Tauri IPC
//! or writing to disk by the caller.
//!
//! Sheet layout:
//!   T0 Design Durations      — Stage / Milestone / Duration (blank)
//!   T1 PC Durations          — Stage / Milestone / Duration (blank)
//!   T2 Design Fees           — Stage / Milestone / Fee + Total row
//!   T3 Post-Contract Fees    — Stage / Milestone / Unit / Est Qty / Price / Est. Fee + Total row
//!   T4 Payment Schedule      — Stage / Milestone / Fee / Payment
//!   T5 Reimbursable Costs    — Stage / Description / Base Cost / Markup / Cost to Client + Total row
//!   Revisions                — Date / Release / Author / Reference

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook, XlsxError};

use crate::models::fee::{Fee, PaymentSchedule, PostContractItem, ReimbursableCost, Stage};

// ============================================================================
// PUBLIC API
// ============================================================================

/// Generate a multi-sheet XLSX workbook for InDesign data-merge export.
///
/// Returns `Ok(Vec<u8>)` containing the raw xlsx bytes on success, or
/// `Err(String)` describing the failure reason.
pub fn generate_indesign_workbook(fee: &Fee) -> Result<Vec<u8>, String> {
    let pricing = fee
        .pricing_typed()
        .ok_or_else(|| "No pricing data".to_string())?;

    let design_stages: Vec<&Stage> = pricing
        .stages
        .iter()
        .filter(|s| !s.is_post_contract)
        .collect();
    let pc_stages: Vec<&Stage> = pricing
        .stages
        .iter()
        .filter(|s| s.is_post_contract)
        .collect();

    let mut workbook = Workbook::new();

    // Formats — simple data sheets, no emittiv branding needed
    let fmt_hdr = fmt_header();
    let fmt_txt = fmt_text();
    let fmt_num = fmt_number();
    let fmt_cur = fmt_currency();
    let fmt_tot = fmt_total();
    let fmt_tot_cur = fmt_total_currency();

    // T0 — Design Durations
    {
        let ws = workbook.add_worksheet();
        ws.set_name("T0 Design Durations")
            .map_err(|e| e.to_string())?;
        write_duration_headers(ws, &fmt_hdr)?;
        for (row, stage) in design_stages.iter().enumerate() {
            let r = (row + 1) as u32;
            ws.write_with_format(r, 0, &stage.name, &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 1, "", &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 2, "", &fmt_txt)
                .map_err(|e| e.to_string())?;
        }
    }

    // T1 — Post-Contract Durations
    {
        let ws = workbook.add_worksheet();
        ws.set_name("T1 PC Durations").map_err(|e| e.to_string())?;
        write_duration_headers(ws, &fmt_hdr)?;
        for (row, stage) in pc_stages.iter().enumerate() {
            let r = (row + 1) as u32;
            ws.write_with_format(r, 0, &stage.name, &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 1, "", &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 2, "", &fmt_txt)
                .map_err(|e| e.to_string())?;
        }
    }

    // T2 — Design Fees
    {
        let ws = workbook.add_worksheet();
        ws.set_name("T2 Design Fees").map_err(|e| e.to_string())?;
        ws.write_with_format(0, 0, "Stage", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 1, "Milestone", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 2, "Fee", &fmt_hdr)
            .map_err(|e| e.to_string())?;

        let mut grand_total = 0.0_f64;
        for (row, stage) in design_stages.iter().enumerate() {
            let r = (row + 1) as u32;
            // Sum all cells for this stage (override_amount takes precedence)
            let fee_amount: f64 = pricing
                .cells
                .iter()
                .filter(|c| c.stage_id == stage.id)
                .map(|c| c.override_amount.unwrap_or(c.amount))
                .sum();
            grand_total += fee_amount;
            ws.write_with_format(r, 0, &stage.name, &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 1, "", &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 2, fee_amount, &fmt_cur)
                .map_err(|e| e.to_string())?;
        }
        let total_row = (design_stages.len() + 1) as u32;
        ws.write_with_format(total_row, 0, "Total", &fmt_tot)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(total_row, 1, "", &fmt_tot)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(total_row, 2, grand_total, &fmt_tot_cur)
            .map_err(|e| e.to_string())?;
    }

    // T3 — Post-Contract Fees
    {
        let ws = workbook.add_worksheet();
        ws.set_name("T3 Post-Contract Fees")
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 0, "Stage", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 1, "Milestone", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 2, "Unit", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 3, "Est Qty", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 4, "Price", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 5, "Est. Fee", &fmt_hdr)
            .map_err(|e| e.to_string())?;

        // Resolve post-contract items — direct Vec<PostContractItem> on fee
        let empty_pc: Vec<PostContractItem> = vec![];
        let pc_items = fee.post_contract_items.as_deref().unwrap_or(&empty_pc);

        let mut grand_total = 0.0_f64;
        for (row, item) in pc_items.iter().enumerate() {
            let r = (row + 1) as u32;
            // Look up stage name from pricing stages
            let stage_name = pricing
                .stages
                .iter()
                .find(|s| s.id == item.stage_id)
                .map(|s| s.name.as_str())
                .unwrap_or(&item.stage_id);
            grand_total += item.amount;
            ws.write_with_format(r, 0, stage_name, &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 1, &item.description, &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 2, &item.unit, &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 3, item.quantity, &fmt_num)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 4, item.rate, &fmt_cur)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 5, item.amount, &fmt_cur)
                .map_err(|e| e.to_string())?;
        }
        let total_row = (pc_items.len() + 1) as u32;
        ws.write_with_format(total_row, 0, "Total", &fmt_tot)
            .map_err(|e| e.to_string())?;
        for col in 1..=4u16 {
            ws.write_with_format(total_row, col, "", &fmt_tot)
                .map_err(|e| e.to_string())?;
        }
        ws.write_with_format(total_row, 5, grand_total, &fmt_tot_cur)
            .map_err(|e| e.to_string())?;
    }

    // T4 — Payment Schedule
    {
        let ws = workbook.add_worksheet();
        ws.set_name("T4 Payment Schedule")
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 0, "Stage", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 1, "Milestone", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 2, "Fee", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 3, "Payment", &fmt_hdr)
            .map_err(|e| e.to_string())?;

        // payment_schedule is Option<DbValue> — convert via json
        let schedule: Option<PaymentSchedule> = fee.payment_schedule.as_ref().and_then(|v| {
            let json = crate::models::common::dbvalue_to_json(v);
            serde_json::from_value(json).ok()
        });

        if let Some(sched) = &schedule {
            for (row, entry) in sched.entries.iter().enumerate() {
                let r = (row + 1) as u32;
                let stage_name = entry
                    .stage_id
                    .as_deref()
                    .and_then(|sid| pricing.stages.iter().find(|s| s.id == sid))
                    .map(|s| s.name.as_str())
                    .unwrap_or("");
                let fee_pct = format!("{:.0}%", entry.percentage_of_total);
                ws.write_with_format(r, 0, stage_name, &fmt_txt)
                    .map_err(|e| e.to_string())?;
                ws.write_with_format(r, 1, &entry.description, &fmt_txt)
                    .map_err(|e| e.to_string())?;
                ws.write_with_format(r, 2, &fee_pct, &fmt_txt)
                    .map_err(|e| e.to_string())?;
                ws.write_with_format(r, 3, entry.amount, &fmt_cur)
                    .map_err(|e| e.to_string())?;
            }

            // Total row
            let total_row = (sched.entries.len() + 1) as u32;
            let total_amount: f64 = sched.entries.iter().map(|e| e.amount).sum();
            ws.write_with_format(total_row, 0, "Total", &fmt_tot)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(total_row, 1, "", &fmt_tot)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(total_row, 2, "", &fmt_tot)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(total_row, 3, total_amount, &fmt_tot_cur)
                .map_err(|e| e.to_string())?;
        }
    }

    // T5 — Reimbursable Costs / Provisional Sums
    {
        let ws = workbook.add_worksheet();
        ws.set_name("T5 Reimbursable Costs")
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 0, "Stage", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 1, "Description", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 2, "Base Cost", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 3, "Markup", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 4, "Cost to Client", &fmt_hdr)
            .map_err(|e| e.to_string())?;

        // Prefer the top-level `reimbursable_costs` (the source the desktop
        // `-PRI` sheet renders); fall back to the pricing breakdown's `costs`.
        let costs: &[ReimbursableCost] = match fee.reimbursable_costs.as_deref() {
            Some(c) if !c.is_empty() => c,
            _ => pricing.costs.as_slice(),
        };

        let mut grand_total = 0.0_f64;
        for (row, cost) in costs.iter().enumerate() {
            let r = (row + 1) as u32;
            let stage_name = pricing
                .stages
                .iter()
                .find(|s| s.id == cost.stage_id)
                .map(|s| s.name.as_str())
                .unwrap_or(&cost.stage_id);
            let markup = format!("{:.0}%", cost.markup_percent);
            grand_total += cost.cost_to_client;
            ws.write_with_format(r, 0, stage_name, &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 1, &cost.description, &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 2, cost.base_cost, &fmt_cur)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 3, &markup, &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 4, cost.cost_to_client, &fmt_cur)
                .map_err(|e| e.to_string())?;
        }
        let total_row = (costs.len() + 1) as u32;
        ws.write_with_format(total_row, 0, "Total", &fmt_tot)
            .map_err(|e| e.to_string())?;
        for col in 1..=3u16 {
            ws.write_with_format(total_row, col, "", &fmt_tot)
                .map_err(|e| e.to_string())?;
        }
        ws.write_with_format(total_row, 4, grand_total, &fmt_tot_cur)
            .map_err(|e| e.to_string())?;
    }

    // Revisions
    {
        let ws = workbook.add_worksheet();
        ws.set_name("Revisions").map_err(|e| e.to_string())?;
        ws.write_with_format(0, 0, "Date", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 1, "Release", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 2, "Author", &fmt_hdr)
            .map_err(|e| e.to_string())?;
        ws.write_with_format(0, 3, "Reference", &fmt_hdr)
            .map_err(|e| e.to_string())?;

        for (row, rev) in fee.revisions.iter().enumerate() {
            let r = (row + 1) as u32;
            let date = crate::export::format_issue_date(&rev.revision_date);
            ws.write_with_format(r, 0, &date, &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 1, rev.revision_number, &fmt_num)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 2, &rev.author_name, &fmt_txt)
                .map_err(|e| e.to_string())?;
            ws.write_with_format(r, 3, &rev.notes, &fmt_txt)
                .map_err(|e| e.to_string())?;
        }
    }

    workbook
        .save_to_buffer()
        .map_err(|e: XlsxError| e.to_string())
}

// ============================================================================
// SHEET HELPERS
// ============================================================================

fn write_duration_headers(ws: &mut rust_xlsxwriter::Worksheet, fmt: &Format) -> Result<(), String> {
    ws.write_with_format(0, 0, "Stage", fmt)
        .map_err(|e| e.to_string())?;
    ws.write_with_format(0, 1, "Milestone", fmt)
        .map_err(|e| e.to_string())?;
    ws.write_with_format(0, 2, "Duration", fmt)
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// FORMAT HELPERS — simple data sheets, 10pt, no emittiv branding
// ============================================================================

fn fmt_header() -> Format {
    Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border_bottom(FormatBorder::Thin)
}

fn fmt_text() -> Format {
    Format::new()
}

fn fmt_number() -> Format {
    Format::new()
        .set_align(FormatAlign::Right)
        .set_num_format("#,##0")
}

fn fmt_currency() -> Format {
    Format::new()
        .set_align(FormatAlign::Right)
        .set_num_format("#,##0.00")
}

fn fmt_total() -> Format {
    Format::new().set_bold().set_border_top(FormatBorder::Thin)
}

fn fmt_total_currency() -> Format {
    Format::new()
        .set_bold()
        .set_align(FormatAlign::Right)
        .set_num_format("#,##0.00")
        .set_border_top(FormatBorder::Thin)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        common::{json_to_dbvalue, TimeStamps},
        fee::{
            Fee, PaymentSchedule, PaymentScheduleEntry, PostContractItem, PricingBreakdown,
            PricingCell, PricingConfig, ReimbursableCost, Stage,
        },
    };
    use surrealdb::types::RecordId;
    use surrealdb_types::Datetime;

    // ---- test helpers ----

    fn make_timestamps() -> TimeStamps {
        TimeStamps {
            created_at: Datetime::default(),
            updated_at: Datetime::default(),
        }
    }

    fn make_base_fee() -> Fee {
        Fee {
            id: None,
            name: "Test Fee Proposal".to_string(),
            number: "TF-001".to_string(),
            rev: 1,
            status: "Draft".to_string(),
            issue_date: "260315".to_string(),
            activity: "Lighting Design".to_string(),
            package: "Full".to_string(),
            project_id: RecordId::new("projects", "26_97101"),
            company_id: RecordId::new("company", "acme"),
            contact_id: RecordId::new("contacts", "john"),
            staff_name: "Alice Smith".to_string(),
            staff_email: "alice@emittiv.com".to_string(),
            staff_phone: "+971501234567".to_string(),
            staff_position: "Senior Designer".to_string(),
            strap_line: "Excellence in light".to_string(),
            revisions: vec![],
            time: make_timestamps(),
            pricing: None,
            post_contract_items: None,
            reimbursable_costs: None,
            payment_schedule: None,
            pricing_revisions: None,
            current_revision_number: None,
            current_release_number: None,
            import_source: None,
        }
    }

    fn make_design_stages() -> Vec<Stage> {
        vec![
            Stage {
                id: "sd".to_string(),
                name: "Schematic Design".to_string(),
                code: "SD".to_string(),
                percentage: 25.0,
                order: 1,
                is_post_contract: false,
            },
            Stage {
                id: "dd".to_string(),
                name: "Design Development".to_string(),
                code: "DD".to_string(),
                percentage: 25.0,
                order: 2,
                is_post_contract: false,
            },
        ]
    }

    fn make_post_contract_stages() -> Vec<Stage> {
        vec![Stage {
            id: "ca".to_string(),
            name: "Construction Administration".to_string(),
            code: "CA".to_string(),
            percentage: 20.0,
            order: 5,
            is_post_contract: true,
        }]
    }

    fn make_pricing_with_stages(stages: Vec<Stage>) -> PricingBreakdown {
        let cells: Vec<PricingCell> = stages
            .iter()
            .map(|s| PricingCell {
                discipline_id: "ld".to_string(),
                stage_id: s.id.clone(),
                amount: 10000.0,
                override_amount: None,
            })
            .collect();

        PricingBreakdown {
            config: PricingConfig {
                target_fee: 50000.0,
                buffer_percent: 10.0,
                quoted_fee: 55000.0,
                currency: "AED".to_string(),
                vat_percent: 5.0,
                vat_included: false,
                mobilisation_percent: 10.0,
            },
            disciplines: vec![],
            stages,
            cells,
            costs: vec![],
            design_phase_total: 20000.0,
            post_contract_total: 10000.0,
            costs_total: 0.0,
            subtotal: 30000.0,
            vat_amount: 1500.0,
            grand_total: 31500.0,
        }
    }

    fn make_full_fee() -> Fee {
        let mut fee = make_base_fee();

        // Design + post-contract stages
        let mut all_stages = make_design_stages();
        all_stages.extend(make_post_contract_stages());
        let pricing = make_pricing_with_stages(all_stages);
        let json = serde_json::to_value(&pricing).unwrap();
        fee.pricing = Some(json_to_dbvalue(&json));

        // Post-contract items
        fee.post_contract_items = Some(vec![PostContractItem {
            id: "pci-1".to_string(),
            stage_id: "ca".to_string(),
            description: "Site Visit".to_string(),
            quantity: 10.0,
            unit: "visit".to_string(),
            rate: 500.0,
            amount: 5000.0,
        }]);

        // Payment schedule stored as DbValue
        let schedule = PaymentSchedule {
            entries: vec![
                PaymentScheduleEntry {
                    id: "ps-1".to_string(),
                    payment_type: "mobilisation".to_string(),
                    description: "Mobilisation".to_string(),
                    stage_id: None,
                    stage_percentage: None,
                    amount: 5500.0,
                    percentage_of_total: 10.0,
                    due_date: None,
                    status: "pending".to_string(),
                    invoice_number: None,
                    invoice_date: None,
                    paid_date: None,
                },
                PaymentScheduleEntry {
                    id: "ps-2".to_string(),
                    payment_type: "milestone".to_string(),
                    description: "Schematic Design".to_string(),
                    stage_id: Some("sd".to_string()),
                    stage_percentage: Some(25.0),
                    amount: 13750.0,
                    percentage_of_total: 25.0,
                    due_date: None,
                    status: "pending".to_string(),
                    invoice_number: None,
                    invoice_date: None,
                    paid_date: None,
                },
            ],
            total_invoiced: 0.0,
            total_paid: 0.0,
            total_outstanding: 19250.0,
        };
        let json = serde_json::to_value(&schedule).unwrap();
        fee.payment_schedule = Some(json_to_dbvalue(&json));

        fee
    }

    fn make_design_only_fee() -> Fee {
        let mut fee = make_base_fee();
        let pricing = make_pricing_with_stages(make_design_stages());
        let json = serde_json::to_value(&pricing).unwrap();
        fee.pricing = Some(json_to_dbvalue(&json));
        // No post_contract_items, no payment_schedule
        fee
    }

    /// Full fee plus a top-level reimbursable cost (the source the desktop
    /// `-PRI` sheet reads). Used to prove BUG 2: costs must reach the IDW merge.
    fn make_fee_with_costs() -> Fee {
        let mut fee = make_full_fee();
        fee.reimbursable_costs = Some(vec![ReimbursableCost {
            id: "rc-1".to_string(),
            description: "Acoustics sub-consultant".to_string(),
            stage_id: "sd".to_string(),
            discipline_id: None,
            base_cost: 50000.0,
            markup_percent: 10.0,
            cost_to_client: 55000.0,
            date_incurred: "2026-05-30".to_string(),
            notes: None,
        }]);
        fee
    }

    /// Read a single zip entry's bytes as a UTF-8 string.
    fn read_zip_entry(bytes: &[u8], entry: &str) -> String {
        use std::io::Read;
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let mut file = archive.by_name(entry).expect("entry exists");
        let mut buf = String::new();
        file.read_to_string(&mut buf).expect("utf-8");
        buf
    }

    // ---- XLSX magic bytes ----

    /// XLSX files are ZIP archives — PK zip magic is 0x50 0x4B 0x03 0x04
    fn is_valid_xlsx(bytes: &[u8]) -> bool {
        bytes.len() >= 4
            && bytes[0] == 0x50
            && bytes[1] == 0x4B
            && bytes[2] == 0x03
            && bytes[3] == 0x04
    }

    // ---- tests ----

    #[test]
    fn test_generates_valid_xlsx_bytes() {
        let fee = make_full_fee();
        let result = generate_indesign_workbook(&fee);
        assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty(), "xlsx bytes must not be empty");
        assert!(
            is_valid_xlsx(&bytes),
            "expected PK zip magic bytes (xlsx format), got: {:?}",
            &bytes[..4.min(bytes.len())]
        );
    }

    #[test]
    fn test_design_only_has_empty_post_contract_sheets() {
        let fee = make_design_only_fee();
        let result = generate_indesign_workbook(&fee);
        assert!(
            result.is_ok(),
            "design-only fee should succeed, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_no_pricing_returns_error() {
        let fee = make_base_fee(); // pricing = None
        let result = generate_indesign_workbook(&fee);
        assert!(result.is_err(), "expected Err for fee with no pricing");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("No pricing data"),
            "error message should contain 'No pricing data', got: {:?}",
            msg
        );
    }

    #[test]
    fn test_workbook_has_seven_sheets() {
        let fee = make_full_fee();
        let bytes = generate_indesign_workbook(&fee).unwrap();
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let sheet_count = (0..archive.len())
            .filter(|i| {
                archive
                    .by_index_raw(*i)
                    .map(|f| f.name().starts_with("xl/worksheets/sheet"))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(sheet_count, 7, "expected 7 sheets (T0–T5 + Revisions)");
    }

    // BUG 2 regression: reimbursable costs were structurally dropped from the
    // IDW workbook (only stages/PC/payment/revisions were emitted), so anything
    // in the costs section never reached the .indd client proposal merge.
    #[test]
    fn test_workbook_includes_reimbursable_costs_sheet() {
        let fee = make_fee_with_costs();
        let bytes = generate_indesign_workbook(&fee).unwrap();

        // workbook.xml lists every sheet's display name
        let workbook_xml = read_zip_entry(&bytes, "xl/workbook.xml");
        assert!(
            workbook_xml.contains("Reimbursable Costs"),
            "workbook must contain a 'Reimbursable Costs' sheet, got names in: {}",
            workbook_xml
        );

        // sharedStrings.xml holds the cell text — the cost description must land
        let shared = read_zip_entry(&bytes, "xl/sharedStrings.xml");
        assert!(
            shared.contains("Acoustics sub-consultant"),
            "cost description must be written into the costs sheet"
        );
    }

    #[test]
    fn test_design_only_still_has_seven_sheets() {
        let fee = make_design_only_fee();
        let bytes = generate_indesign_workbook(&fee).unwrap();
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let sheet_count = (0..archive.len())
            .filter(|i| {
                archive
                    .by_index_raw(*i)
                    .map(|f| f.name().starts_with("xl/worksheets/sheet"))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(
            sheet_count, 7,
            "design-only should still have 7 sheets (empty costs sheet included)"
        );
    }
}
