//! Template and folder operations for project management.
//!
//! This module provides Tauri commands for managing project templates,
//! copying folder structures, and checking file existence.

use super::AppState;
use crate::commands::settings::get_settings;
use crate::db::types::{record_id_string, record_key_string};
use crate::db::{NewProject, Project};

use log::{error, info, warn};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, State};

// ============================================================================
// PROJECT CREATION WITH TEMPLATE
// ============================================================================

/// Create a new project with template folder copying.
///
/// This command creates a project in the database and copies the template
/// folder structure to the project's designated location.
#[tauri::command]
pub async fn create_project_with_template(
    project: NewProject,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<Project, String> {
    info!("Creating project with template: {}", project.name);
    info!("Project data: {:?}", project);

    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };

    // First create the project in database
    info!("About to create project in database...");
    match manager_clone.create_new_project(project.clone()).await {
        Ok(created_project) => {
            info!(
                "Successfully created project in database: {:?}",
                created_project.id
            );

            // Get project folder path from settings
            info!("Getting settings for project folder path...");
            let settings = get_settings(app_handle)
                .await
                .map_err(|e| format!("Failed to get settings: {}", e))?;

            info!(
                "Settings loaded - project_folder_path: {:?}",
                settings.project_folder_path
            );

            if let Some(base_path) = settings.project_folder_path {
                // Copy template folder - use cross-platform paths
                let base_path_buf = PathBuf::from(&base_path);
                let template_path = base_path_buf.join("01 RFPs").join("_yy-cccnn Project Name");
                let project_number = created_project.number.id.clone();
                let dest_folder_name = format!("{} {}", project_number, created_project.name_short);
                let dest_path = base_path_buf.join("01 RFPs").join(&dest_folder_name);

                info!(
                    "Copying template from {:?} to {:?}",
                    template_path, dest_path
                );

                // Use cross-platform folder copying
                if template_path.exists() {
                    match copy_folder_recursive(&template_path, &dest_path) {
                        Ok(()) => {
                            info!("Successfully copied template folder");

                            // Rename files within the copied folder
                            if let Err(e) = rename_template_files_cross_platform(
                                &dest_path,
                                "yy-cccnn",
                                &project_number,
                            ) {
                                error!("Failed to rename template files: {}", e);
                                // Don't fail the entire operation just because rename failed
                            }
                        }
                        Err(e) => {
                            error!("Failed to copy template folder: {}", e);
                        }
                    }
                } else {
                    warn!("Template folder not found: {:?}", template_path);
                }
            } else {
                info!("No project_folder_path configured in settings - skipping template folder creation");
            }

            Ok(created_project)
        }
        Err(e) => {
            error!("Failed to create project: {}", e);
            Err(format!("Failed to create project: {}", e))
        }
    }
}

// ============================================================================
// TEMPLATE COPYING COMMANDS
// ============================================================================

/// Copy project template folder and rename files based on project details.
#[tauri::command]
pub async fn copy_project_template(
    project_number: String,
    project_short_name: String,
    app_handle: AppHandle,
) -> Result<String, String> {
    info!(
        "Copying project template for number: {}, short name: {}",
        project_number, project_short_name
    );

    // Get project folder path from settings
    let settings = get_settings(app_handle)
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))?;

    let base_path = settings
        .project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;

    // Normalize paths for cross-platform compatibility
    let base_path_buf = PathBuf::from(&base_path);
    let template_path = base_path_buf.join("01 RFPs").join("_yy-cccnn Project Name");
    let dest_folder_name = format!("{} {}", project_number, project_short_name);
    let dest_path = base_path_buf.join("01 RFPs").join(&dest_folder_name);

    info!("Template path: {:?}", template_path);
    info!("Destination path: {:?}", dest_path);

    // Check if template folder exists
    if !template_path.exists() {
        return Err(format!("Template folder not found: {:?}", template_path));
    }

    // Check if destination already exists
    if dest_path.exists() {
        return Err(format!(
            "Destination folder already exists: {:?}",
            dest_path
        ));
    }

    // Copy template folder using cross-platform approach
    copy_folder_recursive(&template_path, &dest_path)?;

    info!("Successfully copied template folder");

    // Rename files within the copied folder
    rename_template_files_cross_platform(&dest_path, "yy-cccnn", &project_number)?;

    info!("Successfully renamed template files");

    Ok(format!(
        "Template copied successfully to: {}",
        dest_path.display()
    ))
}

/// Populate project data from fee proposal record.
#[tauri::command]
pub async fn populate_project_data(
    fp_id: String,
    project_number: String,
    project_short_name: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    info!(
        "Populating project data for FP: {}, Project: {} {}",
        fp_id, project_number, project_short_name
    );

    // Fetch FP record data from database
    info!("Fetching FP record data for ID: {}", fp_id);
    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };

    // Get FP data
    let fps = manager_clone
        .get_fees()
        .await
        .map_err(|e| format!("Failed to fetch FPs: {}", e))?;

    // Find the specific FP
    info!("Looking for FP ID: {}", fp_id);
    let fp = fps
        .iter()
        .find(|f| {
            if let Some(id) = &f.id {
                let db_id_clean = record_key_string(&id.key)
                    .trim_start_matches('⟨')
                    .trim_end_matches('⟩')
                    .to_string();
                let input_id_clean = fp_id.trim_start_matches("fee:").to_string();
                db_id_clean == input_id_clean || record_id_string(id).contains(&fp_id)
            } else {
                false
            }
        })
        .ok_or_else(|| format!("FP record not found with ID: {}", fp_id))?;

    info!("Found FP record: {}", fp.name);

    // Get settings for file paths
    let settings = get_settings(app_handle.clone())
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let base_path = settings
        .project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;

    // Construct file path
    let base_path_buf = PathBuf::from(&base_path);
    let project_dir = base_path_buf
        .join("01 RFPs")
        .join(format!("{} {}", project_number, project_short_name))
        .join("02 Proposal");
    let json_file_path = project_dir.join(format!("{}-var Default Values.json", project_number));

    info!("Looking for JSON file: {:?}", json_file_path);

    // Read existing JSON file
    if !json_file_path.exists() {
        // Try the renamed version
        let alt_path = project_dir.join(format!("{}-var.json", project_number));
        if alt_path.exists() {
            info!("Found renamed JSON file: {:?}", alt_path);
        } else {
            return Err(format!("JSON file not found: {:?}", json_file_path));
        }
    }

    // Read and parse JSON
    let json_content = fs::read_to_string(&json_file_path)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;
    let mut json_data: Value =
        serde_json::from_str(&json_content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Update JSON fields with FP data
    if let Some(map) = json_data.as_object_mut() {
        map.insert(
            "02 Document Number".to_string(),
            Value::String(format!("{}-FP", project_number)),
        );
        map.insert(
            "06 Project Name".to_string(),
            Value::String(fp.name.clone()),
        );

        let status_str = format!("{:?}", fp.status);
        map.insert("09 Project Stage".to_string(), Value::String(status_str));

        info!(
            "Successfully populated {} JSON fields with FP data: {}",
            map.len(),
            fp.name
        );
    }

    // Write updated JSON back to file
    let updated_json = serde_json::to_string_pretty(&json_data)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    fs::write(&json_file_path, updated_json)
        .map_err(|e| format!("Failed to write JSON file: {}", e))?;

    info!("Successfully updated JSON file with RFP data");
    Ok(format!(
        "JSON file updated: {:?}",
        json_file_path.file_name().unwrap_or_default()
    ))
}

// ============================================================================
// FOLDER CHECK COMMANDS
// ============================================================================

/// Check if a project folder already exists.
#[tauri::command]
pub async fn check_project_folder_exists(
    project_number: String,
    project_short_name: String,
    app_handle: AppHandle,
) -> Result<bool, String> {
    info!(
        "Checking if project folder exists for: {} {}",
        project_number, project_short_name
    );

    let settings = get_settings(app_handle)
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))?;

    let base_path = settings
        .project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;

    let base_path_buf = PathBuf::from(&base_path);
    let dest_folder_name = format!("{} {}", project_number, project_short_name);
    let dest_path = base_path_buf.join("01 RFPs").join(&dest_folder_name);

    let exists = dest_path.exists();
    info!(
        "Project folder '{}' exists: {}",
        dest_path.display(),
        exists
    );

    Ok(exists)
}

/// Check if a var.json file already exists in a project folder.
#[tauri::command]
pub async fn check_var_json_exists(
    project_number: String,
    project_short_name: String,
    app_handle: AppHandle,
) -> Result<bool, String> {
    info!(
        "Checking if var.json exists for: {} {}",
        project_number, project_short_name
    );

    let settings = get_settings(app_handle)
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))?;

    let base_path = settings
        .project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;

    let base_path_buf = PathBuf::from(&base_path);
    let project_dir = base_path_buf
        .join("01 RFPs")
        .join(format!("{} {}", project_number, project_short_name))
        .join("02 Proposal");
    let json_file_path = project_dir.join(format!("{}-var.json", project_number));

    let exists = json_file_path.exists();
    info!(
        "var.json file '{}' exists: {}",
        json_file_path.display(),
        exists
    );

    Ok(exists)
}

/// Check if a var.json template file (with "Default Values") exists.
#[tauri::command]
pub async fn check_var_json_template_exists(
    project_number: String,
    project_short_name: String,
    app_handle: AppHandle,
) -> Result<bool, String> {
    info!(
        "Checking if var template (Default Values) exists for: {} {}",
        project_number, project_short_name
    );

    let settings = get_settings(app_handle)
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))?;

    let base_path = settings
        .project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;

    let base_path_buf = PathBuf::from(&base_path);
    let project_dir = base_path_buf
        .join("01 RFPs")
        .join(format!("{} {}", project_number, project_short_name))
        .join("02 Proposal");
    let json_template_path =
        project_dir.join(format!("{}-var Default Values.json", project_number));

    let exists = json_template_path.exists();
    info!(
        "var template file '{}' exists: {}",
        json_template_path.display(),
        exists
    );

    Ok(exists)
}

// ============================================================================
// FOLDER RENAME COMMANDS
// ============================================================================

/// Rename an existing folder with _old suffix.
#[tauri::command]
pub async fn rename_folder_with_old_suffix(
    project_number: String,
    project_short_name: String,
    app_handle: AppHandle,
) -> Result<String, String> {
    info!(
        "Renaming folder with _old suffix: {} {}",
        project_number, project_short_name
    );

    let settings = get_settings(app_handle)
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let base_path = settings
        .project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;

    let base_path_buf = PathBuf::from(&base_path);
    let original_folder_name = format!("{} {}", project_number, project_short_name);
    let original_path = base_path_buf.join("01 RFPs").join(&original_folder_name);

    if !original_path.exists() {
        return Err("Original folder does not exist".to_string());
    }

    // Create a unique _old folder name
    let mut old_suffix = "_old".to_string();
    let mut counter = 1;
    loop {
        let old_folder_name = format!("{}{}", original_folder_name, old_suffix);
        let old_path = base_path_buf.join("01 RFPs").join(&old_folder_name);

        if !old_path.exists() {
            fs::rename(&original_path, &old_path)
                .map_err(|e| format!("Failed to rename folder: {}", e))?;

            info!(
                "Renamed folder from '{}' to '{}'",
                original_folder_name, old_folder_name
            );
            return Ok(format!("Folder renamed to: {}", old_folder_name));
        }

        counter += 1;
        old_suffix = format!("_old{}", counter);
    }
}

/// Rename an existing var.json file with _old suffix.
#[tauri::command]
pub async fn rename_var_json_with_old_suffix(
    project_number: String,
    project_short_name: String,
    app_handle: AppHandle,
) -> Result<String, String> {
    info!(
        "Renaming var.json with _old suffix: {} {}",
        project_number, project_short_name
    );

    let settings = get_settings(app_handle)
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let base_path = settings
        .project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;

    let base_path_buf = PathBuf::from(&base_path);
    let project_dir = base_path_buf
        .join("01 RFPs")
        .join(format!("{} {}", project_number, project_short_name))
        .join("02 Proposal");

    let original_json_name = format!("{}-var.json", project_number);
    let original_json_path = project_dir.join(&original_json_name);

    if !original_json_path.exists() {
        return Err("Original var.json file does not exist".to_string());
    }

    // Create a unique _old file name
    let mut old_suffix = "_old".to_string();
    let mut counter = 1;
    loop {
        let old_json_name = format!("{}-var{}.json", project_number, old_suffix);
        let old_json_path = project_dir.join(&old_json_name);

        if !old_json_path.exists() {
            fs::rename(&original_json_path, &old_json_path)
                .map_err(|e| format!("Failed to rename file: {}", e))?;

            info!(
                "Renamed file from '{}' to '{}'",
                original_json_name, old_json_name
            );
            return Ok(format!("File renamed to: {}", old_json_name));
        }

        counter += 1;
        old_suffix = format!("_old{}", counter);
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Cross-platform recursive folder copying.
fn copy_folder_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    use std::process::Command;

    info!("Copying folder from {:?} to {:?}", src, dest);

    let result = if cfg!(target_os = "windows") {
        Command::new("xcopy")
            .args(&[
                src.to_string_lossy().as_ref(),
                dest.to_string_lossy().as_ref(),
                "/E",
                "/I",
                "/Q",
                "/Y",
            ])
            .output()
    } else {
        Command::new("cp")
            .args(&[
                "-R",
                src.to_string_lossy().as_ref(),
                dest.to_string_lossy().as_ref(),
            ])
            .output()
    };

    match result {
        Ok(output) => {
            if output.status.success() {
                info!("Successfully copied folder using system command");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Folder copy command failed: {}", stderr);
                Err(format!("Failed to copy folder: {}", stderr))
            }
        }
        Err(e) => {
            error!("Failed to execute copy command: {}", e);
            Err(format!("Failed to execute copy command: {}", e))
        }
    }
}

/// Cross-platform file renaming with pattern replacement.
fn rename_template_files_cross_platform(
    dir_path: &Path,
    old_pattern: &str,
    new_pattern: &str,
) -> Result<(), String> {
    info!(
        "Renaming template files in {:?}, replacing '{}' with '{}'",
        dir_path, old_pattern, new_pattern
    );

    if !dir_path.exists() {
        return Err(format!("Directory does not exist: {:?}", dir_path));
    }

    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {:?}", dir_path));
    }

    visit_dirs_cross_platform(dir_path, old_pattern, new_pattern)?;

    info!("Successfully renamed all template files");
    Ok(())
}

/// Recursive directory visitor for file renaming.
fn visit_dirs_cross_platform(
    dir: &Path,
    old_pattern: &str,
    new_pattern: &str,
) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;

    let mut entries_to_rename = Vec::new();

    // First pass: collect all entries and recurse into subdirectories
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            visit_dirs_cross_platform(&path, old_pattern, new_pattern)?;
        }

        if let Some(file_name) = path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                if file_name_str.contains(old_pattern) {
                    entries_to_rename.push(path);
                }
            }
        }
    }

    // Second pass: rename entries
    for path in entries_to_rename {
        if let Some(file_name) = path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                let new_name = file_name_str.replace(old_pattern, new_pattern);
                let new_path = path.with_file_name(new_name);

                info!("Renaming {:?} to {:?}", path, new_path);

                fs::rename(&path, &new_path)
                    .map_err(|e| format!("Failed to rename {:?} to {:?}: {}", path, new_path, e))?;

                info!("Successfully renamed to {:?}", new_path);
            }
        }
    }

    Ok(())
}
