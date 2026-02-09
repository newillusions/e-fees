//! Excel export Tauri commands.

use tauri::State;
use log::{info, error};
use std::path::PathBuf;

use super::AppState;
use crate::excel_export::generate_fee_excel;

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
