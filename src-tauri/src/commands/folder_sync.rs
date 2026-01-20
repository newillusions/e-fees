//! Folder Sync Commands
//!
//! This module provides functionality to detect and resolve inconsistencies between
//! the database project statuses and the actual file system folder locations.
//!
//! ## Features
//! - Scan all status folders to find project folders
//! - Compare with database expected locations
//! - Detect wrong locations, missing folders, orphan folders, and duplicates
//! - Resolve inconsistencies by updating DB or moving folders
//!
//! ## Status-to-Folder Mapping
//! - `rfp` → `01 RFPs`
//! - `active` → `11 Current`
//! - `completed` → `99 Completed`
//! - `cancelled` → `00 Inactive`

use tauri::{command, AppHandle, State};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use log::{info, error, warn};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use regex::Regex;
use once_cell::sync::Lazy;

use crate::commands::AppState;

// ============================================================================
// STATIC REGEX PATTERNS (compiled once at startup)
// ============================================================================

/// Pattern to match project folder names: YY-NNNNN followed by space and project name
/// Examples: "25-97108 RAK Beach District", "24-96601 Some Project"
static PROJECT_FOLDER_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d{2}-\d{5}\s+.+")
        .expect("Invalid PROJECT_FOLDER_PATTERN regex - this is a compile-time constant")
});

/// Pattern to extract project number from folder name
static PROJECT_NUMBER_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d{2}-\d{5})")
        .expect("Invalid PROJECT_NUMBER_PATTERN regex - this is a compile-time constant")
});

/// Pattern to extract project name from folder name (after number and space)
static PROJECT_NAME_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d{2}-\d{5}\s+(.+)$")
        .expect("Invalid PROJECT_NAME_PATTERN regex - this is a compile-time constant")
});

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Type of inconsistency detected between DB and file system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InconsistencyType {
    /// Folder exists but in different status folder than expected
    WrongLocation,
    /// No folder found anywhere on disk
    Missing,
    /// Folder exists but no matching DB record
    Orphan,
    /// Same project folder exists in multiple locations
    Duplicate,
}

/// Represents a single folder/DB inconsistency
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderInconsistency {
    /// Database project ID (empty string for orphans)
    pub project_id: String,
    /// Project number, e.g., "25-97108"
    pub project_number: String,
    /// Project name from DB or parsed from folder
    pub project_name: String,
    /// Full folder name, e.g., "25-97108 RAK Beach District"
    pub folder_name: String,
    /// Type of inconsistency detected
    #[serde(rename = "type")]
    pub inconsistency_type: InconsistencyType,
    /// Expected status from database (null for orphans)
    pub db_status: Option<String>,
    /// Expected folder path based on DB status (null for orphans)
    pub expected_path: Option<String>,
    /// Actual path where folder was found (null if missing)
    pub actual_path: Option<String>,
    /// Status inferred from actual folder location (null if missing)
    pub actual_status: Option<String>,
    /// All paths where folder was found (only for duplicate type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_paths: Option<Vec<String>>,
}

/// Result of a folder sync scan operation
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderSyncResult {
    /// ISO timestamp when scan was performed
    pub scanned_at: String,
    /// Base path that was scanned
    pub base_path: String,
    /// Total number of projects in database
    pub total_projects: u32,
    /// Total number of project folders found on disk
    pub total_folders: u32,
    /// List of detected inconsistencies
    pub inconsistencies: Vec<FolderInconsistency>,
    /// Any errors encountered during scan
    pub errors: Vec<String>,
}

/// Request payload for resolving an inconsistency
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionRequest {
    /// Action type: 'update_db', 'move_folder', 'create_folder', 'ignore'
    pub action: String,
    /// Project ID for update_db and create_folder actions
    pub project_id: Option<String>,
    /// New status for update_db action
    pub new_status: Option<String>,
    /// Source path for move_folder action
    pub from_path: Option<String>,
    /// Destination path for move_folder action
    pub to_path: Option<String>,
}

/// Response from resolution operation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionResponse {
    pub success: bool,
    pub message: String,
}

// ============================================================================
// STATUS FOLDER MAPPING
// ============================================================================

/// All status folders to scan
const STATUS_FOLDERS: [&str; 4] = ["00 Inactive", "01 RFPs", "11 Current", "99 Completed"];

/// Get the status folder name for a given project status
fn get_folder_for_status(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "draft" | "rfp" | "proposal" | "submitted" => "01 RFPs",
        "active" | "current" | "awarded" | "ongoing" => "11 Current",
        "completed" | "finished" | "delivered" => "99 Completed",
        "cancelled" | "canceled" | "inactive" | "lost" | "on hold" => "00 Inactive",
        _ => "01 RFPs" // Default to RFPs
    }
}

/// Get the status from a folder name
fn get_status_from_folder(folder: &str) -> Option<&'static str> {
    match folder {
        "01 RFPs" => Some("rfp"),
        "11 Current" => Some("active"),
        "99 Completed" => Some("completed"),
        "00 Inactive" => Some("cancelled"),
        _ => None
    }
}

/// Check if a folder name matches the project number pattern (YY-NNNNN)
fn is_project_folder(folder_name: &str) -> bool {
    // Pattern: 2 digits, hyphen, 5 digits, space, then project name
    // Examples: "25-97108 RAK Beach District", "24-96601 Some Project"
    PROJECT_FOLDER_PATTERN.is_match(folder_name)
}

/// Extract project number from folder name
fn extract_project_number(folder_name: &str) -> Option<String> {
    PROJECT_NUMBER_PATTERN.captures(folder_name)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Extract project name from folder name (after the number and space)
fn extract_project_name(folder_name: &str) -> String {
    PROJECT_NAME_PATTERN.captures(folder_name)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| folder_name.to_string())
}

// ============================================================================
// DATABASE HELPERS
// ============================================================================

/// Project info from database (minimal fields needed)
#[derive(Debug, Clone)]
struct DbProjectInfo {
    id: String,
    number: String,
    name: String,
    name_short: Option<String>,
    status: String,
    folder: Option<String>,
}

/// Get all projects from the database
async fn get_projects_from_db(state: &State<'_, AppState>) -> Result<Vec<DbProjectInfo>, String> {
    // Clone the manager to avoid holding lock across await
    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    }; // Read lock is dropped here

    // Get all projects using the existing get_projects method
    let projects = manager_clone.get_projects()
        .await
        .map_err(|e| format!("Failed to get projects: {}", e))?;

    Ok(projects.iter().map(|p| {
        // Extract ID properly - use id.id.to_string() and strip the special ⟨⟩ characters
        // that SurrealDB uses for string IDs
        let id_str = match &p.id {
            Some(id) => id.id.to_string()
                .trim_start_matches('⟨')
                .trim_end_matches('⟩')
                .to_string(),
            None => String::new(),
        };

        DbProjectInfo {
            id: id_str,
            number: p.number.id.clone(),
            name: p.name.clone(),
            name_short: if p.name_short.is_empty() { None } else { Some(p.name_short.clone()) },
            status: p.status.clone(),
            folder: if p.folder.is_empty() { None } else { Some(p.folder.clone()) },
        }
    }).collect())
}

// ============================================================================
// SCAN COMMAND
// ============================================================================

/// Scan the file system and compare with database to find inconsistencies
///
/// This command:
/// 1. Gets all projects from the database
/// 2. Scans all status folders for project folders
/// 3. Compares expected vs actual locations
/// 4. Returns a list of inconsistencies
#[command]
pub async fn scan_folder_sync(
    base_path: String,
    state: State<'_, AppState>,
) -> Result<FolderSyncResult, String> {
    info!("Starting folder sync scan at: {}", base_path);

    let base = Path::new(&base_path);
    if !base.exists() {
        return Err(format!("Base path does not exist: {}", base_path));
    }

    let mut errors: Vec<String> = Vec::new();
    let mut inconsistencies: Vec<FolderInconsistency> = Vec::new();

    // Get all projects from DB
    let db_projects = match get_projects_from_db(&state).await {
        Ok(projects) => projects,
        Err(e) => {
            return Err(format!("Failed to get projects from database: {}", e));
        }
    };
    let total_projects = db_projects.len() as u32;
    info!("Found {} projects in database", total_projects);

    // Build a map of project_number -> DbProjectInfo
    let mut db_map: HashMap<String, DbProjectInfo> = HashMap::new();
    for project in &db_projects {
        if !project.number.is_empty() {
            db_map.insert(project.number.clone(), project.clone());
        }
    }

    // Scan all status folders and build a map of folder_name -> list of paths
    let mut folder_paths: HashMap<String, Vec<(String, String)>> = HashMap::new(); // folder_name -> [(full_path, status_folder)]
    let mut total_folders: u32 = 0;

    for status_folder in STATUS_FOLDERS.iter() {
        let status_path = base.join(status_folder);

        if !status_path.exists() {
            info!("Status folder does not exist, skipping: {:?}", status_path);
            continue;
        }

        match fs::read_dir(&status_path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let folder_name = entry.file_name().to_string_lossy().to_string();

                    // Skip non-project folders
                    if !is_project_folder(&folder_name) {
                        continue;
                    }

                    total_folders += 1;
                    let full_path = entry.path().to_string_lossy().to_string();

                    folder_paths
                        .entry(folder_name.clone())
                        .or_insert_with(Vec::new)
                        .push((full_path, status_folder.to_string()));
                }
            }
            Err(e) => {
                let err_msg = format!("Failed to read {}: {}", status_path.display(), e);
                warn!("{}", err_msg);
                errors.push(err_msg);
            }
        }
    }

    info!("Found {} project folders on disk", total_folders);

    // Check for duplicates first
    for (folder_name, paths) in &folder_paths {
        if paths.len() > 1 {
            let project_number = extract_project_number(folder_name).unwrap_or_default();
            let project_name = extract_project_name(folder_name);

            let db_project = db_map.get(&project_number);

            inconsistencies.push(FolderInconsistency {
                project_id: db_project.map(|p| p.id.clone()).unwrap_or_default(),
                project_number: project_number.clone(),
                project_name,
                folder_name: folder_name.clone(),
                inconsistency_type: InconsistencyType::Duplicate,
                db_status: db_project.map(|p| p.status.clone()),
                expected_path: db_project.map(|p| {
                    let expected_folder = get_folder_for_status(&p.status);
                    format!("{}/{}/{}", base_path, expected_folder, folder_name)
                }),
                actual_path: Some(paths[0].0.clone()), // First occurrence
                actual_status: get_status_from_folder(&paths[0].1).map(|s| s.to_string()),
                duplicate_paths: Some(paths.iter().map(|(p, _)| p.clone()).collect()),
            });
        }
    }

    // Check each DB project for missing or wrong location
    for project in &db_projects {
        if project.number.is_empty() {
            continue;
        }

        let expected_folder = project.folder.clone()
            .unwrap_or_else(|| format!("{} {}", project.number, project.name_short.as_ref().unwrap_or(&project.name)));

        let expected_status_folder = get_folder_for_status(&project.status);
        let expected_path = format!("{}/{}/{}", base_path, expected_status_folder, expected_folder);

        // Find if folder exists anywhere
        let found_paths: Vec<_> = folder_paths.iter()
            .filter(|(name, _)| {
                extract_project_number(name).as_ref() == Some(&project.number)
            })
            .flat_map(|(name, paths)| {
                paths.iter().map(move |(path, status)| (name.clone(), path.clone(), status.clone()))
            })
            .collect();

        if found_paths.is_empty() {
            // Missing folder
            inconsistencies.push(FolderInconsistency {
                project_id: project.id.clone(),
                project_number: project.number.clone(),
                project_name: project.name_short.clone().unwrap_or_else(|| project.name.clone()),
                folder_name: expected_folder,
                inconsistency_type: InconsistencyType::Missing,
                db_status: Some(project.status.clone()),
                expected_path: Some(expected_path),
                actual_path: None,
                actual_status: None,
                duplicate_paths: None,
            });
        } else if found_paths.len() == 1 {
            // Check if in correct location
            let (folder_name, actual_path, status_folder) = &found_paths[0];
            let actual_status = get_status_from_folder(status_folder);

            // Compare expected vs actual status folder
            if status_folder != expected_status_folder {
                inconsistencies.push(FolderInconsistency {
                    project_id: project.id.clone(),
                    project_number: project.number.clone(),
                    project_name: project.name_short.clone().unwrap_or_else(|| project.name.clone()),
                    folder_name: folder_name.clone(),
                    inconsistency_type: InconsistencyType::WrongLocation,
                    db_status: Some(project.status.clone()),
                    expected_path: Some(expected_path),
                    actual_path: Some(actual_path.clone()),
                    actual_status: actual_status.map(|s| s.to_string()),
                    duplicate_paths: None,
                });
            }
        }
        // Duplicates are already handled above
    }

    // Check for orphan folders (folders on disk with no DB record)
    let db_numbers: std::collections::HashSet<_> = db_projects.iter()
        .filter(|p| !p.number.is_empty())
        .map(|p| p.number.clone())
        .collect();

    for (folder_name, paths) in &folder_paths {
        if paths.len() > 1 {
            continue; // Already reported as duplicate
        }

        let project_number = match extract_project_number(folder_name) {
            Some(num) => num,
            None => continue,
        };

        if !db_numbers.contains(&project_number) {
            let (actual_path, status_folder) = &paths[0];

            inconsistencies.push(FolderInconsistency {
                project_id: String::new(),
                project_number: project_number.clone(),
                project_name: extract_project_name(folder_name),
                folder_name: folder_name.clone(),
                inconsistency_type: InconsistencyType::Orphan,
                db_status: None,
                expected_path: None,
                actual_path: Some(actual_path.clone()),
                actual_status: get_status_from_folder(status_folder).map(|s| s.to_string()),
                duplicate_paths: None,
            });
        }
    }

    info!("Scan complete. Found {} inconsistencies", inconsistencies.len());

    Ok(FolderSyncResult {
        scanned_at: Utc::now().to_rfc3339(),
        base_path,
        total_projects,
        total_folders,
        inconsistencies,
        errors,
    })
}

// ============================================================================
// RESOLUTION COMMAND
// ============================================================================

/// Resolve a folder inconsistency
///
/// Actions supported:
/// - `update_db`: Update project status in database to match folder location
/// - `move_folder`: Move folder to expected location based on DB status
/// - `create_folder`: Create missing folder using project template
/// - `ignore`: Do nothing (mark as intentionally ignored)
#[command]
pub async fn resolve_folder_inconsistency(
    resolution: ResolutionRequest,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<ResolutionResponse, String> {
    info!("Resolving folder inconsistency: action={}", resolution.action);

    match resolution.action.as_str() {
        "update_db" => {
            let project_id = resolution.project_id
                .ok_or("project_id is required for update_db action")?;
            let new_status = resolution.new_status
                .ok_or("new_status is required for update_db action")?;

            info!("Attempting to update project ID '{}' to status '{}'", project_id, new_status);

            // Clone the manager to avoid holding lock across await
            let manager_clone = {
                let manager = state.read().await;
                manager.clone()
            }; // Read lock is dropped here

            // Use safe update_project_status function with input validation
            manager_clone.update_project_status(&project_id, &new_status)
                .await
                .map_err(|e| format!("Failed to update project status: {}", e))?;

            info!("Successfully updated project {} status to {}", project_id, new_status);
            Ok(ResolutionResponse {
                success: true,
                message: format!("Project status updated to {}", new_status),
            })
        }

        "move_folder" => {
            let from_path = resolution.from_path
                .ok_or("from_path is required for move_folder action")?;
            let to_path = resolution.to_path
                .ok_or("to_path is required for move_folder action")?;

            let from = Path::new(&from_path);
            let to = Path::new(&to_path);

            if !from.exists() {
                return Err(format!("Source folder does not exist: {}", from_path));
            }

            if to.exists() {
                return Err(format!("Destination folder already exists: {}", to_path));
            }

            // Ensure parent directory exists
            if let Some(parent) = to.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
            }

            // Try rename first (faster if on same filesystem)
            match fs::rename(from, to) {
                Ok(_) => {
                    info!("Moved folder from {} to {}", from_path, to_path);
                    Ok(ResolutionResponse {
                        success: true,
                        message: "Folder moved successfully".to_string(),
                    })
                }
                Err(e) => {
                    // If rename fails (cross-filesystem), use copy+delete
                    warn!("Rename failed, attempting copy+delete: {}", e);

                    let options = fs_extra::dir::CopyOptions::new();
                    let parent_dir = to.parent()
                        .ok_or_else(|| "Destination path has no parent directory".to_string())?;
                    fs_extra::dir::copy(from, parent_dir, &options)
                        .map_err(|e| format!("Failed to copy folder: {}", e))?;

                    fs::remove_dir_all(from)
                        .map_err(|e| format!("Failed to remove original folder: {}", e))?;

                    info!("Moved folder (via copy+delete) from {} to {}", from_path, to_path);
                    Ok(ResolutionResponse {
                        success: true,
                        message: "Folder moved successfully".to_string(),
                    })
                }
            }
        }

        "create_folder" => {
            // For create_folder, we expect the project_number and project_name
            // to be passed directly since the frontend already has this info
            let project_number = resolution.project_id
                .ok_or("project_number is required for create_folder action")?;
            let project_name = resolution.new_status
                .ok_or("project_name (in new_status field) is required for create_folder action")?;

            if project_number.is_empty() {
                return Err("Project number cannot be empty".to_string());
            }

            // Use the existing copy_project_template command
            crate::commands::copy_project_template(
                project_number.clone(),
                project_name.clone(),
                app_handle.clone(),
            ).await?;

            info!("Created folder for project {} {}", project_number, project_name);
            Ok(ResolutionResponse {
                success: true,
                message: format!("Folder created for {} {}", project_number, project_name),
            })
        }

        "ignore" => {
            info!("Ignoring inconsistency as requested");
            Ok(ResolutionResponse {
                success: true,
                message: "Inconsistency ignored".to_string(),
            })
        }

        _ => Err(format!("Unknown action: {}", resolution.action)),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_project_folder() {
        assert!(is_project_folder("25-97108 RAK Beach District"));
        assert!(is_project_folder("24-96601 Some Project Name"));
        assert!(!is_project_folder("Random Folder"));
        assert!(!is_project_folder("25-971 Short Number"));
        assert!(!is_project_folder("2597108 No Hyphen"));
    }

    #[test]
    fn test_extract_project_number() {
        assert_eq!(
            extract_project_number("25-97108 RAK Beach District"),
            Some("25-97108".to_string())
        );
        assert_eq!(
            extract_project_number("24-96601 Some Project"),
            Some("24-96601".to_string())
        );
        assert_eq!(extract_project_number("Random Folder"), None);
    }

    #[test]
    fn test_extract_project_name() {
        assert_eq!(
            extract_project_name("25-97108 RAK Beach District"),
            "RAK Beach District"
        );
        assert_eq!(
            extract_project_name("24-96601 Some Project Name"),
            "Some Project Name"
        );
    }

    #[test]
    fn test_get_folder_for_status() {
        assert_eq!(get_folder_for_status("rfp"), "01 RFPs");
        assert_eq!(get_folder_for_status("RFP"), "01 RFPs");
        assert_eq!(get_folder_for_status("active"), "11 Current");
        assert_eq!(get_folder_for_status("Active"), "11 Current");
        assert_eq!(get_folder_for_status("completed"), "99 Completed");
        assert_eq!(get_folder_for_status("cancelled"), "00 Inactive");
        assert_eq!(get_folder_for_status("unknown"), "01 RFPs");
    }

    #[test]
    fn test_get_status_from_folder() {
        assert_eq!(get_status_from_folder("01 RFPs"), Some("rfp"));
        assert_eq!(get_status_from_folder("11 Current"), Some("active"));
        assert_eq!(get_status_from_folder("99 Completed"), Some("completed"));
        assert_eq!(get_status_from_folder("00 Inactive"), Some("cancelled"));
        assert_eq!(get_status_from_folder("Unknown"), None);
    }
}
