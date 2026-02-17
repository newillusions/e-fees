//! Excel export Tauri commands.

use tauri::{AppHandle, State};
use log::{info, warn, error};
use std::path::{Path, PathBuf};
use regex::Regex;

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
/// Uses PRI-NN versioning: each export creates the next numbered version
/// (e.g. `25-97105-PRI-01 Pricing.xlsx`, `PRI-02`, etc.). The latest
/// existing PRI file is used as the source template so manual edits are
/// preserved. Falls back to the embedded blank template when no project
/// folder or previous PRI file exists.
///
/// If `output_path` is provided (from "Export As…" dialog), saves there
/// instead of auto-incrementing in the project folder.
#[tauri::command]
pub async fn export_fee_template(
    fee_id: String,
    output_path: Option<String>,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("export_fee_template called for id: {}, output_path: {:?}", fee_id, output_path);

    let record_id = fee_id.strip_prefix("fee:").unwrap_or(&fee_id).to_string();

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

    let safe_number = project_number
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");

    // Find the project's proposal directory and scan for existing PRI files
    let proposal_dir = find_proposal_dir(&app_handle, &project_number).await;
    let (source_path, next_version) = match &proposal_dir {
        Some(dir) => find_latest_pri_file(dir, &safe_number),
        None => (None, 1),
    };

    // Determine output path
    let resolved_path = match output_path {
        Some(p) => PathBuf::from(p),
        None => {
            match &proposal_dir {
                Some(dir) => {
                    let filename = format!("{}-PRI-{:02} Pricing.xlsx", safe_number, next_version);
                    dir.join(&filename)
                }
                None => {
                    // No project folder → save to temp
                    let filename = format!("{}-PRI-{:02} Pricing.xlsx", safe_number, next_version);
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

/// Locate the project's `02 Proposal` directory via `find_project_folder()`.
async fn find_proposal_dir(
    app_handle: &AppHandle,
    project_number: &str,
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

    Some(proposal_dir)
}

/// Scan a proposal directory for existing PRI-NN files and return the latest
/// as `source_path` plus the next version number.
///
/// Pattern: `{project_number}-PRI-{NN} Pricing.xlsx`
/// Also matches legacy `{project_number}-FP-{NN} Pricing.xlsx` files as sources.
fn find_latest_pri_file(
    proposal_dir: &Path,
    safe_number: &str,
) -> (Option<PathBuf>, i32) {
    let pattern = format!(r"^{}-(?:PRI|FP)-(\d+)\s+Pricing\.xlsx$", regex::escape(safe_number));
    let re = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return (None, 1),
    };

    let mut best_version: i32 = 0;
    let mut best_path: Option<PathBuf> = None;

    if let Ok(entries) = std::fs::read_dir(proposal_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if let Some(caps) = re.captures(&name_str) {
                if let Ok(ver) = caps[1].parse::<i32>() {
                    if ver > best_version {
                        best_version = ver;
                        best_path = Some(entry.path());
                    }
                }
            }
        }
    }

    if let Some(ref p) = best_path {
        info!("Found latest pricing file: {:?} (version {})", p, best_version);
    } else {
        info!("No existing PRI/FP files found in {:?}", proposal_dir);
    }

    (best_path, best_version + 1)
}
