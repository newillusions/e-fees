//! Nextcloud project folder creation via SSH to Primary server.

use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};
use tokio::process::Command;
use tracing::{info, warn};

use e_fees_core::models::Project;

use crate::error::ApiError;
use crate::validation::validate_id;
use crate::AppState;

/// Create the Nextcloud project folder on Primary via SSH.
#[utoipa::path(
    post,
    path = "/projects/{id}/folder",
    tag = "Projects",
    params(("id" = String, Path, description = "Project record key (e.g. 26_97101)")),
    responses(
        (status = 200, description = "Folder created", body = crate::schemas::FolderCreatedResponse),
        (status = 404, description = "Project not found", body = crate::schemas::ErrorResponse),
        (status = 503, description = "Folder creation failed (SSH error)", body = crate::schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn create_folder(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    validate_id(&id)?;

    // Fetch project from DB
    let project: Option<Project> = state.db.select(("projects", &*id)).await?;
    let project = project.ok_or_else(|| ApiError::not_found("Project", &id))?;

    // DB stores number.id with underscores (25_97101), script expects dashes (25-97101)
    let number = project.number.id.replace('_', "-");
    let name = &project.name;

    // Sanitize: only allow alphanumeric and dash
    if !number.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(ApiError::bad_request("Invalid project number format"));
    }
    if name.chars().any(|c| c == '\'' || c == '"' || c == '\\' || c == ';' || c == '`' || c == '$') {
        return Err(ApiError::bad_request("Project name contains invalid characters"));
    }

    let folder_config = match &state.folder_config {
        Some(cfg) => cfg,
        None => {
            return Err(ApiError::service_unavailable(
                "Folder creation not configured (NC_SSH_HOST not set)",
            ));
        }
    };

    let folder_path = format!("01 Projects/01 RFPs/{} {}/", number, name);

    // SSH to Primary and run the folder creation script.
    // Must send as a single quoted command string — SSH splits bare args on spaces.
    let user_host = format!("{}@{}", folder_config.ssh_user, folder_config.ssh_host);
    let remote_cmd = format!(
        "bash '{}' '{}' '{}'",
        folder_config.script_path, number, name
    );
    let output = Command::new("ssh")
        .args([
            "-i", &folder_config.ssh_key,
            "-o", "StrictHostKeyChecking=no",
            "-o", "ConnectTimeout=10",
            &user_host,
            &remote_cmd,
        ])
        .output()
        .await
        .map_err(|e| {
            warn!("SSH command failed to execute: {}", e);
            ApiError::service_unavailable(format!("SSH connection failed: {}", e))
        })?;

    if output.status.success() {
        info!("Created folder for project {} ({})", number, name);
        Ok(Json(json!({
            "status": "created",
            "project": number,
            "name": name,
            "path": folder_path,
        })))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        warn!(
            "Folder creation failed for {}: stderr={}, stdout={}",
            number, stderr, stdout
        );
        Err(ApiError::service_unavailable(format!(
            "Folder creation failed: {}",
            stderr.trim()
        )))
    }
}
