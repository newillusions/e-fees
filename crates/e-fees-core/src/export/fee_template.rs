//! Internal pricing template (`-PRI`) export — populates a fee's pricing data
//! into the emittiv pricing-model xlsx, preserving the template's formulas and
//! formatting via `umya_spreadsheet`.
//!
//! Shared by the Tauri desktop app and the standalone API. The function reads a
//! source spreadsheet (the latest `-PRI`/`-FP` pricing file, or an embedded
//! blank template) and writes the populated workbook to `output_path`.

use std::path::Path;

use crate::models::fee::{Fee, PostContractItem, PricingBreakdown, Stage};

// ============================================================================
// TEMPLATE CONSTANTS
// ============================================================================

/// Embedded fallback pricing template (from project folder template).
const EMBEDDED_TEMPLATE: &[u8] = include_bytes!("../resources/pricing-template.xlsx");

/// Default number of design stage rows in the blank pricing template.
const TEMPLATE_STAGE_COUNT: u32 = 4;

/// First stage row in the template (1-indexed).
const FIRST_STAGE_ROW: u32 = 9;

/// Column index (1-based) → letter. Only covers A–Z (cols 1–26).
fn col_letter(col: u32) -> char {
    (b'A' + (col - 1) as u8) as char
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Generate the internal `-PRI` pricing workbook for a fee.
///
/// Opens `source_path` (an existing pricing xlsx) or the embedded blank
/// template, populates the fee's pricing config / stages / post-contract items
/// while preserving the template's formulas, and writes the result to
/// `output_path`. Returns the output path on success.
pub fn generate_fee_template(
    fee: &Fee,
    output_path: &Path,
    source_path: Option<&Path>,
) -> Result<String, String> {
    let pricing = fee
        .pricing_typed()
        .ok_or_else(|| "No pricing data available for template generation".to_string())?;

    // 1. Open source spreadsheet (project file or embedded fallback)
    let mut book = open_or_create_template(source_path)?;

    let sheet = book
        .get_sheet_mut(&0)
        .ok_or_else(|| "Failed to get worksheet: no sheets in workbook".to_string())?;

    // 2. Count design stages and adjust template rows
    let design_stages: Vec<&Stage> = pricing
        .stages
        .iter()
        .filter(|s| !s.is_post_contract)
        .collect();
    let stage_count = design_stages.len() as u32;

    adjust_stage_rows(sheet, stage_count);

    // 3. Populate header / config cells
    populate_config(sheet, &pricing, stage_count);

    // 4. Populate stage rows
    populate_stages(sheet, &pricing, &design_stages);

    // 5. Populate post-contract items
    if let Some(ref items) = fee.post_contract_items {
        populate_post_contract(sheet, items, stage_count);
    }

    // 6. Save
    umya_spreadsheet::writer::xlsx::write(&book, output_path)
        .map_err(|e| format!("Failed to save template file: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}

/// Open an existing pricing xlsx or fall back to the embedded blank template.
fn open_or_create_template(
    source_path: Option<&Path>,
) -> Result<umya_spreadsheet::Spreadsheet, String> {
    match source_path {
        Some(p) if p.exists() => umya_spreadsheet::reader::xlsx::read(p)
            .map_err(|e| format!("Failed to read pricing file: {}", e)),
        _ => {
            // Read the embedded template straight from memory — no temp file, so
            // concurrent exports can't race on a shared path.
            umya_spreadsheet::reader::xlsx::read_reader(
                std::io::Cursor::new(EMBEDDED_TEMPLATE),
                true,
            )
            .map_err(|e| format!("Failed to read embedded template: {}", e))
        }
    }
}

/// Adjust template stage rows to match the actual stage count.
///
/// The blank template ships with 4 stage rows (9-12). If the fee has more
/// design stages, new rows are inserted (formulas auto-adjust). If fewer,
/// the extra template rows are cleared but **not** removed - umya-spreadsheet's
/// `remove_row` can crash on certain formula patterns. Leaving them blank is
/// safe; the totals formula sums the populated range.
///
/// BUG FIX (2026-07-15, e-fees Pricing.xlsx export defect): `insert_new_row`
/// shifts single-cell formula references correctly, but does **not** widen
/// `SUM(range)` formulas whose original span ends exactly at the insertion
/// point (e.g. the template's "Sub" row sums `C9:C12`; inserting rows at 13
/// moves that formula down but leaves it summing only the original 4 rows,
/// silently excluding every stage past the 4th from Subtotal/VAT/Grand
/// Total/Remaining). Confirmed via direct inspection of generated workbook
/// XML for a 6-stage fee. Fix: after inserting, explicitly rewrite the
/// shifted Sub row's range formulas to span the full stage block.
fn adjust_stage_rows(sheet: &mut umya_spreadsheet::structs::Worksheet, stage_count: u32) {
    if stage_count > TEMPLATE_STAGE_COUNT {
        let extra = stage_count - TEMPLATE_STAGE_COUNT;
        // Insert after the last template stage row - pushes subtotal down
        let insert_at = FIRST_STAGE_ROW + TEMPLATE_STAGE_COUNT;
        sheet.insert_new_row(&insert_at, &extra);
        widen_subtotal_formulas(sheet, stage_count);
    } else if stage_count < TEMPLATE_STAGE_COUNT {
        // Clear unused template stage rows (don't remove - avoids formula crash)
        for row in (FIRST_STAGE_ROW + stage_count)..(FIRST_STAGE_ROW + TEMPLATE_STAGE_COUNT) {
            // Clear data columns: A, B, C-H, M, R
            for &col in &["A", "B", "C", "D", "E", "F", "G", "H", "M", "R"] {
                let addr = format!("{}{}", col, row);
                sheet.get_cell_value_mut(addr.as_str()).set_value("");
            }
        }
    }
}

/// Rewrite the "Sub" totals row's range formulas to span the full stage
/// block after `insert_new_row` has shifted that row down.
///
/// The template's Sub row (originally row `FIRST_STAGE_ROW + TEMPLATE_STAGE_COUNT`)
/// sums the 4 default stage rows via fixed ranges (`SUM(C9:C12)` etc). After
/// inserting `stage_count - TEMPLATE_STAGE_COUNT` rows, the Sub row itself
/// moves to `FIRST_STAGE_ROW + stage_count`, but umya-spreadsheet does not
/// widen the range text - it stays pinned to the original 4-row span. This
/// explicitly rewrites each affected formula to cover every stage row.
fn widen_subtotal_formulas(sheet: &mut umya_spreadsheet::structs::Worksheet, stage_count: u32) {
    let sub_row = FIRST_STAGE_ROW + stage_count;
    let last_stage_row = sub_row - 1;

    // Discipline amount columns C-H: SUM(C9:C{last_stage_row}) etc.
    for &col in &["C", "D", "E", "F", "G", "H"] {
        sheet
            .get_cell_value_mut(format!("{}{}", col, sub_row).as_str())
            .set_formula(format!(
                "SUM({}{}:{}{})",
                col, FIRST_STAGE_ROW, col, last_stage_row
            ));
    }

    // M: stage percentage sum (should total 100% across all stages).
    sheet
        .get_cell_value_mut(format!("M{}", sub_row).as_str())
        .set_formula(format!("SUM(M{}:M{})", FIRST_STAGE_ROW, last_stage_row));

    // P: net-fee sum. Template starts at P8 (Mobilisation row), not
    // FIRST_STAGE_ROW - preserve that lower bound.
    sheet
        .get_cell_value_mut(format!("P{}", sub_row).as_str())
        .set_formula(format!("SUM(P8:P{})", last_stage_row));

    // R: reimbursable costs per stage.
    sheet
        .get_cell_value_mut(format!("R{}", sub_row).as_str())
        .set_formula(format!("SUM(R{}:R{})", FIRST_STAGE_ROW, last_stage_row));
}

/// Write config / header cells (target fee, discipline names, percentages, etc.).
fn populate_config(
    sheet: &mut umya_spreadsheet::structs::Worksheet,
    pricing: &PricingBreakdown,
    stage_count: u32,
) {
    // B2: target fee
    sheet
        .get_cell_value_mut("B2")
        .set_value_number(pricing.config.target_fee);

    // C1:H1 — discipline names, C2:H2 — percentages (up to 6 columns)
    let disc_count = pricing.disciplines.len().min(6);
    for (i, disc) in pricing.disciplines.iter().enumerate().take(6) {
        let col = col_letter(3 + i as u32);
        sheet
            .get_cell_value_mut(format!("{}1", col).as_str())
            .set_value(disc.name.as_str());
        sheet
            .get_cell_value_mut(format!("{}2", col).as_str())
            .set_value_number(disc.percentage / 100.0);
    }
    // Clear unused discipline columns so template defaults don't leak through
    for i in disc_count..6 {
        let col = col_letter(3 + i as u32);
        sheet
            .get_cell_value_mut(format!("{}1", col).as_str())
            .set_value("");
        sheet
            .get_cell_value_mut(format!("{}2", col).as_str())
            .set_value_number(0.0);
    }

    // M6: first discipline name (used by HLOOKUP dropdown)
    if let Some(first) = pricing.disciplines.first() {
        sheet
            .get_cell_value_mut("M6")
            .set_value(first.name.as_str());
    }

    // Config block: P2 = stages, P3 = VAT%, P4 = mobilisation%
    sheet
        .get_cell_value_mut("P2")
        .set_value_number(stage_count as f64);
    sheet
        .get_cell_value_mut("P3")
        .set_value_number(pricing.config.vat_percent / 100.0);
    sheet
        .get_cell_value_mut("P4")
        .set_value_number(pricing.config.mobilisation_percent / 100.0);

    // L6: discipline target price (target fee × selected discipline %)
    // M6 holds the discipline name; HLOOKUP finds its percentage in row 2
    sheet
        .get_cell_value_mut("L6")
        .set_formula("B2*HLOOKUP(M6,C1:H2,2,FALSE)");
}

/// Write data + formulas for each design stage row.
fn populate_stages(
    sheet: &mut umya_spreadsheet::structs::Worksheet,
    pricing: &PricingBreakdown,
    design_stages: &[&Stage],
) {
    for (idx, stage) in design_stages.iter().enumerate() {
        let row = FIRST_STAGE_ROW + idx as u32;

        // A: stage number
        sheet
            .get_cell_value_mut(format!("A{}", row).as_str())
            .set_value_number((idx + 1) as f64);

        // B: stage code (e.g. "SD", "DD")
        sheet
            .get_cell_value_mut(format!("B{}", row).as_str())
            .set_value(stage.code.as_str());

        // C–H: amounts per discipline
        for (d, disc) in pricing.disciplines.iter().enumerate().take(6) {
            let amount = pricing
                .cells
                .iter()
                .find(|c| c.discipline_id == disc.id && c.stage_id == stage.id)
                .map(|c| c.override_amount.unwrap_or(c.amount))
                .unwrap_or(0.0);
            let addr = format!("{}{}", col_letter(3 + d as u32), row);
            sheet
                .get_cell_value_mut(addr.as_str())
                .set_value_number(amount);
        }

        // I: row total formula  (written for ALL rows including originals to cover inserted rows)
        sheet
            .get_cell_value_mut(format!("I{}", row).as_str())
            .set_formula(format!("SUM(C{}:H{})", row, row));

        // L: recommended fee formula
        sheet
            .get_cell_value_mut(format!("L{}", row).as_str())
            .set_formula(format!("$L$6*M{}", row));

        // M: stage percentage (as decimal)
        sheet
            .get_cell_value_mut(format!("M{}", row).as_str())
            .set_value_number(stage.percentage / 100.0);

        // P: net fee formula
        sheet
            .get_cell_value_mut(format!("P{}", row).as_str())
            .set_formula(format!("I{}-($P$8/$P$2)", row));

        // R: reimbursable costs for this stage
        let costs: f64 = pricing
            .costs
            .iter()
            .filter(|c| c.stage_id == stage.id)
            .map(|c| c.cost_to_client)
            .sum();
        if costs > 0.0 {
            sheet
                .get_cell_value_mut(format!("R{}", row).as_str())
                .set_value_number(costs);
        }
    }
}

/// Write post-contract item data (description, qty, rate) into the template.
///
/// The template has 5 fixed post-contract item rows. We populate up to 5
/// items; the template's column-E formulas (=C*D) compute the amounts.
fn populate_post_contract(
    sheet: &mut umya_spreadsheet::structs::Worksheet,
    items: &[PostContractItem],
    stage_count: u32,
) {
    // With the default 4 stages, PC items start at row 19.
    // Row adjustment shifts everything: item_start = 15 + stage_count.
    let item_start = 15 + stage_count;

    for (i, item) in items.iter().enumerate().take(5) {
        let row = item_start + i as u32;
        sheet
            .get_cell_value_mut(format!("B{}", row).as_str())
            .set_value(item.description.as_str());
        sheet
            .get_cell_value_mut(format!("C{}", row).as_str())
            .set_value_number(item.quantity);
        sheet
            .get_cell_value_mut(format!("D{}", row).as_str())
            .set_value_number(item.rate);
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::{json_to_dbvalue, TimeStamps};
    use crate::models::fee::{Fee, PricingBreakdown, PricingCell, PricingConfig, Stage};
    use surrealdb::types::RecordId;
    use surrealdb_types::Datetime;

    fn make_timestamps() -> TimeStamps {
        TimeStamps {
            created_at: Datetime::default(),
            updated_at: Datetime::default(),
        }
    }

    fn make_base_fee() -> Fee {
        Fee {
            id: None,
            name: "Test Fee".to_string(),
            number: "TF-001".to_string(),
            rev: 1,
            status: "Draft".to_string(),
            issue_date: "260315".to_string(),
            activity: "Lighting Design".to_string(),
            package: "Full".to_string(),
            project_id: RecordId::new("projects", "26_97101"),
            company_id: RecordId::new("company", "acme"),
            contact_id: RecordId::new("contacts", "john"),
            staff_name: "Alice".to_string(),
            staff_email: "a@e.com".to_string(),
            staff_phone: "+9715".to_string(),
            staff_position: "Designer".to_string(),
            strap_line: "Light".to_string(),
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

    fn make_fee_with_n_stages(n: usize) -> Fee {
        let stages: Vec<Stage> = (0..n)
            .map(|i| Stage {
                id: format!("s{}", i),
                name: format!("Stage {}", i),
                code: format!("S{}", i),
                percentage: 100.0 / n as f64,
                order: i as i64,
                is_post_contract: false,
            })
            .collect();
        let cells: Vec<PricingCell> = stages
            .iter()
            .map(|s| PricingCell {
                discipline_id: "ld".to_string(),
                stage_id: s.id.clone(),
                amount: 10000.0,
                override_amount: None,
            })
            .collect();
        let pricing = PricingBreakdown {
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
            design_phase_total: 0.0,
            post_contract_total: 0.0,
            costs_total: 0.0,
            subtotal: 0.0,
            vat_amount: 0.0,
            grand_total: 0.0,
        };
        let mut fee = make_base_fee();
        let json = serde_json::to_value(&pricing).unwrap();
        fee.pricing = Some(json_to_dbvalue(&json));
        fee
    }

    /// XLSX files are ZIP archives — PK zip magic 0x50 0x4B 0x03 0x04.
    fn is_valid_xlsx(bytes: &[u8]) -> bool {
        bytes.len() >= 4 && bytes[0] == 0x50 && bytes[1] == 0x4B
    }

    #[test]
    fn test_template_fewer_stages_than_default_embedded() {
        let fee = make_fee_with_n_stages(3); // fewer than TEMPLATE_STAGE_COUNT (4)
        let path = std::env::temp_dir().join("_efees_test_pri_3.xlsx");
        let result = generate_fee_template(&fee, &path, None);
        assert!(
            result.is_ok(),
            "3-stage template failed: {:?}",
            result.err()
        );
        let bytes = std::fs::read(&path).unwrap();
        assert!(is_valid_xlsx(&bytes), "output must be a valid xlsx");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_template_more_stages_than_default_embedded() {
        let fee = make_fee_with_n_stages(6); // more than TEMPLATE_STAGE_COUNT (4)
        let path = std::env::temp_dir().join("_efees_test_pri_6.xlsx");
        let result = generate_fee_template(&fee, &path, None);
        assert!(
            result.is_ok(),
            "6-stage template failed: {:?}",
            result.err()
        );
        let _ = std::fs::remove_file(&path);
    }

    // BUG regression (2026-07-15, Lulu Island dhow FP 26-97108): the Sub
    // totals row's SUM ranges must widen to cover every inserted stage row,
    // not stay pinned to the template's original 4-row span. Before the fix,
    // `insert_new_row` shifted the Sub row down but left `SUM(C9:C12)` etc
    // unexpanded, silently dropping stages 5+ from Subtotal/VAT/Grand Total.
    #[test]
    fn test_more_stages_widens_subtotal_formula_range() {
        let fee = make_fee_with_n_stages(6); // 2 more than TEMPLATE_STAGE_COUNT (4)
        let path = std::env::temp_dir().join("_efees_test_pri_subtotal_range.xlsx");
        let result = generate_fee_template(&fee, &path, None);
        assert!(
            result.is_ok(),
            "6-stage template failed: {:?}",
            result.err()
        );

        let book = umya_spreadsheet::reader::xlsx::read(&path).expect("re-read output xlsx");
        let sheet = book.get_sheet(&0).expect("sheet 0 exists");

        // Sub row = FIRST_STAGE_ROW (9) + stage_count (6) = 15.
        let sub_row = 15;
        for col in ["C", "D", "E", "F", "G", "H"] {
            let formula = sheet
                .get_cell_value(format!("{}{}", col, sub_row).as_str())
                .get_formula();
            assert_eq!(
                formula,
                format!("SUM({}9:{}14)", col, col),
                "column {} Sub-row formula must span all 6 stage rows (9-14), got: {}",
                col,
                formula
            );
        }
        assert_eq!(
            sheet
                .get_cell_value(format!("M{}", sub_row).as_str())
                .get_formula(),
            "SUM(M9:M14)",
            "stage-percentage Sub formula must span all 6 stage rows"
        );
        assert_eq!(
            sheet
                .get_cell_value(format!("P{}", sub_row).as_str())
                .get_formula(),
            "SUM(P8:P14)",
            "net-fee Sub formula must span P8 (mobilisation) through the last stage row"
        );
        assert_eq!(
            sheet
                .get_cell_value(format!("R{}", sub_row).as_str())
                .get_formula(),
            "SUM(R9:R14)",
            "reimbursable-cost Sub formula must span all 6 stage rows"
        );

        let _ = std::fs::remove_file(&path);
    }

    // BUG regression, fewer-stages side: with exactly the default stage
    // count (4), the Sub row must remain untouched (no insert triggered,
    // no widening needed) - guards against widen_subtotal_formulas firing
    // when it shouldn't.
    #[test]
    fn test_default_stage_count_leaves_subtotal_formula_untouched() {
        let fee = make_fee_with_n_stages(4);
        let path = std::env::temp_dir().join("_efees_test_pri_default_subtotal.xlsx");
        let result = generate_fee_template(&fee, &path, None);
        assert!(
            result.is_ok(),
            "4-stage template failed: {:?}",
            result.err()
        );

        let book = umya_spreadsheet::reader::xlsx::read(&path).expect("re-read output xlsx");
        let sheet = book.get_sheet(&0).expect("sheet 0 exists");
        assert_eq!(
            sheet.get_cell_value("C13").get_formula(),
            "SUM(C9:C12)",
            "default 4-stage Sub row must keep the template's original range"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_template_no_pricing_errors() {
        let fee = make_base_fee(); // pricing = None
        let path = std::env::temp_dir().join("_efees_test_pri_none.xlsx");
        let result = generate_fee_template(&fee, &path, None);
        assert!(result.is_err(), "expected error for fee with no pricing");
        let _ = std::fs::remove_file(&path);
    }
}
