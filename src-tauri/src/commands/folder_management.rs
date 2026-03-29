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

use crate::commands::get_settings;
use log::{error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{command, AppHandle};

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
    project_number: String,
    destination: String,
) -> Result<FolderOperationResult, String> {
    info!(
        "Moving project {} from RFP to: {}",
        project_number, destination
    );
    // Validate destination — uses new domain model statuses
    match destination.as_str() {
        "current" => move_project_folder(app_handle, project_number, "awarded".to_string()).await,
        "archive" => move_project_folder(app_handle, project_number, "completed".to_string()).await,
        "inactive" => {
            move_project_folder(app_handle, project_number, "cancelled".to_string()).await
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
    project_number: String,
) -> Result<FolderOperationResult, String> {
    info!("Moving project {} to archive", project_number);
    move_project_folder(app_handle, project_number, "completed".to_string()).await
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
