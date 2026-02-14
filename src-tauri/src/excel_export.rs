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
    if let Some(ref pricing) = fee.pricing {
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
    if let Some(ref pricing) = fee.pricing {
        row = write_summary_section(worksheet, row, pricing)?;
    }

    // Suppress unused variable warning — row is the final position
    let _ = row;

    workbook.save(output_path).map_err(|e| format!("Failed to save Excel file: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
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
    use surrealdb::sql::Thing;
    use std::fs;

    fn make_thing(table: &str, id: &str) -> Thing {
        Thing::from((table.to_string(), id.to_string()))
    }

    fn minimal_fee() -> Fee {
        Fee {
            id: Some(make_thing("fee", "test123")),
            name: "DELETE ME - Test Fee Proposal".to_string(),
            number: "26-97101-R1".to_string(),
            rev: 1,
            status: "Draft".to_string(),
            issue_date: "260209".to_string(),
            activity: "Design and Consultancy".to_string(),
            package: "Lighting".to_string(),
            project_id: make_thing("projects", "proj001"),
            company_id: make_thing("company", "comp001"),
            contact_id: make_thing("contacts", "cont001"),
            staff_name: "Martin Robert".to_string(),
            staff_email: "martin@emittiv.com".to_string(),
            staff_phone: "+971501234567".to_string(),
            staff_position: "Director".to_string(),
            strap_line: String::new(),
            revisions: vec![],
            time: TimeStamps {
                created_at: "2026-02-09T00:00:00Z".to_string(),
                updated_at: "2026-02-09T00:00:00Z".to_string(),
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
        fee.pricing = Some(PricingBreakdown {
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
        });
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
}
