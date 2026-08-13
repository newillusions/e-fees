//! Nextcloud groupfolder rescan trigger.
//!
//! Standalone HTTP wrapper around the same `occ groupfolders:scan` mechanism
//! already used internally after document uploads and fee exports (see
//! `ssh::SshOps::nc_rescan`). Exists so OTHER writers of the shared project
//! storage - specifically the desktop app's direct filesystem folder moves
//! (`src-tauri/src/commands/folder_management.rs`) - can ask Nextcloud to
//! pick up a change it made outside of Nextcloud's own upload/API path.
//!
//! Best-effort by design: the desktop app calls this after a successful
//! folder move and must never fail the move itself over a rescan hiccup,
//! matching every other `nc_rescan` call site in this crate.

use std::sync::Arc;

use axum::{extract::State, Json};
use serde_json::{json, Value};
use tracing::info;

use crate::error::ApiError;
use crate::ssh::SshOps;
use crate::AppState;

/// Trigger a Nextcloud groupfolders rescan.
///
/// Blanket rescan (the underlying `occ groupfolders:scan` command scans the
/// whole groupfolder, not a specific subpath - see `SshOps::nc_rescan`), so
/// no request body is needed.
#[utoipa::path(
    post,
    path = "/nc/rescan",
    tag = "Projects",
    responses(
        (status = 200, description = "Rescan triggered", body = crate::schemas::RescanTriggeredResponse),
        (status = 503, description = "Rescan not configured or SSH failed", body = crate::schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn trigger_rescan(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let folder_config = match &state.folder_config {
        Some(cfg) => cfg,
        None => {
            return Err(ApiError::service_unavailable(
                "Rescan not configured (NC_SSH_HOST not set)",
            ));
        }
    };

    let ssh = SshOps::from_folder_config(folder_config);
    ssh.nc_rescan("manual-trigger").await?;

    info!("Nextcloud groupfolders rescan triggered");

    Ok(Json(json!({ "status": "rescanned" })))
}
