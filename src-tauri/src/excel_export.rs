//! Excel export module for generating .xlsx files from fee proposal data.
//!
//! Uses `rust_xlsxwriter` (pure Rust, no C dependencies) to produce
//! professional, formatted spreadsheets suitable for client-facing output.

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook, Worksheet, Color};
use std::path::Path;

use crate::db::types::{Fee, PricingBreakdown, Discipline, Stage, PricingCell, PostContractItem, ReimbursableCost};

// ============================================================================
// EMITTIV BRAND COLOURS
// ============================================================================

const BLACK: Color = Color::RGB(0x000000);
const DARKER: Color = Color::RGB(0x333333);
const DARK: Color = Color::RGB(0x666666);
const LIGHT: Color = Color::RGB(0x999999);
const WHITE: Color = Color::RGB(0xFFFFFF);
const SPLASH: Color = Color::RGB(0xFF9900);

// ============================================================================
// FORMAT HELPERS
// ============================================================================

fn fmt_title() -> Format {
    Format::new()
        .set_font_size(16)
        .set_bold()
        .set_font_color(WHITE)
        .set_background_color(BLACK)
        .set_font_name("Ubuntu")
}

fn fmt_section_header() -> Format {
    Format::new()
        .set_font_size(12)
        .set_bold()
        .set_font_color(WHITE)
        .set_background_color(DARKER)
        .set_font_name("Ubuntu")
}

fn fmt_col_header() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(WHITE)
        .set_background_color(DARK)
        .set_font_name("Montserrat")
        .set_font_size(10)
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
}

fn fmt_label() -> Format {
    Format::new()
        .set_bold()
        .set_font_name("Montserrat")
        .set_font_size(10)
        .set_font_color(DARKER)
}

fn fmt_value() -> Format {
    Format::new()
        .set_font_name("Montserrat")
        .set_font_size(10)
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
        .set_num_format("0.0%")
        .set_align(FormatAlign::Right)
}

fn fmt_total_label() -> Format {
    Format::new()
        .set_bold()
        .set_font_name("Ubuntu")
        .set_font_size(11)
        .set_font_color(SPLASH)
        .set_border_bottom(FormatBorder::Double)
}

fn fmt_total_currency() -> Format {
    Format::new()
        .set_bold()
        .set_font_name("Ubuntu")
        .set_font_size(11)
        .set_font_color(SPLASH)
        .set_num_format("#,##0.00")
        .set_align(FormatAlign::Right)
        .set_border_bottom(FormatBorder::Double)
}

fn fmt_cell_data() -> Format {
    Format::new()
        .set_font_name("Montserrat")
        .set_font_size(10)
        .set_num_format("#,##0.00")
        .set_align(FormatAlign::Right)
        .set_border(FormatBorder::Thin)
        .set_border_color(LIGHT)
}

fn fmt_cell_header() -> Format {
    Format::new()
        .set_font_name("Montserrat")
        .set_font_size(10)
        .set_bold()
        .set_border(FormatBorder::Thin)
        .set_border_color(LIGHT)
}

// ============================================================================
// TEMPLATE POPULATION (umya-spreadsheet)
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
// MAIN EXPORT FUNCTION
// ============================================================================

/// Generate a formatted Excel file from a Fee record.
///
/// Returns the canonical path to the written file.
pub fn generate_fee_excel(fee: &Fee, output_path: &Path) -> Result<String, String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.set_name("Fee Proposal").map_err(|e| e.to_string())?;

    // Column widths
    worksheet.set_column_width(0, 22).map_err(|e| e.to_string())?;
    worksheet.set_column_width(1, 30).map_err(|e| e.to_string())?;
    worksheet.set_column_width(2, 18).map_err(|e| e.to_string())?;
    worksheet.set_column_width(3, 18).map_err(|e| e.to_string())?;
    worksheet.set_column_width(4, 18).map_err(|e| e.to_string())?;

    let mut row: u32 = 0;

    // ------------------------------------------------------------------
    // HEADER SECTION
    // ------------------------------------------------------------------
    worksheet.merge_range(row, 0, row, 4, &fee.name, &fmt_title()).map_err(|e| e.to_string())?;
    row += 1;

    let header_fields: Vec<(&str, String)> = vec![
        ("Fee Number", fee.number.clone()),
        ("Revision", format!("{}", fee.rev)),
        ("Status", fee.status.clone()),
        ("Issue Date", fee.issue_date.clone()),
        ("Activity", fee.activity.clone()),
        ("Package", fee.package.clone()),
        ("Staff Contact", fee.staff_name.clone()),
        ("Staff Email", fee.staff_email.clone()),
    ];

    for (label, value) in &header_fields {
        worksheet.write_string_with_format(row, 0, *label, &fmt_label()).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(row, 1, value, &fmt_value()).map_err(|e| e.to_string())?;
        row += 1;
    }

    row += 1; // blank row

    // ------------------------------------------------------------------
    // PRICING SECTION (if pricing data exists)
    // ------------------------------------------------------------------
    let pricing_typed = fee.pricing_typed();
    if let Some(ref pricing) = pricing_typed {
        row = write_pricing_section(worksheet, row, pricing)?;
    } else {
        worksheet.merge_range(row, 0, row, 4, "No pricing data available", &fmt_section_header()).map_err(|e| e.to_string())?;
        row += 2;
    }

    // ------------------------------------------------------------------
    // POST-CONTRACT ITEMS
    // ------------------------------------------------------------------
    if let Some(ref items) = fee.post_contract_items {
        if !items.is_empty() {
            row = write_post_contract_section(worksheet, row, items)?;
        }
    }

    // ------------------------------------------------------------------
    // REIMBURSABLE COSTS
    // ------------------------------------------------------------------
    if let Some(ref costs) = fee.reimbursable_costs {
        if !costs.is_empty() {
            row = write_reimbursable_costs_section(worksheet, row, costs)?;
        }
    }

    // ------------------------------------------------------------------
    // SUMMARY
    // ------------------------------------------------------------------
    if let Some(ref pricing) = pricing_typed {
        row = write_summary_section(worksheet, row, pricing)?;
    }

    // Suppress unused variable warning — row is the final position
    let _ = row;

    workbook.save(output_path).map_err(|e| format!("Failed to save Excel file: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}

/// Generate a pricing template by populating an existing xlsx file.
///
/// Opens `source_path` (the project's existing pricing spreadsheet) or falls
/// back to the embedded blank template, populates it with data from the Fee
/// record, and saves to `output_path`.
///
/// Returns the canonical path to the written file.
pub fn generate_fee_template(
    fee: &Fee,
    output_path: &Path,
    source_path: Option<&Path>,
) -> Result<String, String> {
    let pricing = fee.pricing_typed()
        .ok_or_else(|| "No pricing data available for template generation".to_string())?;

    // 1. Open source spreadsheet (project file or embedded fallback)
    let mut book = open_or_create_template(source_path)?;

    let sheet = book.get_sheet_mut(&0)
        .ok_or_else(|| "Failed to get worksheet: no sheets in workbook".to_string())?;

    // 2. Count design stages and adjust template rows
    let design_stages: Vec<&Stage> = pricing.stages.iter()
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
        Some(p) if p.exists() => {
            umya_spreadsheet::reader::xlsx::read(p)
                .map_err(|e| format!("Failed to read pricing file: {}", e))
        }
        _ => {
            // Write embedded template to a temp file, read it, then clean up
            let tmp = std::env::temp_dir().join("_efees_embedded_template.xlsx");
            std::fs::write(&tmp, EMBEDDED_TEMPLATE)
                .map_err(|e| format!("Failed to write embedded template: {}", e))?;
            let book = umya_spreadsheet::reader::xlsx::read(&tmp)
                .map_err(|e| format!("Failed to read embedded template: {}", e))?;
            let _ = std::fs::remove_file(&tmp);
            Ok(book)
        }
    }
}

/// Adjust template stage rows to match the actual stage count.
///
/// The blank template ships with 4 stage rows (9–12). If the fee has more
/// design stages, new rows are inserted (formulas auto-adjust). If fewer,
/// the extra template rows are cleared but **not** removed — umya-spreadsheet's
/// `remove_row` can crash on certain formula patterns. Leaving them blank is
/// harmless because the subtotal SUM includes them (empty → 0).
fn adjust_stage_rows(
    sheet: &mut umya_spreadsheet::structs::Worksheet,
    stage_count: u32,
) {
    if stage_count > TEMPLATE_STAGE_COUNT {
        let extra = stage_count - TEMPLATE_STAGE_COUNT;
        // Insert after the last template stage row → pushes subtotal down
        let insert_at = FIRST_STAGE_ROW + TEMPLATE_STAGE_COUNT;
        sheet.insert_new_row(&insert_at, &extra);
    } else if stage_count < TEMPLATE_STAGE_COUNT {
        // Clear unused template stage rows (don't remove — avoids formula crash)
        for row in (FIRST_STAGE_ROW + stage_count)..(FIRST_STAGE_ROW + TEMPLATE_STAGE_COUNT) {
            // Clear data columns: A, B, C–H, M, R
            for &col_letter in &["A", "B", "C", "D", "E", "F", "G", "H", "M", "R"] {
                let addr = format!("{}{}", col_letter, row);
                sheet.get_cell_value_mut(addr.as_str()).set_value("");
            }
        }
    }
}

/// Write config / header cells (target fee, discipline names, percentages, etc.).
fn populate_config(
    sheet: &mut umya_spreadsheet::structs::Worksheet,
    pricing: &PricingBreakdown,
    stage_count: u32,
) {
    // B2: target fee
    sheet.get_cell_value_mut("B2").set_value_number(pricing.config.target_fee);

    // C1:H1 — discipline names, C2:H2 — percentages (up to 6 columns)
    let disc_count = pricing.disciplines.len().min(6);
    for (i, disc) in pricing.disciplines.iter().enumerate().take(6) {
        let col = col_letter(3 + i as u32);
        sheet.get_cell_value_mut(format!("{}1", col).as_str()).set_value(disc.name.as_str());
        sheet.get_cell_value_mut(format!("{}2", col).as_str()).set_value_number(disc.percentage / 100.0);
    }
    // Clear unused discipline columns so template defaults don't leak through
    for i in disc_count..6 {
        let col = col_letter(3 + i as u32);
        sheet.get_cell_value_mut(format!("{}1", col).as_str()).set_value("");
        sheet.get_cell_value_mut(format!("{}2", col).as_str()).set_value_number(0.0);
    }

    // M6: first discipline name (used by HLOOKUP dropdown)
    if let Some(first) = pricing.disciplines.first() {
        sheet.get_cell_value_mut("M6").set_value(first.name.as_str());
    }

    // Config block: P2 = stages, P3 = VAT%, P4 = mobilisation%
    sheet.get_cell_value_mut("P2").set_value_number(stage_count as f64);
    sheet.get_cell_value_mut("P3").set_value_number(pricing.config.vat_percent / 100.0);
    sheet.get_cell_value_mut("P4").set_value_number(pricing.config.mobilisation_percent / 100.0);

    // L6: discipline target price (target fee × selected discipline %)
    // M6 holds the discipline name; HLOOKUP finds its percentage in row 2
    sheet.get_cell_value_mut("L6")
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
        sheet.get_cell_value_mut(format!("A{}", row).as_str())
            .set_value_number((idx + 1) as f64);

        // B: stage code (e.g. "SD", "DD")
        sheet.get_cell_value_mut(format!("B{}", row).as_str())
            .set_value(stage.code.as_str());

        // C–H: amounts per discipline
        for (d, disc) in pricing.disciplines.iter().enumerate().take(6) {
            let amount = pricing.cells.iter()
                .find(|c| c.discipline_id == disc.id && c.stage_id == stage.id)
                .map(|c| c.override_amount.unwrap_or(c.amount))
                .unwrap_or(0.0);
            let addr = format!("{}{}", col_letter(3 + d as u32), row);
            sheet.get_cell_value_mut(addr.as_str()).set_value_number(amount);
        }

        // I: row total formula  (written for ALL rows including originals to cover inserted rows)
        sheet.get_cell_value_mut(format!("I{}", row).as_str())
            .set_formula(format!("SUM(C{}:H{})", row, row));

        // L: recommended fee formula
        sheet.get_cell_value_mut(format!("L{}", row).as_str())
            .set_formula(format!("$L$6*M{}", row));

        // M: stage percentage (as decimal)
        sheet.get_cell_value_mut(format!("M{}", row).as_str())
            .set_value_number(stage.percentage / 100.0);

        // P: net fee formula
        sheet.get_cell_value_mut(format!("P{}", row).as_str())
            .set_formula(format!("I{}-($P$8/$P$2)", row));

        // R: reimbursable costs for this stage
        let costs: f64 = pricing.costs.iter()
            .filter(|c| c.stage_id == stage.id)
            .map(|c| c.cost_to_client)
            .sum();
        if costs > 0.0 {
            sheet.get_cell_value_mut(format!("R{}", row).as_str())
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
        sheet.get_cell_value_mut(format!("B{}", row).as_str())
            .set_value(item.description.as_str());
        sheet.get_cell_value_mut(format!("C{}", row).as_str())
            .set_value_number(item.quantity);
        sheet.get_cell_value_mut(format!("D{}", row).as_str())
            .set_value_number(item.rate);
    }
}

// ============================================================================
// SECTION WRITERS
// ============================================================================

fn write_pricing_section(ws: &mut Worksheet, start_row: u32, pricing: &PricingBreakdown) -> Result<u32, String> {
    let mut row = start_row;

    // Section title
    ws.merge_range(row, 0, row, 4, "Fee Breakdown", &fmt_section_header()).map_err(|e| e.to_string())?;
    row += 1;

    // Config summary
    let config_fields: Vec<(&str, String)> = vec![
        ("Currency", pricing.config.currency.clone()),
        ("Quoted Fee", format!("{:.2}", pricing.config.quoted_fee)),
        ("Target Fee", format!("{:.2}", pricing.config.target_fee)),
        ("Buffer", format!("{:.1}%", pricing.config.buffer_percent)),
    ];

    for (label, value) in &config_fields {
        ws.write_string_with_format(row, 0, *label, &fmt_label()).map_err(|e| e.to_string())?;
        ws.write_string_with_format(row, 1, value, &fmt_value()).map_err(|e| e.to_string())?;
        row += 1;
    }
    row += 1;

    // Discipline × Stage matrix
    if !pricing.disciplines.is_empty() && !pricing.stages.is_empty() {
        row = write_discipline_stage_matrix(ws, row, &pricing.disciplines, &pricing.stages, &pricing.cells)?;
    }

    Ok(row)
}

fn write_discipline_stage_matrix(
    ws: &mut Worksheet,
    start_row: u32,
    disciplines: &[Discipline],
    stages: &[Stage],
    cells: &[PricingCell],
) -> Result<u32, String> {
    let mut row = start_row;

    // Filter to design stages only (is_post_contract == false)
    let design_stages: Vec<&Stage> = stages.iter().filter(|s| !s.is_post_contract).collect();

    // Column headers: blank | stage1 | stage2 | ... | Total
    ws.write_string_with_format(row, 0, "Discipline", &fmt_col_header()).map_err(|e| e.to_string())?;
    for (i, stage) in design_stages.iter().enumerate() {
        ws.write_string_with_format(row, (i + 1) as u16, &stage.code, &fmt_col_header()).map_err(|e| e.to_string())?;
    }
    ws.write_string_with_format(row, (design_stages.len() + 1) as u16, "Total", &fmt_col_header()).map_err(|e| e.to_string())?;
    row += 1;

    // Data rows per discipline
    for disc in disciplines {
        ws.write_string_with_format(row, 0, &disc.name, &fmt_cell_header()).map_err(|e| e.to_string())?;
        let mut disc_total = 0.0_f64;

        for (j, stage) in design_stages.iter().enumerate() {
            let amount = cells.iter()
                .find(|c| c.discipline_id == disc.id && c.stage_id == stage.id)
                .map(|c| c.override_amount.unwrap_or(c.amount))
                .unwrap_or(0.0);
            disc_total += amount;
            ws.write_number_with_format(row, (j + 1) as u16, amount, &fmt_cell_data()).map_err(|e| e.to_string())?;
        }
        ws.write_number_with_format(row, (design_stages.len() + 1) as u16, disc_total, &fmt_cell_data()).map_err(|e| e.to_string())?;
        row += 1;
    }

    // Stage totals row
    ws.write_string_with_format(row, 0, "Stage Total", &fmt_total_label()).map_err(|e| e.to_string())?;
    for (j, stage) in design_stages.iter().enumerate() {
        let stage_total: f64 = cells.iter()
            .filter(|c| c.stage_id == stage.id)
            .map(|c| c.override_amount.unwrap_or(c.amount))
            .sum();
        ws.write_number_with_format(row, (j + 1) as u16, stage_total, &fmt_total_currency()).map_err(|e| e.to_string())?;
    }
    // Grand total of all design cells
    let all_design_total: f64 = cells.iter()
        .filter(|c| design_stages.iter().any(|s| s.id == c.stage_id))
        .map(|c| c.override_amount.unwrap_or(c.amount))
        .sum();
    ws.write_number_with_format(row, (design_stages.len() + 1) as u16, all_design_total, &fmt_total_currency()).map_err(|e| e.to_string())?;
    row += 2;

    Ok(row)
}

fn write_post_contract_section(ws: &mut Worksheet, start_row: u32, items: &[PostContractItem]) -> Result<u32, String> {
    let mut row = start_row;

    ws.merge_range(row, 0, row, 4, "Post-Contract Items", &fmt_section_header()).map_err(|e| e.to_string())?;
    row += 1;

    // Column headers
    let headers = ["Description", "Qty", "Unit", "Rate", "Amount"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_string_with_format(row, i as u16, *h, &fmt_col_header()).map_err(|e| e.to_string())?;
    }
    row += 1;

    let mut total = 0.0_f64;
    for item in items {
        ws.write_string_with_format(row, 0, &item.description, &fmt_value()).map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 1, item.quantity, &fmt_value()).map_err(|e| e.to_string())?;
        ws.write_string_with_format(row, 2, &item.unit, &fmt_value()).map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 3, item.rate, &fmt_currency()).map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 4, item.amount, &fmt_currency()).map_err(|e| e.to_string())?;
        total += item.amount;
        row += 1;
    }

    ws.write_string_with_format(row, 0, "Post-Contract Total", &fmt_total_label()).map_err(|e| e.to_string())?;
    ws.write_number_with_format(row, 4, total, &fmt_total_currency()).map_err(|e| e.to_string())?;
    row += 2;

    Ok(row)
}

fn write_reimbursable_costs_section(ws: &mut Worksheet, start_row: u32, costs: &[ReimbursableCost]) -> Result<u32, String> {
    let mut row = start_row;

    ws.merge_range(row, 0, row, 4, "Reimbursable Costs", &fmt_section_header()).map_err(|e| e.to_string())?;
    row += 1;

    let headers = ["Description", "Base Cost", "Markup %", "Cost to Client", "Notes"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_string_with_format(row, i as u16, *h, &fmt_col_header()).map_err(|e| e.to_string())?;
    }
    row += 1;

    let mut total = 0.0_f64;
    for cost in costs {
        ws.write_string_with_format(row, 0, &cost.description, &fmt_value()).map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 1, cost.base_cost, &fmt_currency()).map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 2, cost.markup_percent / 100.0, &fmt_percent()).map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 3, cost.cost_to_client, &fmt_currency()).map_err(|e| e.to_string())?;
        ws.write_string_with_format(row, 4, cost.notes.as_deref().unwrap_or(""), &fmt_value()).map_err(|e| e.to_string())?;
        total += cost.cost_to_client;
        row += 1;
    }

    ws.write_string_with_format(row, 0, "Costs Total", &fmt_total_label()).map_err(|e| e.to_string())?;
    ws.write_number_with_format(row, 3, total, &fmt_total_currency()).map_err(|e| e.to_string())?;
    row += 2;

    Ok(row)
}

fn write_summary_section(ws: &mut Worksheet, start_row: u32, pricing: &PricingBreakdown) -> Result<u32, String> {
    let mut row = start_row;

    ws.merge_range(row, 0, row, 4, "Summary", &fmt_section_header()).map_err(|e| e.to_string())?;
    row += 1;

    let summary_lines: Vec<(&str, f64)> = vec![
        ("Design Phase Total", pricing.design_phase_total),
        ("Post-Contract Total", pricing.post_contract_total),
        ("Reimbursable Costs Total", pricing.costs_total),
        ("Subtotal", pricing.subtotal),
        ("VAT", pricing.vat_amount),
    ];

    for (label, amount) in &summary_lines {
        ws.write_string_with_format(row, 0, *label, &fmt_label()).map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 1, *amount, &fmt_currency()).map_err(|e| e.to_string())?;
        row += 1;
    }

    // Grand total — prominent
    ws.write_string_with_format(row, 0, "GRAND TOTAL", &fmt_total_label()).map_err(|e| e.to_string())?;
    ws.write_number_with_format(row, 1, pricing.grand_total, &fmt_total_currency()).map_err(|e| e.to_string())?;
    row += 1;

    // Currency note
    row += 1;
    ws.write_string_with_format(row, 0, &format!("All amounts in {}", pricing.config.currency), &fmt_label()).map_err(|e| e.to_string())?;
    row += 1;

    Ok(row)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::types::*;
    use surrealdb::types::RecordId;
    use std::fs;

    fn make_record_id(table: &str, id: &str) -> RecordId {
        RecordId::new(table, id)
    }

    fn minimal_fee() -> Fee {
        Fee {
            id: Some(make_record_id("fee", "test123")),
            name: "DELETE ME - Test Fee Proposal".to_string(),
            number: "26-97101-R1".to_string(),
            rev: 1,
            status: "Draft".to_string(),
            issue_date: "260209".to_string(),
            activity: "Design and Consultancy".to_string(),
            package: "Lighting".to_string(),
            project_id: make_record_id("projects", "proj001"),
            company_id: make_record_id("company", "comp001"),
            contact_id: make_record_id("contacts", "cont001"),
            staff_name: "Martin Robert".to_string(),
            staff_email: "martin@emittiv.com".to_string(),
            staff_phone: "+971501234567".to_string(),
            staff_position: "Director".to_string(),
            strap_line: String::new(),
            revisions: vec![],
            time: TimeStamps {
                created_at: surrealdb_types::Datetime::default(),
                updated_at: surrealdb_types::Datetime::default(),
            },
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

    fn full_fee() -> Fee {
        let mut fee = minimal_fee();
        fee.name = "DELETE ME - Full Test Fee".to_string();
        fee.pricing = Some(serde_json::to_value(PricingBreakdown {
            config: PricingConfig {
                target_fee: 100000.0,
                buffer_percent: 5.0,
                quoted_fee: 105000.0,
                currency: "AED".to_string(),
                vat_percent: 5.0,
                vat_included: false,
                mobilisation_percent: 10.0,
            },
            disciplines: vec![
                Discipline { id: "lx".to_string(), name: "Lighting Design".to_string(), percentage: 70.0, order: 1 },
                Discipline { id: "ctrl".to_string(), name: "Controls".to_string(), percentage: 30.0, order: 2 },
            ],
            stages: vec![
                Stage { id: "sd".to_string(), name: "Schematic Design".to_string(), code: "SD".to_string(), percentage: 20.0, order: 1, is_post_contract: false },
                Stage { id: "dd".to_string(), name: "Design Development".to_string(), code: "DD".to_string(), percentage: 30.0, order: 2, is_post_contract: false },
                Stage { id: "cd".to_string(), name: "Construction Documents".to_string(), code: "CD".to_string(), percentage: 50.0, order: 3, is_post_contract: false },
            ],
            cells: vec![
                PricingCell { discipline_id: "lx".to_string(), stage_id: "sd".to_string(), amount: 14700.0, override_amount: None },
                PricingCell { discipline_id: "lx".to_string(), stage_id: "dd".to_string(), amount: 22050.0, override_amount: None },
                PricingCell { discipline_id: "lx".to_string(), stage_id: "cd".to_string(), amount: 36750.0, override_amount: None },
                PricingCell { discipline_id: "ctrl".to_string(), stage_id: "sd".to_string(), amount: 6300.0, override_amount: None },
                PricingCell { discipline_id: "ctrl".to_string(), stage_id: "dd".to_string(), amount: 9450.0, override_amount: None },
                PricingCell { discipline_id: "ctrl".to_string(), stage_id: "cd".to_string(), amount: 15750.0, override_amount: None },
            ],
            costs: vec![
                ReimbursableCost {
                    id: "rc1".to_string(),
                    description: "Travel expenses".to_string(),
                    stage_id: "sd".to_string(),
                    discipline_id: None,
                    base_cost: 5000.0,
                    markup_percent: 10.0,
                    cost_to_client: 5500.0,
                    date_incurred: "2026-03-01".to_string(),
                    notes: Some("Estimated".to_string()),
                },
            ],
            design_phase_total: 105000.0,
            post_contract_total: 0.0,
            costs_total: 5500.0,
            subtotal: 110500.0,
            vat_amount: 5525.0,
            grand_total: 116025.0,
        }).unwrap());
        fee.post_contract_items = Some(vec![
            PostContractItem {
                id: "pc1".to_string(),
                stage_id: "ca".to_string(),
                description: "Site visits".to_string(),
                quantity: 12.0,
                unit: "visits".to_string(),
                rate: 2500.0,
                amount: 30000.0,
            },
        ]);
        fee.reimbursable_costs = Some(vec![
            ReimbursableCost {
                id: "rc1".to_string(),
                description: "Travel expenses".to_string(),
                stage_id: "sd".to_string(),
                discipline_id: None,
                base_cost: 5000.0,
                markup_percent: 10.0,
                cost_to_client: 5500.0,
                date_incurred: "2026-03-01".to_string(),
                notes: Some("Estimated".to_string()),
            },
        ]);
        fee
    }

    #[test]
    fn test_minimal_fee_generates_valid_xlsx() {
        let fee = minimal_fee();
        let path = std::env::temp_dir().join("delete_me_test_minimal_fee.xlsx");
        let result = generate_fee_excel(&fee, &path);
        assert!(result.is_ok(), "generate_fee_excel failed: {:?}", result.err());

        let metadata = fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "Generated file is empty");

        // Clean up
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_full_fee_generates_valid_xlsx() {
        let fee = full_fee();
        let path = std::env::temp_dir().join("delete_me_test_full_fee.xlsx");
        let result = generate_fee_excel(&fee, &path);
        assert!(result.is_ok(), "generate_fee_excel failed: {:?}", result.err());

        let metadata = fs::metadata(&path).unwrap();
        // Full fee with pricing data should produce a larger file
        assert!(metadata.len() > 1000, "Generated file seems too small: {} bytes", metadata.len());

        // Clean up
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_returned_path_matches_output() {
        let fee = minimal_fee();
        let path = std::env::temp_dir().join("delete_me_test_path_check.xlsx");
        let result = generate_fee_excel(&fee, &path).unwrap();
        assert_eq!(result, path.to_string_lossy().to_string());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_invalid_path_returns_error() {
        let fee = minimal_fee();
        let path = Path::new("/nonexistent_dir_12345/impossible.xlsx");
        let result = generate_fee_excel(&fee, path);
        assert!(result.is_err(), "Expected error for invalid path");
    }

    #[test]
    fn test_xlsx_file_has_valid_magic_bytes() {
        let fee = full_fee();
        let path = std::env::temp_dir().join("delete_me_test_magic_bytes.xlsx");
        generate_fee_excel(&fee, &path).unwrap();

        let bytes = fs::read(&path).unwrap();
        // xlsx files are zip archives — must start with PK magic bytes
        assert_eq!(&bytes[0..2], b"PK", "File does not have ZIP/XLSX magic bytes");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_template_with_3_stages_fallback() {
        let fee = full_fee();
        // 3 stages = default test data (fewer than template's 4 → row removal)
        let path = std::env::temp_dir().join("delete_me_test_template_3stages.xlsx");
        let result = generate_fee_template(&fee, &path, None);
        assert!(result.is_ok(), "generate_fee_template failed: {:?}", result.err());

        let metadata = fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "Generated file is empty");

        // Verify it's a valid xlsx (PK magic bytes)
        let bytes = fs::read(&path).unwrap();
        assert_eq!(&bytes[0..2], b"PK", "Not a valid xlsx file");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_template_with_5_stages_fallback() {
        let mut fee = full_fee();
        // Add 2 more stages to get 5 total (more than template's 4 → row insertion)
        if let Some(mut pricing) = fee.pricing_typed() {
            pricing.stages.push(Stage {
                id: "td".to_string(), name: "Tender".to_string(), code: "TD".to_string(),
                percentage: 10.0, order: 4, is_post_contract: false,
            });
            pricing.stages.push(Stage {
                id: "ca".to_string(), name: "Construction Admin".to_string(), code: "CA".to_string(),
                percentage: 10.0, order: 5, is_post_contract: false,
            });
            // Add cells for the new stages
            for disc_id in ["lx", "ctrl"] {
                for stage_id in ["td", "ca"] {
                    pricing.cells.push(PricingCell {
                        discipline_id: disc_id.to_string(),
                        stage_id: stage_id.to_string(),
                        amount: 5000.0,
                        override_amount: None,
                    });
                }
            }
            fee.pricing = Some(serde_json::to_value(&pricing).unwrap());
        }

        let path = std::env::temp_dir().join("delete_me_test_template_5stages.xlsx");
        let result = generate_fee_template(&fee, &path, None);
        assert!(result.is_ok(), "generate_fee_template (5 stages) failed: {:?}", result.err());

        let metadata = fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "Generated file is empty");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_template_fallback_embedded() {
        // No source_path → uses embedded template
        let fee = full_fee();
        let path = std::env::temp_dir().join("delete_me_test_template_embedded.xlsx");
        let result = generate_fee_template(&fee, &path, None);
        assert!(result.is_ok(), "Embedded template fallback failed: {:?}", result.err());

        let bytes = fs::read(&path).unwrap();
        assert_eq!(&bytes[0..2], b"PK", "Not a valid xlsx file");

        let _ = fs::remove_file(&path);
    }
}
