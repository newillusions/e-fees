//! Excel export Tauri commands.

use tauri::{State, AppHandle, Manager};
use log::{info, error};
use std::path::PathBuf;

use super::AppState;
use crate::excel_export::{generate_fee_excel, generate_fee_template};

/// Export a fee proposal to a formatted .xlsx file.
///
/// Fetches the fee from the database, generates the Excel file in the system
/// temp directory, and returns the file path.
#[tauri::command]
pub async fn export_fee_excel(
    id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("export_fee_excel called for id: {}", id);

    // Strip table prefix if present
    let record_id = id.strip_prefix("fee:").unwrap_or(&id).to_string();

    let manager = state.read().await;
    let fee = manager.get_fee_by_id(&record_id).await.map_err(|e| {
        error!("Failed to fetch fee for export: {}", e);
        format!("Failed to fetch fee: {}", e)
    })?;

    let fee = fee.ok_or_else(|| "Fee not found".to_string())?;

    // Build output filename: fee-{number}-rev{rev}.xlsx
    let safe_number = fee.number.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");
    let filename = format!("fee-{}-rev{}.xlsx", safe_number, fee.rev);
    let output_path = std::env::temp_dir().join(&filename);

    let path = generate_fee_excel(&fee, &output_path)?;

    info!("Excel export saved to: {}", path);
    Ok(path)
}

/// Export a fee to the project folder as a pricing template Excel file.
///
/// Generates a working spreadsheet with discipline × stage matrix and formulas,
/// matching the format used in project folders (`*-FP-NN Pricing.xlsx`).
#[tauri::command]
pub async fn export_fee_template(
    fee_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    info!("export_fee_template called for id: {}", fee_id);

    let record_id = fee_id.strip_prefix("fee:").unwrap_or(&fee_id).to_string();

    // Fetch the fee
    let manager = state.read().await;
    let fee = manager.get_fee_by_id(&record_id).await.map_err(|e| {
        error!("Failed to fetch fee for template export: {}", e);
        format!("Failed to fetch fee: {}", e)
    })?;
    let fee = fee.ok_or_else(|| "Fee not found".to_string())?;

    // Count design stages from pricing data
    let stage_count = fee.pricing.as_ref()
        .map(|p| p.stages.iter().filter(|s| !s.is_post_contract).count())
        .unwrap_or(3);

    // Build output filename: {number}-FP-{rev:02} Pricing.xlsx
    let safe_number = fee.number.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");
    let filename = format!("{}-FP-{:02} Pricing.xlsx", safe_number, fee.rev);
    let output_path = std::env::temp_dir().join(&filename);

    let path = generate_fee_template(&fee, &output_path, stage_count)?;

    info!("Template export saved to: {}", path);
    Ok(path)
}
