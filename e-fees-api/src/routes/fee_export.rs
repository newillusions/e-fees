//! Fee JSON export and status endpoints.
//!
//! Builds InDesign variable data JSON from a fee proposal and its linked
//! project/company/contact records. Export writes the JSON to the Nextcloud
//! project folder via SSH; status is a read-only field-by-field check.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::Path,
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use chrono::Utc;
use serde_json::{json, Value};
use tracing::{info, warn};

use e_fees_core::export::{
    build_fee_json, clean_number_for_path, fee_template::generate_fee_template,
    indesign_workbook::generate_indesign_workbook, is_placeholder,
};
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

/// Filesystem paths for a proposal's var.json export, all derived from the
/// project's **short name** (`name_short`) — never the legal/long `name`.
///
/// The canonical project folder is created as `{number} {name_short}` (see the
/// desktop `template_ops.rs`), so every export path must use `name_short` to
/// land in that folder instead of creating a stray `{number} {name}` sibling.
struct ProposalPaths {
    /// `{nc_base_path}/01 RFPs/{number} {name_short}/02 Proposal`
    proposal_dir: String,
    /// `{proposal_dir}/{number}-var.json`
    var_path: String,
    /// `{proposal_dir}/{number}-var Default Values.json`
    template_path: String,
    /// Path relative to the NC base, returned in the API response.
    relative_path: String,
    /// Subpath passed to the Nextcloud rescan (fixed groupfolder prefix).
    rescan_subpath: String,
}

impl ProposalPaths {
    fn new(nc_base_path: &str, number: &str, name_short: &str) -> Self {
        let proposal_dir = format!("{nc_base_path}/01 RFPs/{number} {name_short}/02 Proposal");
        let var_path = format!("{proposal_dir}/{number}-var.json");
        let template_path = format!("{proposal_dir}/{number}-var Default Values.json");
        let relative_path = format!("01 RFPs/{number} {name_short}/02 Proposal/{number}-var.json");
        let rescan_subpath = format!(
            "/emittiv/nc/__groupfolders/1/01 Projects/01 RFPs/{number} {name_short}/02 Proposal"
        );
        Self {
            proposal_dir,
            var_path,
            template_path,
            relative_path,
            rescan_subpath,
        }
    }
}

/// Given a proposal-folder listing and the safe project number, return the
/// latest existing `{number}-(PRI|FP)-NN Pricing.xlsx` filename (the source to
/// base the next version on) and the next version number. Mirrors the desktop
/// `find_latest_pri_file`. Pure string parsing — no regex.
fn latest_pri_source_and_version(filenames: &[String], number: &str) -> (Option<String>, i32) {
    let mut best_ver = 0i32;
    let mut best: Option<String> = None;
    for name in filenames {
        for tag in ["PRI", "FP"] {
            let prefix = format!("{number}-{tag}-");
            if let Some(rest) = name.strip_prefix(&prefix) {
                if let Some(num_str) = rest.strip_suffix(" Pricing.xlsx") {
                    if let Ok(v) = num_str.parse::<i32>() {
                        if v > best_ver {
                            best_ver = v;
                            best = Some(name.clone());
                        }
                    }
                }
            }
        }
    }
    (best, best_ver + 1)
}

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

    // Build filesystem paths from the project's SHORT name — the canonical
    // project folder is `{number} {name_short}`, never the legal/long `name`.
    let number = clean_number_for_path(&project.number.id);
    let paths = ProposalPaths::new(&folder_config.nc_base_path, &number, &project.name_short);
    let proposal_dir = &paths.proposal_dir;
    let var_path = &paths.var_path;
    let template_path = &paths.template_path;

    let ssh = SshOps::from_folder_config(folder_config);

    // Ensure the proposal directory exists
    ssh.mkdir_p(proposal_dir).await?;

    // Archive logic:
    // 1. If template file exists and var.json does not → rename template to var.json
    // 2. If var.json already exists → archive it before overwriting
    let mut archived_previous = false;

    let var_exists = ssh.path_exists(var_path).await?;
    let template_exists = ssh.path_exists(template_path).await?;

    if !var_exists && template_exists {
        // First export: rename template placeholder to var.json
        ssh.rename(template_path, var_path).await?;
        info!("Renamed template to var.json for {}", number);
    } else if var_exists {
        // Archive the existing var.json before overwriting
        let timestamp = Utc::now().format("%Y%m%dT%H%M%S").to_string();
        let archive_dir = format!("{}/00 Archive", proposal_dir);
        ssh.mkdir_p(&archive_dir).await?;
        let archive_path = format!("{}/{}-var-{}.json", archive_dir, number, timestamp);
        ssh.rename(var_path, &archive_path).await?;
        archived_previous = true;
        info!(
            "Archived previous var.json for {} to {}",
            number, archive_path
        );
    }

    // Serialize and write the new JSON
    let json_bytes = serde_json::to_string_pretty(&json_data)
        .map_err(|e| ApiError::service_unavailable(format!("JSON serialization failed: {}", e)))?;
    ssh.write_file(var_path, json_bytes.as_bytes()).await?;

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
    if let Err(e) = ssh.nc_rescan(&paths.rescan_subpath).await {
        warn!("NC rescan failed (non-fatal): {}", e.message);
    }

    let fee_id_str = format!("fee:{}", key);

    info!(
        "Exported fee JSON for {} to {} ({} fields populated)",
        fee_id_str, var_path, fields_populated
    );

    Ok(Json(json!({
        "status": "exported",
        "fee_id": fee_id_str,
        "path": paths.relative_path,
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

// ============================================================================
// PRICING TEMPLATE (-PRI) EXPORT ENDPOINT
// ============================================================================

/// Export the internal `-PRI` pricing workbook to the Nextcloud proposal folder.
///
/// Mirrors the desktop "Export Pricing Template" action: scans the proposal
/// folder for the latest `{number}-(PRI|FP)-NN Pricing.xlsx`, uses it as the
/// source (or the embedded blank template), populates the fee's pricing data
/// while preserving the template's formulas, and writes the next version
/// `{number}-PRI-{NN+1} Pricing.xlsx` back to the folder.
#[utoipa::path(
    post,
    path = "/fees/{id}/export/template",
    tag = "Fees",
    params(("id" = String, Path, description = "Fee record key (fee: prefix stripped if present)")),
    responses(
        (status = 200, description = "PRI workbook exported to the proposal folder"),
        (status = 404, description = "Fee or project not found", body = crate::schemas::ErrorResponse),
        (status = 422, description = "Fee has no pricing data", body = crate::schemas::ErrorResponse),
        (status = 503, description = "Folder config not set or SSH error", body = crate::schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn export_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("fee:").unwrap_or(&id);
    validate_id(key)?;

    // Fetch fee (carries pricing) and its project (for the folder path).
    let fee: Option<Fee> = state.db.select(("fee", key)).await?;
    let fee = fee.ok_or_else(|| ApiError::not_found("Fee", key))?;
    let project_key = {
        let s = record_id_string(&fee.project_id);
        s.splitn(2, ':').nth(1).unwrap_or("").to_string()
    };
    let project: Option<Project> = state.db.select(("projects", &*project_key)).await?;
    let project = project
        .ok_or_else(|| ApiError::not_found("Project", &format!("projects:{}", project_key)))?;

    let folder_config = state.folder_config.as_ref().ok_or_else(|| {
        ApiError::service_unavailable("Folder export not configured (NC_SSH_HOST not set)")
    })?;

    let number = clean_number_for_path(&project.number.id);
    let paths = ProposalPaths::new(&folder_config.nc_base_path, &number, &project.name_short);
    let ssh = SshOps::from_folder_config(folder_config);
    ssh.mkdir_p(&paths.proposal_dir).await?;

    // Pick the latest existing PRI/FP as the source + the next version number.
    let listing = ssh.list_dir(&paths.proposal_dir).await?;
    let (source_name, next_version) = latest_pri_source_and_version(&listing, &number);

    // Stage the source (if any) to a local temp file for umya to read.
    let tmp_dir = std::env::temp_dir();
    let source_local: Option<std::path::PathBuf> = match &source_name {
        Some(name) => {
            let remote = format!("{}/{}", paths.proposal_dir, name);
            let bytes = ssh.read_file(&remote).await?;
            let local = tmp_dir.join(format!("efees-pri-src-{}.xlsx", key));
            std::fs::write(&local, &bytes)
                .map_err(|e| ApiError::service_unavailable(format!("temp write failed: {e}")))?;
            Some(local)
        }
        None => None,
    };

    // Generate into a local temp output, then read the bytes back.
    let out_local = tmp_dir.join(format!("efees-pri-out-{}.xlsx", key));
    let gen = generate_fee_template(&fee, &out_local, source_local.as_deref());
    // Clean the source temp regardless of outcome.
    if let Some(s) = &source_local {
        let _ = std::fs::remove_file(s);
    }
    gen.map_err(|e| ApiError {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        code: "export_failed".into(),
        message: e,
    })?;
    let out_bytes = std::fs::read(&out_local)
        .map_err(|e| ApiError::service_unavailable(format!("temp read failed: {e}")))?;
    let _ = std::fs::remove_file(&out_local);

    // Write the next-version PRI to the proposal folder.
    let filename = format!("{}-PRI-{:02} Pricing.xlsx", number, next_version);
    let remote_out = format!("{}/{}", paths.proposal_dir, filename);
    ssh.write_file(&remote_out, &out_bytes).await?;

    // Best-effort NC rescan.
    if let Err(e) = ssh.nc_rescan(&paths.rescan_subpath).await {
        warn!("NC rescan failed (non-fatal): {}", e.message);
    }

    let relative_path = format!(
        "01 RFPs/{} {}/02 Proposal/{}",
        number, project.name_short, filename
    );
    info!("Exported -PRI for fee:{} to {}", key, remote_out);

    Ok(Json(json!({
        "status": "exported",
        "fee_id": format!("fee:{}", key),
        "path": relative_path,
        "version": next_version,
        "source": source_name,
    })))
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // BUG 1 regression: var.json export must land in the canonical
    // `{number} {name_short}` folder — never the legal/long `{name}` folder.
    // The project folder is created as `{number} {name_short}` (desktop
    // template_ops.rs), so building paths from `name` writes to a stray folder.
    #[test]
    fn proposal_paths_use_name_short_not_legal_name() {
        let paths = ProposalPaths::new(
            "/mnt/user/emittiv",
            "26-97104",
            "Lulu-Island-AV", // name_short; legal name is "LULU ISLAND DEVELOPMENT"
        );

        assert_eq!(
            paths.proposal_dir,
            "/mnt/user/emittiv/01 RFPs/26-97104 Lulu-Island-AV/02 Proposal"
        );
        assert_eq!(
            paths.var_path,
            "/mnt/user/emittiv/01 RFPs/26-97104 Lulu-Island-AV/02 Proposal/26-97104-var.json"
        );
        assert_eq!(
            paths.template_path,
            "/mnt/user/emittiv/01 RFPs/26-97104 Lulu-Island-AV/02 Proposal/26-97104-var Default Values.json"
        );
        assert_eq!(
            paths.relative_path,
            "01 RFPs/26-97104 Lulu-Island-AV/02 Proposal/26-97104-var.json"
        );
        assert_eq!(
            paths.rescan_subpath,
            "/emittiv/nc/__groupfolders/1/01 Projects/01 RFPs/26-97104 Lulu-Island-AV/02 Proposal"
        );
    }

    #[test]
    fn latest_pri_picks_highest_version_and_next() {
        let files = vec![
            "26-97104-FP-01 Pricing.xlsx".to_string(),
            "26-97104-PRI-02 Pricing.xlsx".to_string(),
            "26-97104-PRI-01 Pricing.xlsx".to_string(),
            "26-97104-var.json".to_string(),
        ];
        let (src, next) = latest_pri_source_and_version(&files, "26-97104");
        assert_eq!(src.as_deref(), Some("26-97104-PRI-02 Pricing.xlsx"));
        assert_eq!(next, 3);
    }

    #[test]
    fn latest_pri_empty_dir_starts_at_one() {
        let (src, next) = latest_pri_source_and_version(&[], "26-97104");
        assert_eq!(src, None);
        assert_eq!(next, 1);
    }

    #[test]
    fn latest_pri_falls_back_to_fp_source() {
        let files = vec!["26-97104-FP-01 Pricing.xlsx".to_string()];
        let (src, next) = latest_pri_source_and_version(&files, "26-97104");
        assert_eq!(src.as_deref(), Some("26-97104-FP-01 Pricing.xlsx"));
        assert_eq!(next, 2);
    }
}

// ============================================================================
// WORKBOOK EXPORT ENDPOINT
// ============================================================================

/// Export fee data as a multi-sheet XLSX workbook for InDesign data-merge.
///
/// Generates a workbook with sheets T0–T5 (design/post-contract durations,
/// design fees, post-contract fees, payment schedule, reimbursable costs) plus
/// a Revisions sheet.
/// Returns raw xlsx bytes with appropriate Content-Type and Content-Disposition
/// headers so the caller can download the file directly.
#[utoipa::path(
    post,
    path = "/fees/{id}/export/indesign",
    tag = "Fees",
    params(("id" = String, Path, description = "Fee record key (fee: prefix stripped if present)")),
    responses(
        (status = 200, description = "XLSX workbook bytes", content_type = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        (status = 404, description = "Fee not found", body = crate::schemas::ErrorResponse),
        (status = 422, description = "Fee has no pricing data", body = crate::schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn export_indesign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Response<Body>, ApiError> {
    let key = id.strip_prefix("fee:").unwrap_or(&id);

    let fee: Option<Fee> = state.db.select(("fee", key)).await?;
    let fee = fee.ok_or_else(|| ApiError::not_found("Fee", &id))?;

    let xlsx_bytes = generate_indesign_workbook(&fee).map_err(|e| ApiError {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        code: "export_failed".into(),
        message: e,
    })?;

    let filename = format!("{}-indesign.xlsx", key.replace(':', "-").replace('_', "-"));
    let disposition = format!("attachment; filename=\"{}\"", filename);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static(
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
        )
        .header(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&disposition)
                .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
        )
        .header(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&xlsx_bytes.len().to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("0")),
        )
        .body(Body::from(xlsx_bytes))
        .map_err(|e| ApiError::service_unavailable(format!("Failed to build response: {e}")))?;

    info!("Generated InDesign workbook for fee:{}", key);

    Ok(response)
}
