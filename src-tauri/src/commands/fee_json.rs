//! Fee JSON export operations.
//!
//! This module provides helper functions for exporting fee proposal data to JSON files.
//! Extracted from mod.rs to improve code organization and testability.

use log::{error, info};
use std::fs;
use std::path::Path;
use surrealdb::types::RecordId;

use crate::db::types::record_key_string;
use crate::db::{Company, Contact, Fee, Project};
pub use e_fees_core::export::build_fee_json;
use e_fees_core::export::clean_record_key;

/// Find a fee record by ID from a list of fees.
///
/// Handles SurrealDB RecordId format variations including angle bracket wrappers.
pub fn find_fee_by_id<'a>(fees: &'a [Fee], rfp_id: &str) -> Option<&'a Fee> {
    fees.iter().find(|f| {
        if let Some(id) = &f.id {
            let db_id_clean = record_key_string(&id.key)
                .trim_start_matches('⟨')
                .trim_end_matches('⟩')
                .to_string();
            let input_id_clean = rfp_id.trim_start_matches("fee:").to_string();
            db_id_clean == input_id_clean
        } else {
            false
        }
    })
}

/// Find an entity by matching its RecordId key against a reference RecordId.
fn find_by_record_id<'a, T, F>(items: &'a [T], reference: &RecordId, get_id: F) -> Option<&'a T>
where
    F: Fn(&'a T) -> Option<&'a RecordId>,
{
    items.iter().find(|item| {
        if let Some(id) = get_id(item) {
            record_key_string(&reference.key) == record_key_string(&id.key)
        } else {
            false
        }
    })
}

/// Find a project by matching its ID against a fee's project_id.
pub fn find_project_for_fee<'a>(projects: &'a [Project], fee: &Fee) -> Option<&'a Project> {
    find_by_record_id(projects, &fee.project_id, |p| p.id.as_ref())
}

/// Find a company by matching its ID against a fee's company_id.
pub fn find_company_for_fee<'a>(companies: &'a [Company], fee: &Fee) -> Option<&'a Company> {
    find_by_record_id(companies, &fee.company_id, |c| c.id.as_ref())
}

/// Find a contact by matching its ID against a fee's contact_id.
pub fn find_contact_for_fee<'a>(contacts: &'a [Contact], fee: &Fee) -> Option<&'a Contact> {
    find_by_record_id(contacts, &fee.contact_id, |c| c.id.as_ref())
}

/// Build the file paths for fee JSON export.
pub struct FeeJsonPaths {
    pub project_dir: String,
    pub old_json_path: String,
    pub new_json_path: String,
}

/// Calculate the file paths for fee JSON export.
pub fn build_fee_json_paths(project_folder_path: &str, project: &Project) -> FeeJsonPaths {
    let project_number = clean_record_key(&project.number.id);

    let project_dir = format!(
        "{}/01 RFPs/{} {}",
        project_folder_path, project_number, project.name_short
    );
    let old_json_path = format!(
        "{}/02 Proposal/{}-var Default Values.json",
        project_dir, project_number
    );
    let new_json_path = format!("{}/02 Proposal/{}-var.json", project_dir, project_number);

    FeeJsonPaths {
        project_dir,
        old_json_path,
        new_json_path,
    }
}

/// Rename template file if needed (removes "Default Values" from filename).
///
/// Returns Ok(()) if rename succeeded or wasn't needed, Err on failure.
pub fn rename_template_file_if_needed(old_path: &str, new_path: &str) -> Result<(), String> {
    if Path::new(old_path).exists() && !Path::new(new_path).exists() {
        info!(
            "Renaming template file from '{}' to '{}'",
            old_path, new_path
        );

        // Check if file might be syncing
        if let Ok(metadata) = fs::metadata(old_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() < 30 {
                        info!("File was recently modified, waiting for potential sync...");
                        std::thread::sleep(std::time::Duration::from_secs(2));
                    }
                }
            }
        }

        fs::rename(old_path, new_path).map_err(|e| {
            error!("Failed to rename file: {}", e);
            format!(
                "Failed to rename file from '{}' to '{}': {}",
                old_path, new_path, e
            )
        })?;
    }
    Ok(())
}

/// Write JSON data to a file, creating parent directories if needed.
pub fn write_json_to_file(
    json_file_path: &str,
    json_data: &serde_json::Value,
) -> Result<(), String> {
    // Ensure directory exists
    if let Some(parent) = Path::new(json_file_path).parent() {
        fs::create_dir_all(parent).map_err(|e| {
            error!("Failed to create directory: {}", e);
            format!("Failed to create directory: {}", e)
        })?;
    }

    // Write JSON file
    let json_string = serde_json::to_string_pretty(json_data).map_err(|e| {
        error!("Failed to serialize JSON: {}", e);
        format!("Failed to serialize JSON: {}", e)
    })?;

    fs::write(json_file_path, json_string).map_err(|e| {
        error!("Failed to write file: {}", e);
        format!("Failed to write file: {}", e)
    })?;

    info!("Successfully wrote fee data to: {}", json_file_path);
    Ok(())
}
