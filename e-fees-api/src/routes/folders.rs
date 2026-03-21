//! Nextcloud project folder creation via SSH to Primary server.

use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};
use tracing::info;

use e_fees_core::models::Project;

use crate::error::ApiError;
use crate::ssh::SshOps;
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
    let ssh = SshOps::from_folder_config(folder_config);
    let remote_cmd = format!(
        "bash {} {} {}",
        crate::ssh::shell_quote(&folder_config.script_path),
        crate::ssh::shell_quote(&number),
        crate::ssh::shell_quote(name),
    );
    let stdout = ssh.exec(&remote_cmd).await?;
    info!("Created folder for project {} ({}): {}", number, name, stdout.trim());

    Ok(Json(json!({
        "status": "created",
        "project": number,
        "name": name,
        "path": folder_path,
    })))
}
