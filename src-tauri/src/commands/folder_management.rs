//! Folder Management Commands
//!
//! This module provides cross-platform folder operations for managing project folders
//! across different status directories. It uses the app settings for path configuration
//! and supports moving folders across different filesystems/drives.
//!
//! ## Cross-Platform Support
//! - Uses `fs_extra` crate for cross-filesystem moves (copy + delete)
//! - Falls back to standard `fs::rename` when on same filesystem (faster)
//! - Works on Windows, macOS, and Linux
//!
//! ## Path Configuration
//! Paths are read from the app settings (`e-fees.config` / `e-fees.config.dev` file):
//! - `PROJECT_FOLDER_PATH`: Base path for all project folders

use crate::commands::types::{AppSettingsPublic, ProjectUpdate};
use crate::commands::{get_settings, AppState};
use log::{error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{command, AppHandle, State};

#[derive(Debug, serde::Serialize)]
pub struct FolderOperationResult {
    pub success: bool,
    pub message: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ProjectFolderInfo {
    pub project_number: String,
    pub current_location: String,
    pub full_path: String,
    pub exists: bool,
}

/// Get the base projects path from app settings.
///
/// This function reads the project folder path from the app's settings file,
/// ensuring consistent path configuration across all folder operations.
///
/// # Arguments
/// * `app_handle` - Tauri AppHandle for accessing app configuration
///
/// # Returns
/// * `Ok(PathBuf)` - The configured project base path
/// * `Err(String)` - Error message if path is not configured or doesn't exist
async fn get_projects_base_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let settings = get_settings(app_handle.clone())
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))?;

    let base_path = settings.project_folder_path.ok_or_else(|| {
        "PROJECT_FOLDER_PATH not configured in settings. Please set it in the Settings panel."
            .to_string()
    })?;

    let path = PathBuf::from(&base_path);

    if !path.exists() {
        return Err(format!(
            "Project folder path does not exist: {}. Please verify the path in your settings.",
            base_path
        ));
    }

    info!("Using project base path: {:?}", path);
    Ok(path)
}

/// Whether enough settings are present to attempt a Nextcloud rescan.
/// Pure/testable: both `efees_api_url` and `efees_api_key` must be set.
fn efees_rescan_configured(settings: &AppSettingsPublic) -> bool {
    settings
        .efees_api_url
        .as_ref()
        .is_some_and(|u| !u.trim().is_empty())
        && settings
            .efees_api_key
            .as_ref()
            .is_some_and(|k| !k.trim().is_empty())
}

/// Best-effort trigger of e-fees-api's `POST /nc/rescan` after a successful
/// folder move, so Nextcloud's groupfolders index picks up the change made
/// directly on the filesystem (Nextcloud does not watch the filesystem live).
///
/// Deliberately swallows every failure mode (not configured, unreachable,
/// non-2xx) - a stale Nextcloud index is a freshness nicety, not a reason to
/// report the folder move itself as failed. Logs at warn/info so it's still
/// visible for diagnosis.
async fn trigger_nc_rescan(settings: &AppSettingsPublic) {
    if !efees_rescan_configured(settings) {
        info!(
            "Skipping Nextcloud rescan: EFEES_API_URL/EFEES_API_KEY not configured in settings"
        );
        return;
    }

    // efees_rescan_configured() already proved both are Some and non-empty.
    let base_url = settings.efees_api_url.as_ref().unwrap().trim_end_matches('/');
    let api_key = settings.efees_api_key.as_ref().unwrap();
    let url = format!("{}/nc/rescan", base_url);

    let client = reqwest::Client::new();
    match client
        .post(&url)
        .header("X-API-Key", api_key)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("Nextcloud rescan triggered successfully");
        }
        Ok(response) => {
            warn!(
                "Nextcloud rescan request returned {} (non-fatal)",
                response.status()
            );
        }
        Err(e) => {
            warn!("Failed to trigger Nextcloud rescan (non-fatal): {}", e);
        }
    }
}

/// Convert a display-form project number ("26-97110") to its DB record key
/// ("26_97110") - the same convention used by `create_new_project`.
fn project_key_from_number(project_number: &str) -> String {
    project_number.replace('-', "_")
}

/// Best-effort sync of the project's DB `folder` field to the folder's actual
/// name on disk. The DB field is known to drift from reality (obs: two live
/// records found stale after a folder rename on 2026-08-12 - name_short and
/// folder both out of sync with the on-disk name) - this closes that gap at
/// the one place folders are known to have just moved (or been confirmed
/// already in the right place).
///
/// Never fails the caller: a DB write failure here does not mean the folder
/// move failed, only that the stored `folder` field is still stale.
async fn sync_project_folder_db_field(
    state: &State<'_, AppState>,
    project_number: &str,
    actual_folder_name: &str,
) {
    let record_key = project_key_from_number(project_number);

    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };

    let update = ProjectUpdate {
        name: None,
        name_short: None,
        status: None,
        area: None,
        city: None,
        country: None,
        folder: Some(actual_folder_name.to_string()),
    };

    match manager_clone.update_project(&record_key, update).await {
        Ok(_) => info!(
            "Synced project {} DB folder field to '{}'",
            project_number, actual_folder_name
        ),
        Err(e) => warn!(
            "Failed to sync project {} DB folder field to '{}' (non-fatal): {}",
            project_number, actual_folder_name, e
        ),
    }
}

/// Get the folder name for a given project status.
///
/// Maps project statuses to their corresponding folder names:
/// - "01 RFPs": lead, rfp, submitted
/// - "11 Current": awarded, design, construction, practical completion
/// - "99 Completed": completed, superseded
/// - "00 Inactive": cancelled, lost, no response, on hold
fn get_folder_for_status(status: &str) -> Result<&str, String> {
    match status.to_lowercase().as_str() {
        "lead" | "rfp" | "submitted" => Ok("01 RFPs"),
        "awarded" | "design" | "construction" | "practical completion" => Ok("11 Current"),
        "completed" | "superseded" => Ok("99 Completed"),
        "cancelled" | "lost" | "no response" | "on hold" => Ok("00 Inactive"),
        _ => Err(format!("Unknown status: {}", status)),
    }
}

/// Find a project folder by number across all status directories.
///
/// Searches through all status folders (00 Inactive, 01 RFPs, 11 Current, 99 Completed)
/// to locate a project folder that starts with the given project number.
async fn find_project_folder(
    app_handle: &AppHandle,
    project_number: &str,
) -> Result<ProjectFolderInfo, String> {
    let base_path = get_projects_base_path(app_handle).await?;

    let status_dirs = ["00 Inactive", "01 RFPs", "11 Current", "99 Completed"];

    for status_dir in &status_dirs {
        let search_path = base_path.join(status_dir);

        if !search_path.exists() {
            info!(
                "Status directory does not exist, skipping: {:?}",
                search_path
            );
            continue;
        }

        // Read directory contents
        match fs::read_dir(&search_path) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let folder_name = entry.file_name();
                        let folder_str = folder_name.to_string_lossy();

                        // Check if folder starts with the project number
                        if folder_str.starts_with(project_number) {
                            info!("Found project folder: {:?} in {}", entry.path(), status_dir);
                            return Ok(ProjectFolderInfo {
                                project_number: project_number.to_string(),
                                current_location: status_dir.to_string(),
                                full_path: entry.path().to_string_lossy().to_string(),
                                exists: true,
                            });
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read directory {}: {}", search_path.display(), e);
                continue;
            }
        }
    }

    info!("Project folder not found for: {}", project_number);
    Ok(ProjectFolderInfo {
        project_number: project_number.to_string(),
        current_location: "not_found".to_string(),
        full_path: String::new(),
        exists: false,
    })
}

/// Move a folder, with fallback to copy+delete for cross-filesystem moves.
///
/// This function first attempts a fast `fs::rename` operation. If that fails
/// (typically due to cross-filesystem boundaries), it falls back to copying
/// the folder recursively and then deleting the original.
///
/// # Arguments
/// * `from` - Source path
/// * `to` - Destination path
///
/// # Returns
/// * `Ok(())` - Move successful
/// * `Err(String)` - Error message if move failed
fn move_folder_cross_platform(from: &Path, to: &Path) -> Result<(), String> {
    info!("Moving folder from {:?} to {:?}", from, to);

    // First, try a simple rename (fast, works on same filesystem)
    match fs::rename(from, to) {
        Ok(_) => {
            info!("Successfully moved folder using fs::rename");
            return Ok(());
        }
        Err(e) => {
            // Check if this is a cross-device link error (common when moving across filesystems)
            let error_kind = e.kind();
            if error_kind == std::io::ErrorKind::CrossesDevices
                || error_kind == std::io::ErrorKind::Other
                || error_kind == std::io::ErrorKind::PermissionDenied
            {
                info!("fs::rename failed ({}), falling back to copy+delete", e);
            } else {
                // For other errors, return immediately
                error!("Failed to move folder: {}", e);
                return Err(format!("Failed to move folder: {}", e));
            }
        }
    }

    // Fallback: Use fs_extra for cross-filesystem move
    info!("Using fs_extra for cross-filesystem move");

    let options = fs_extra::dir::CopyOptions {
        overwrite: false,
        skip_exist: false,
        buffer_size: 64000,
        copy_inside: false,
        content_only: false,
        depth: 0, // 0 means unlimited depth
    };

    // Get the parent directory of the destination
    let dest_parent = to
        .parent()
        .ok_or_else(|| "Could not get destination parent directory".to_string())?;

    // Copy the folder to the destination
    match fs_extra::dir::copy(from, dest_parent, &options) {
        Ok(bytes) => {
            info!("Successfully copied {} bytes to {:?}", bytes, dest_parent);

            // Verify the copy was successful by checking destination exists
            if !to.exists() {
                // fs_extra::dir::copy preserves the folder name, so check with original name
                let folder_name = from
                    .file_name()
                    .ok_or_else(|| "Could not get folder name".to_string())?;
                let actual_dest = dest_parent.join(folder_name);

                if !actual_dest.exists() {
                    return Err(format!(
                        "Copy appeared successful but destination doesn't exist: {:?}",
                        to
                    ));
                }
            }

            // Delete the original folder
            match fs::remove_dir_all(from) {
                Ok(_) => {
                    info!("Successfully deleted original folder after copy");
                    Ok(())
                }
                Err(e) => {
                    // Copy succeeded but delete failed - warn but don't fail
                    warn!("Folder was copied but original could not be deleted: {}", e);
                    Ok(()) // Still consider this a success - data is moved
                }
            }
        }
        Err(e) => {
            error!("Failed to copy folder with fs_extra: {}", e);
            Err(format!("Failed to move folder: {}", e))
        }
    }
}

#[command]
pub async fn get_project_folder_location(
    app_handle: AppHandle,
    project_number: String,
) -> Result<ProjectFolderInfo, String> {
    info!("Getting folder location for project: {}", project_number);
    find_project_folder(&app_handle, &project_number).await
}

#[command]
pub async fn move_project_folder(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    project_number: String,
    new_status: String,
) -> Result<FolderOperationResult, String> {
    info!(
        "Moving project {} to status: {}",
        project_number, new_status
    );

    // Find current location
    let current_info = find_project_folder(&app_handle, &project_number).await?;

    if !current_info.exists {
        warn!("Project folder not found: {}", project_number);
        return Ok(FolderOperationResult {
            success: false,
            message: format!("Project folder {} not found. Make sure the folder exists and PROJECT_FOLDER_PATH is correctly configured.", project_number),
            old_path: None,
            new_path: None,
        });
    }

    // Get destination folder
    let dest_folder = get_folder_for_status(&new_status)?;

    // Check if already in correct location
    if current_info.current_location == dest_folder {
        info!("Project {} is already in {}", project_number, dest_folder);

        // The physical location is already correct, but the DB folder field
        // can still be stale (e.g. the folder was renamed by hand without
        // the app knowing) - sync it now that we have the real name in hand.
        if let Some(actual_name) = Path::new(&current_info.full_path)
            .file_name()
            .and_then(|n| n.to_str())
        {
            sync_project_folder_db_field(&state, &project_number, actual_name).await;
        }

        return Ok(FolderOperationResult {
            success: true,
            message: format!("Project {} is already in {}", project_number, dest_folder),
            old_path: Some(current_info.full_path.clone()),
            new_path: Some(current_info.full_path),
        });
    }

    // Build destination path
    let base_path = get_projects_base_path(&app_handle).await?;
    let dest_dir = base_path.join(dest_folder);

    // Ensure destination directory exists
    if !dest_dir.exists() {
        error!("Destination directory does not exist: {:?}", dest_dir);
        return Err(format!(
            "Destination directory {} does not exist",
            dest_dir.display()
        ));
    }

    // Get the folder name from current path
    let current_path = Path::new(&current_info.full_path);
    let folder_name = current_path
        .file_name()
        .ok_or("Could not get folder name")?;

    let new_path = dest_dir.join(folder_name);

    // Check if destination already exists
    if new_path.exists() {
        warn!("Destination folder already exists: {:?}", new_path);
        return Ok(FolderOperationResult {
            success: false,
            message: format!("Destination folder already exists: {}", new_path.display()),
            old_path: Some(current_info.full_path),
            new_path: Some(new_path.to_string_lossy().to_string()),
        });
    }

    // Perform the move operation using cross-platform function
    match move_folder_cross_platform(current_path, &new_path) {
        Ok(_) => {
            let mut success_message = format!(
                "Successfully moved {} from {} to {}",
                project_number, current_info.current_location, dest_folder
            );

            // If moving to Current from any other folder, copy awarded project templates
            if current_info.current_location != "11 Current" && dest_folder == "11 Current" {
                match copy_awarded_templates(&app_handle, &new_path).await {
                    Ok(_) => {
                        success_message
                            .push_str(". Awarded project templates copied successfully.");
                    }
                    Err(e) => {
                        success_message
                            .push_str(&format!(". Warning: Failed to copy some templates: {}", e));
                    }
                }
            }

            info!("{}", success_message);

            // Best-effort: sync the DB folder field to the folder's real
            // (possibly renamed/drifted) name, now that we know it for certain.
            if let Some(actual_name) = folder_name.to_str() {
                sync_project_folder_db_field(&state, &project_number, actual_name).await;
            }

            // Best-effort: ask Nextcloud to rescan so it picks up the move.
            // Never blocks or fails the move itself over a rescan hiccup -
            // matches the pattern e-fees-api itself uses for every other
            // nc_rescan call site (document upload, fee export).
            match get_settings(app_handle.clone()).await {
                Ok(settings) => trigger_nc_rescan(&settings).await,
                Err(e) => warn!("Skipping Nextcloud rescan - could not read settings: {}", e),
            }

            Ok(FolderOperationResult {
                success: true,
                message: success_message,
                old_path: Some(current_info.full_path),
                new_path: Some(new_path.to_string_lossy().to_string()),
            })
        }
        Err(e) => {
            error!("Failed to move project folder: {}", e);
            Ok(FolderOperationResult {
                success: false,
                message: format!("Failed to move folder: {}", e),
                old_path: Some(current_info.full_path),
                new_path: Some(new_path.to_string_lossy().to_string()),
            })
        }
    }
}

#[command]
pub async fn move_project_from_rfp(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    project_number: String,
    destination: String,
) -> Result<FolderOperationResult, String> {
    info!(
        "Moving project {} from RFP to: {}",
        project_number, destination
    );
    // Validate destination — uses new domain model statuses
    match destination.as_str() {
        "current" => {
            move_project_folder(app_handle, state, project_number, "awarded".to_string()).await
        }
        "archive" => {
            move_project_folder(app_handle, state, project_number, "completed".to_string()).await
        }
        "inactive" => {
            move_project_folder(app_handle, state, project_number, "cancelled".to_string()).await
        }
        _ => Err(format!(
            "Invalid destination: {}. Use 'current', 'archive', or 'inactive'",
            destination
        )),
    }
}

#[command]
pub async fn move_project_to_archive(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    project_number: String,
) -> Result<FolderOperationResult, String> {
    info!("Moving project {} to archive", project_number);
    move_project_folder(app_handle, state, project_number, "completed".to_string()).await
}

#[command]
pub async fn list_projects_in_folder(
    app_handle: AppHandle,
    folder_path: String,
) -> Result<Vec<String>, String> {
    // Security: reject path traversal in subfolder name
    if folder_path.contains("..") {
        return Err("Invalid path: path traversal not allowed".to_string());
    }

    let base_path = get_projects_base_path(&app_handle).await?;
    let full_path = base_path.join(&folder_path);

    info!("Listing projects in: {:?}", full_path);

    if !full_path.exists() {
        return Err(format!("Folder does not exist: {}", full_path.display()));
    }

    let mut projects = Vec::new();

    match fs::read_dir(full_path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        let folder_name = entry.file_name().to_string_lossy().to_string();
                        // Filter for project folders (YY-CCCNN format)
                        if folder_name.len() >= 8 && folder_name.chars().nth(2) == Some('-') {
                            projects.push(folder_name);
                        }
                    }
                }
            }
        }
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    }

    projects.sort();
    Ok(projects)
}

/// Copy awarded project template folders when moving from RFP to Current.
///
/// When a project is awarded (moved from "01 RFPs" to "11 Current"), this function
/// copies additional template folders that are needed for active projects.
async fn copy_awarded_templates(app_handle: &AppHandle, project_path: &Path) -> Result<(), String> {
    let base_path = get_projects_base_path(app_handle).await?;
    let template_path = base_path.join("11 Current").join("00 Additional Folders");

    if !template_path.exists() {
        warn!(
            "Awarded project template folder not found: {:?}",
            template_path
        );
        return Err("Awarded project template folder not found".to_string());
    }

    // Copy each template folder to the project
    let template_folders = [
        "03 Contract",
        "04 Deliverables",
        "05 Submittals",
        "11 SubContractors",
        "98 Outgoing",
        "99 Temp",
    ];

    let options = fs_extra::dir::CopyOptions {
        overwrite: false,
        skip_exist: true, // Skip if already exists
        buffer_size: 64000,
        copy_inside: false,
        content_only: false,
        depth: 0,
    };

    for folder in &template_folders {
        let src = template_path.join(folder);
        let dest = project_path.join(folder);

        if src.exists() && !dest.exists() {
            info!("Copying template folder: {} to {:?}", folder, dest);
            if let Err(e) = fs_extra::dir::copy(&src, project_path, &options) {
                warn!("Failed to copy template folder {}: {}", folder, e);
                // Continue with other folders even if one fails
            }
        }
    }

    Ok(())
}

/// Recursively copy a directory and its contents (fallback for simple cases).
///
/// This is a pure Rust implementation that doesn't rely on external crates.
/// Used as a fallback when fs_extra is not available or for simple cases.
#[allow(dead_code)]
fn copy_dir_recursively(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    if !src.exists() {
        return Ok(());
    }

    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursively(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

#[command]
pub async fn validate_project_base_path(app_handle: AppHandle) -> Result<String, String> {
    match get_projects_base_path(&app_handle).await {
        Ok(path) => {
            // Also check that required subdirectories exist
            let required_dirs = ["00 Inactive", "01 RFPs", "11 Current", "99 Completed"];
            let mut missing_dirs = Vec::new();

            for dir in &required_dirs {
                let dir_path = path.join(dir);
                if !dir_path.exists() {
                    missing_dirs.push(*dir);
                }
            }

            if missing_dirs.is_empty() {
                Ok(format!(
                    "Project base path is valid: {}\nAll required directories exist.",
                    path.display()
                ))
            } else {
                Ok(format!(
                    "Project base path exists: {}\nWarning: Missing directories: {}",
                    path.display(),
                    missing_dirs.join(", ")
                ))
            }
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with(
        efees_api_url: Option<&str>,
        efees_api_key: Option<&str>,
    ) -> AppSettingsPublic {
        AppSettingsPublic {
            surrealdb_url: None,
            surrealdb_ns: None,
            surrealdb_db: None,
            surrealdb_user: None,
            has_password: false,
            staff_name: None,
            staff_email: None,
            staff_phone: None,
            staff_position: None,
            project_folder_path: None,
            dev_mode: None,
            log_level: None,
            scope_api_url: None,
            scope_api_key: None,
            efees_api_url: efees_api_url.map(str::to_string),
            efees_api_key: efees_api_key.map(str::to_string),
        }
    }

    #[test]
    fn rescan_configured_when_both_url_and_key_present() {
        let settings = settings_with(Some("http://10.0.21.80:3200"), Some("secret-key"));
        assert!(efees_rescan_configured(&settings));
    }

    #[test]
    fn rescan_not_configured_when_url_missing() {
        let settings = settings_with(None, Some("secret-key"));
        assert!(!efees_rescan_configured(&settings));
    }

    #[test]
    fn rescan_not_configured_when_key_missing() {
        let settings = settings_with(Some("http://10.0.21.80:3200"), None);
        assert!(!efees_rescan_configured(&settings));
    }

    #[test]
    fn rescan_not_configured_when_both_missing() {
        let settings = settings_with(None, None);
        assert!(!efees_rescan_configured(&settings));
    }

    #[test]
    fn rescan_not_configured_when_values_are_blank_strings() {
        // A saved-but-empty settings field (e.g. cleared in the Settings
        // panel) must be treated the same as "not configured", not as a
        // real empty-string URL/key.
        let settings = settings_with(Some("   "), Some(""));
        assert!(!efees_rescan_configured(&settings));
    }

    #[test]
    fn project_key_converts_hyphens_to_underscores() {
        assert_eq!(project_key_from_number("26-97110"), "26_97110");
    }

    #[test]
    fn project_key_is_idempotent_on_an_already_underscored_key() {
        // Defensive: if a caller ever passes the record-key form by mistake,
        // this must not corrupt it further.
        assert_eq!(project_key_from_number("26_97110"), "26_97110");
    }

    #[test]
    fn project_key_only_replaces_hyphens_leaves_the_rest_untouched() {
        assert_eq!(project_key_from_number("00-00001"), "00_00001");
    }
}
