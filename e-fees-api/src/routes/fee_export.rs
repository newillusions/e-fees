//! Fee JSON export and status endpoints.
//!
//! Builds InDesign variable data JSON from a fee proposal and its linked
//! project/company/contact records. Export writes the JSON to the Nextcloud
//! project folder via SSH; status is a read-only field-by-field check.

use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use chrono::Utc;
use serde_json::{json, Value};
use tracing::{info, warn};

use e_fees_core::export::{build_fee_json, clean_number_for_path, is_placeholder};
use e_fees_core::models::{record_id_string, Company, Contact, Fee, Project};

use crate::error::ApiError;
use crate::ssh::SshOps;
use crate::validation::validate_id;
use crate::AppState;

// ============================================================================
// SHARED HELPER
// ============================================================================

/// Fetch a fee and its three linked entities (project, company, contact).
///
/// Returns 404 with a descriptive message if any record is missing.
async fn fetch_fee_with_links(
    state: &Arc<AppState>,
    fee_id: &str,
) -> Result<(Fee, Project, Company, Contact), ApiError> {
    // Fetch fee (table is "fee", not "fees")
    let fee: Option<Fee> = state.db.select(("fee", fee_id)).await?;
    let fee = fee.ok_or_else(|| ApiError::not_found("Fee", fee_id))?;

    // Extract linked record keys by formatting as "table:key" then splitting on ":"
    // record_id_string returns "projects:26_97101" etc.
    let project_key = {
        let s = record_id_string(&fee.project_id);
        s.splitn(2, ':').nth(1).unwrap_or("").to_string()
    };
    let company_key = {
        let s = record_id_string(&fee.company_id);
        s.splitn(2, ':').nth(1).unwrap_or("").to_string()
    };
    let contact_key = {
        let s = record_id_string(&fee.contact_id);
        s.splitn(2, ':').nth(1).unwrap_or("").to_string()
    };

    let project: Option<Project> = state.db.select(("projects", &*project_key)).await?;
    let project = project
        .ok_or_else(|| ApiError::not_found("Project", &format!("projects:{}", project_key)))?;

    let company: Option<Company> = state.db.select(("company", &*company_key)).await?;
    let company = company
        .ok_or_else(|| ApiError::not_found("Company", &format!("company:{}", company_key)))?;

    let contact: Option<Contact> = state.db.select(("contacts", &*contact_key)).await?;
    let contact = contact
        .ok_or_else(|| ApiError::not_found("Contact", &format!("contacts:{}", contact_key)))?;

    Ok((fee, project, company, contact))
}

// ============================================================================
// EXPORT ENDPOINT
// ============================================================================

/// Export fee JSON to the Nextcloud project folder.
///
/// Fetches the fee and its linked project/company/contact, builds the
/// 23-field InDesign variable data JSON, and writes it to:
/// `{nc_base_path}/01 RFPs/{number} {name}/02 Proposal/{number}-var.json`
///
/// If a template file (`{number}-var Default Values.json`) exists and the
/// var.json does not yet exist, the template is renamed to var.json first.
/// If var.json already exists, it is archived before overwriting.
#[utoipa::path(
    post,
    path = "/fees/{id}/json-export",
    tag = "Fees",
    params(("id" = String, Path, description = "Fee record key (fee: prefix stripped if present)")),
    responses(
        (status = 200, description = "JSON exported to Nextcloud folder"),
        (status = 404, description = "Fee, project, company, or contact not found", body = crate::schemas::ErrorResponse),
        (status = 503, description = "Folder config not set or SSH error", body = crate::schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn export_fee_json(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("fee:").unwrap_or(&id);
    validate_id(key)?;

    let (fee, project, company, contact) = fetch_fee_with_links(&state, key).await?;

    // Build the InDesign JSON
    let json_data = build_fee_json(&fee, &project, &company, &contact);

    // Require folder config
    let folder_config = state.folder_config.as_ref().ok_or_else(|| {
        ApiError::service_unavailable("Folder export not configured (NC_SSH_HOST not set)")
    })?;

    // Build filesystem paths
    let number = clean_number_for_path(&project.number.id);
    let name = &project.name;
    let proposal_dir = format!(
        "{}/01 RFPs/{} {}/02 Proposal",
        folder_config.nc_base_path, number, name
    );
    let var_path = format!("{}/{}-var.json", proposal_dir, number);
    let template_path = format!("{}/{}-var Default Values.json", proposal_dir, number);

    let ssh = SshOps::from_folder_config(folder_config);

    // Ensure the proposal directory exists
    ssh.mkdir_p(&proposal_dir).await?;

    // Archive logic:
    // 1. If template file exists and var.json does not → rename template to var.json
    // 2. If var.json already exists → archive it before overwriting
    let mut archived_previous = false;

    let var_exists = ssh.path_exists(&var_path).await?;
    let template_exists = ssh.path_exists(&template_path).await?;

    if !var_exists && template_exists {
        // First export: rename template placeholder to var.json
        ssh.rename(&template_path, &var_path).await?;
        info!("Renamed template to var.json for {}", number);
    } else if var_exists {
        // Archive the existing var.json before overwriting
        let timestamp = Utc::now().format("%Y%m%dT%H%M%S").to_string();
        let archive_dir = format!("{}/00 Archive", proposal_dir);
        ssh.mkdir_p(&archive_dir).await?;
        let archive_path = format!("{}/{}-var-{}.json", archive_dir, number, timestamp);
        ssh.rename(&var_path, &archive_path).await?;
        archived_previous = true;
        info!(
            "Archived previous var.json for {} to {}",
            number, archive_path
        );
    }

    // Serialize and write the new JSON
    let json_bytes = serde_json::to_string_pretty(&json_data)
        .map_err(|e| ApiError::service_unavailable(format!("JSON serialization failed: {}", e)))?;
    ssh.write_file(&var_path, json_bytes.as_bytes()).await?;

    // Count populated fields for response
    let fields_populated = json_data
        .as_object()
        .map(|obj| {
            obj.values()
                .filter(|v| v.as_str().map(|s| !is_placeholder(s)).unwrap_or(false))
                .count()
        })
        .unwrap_or(0);

    // Best-effort NC rescan — don't fail the request on rescan errors
    let rescan_subpath = format!(
        "/emittiv/nc/__groupfolders/1/01 Projects/01 RFPs/{} {}/02 Proposal",
        number, name
    );
    if let Err(e) = ssh.nc_rescan(&rescan_subpath).await {
        warn!("NC rescan failed (non-fatal): {}", e.message);
    }

    let fee_id_str = format!("fee:{}", key);
    let relative_path = format!(
        "01 RFPs/{} {}/02 Proposal/{}-var.json",
        number, name, number
    );

    info!(
        "Exported fee JSON for {} to {} ({} fields populated)",
        fee_id_str, var_path, fields_populated
    );

    Ok(Json(json!({
        "status": "exported",
        "fee_id": fee_id_str,
        "path": relative_path,
        "fields_populated": fields_populated,
        "archived_previous": archived_previous,
    })))
}

// ============================================================================
// STATUS ENDPOINT
// ============================================================================

/// Check which fields in the fee JSON export are populated vs placeholder.
///
/// Read-only: builds the JSON in memory, classifies each field, and returns
/// counts and per-field status. Does not write to the filesystem.
#[utoipa::path(
    get,
    path = "/fees/{id}/json-status",
    tag = "Fees",
    params(("id" = String, Path, description = "Fee record key (fee: prefix stripped if present)")),
    responses(
        (status = 200, description = "Field population status"),
        (status = 404, description = "Fee, project, company, or contact not found", body = crate::schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn fee_json_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("fee:").unwrap_or(&id);
    validate_id(key)?;

    let (fee, project, company, contact) = fetch_fee_with_links(&state, key).await?;

    let json_data = build_fee_json(&fee, &project, &company, &contact);

    let obj = json_data.as_object().ok_or_else(|| {
        ApiError::service_unavailable("Unexpected: build_fee_json did not return an object")
    })?;

    let total_fields = obj.len();
    let mut populated = 0usize;
    let mut placeholder = 0usize;
    let mut fields = serde_json::Map::new();

    for (field_name, field_value) in obj {
        let value_str = field_value.as_str().unwrap_or("");
        if is_placeholder(value_str) {
            placeholder += 1;
            fields.insert(
                field_name.clone(),
                json!({ "status": "placeholder", "value": value_str }),
            );
        } else {
            populated += 1;
            fields.insert(
                field_name.clone(),
                json!({ "status": "populated", "value": value_str }),
            );
        }
    }

    Ok(Json(json!({
        "fee_id": format!("fee:{}", key),
        "total_fields": total_fields,
        "populated": populated,
        "placeholder": placeholder,
        "fields": fields,
    })))
}
