//! Excel export Tauri commands.

use tauri::{AppHandle, State};
use log::{info, warn, error};
use std::path::PathBuf;

use super::AppState;
use crate::commands::folder_management::get_project_folder_location;
use crate::excel_export::{generate_fee_excel, generate_fee_template};

/// Reveal a file in the native file manager (Finder on macOS, Explorer on Windows).
fn reveal_in_file_manager(path: &str) {
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("open").arg("-R").arg(path).spawn() {
            warn!("Failed to reveal file in Finder: {}", e);
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("explorer").arg(format!("/select,{}", path)).spawn() {
            warn!("Failed to reveal file in Explorer: {}", e);
        }
    }
}

/// Export a fee proposal to a formatted .xlsx file.
///
/// If `output_path` is provided (from a save dialog), writes there.
/// Otherwise falls back to the system temp directory.
#[tauri::command]
pub async fn export_fee_excel(
    id: String,
    output_path: Option<String>,
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

    let resolved_path = match output_path {
        Some(p) => PathBuf::from(p),
        None => {
            let safe_number = fee.number.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");
            let filename = format!("fee-{}-rev{}.xlsx", safe_number, fee.rev);
            std::env::temp_dir().join(&filename)
        }
    };

    let path = generate_fee_excel(&fee, &resolved_path)?;

    info!("Excel export saved to: {}", path);
    reveal_in_file_manager(&path);
    Ok(path)
}

/// Export a fee to the project folder as a pricing template Excel file.
///
/// Locates the project's existing pricing spreadsheet via `find_project_folder()`,
/// opens it with umya-spreadsheet (preserving formatting), populates it with
/// pricing data from the fee, and saves in-place (or to `output_path` if the
/// user chose "Export As…").
///
/// Falls back to the embedded blank template when the project folder doesn't
/// exist yet (e.g. new RFPs before folder creation).
#[tauri::command]
pub async fn export_fee_template(
    fee_id: String,
    output_path: Option<String>,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("export_fee_template called for id: {}, output_path: {:?}", fee_id, output_path);

    let record_id = fee_id.strip_prefix("fee:").unwrap_or(&fee_id).to_string();

    // Fetch the fee and its related project
    let manager = state.read().await;
    let fee = manager.get_fee_by_id(&record_id).await.map_err(|e| {
        error!("Failed to fetch fee for template export: {}", e);
        format!("Failed to fetch fee: {}", e)
    })?;
    let fee = fee.ok_or_else(|| "Fee not found".to_string())?;

    // Extract project number from fee number (e.g. "25-97105-R1" → "25-97105")
    let project_number = fee.number
        .split("-R")
        .next()
        .unwrap_or(&fee.number)
        .to_string();

    // Try to find the project's existing pricing file
    let source_path = find_pricing_file(&app_handle, &project_number, &fee.number, fee.rev).await;

    // Determine output path
    let resolved_path = match output_path {
        Some(p) => PathBuf::from(p),
        None => {
            // Default: save in-place to the project folder's pricing file
            match &source_path {
                Some(p) => p.clone(),
                None => {
                    // No project folder → save to temp
                    let safe_number = fee.number.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");
                    let filename = format!("{}-{:02} Pricing.xlsx", safe_number, fee.rev);
                    std::env::temp_dir().join(&filename)
                }
            }
        }
    };

    let path = generate_fee_template(
        &fee,
        &resolved_path,
        source_path.as_deref(),
    )?;

    info!("Template export saved to: {}", path);
    reveal_in_file_manager(&path);
    Ok(path)
}

/// Search for the project's existing pricing xlsx file.
///
/// Constructs the expected path:
/// `{project_folder}/02 Proposal/{fee_number}-FP-{rev:02} Pricing.xlsx`
async fn find_pricing_file(
    app_handle: &AppHandle,
    project_number: &str,
    fee_number: &str,
    rev: i32,
) -> Option<PathBuf> {
    let folder_info = get_project_folder_location(app_handle.clone(), project_number.to_string())
        .await
        .ok()?;

    if !folder_info.exists {
        info!("Project folder not found for {}, will use embedded template", project_number);
        return None;
    }

    let proposal_dir = PathBuf::from(&folder_info.full_path).join("02 Proposal");
    if !proposal_dir.exists() {
        info!("02 Proposal directory not found in project folder");
        return None;
    }

    // Strip table prefix from fee_number for filename (e.g. "25-97105" not "fee:xxx")
    let safe_number = fee_number
        .split("-R")
        .next()
        .unwrap_or(fee_number)
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");

    let filename = format!("{}-FP-{:02} Pricing.xlsx", safe_number, rev);
    let pricing_path = proposal_dir.join(&filename);

    if pricing_path.exists() {
        info!("Found existing pricing file: {:?}", pricing_path);
        Some(pricing_path)
    } else {
        info!("Pricing file not found at {:?}, will use embedded template", pricing_path);
        None
    }
}
