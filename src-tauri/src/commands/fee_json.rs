//! Fee JSON export operations.
//!
//! This module provides helper functions for exporting fee proposal data to JSON files.
//! Extracted from mod.rs to improve code organization and testability.

use std::fs;
use std::path::Path;
use chrono::Utc;
use log::{info, error};
use serde_json::json;
use surrealdb::sql::Thing;

use crate::db::{Fee, Project, Company, Contact};

/// Find a fee record by ID from a list of fees.
///
/// Handles SurrealDB Thing ID format variations including angle bracket wrappers.
pub fn find_fee_by_id<'a>(fees: &'a [Fee], rfp_id: &str) -> Option<&'a Fee> {
    fees.iter().find(|f| {
        if let Some(id) = &f.id {
            let db_id_clean = id.id.to_string()
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

/// Find an entity by matching its Thing ID against a reference Thing.
fn find_by_thing_id<'a, T, F>(items: &'a [T], reference: &Thing, get_id: F) -> Option<&'a T>
where
    F: Fn(&'a T) -> Option<&'a Thing>,
{
    items.iter().find(|item| {
        if let Some(id) = get_id(item) {
            reference.id.to_string() == id.id.to_string()
        } else {
            false
        }
    })
}

/// Find a project by matching its ID against a fee's project_id.
pub fn find_project_for_fee<'a>(projects: &'a [Project], fee: &Fee) -> Option<&'a Project> {
    find_by_thing_id(projects, &fee.project_id, |p| p.id.as_ref())
}

/// Find a company by matching its ID against a fee's company_id.
pub fn find_company_for_fee<'a>(companies: &'a [Company], fee: &Fee) -> Option<&'a Company> {
    find_by_thing_id(companies, &fee.company_id, |c| c.id.as_ref())
}

/// Find a contact by matching its ID against a fee's contact_id.
pub fn find_contact_for_fee<'a>(contacts: &'a [Contact], fee: &Fee) -> Option<&'a Contact> {
    find_by_thing_id(contacts, &fee.contact_id, |c| c.id.as_ref())
}

/// Format a fee issue date from YYMMDD format to "dd MMM yyyy".
pub fn format_issue_date(date_str: &str) -> String {
    if date_str.len() == 6 {
        if let (Ok(year), Ok(month), Ok(day)) = (
            date_str[0..2].parse::<i32>(),
            date_str[2..4].parse::<u32>(),
            date_str[4..6].parse::<u32>()
        ) {
            let full_year = if year >= 50 { 1900 + year } else { 2000 + year };
            if let Some(date) = chrono::NaiveDate::from_ymd_opt(full_year, month, day) {
                return date.format("%d %b %Y").to_string();
            }
        }
    }
    Utc::now().format("%d %b %Y").to_string()
}

/// Build the JSON data structure for a fee proposal export.
pub fn build_fee_json(
    fee: &Fee,
    project: &Project,
    company: &Company,
    contact: &Contact,
) -> serde_json::Value {
    let issue_date = format_issue_date(&fee.issue_date);
    let contact_name = contact.full_name.clone().unwrap_or_else(|| {
        let first = contact.first_name.clone().unwrap_or_default();
        let last = contact.last_name.clone().unwrap_or_default();
        format!("{} {}", first, last)
    });

    json!({
        "01 Document Name": fee.name.clone(),
        "02 Document Number": fee.number.clone(),
        "03 Document Release": fee.rev.to_string(),
        "04 Document Issue Date": issue_date,
        "06 Project Name": project.name.clone(),
        "07 Project Activity": fee.activity.clone(),
        "08 Project Package": fee.package.clone(),
        "09 Project Stage": project.status.clone(),
        "11 Project Area": project.area.clone(),
        "12 Project City": project.city.clone(),
        "13 Project Country": project.country.clone(),
        "21 Client Company": company.name.clone(),
        "22 Client City": company.city.clone(),
        "23 Client Country": company.country.clone(),
        "26 Contact Name": contact_name,
        "27 Contact Position": contact.position.clone().unwrap_or_default(),
        "28 Contact Phone": contact.phone.clone().unwrap_or_default(),
        "29 Contact Email": contact.email.clone().unwrap_or_default(),
        "99 Strap Line": fee.strap_line.clone()
    })
}

/// Build the file paths for fee JSON export.
pub struct FeeJsonPaths {
    pub project_dir: String,
    pub old_json_path: String,
    pub new_json_path: String,
}

/// Calculate the file paths for fee JSON export.
pub fn build_fee_json_paths(
    project_folder_path: &str,
    project: &Project,
) -> FeeJsonPaths {
    let project_number = project.number.id
        .replace("⟨", "")
        .replace("⟩", "");
    let project_name = &project.name_short;

    let project_dir = format!(
        "{}/01 RFPs/{} {}",
        project_folder_path, project_number, project_name
    );
    let old_json_path = format!(
        "{}/02 Proposal/{}-var Default Values.json",
        project_dir, project_number
    );
    let new_json_path = format!(
        "{}/02 Proposal/{}-var.json",
        project_dir, project_number
    );

    FeeJsonPaths {
        project_dir,
        old_json_path,
        new_json_path,
    }
}

/// Rename template file if needed (removes "Default Values" from filename).
///
/// Returns Ok(()) if rename succeeded or wasn't needed, Err on failure.
pub fn rename_template_file_if_needed(
    old_path: &str,
    new_path: &str,
) -> Result<(), String> {
    if Path::new(old_path).exists() && !Path::new(new_path).exists() {
        info!("Renaming template file from '{}' to '{}'", old_path, new_path);

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
            format!("Failed to rename file from '{}' to '{}': {}", old_path, new_path, e)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_issue_date_valid() {
        assert_eq!(format_issue_date("250115"), "15 Jan 2025");
        assert_eq!(format_issue_date("991231"), "31 Dec 1999");
        assert_eq!(format_issue_date("000101"), "01 Jan 2000");
    }

    #[test]
    fn test_format_issue_date_invalid() {
        // Invalid dates should return current date (we just check it doesn't panic)
        let result = format_issue_date("invalid");
        assert!(!result.is_empty());

        let result = format_issue_date("12345");
        assert!(!result.is_empty());
    }
}
