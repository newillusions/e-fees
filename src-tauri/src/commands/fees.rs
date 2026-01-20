//! Fee proposal management commands.
//!
//! This module provides Tauri commands for managing fee proposals (RFPs)
//! including CRUD operations, pagination, and JSON export for InDesign.

use crate::db::{Fee, FeeCreate, FeeUpdate, PaginatedResponse};
use crate::commands::utils::execute_with_manager;
use crate::commands::fee_json;
use crate::crud_command;
use super::AppState;

use std::fs;
use std::path::Path;
use tauri::{State, AppHandle, Manager};
use log::{error, info, warn};

// ============================================================================
// FEE CRUD COMMANDS
// ============================================================================

/// Retrieve all fee proposals from the database.
crud_command!(
    get_fees,
    Vec<Fee>,
    get_fees,
    "fetch",
    "fee proposals"
);

/// Retrieve a paginated page of fees.
crud_command!(
    get_fees_page,
    PaginatedResponse<Fee>,
    get_fees_page,
    "fetch page",
    "fees",
    paginated
);

/// Create a new fee proposal in the database.
crud_command!(
    create_fee,
    Fee,
    FeeCreate,
    create_fee,
    "create",
    "fee proposal",
    data: fee
);

/// Delete a fee proposal from the database.
crud_command!(
    delete_fee,
    Fee,
    delete_fee,
    "delete",
    "fee proposal",
    id: String
);

/// Update an existing fee proposal in the database.
#[tauri::command]
pub async fn update_fee(id: String, fee: FeeUpdate, state: State<'_, AppState>) -> Result<Fee, String> {
    let fee_name = format!("fee proposal '{}'", id);
    execute_with_manager(
        &state,
        |manager| {
            let id_clone = id.clone();
            Box::pin(async move {
                manager.update_fee(&id_clone, fee).await
            })
        },
        "update",
        &fee_name
    ).await
}

// ============================================================================
// FEE JSON EXPORT COMMANDS
// ============================================================================

/// Write fee proposal data to JSON file in project folder.
///
/// This command fetches complete fee data including all linked records
/// and writes it to the project's JSON template file for InDesign integration.
#[tauri::command]
pub async fn write_fee_to_json(
    rfp_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle
) -> Result<String, String> {
    use fee_json::{
        find_fee_by_id, find_project_for_fee, find_company_for_fee, find_contact_for_fee,
        build_fee_json_paths, build_fee_json, rename_template_file_if_needed, write_json_to_file
    };

    info!("Writing fee {} to JSON file", rfp_id);

    // Get all data from database
    let (fees, projects, companies, contacts, project_folder_path) = {
        let manager = state.read().await;
        let fees = manager.get_fees().await.map_err(|e| format!("Failed to fetch fees: {}", e))?;
        let projects = manager.get_projects().await.map_err(|e| format!("Failed to fetch projects: {}", e))?;
        let companies = manager.get_companies().await.map_err(|e| format!("Failed to fetch companies: {}", e))?;
        let contacts = manager.get_contacts().await.map_err(|e| format!("Failed to fetch contacts: {}", e))?;

        // Get project folder path from settings
        let config = app_handle.config();
        let app_data_dir = app_handle.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;
        let settings_path = app_data_dir.join("settings.json");

        let project_folder_path = if settings_path.exists() {
            let settings_content = fs::read_to_string(&settings_path)
                .map_err(|e| format!("Failed to read settings: {}", e))?;
            let settings: serde_json::Value = serde_json::from_str(&settings_content)
                .map_err(|e| format!("Failed to parse settings: {}", e))?;
            settings.get("project_folder_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        (fees, projects, companies, contacts, project_folder_path)
    };

    if project_folder_path.is_empty() {
        return Err("Project folder path not configured in settings".to_string());
    }

    // Find the fee
    let fee = find_fee_by_id(&fees, &rfp_id)
        .ok_or_else(|| format!("Fee not found: {}", rfp_id))?;

    // Find related records
    let project = find_project_for_fee(&projects, fee)
        .ok_or_else(|| "Project not found for fee".to_string())?;
    let company = find_company_for_fee(&companies, fee)
        .ok_or_else(|| "Company not found for fee".to_string())?;
    let contact = find_contact_for_fee(&contacts, fee)
        .ok_or_else(|| "Contact not found for fee".to_string())?;

    // Build paths and JSON data
    let paths = build_fee_json_paths(&project_folder_path, project);
    let json_data = build_fee_json(fee, project, company, contact);

    // Rename template file if needed
    rename_template_file_if_needed(&paths.old_json_path, &paths.new_json_path)?;

    // Write JSON file
    write_json_to_file(&paths.new_json_path, &json_data)?;

    Ok(format!("Successfully wrote fee data to: {}", paths.new_json_path))
}

/// Write fee proposal data to JSON file with enhanced error handling.
///
/// This is a safer version of write_fee_to_json that provides more detailed
/// error messages and handles edge cases more gracefully.
#[tauri::command]
pub async fn write_fee_to_json_safe(
    fee_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle
) -> Result<String, String> {
    use fee_json::{
        find_fee_by_id, find_project_for_fee, find_company_for_fee, find_contact_for_fee,
        build_fee_json_paths, build_fee_json, rename_template_file_if_needed, write_json_to_file,
        format_issue_date
    };

    info!("Writing fee {} to JSON file (safe mode)", fee_id);

    // Validate fee_id format
    if fee_id.is_empty() {
        return Err("Fee ID cannot be empty".to_string());
    }

    // Get all data from database
    let (fees, projects, companies, contacts, project_folder_path) = {
        let manager = state.read().await;

        let fees = manager.get_fees().await.map_err(|e| {
            error!("Failed to fetch fees: {}", e);
            format!("Database error: Failed to fetch fees - {}", e)
        })?;

        let projects = manager.get_projects().await.map_err(|e| {
            error!("Failed to fetch projects: {}", e);
            format!("Database error: Failed to fetch projects - {}", e)
        })?;

        let companies = manager.get_companies().await.map_err(|e| {
            error!("Failed to fetch companies: {}", e);
            format!("Database error: Failed to fetch companies - {}", e)
        })?;

        let contacts = manager.get_contacts().await.map_err(|e| {
            error!("Failed to fetch contacts: {}", e);
            format!("Database error: Failed to fetch contacts - {}", e)
        })?;

        // Get project folder path from settings
        let app_data_dir = app_handle.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;
        let settings_path = app_data_dir.join("settings.json");

        let project_folder_path = if settings_path.exists() {
            let settings_content = fs::read_to_string(&settings_path)
                .map_err(|e| format!("Failed to read settings: {}", e))?;
            let settings: serde_json::Value = serde_json::from_str(&settings_content)
                .map_err(|e| format!("Failed to parse settings: {}", e))?;
            settings.get("project_folder_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        (fees, projects, companies, contacts, project_folder_path)
    };

    if project_folder_path.is_empty() {
        return Err("Configuration error: Project folder path not set. Please configure in Settings.".to_string());
    }

    // Verify project folder exists
    if !Path::new(&project_folder_path).exists() {
        return Err(format!(
            "Configuration error: Project folder does not exist: {}",
            project_folder_path
        ));
    }

    // Find the fee
    let fee = find_fee_by_id(&fees, &fee_id).ok_or_else(|| {
        error!("Fee not found: {}", fee_id);
        format!("Fee not found with ID: {}. It may have been deleted.", fee_id)
    })?;

    // Find related records with detailed error messages
    let project = find_project_for_fee(&projects, fee).ok_or_else(|| {
        error!("Project not found for fee: {}", fee_id);
        "Data integrity error: The project linked to this fee no longer exists.".to_string()
    })?;

    let company = find_company_for_fee(&companies, fee).ok_or_else(|| {
        error!("Company not found for fee: {}", fee_id);
        "Data integrity error: The company linked to this fee no longer exists.".to_string()
    })?;

    let contact = find_contact_for_fee(&contacts, fee).ok_or_else(|| {
        error!("Contact not found for fee: {}", fee_id);
        "Data integrity error: The contact linked to this fee no longer exists.".to_string()
    })?;

    // Build paths
    let paths = build_fee_json_paths(&project_folder_path, project);

    // Check if project directory exists
    if !Path::new(&paths.old_json_path).parent().map(|p| p.exists()).unwrap_or(false) {
        warn!("Project directory does not exist, attempting to create: {}", paths.old_json_path);
    }

    // Build JSON data
    let json_data = build_fee_json(fee, project, company, contact);

    // Check for placeholder content
    if check_placeholder_content(&json_data) {
        warn!("Fee data contains placeholder content that should be updated");
    }

    // Rename template file if needed
    if let Err(e) = rename_template_file_if_needed(&paths.old_json_path, &paths.new_json_path) {
        warn!("Could not rename template file (may not exist): {}", e);
    }

    // Write JSON file
    write_json_to_file(&paths.new_json_path, &json_data)?;

    info!("Successfully wrote fee data to: {}", paths.new_json_path);
    Ok(format!("Fee data exported successfully to:\n{}", paths.new_json_path))
}

/// Check if JSON data contains placeholder content that should be updated.
fn check_placeholder_content(data: &serde_json::Value) -> bool {
    let placeholder_indicators = [
        "TBD", "TBC", "XXX", "TODO", "PLACEHOLDER",
        "[Project Name]", "[Company]", "[Contact]",
    ];

    if let Some(obj) = data.as_object() {
        for (_, value) in obj {
            if let Some(s) = value.as_str() {
                let upper = s.to_uppercase();
                for indicator in &placeholder_indicators {
                    if upper.contains(indicator) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
