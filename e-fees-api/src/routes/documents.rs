//! Document upload endpoint — accepts multipart file upload and writes to Nextcloud project folder.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use axum_extra::extract::Multipart;
use serde_json::{json, Value};
use tracing::info;

use e_fees_core::export::clean_number_for_path;
use e_fees_core::models::Project;

use crate::error::ApiError;
use crate::ssh::SshOps;
use crate::validation::validate_id;
use crate::AppState;

const DEFAULT_SUBFOLDER: &str = "01 Client Info/01 Pre Award";

/// Upload a document to the Nextcloud project folder via SSH.
///
/// Accepts a multipart form with:
/// - `file` (required): the binary file to upload
/// - `subfolder` (optional): destination subfolder within the project folder
///   (defaults to "01 Client Info/01 Pre Award")
#[utoipa::path(
    post,
    path = "/projects/{id}/documents",
    tag = "Projects",
    params(("id" = String, Path, description = "Project record key (e.g. 26_97101)")),
    request_body(
        content_type = "multipart/form-data",
        description = "File upload with optional subfolder",
    ),
    responses(
        (status = 200, description = "Document uploaded", body = crate::schemas::UploadDocumentResponse),
        (status = 400, description = "Invalid filename or subfolder", body = crate::schemas::ErrorResponse),
        (status = 404, description = "Project not found", body = crate::schemas::ErrorResponse),
        (status = 503, description = "Upload failed (SSH error or folder config missing)", body = crate::schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn upload_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Value>, ApiError> {
    validate_id(&id)?;

    // Fetch project from DB
    let project: Option<Project> = state.db.select(("projects", &*id)).await?;
    let project = project.ok_or_else(|| ApiError::not_found("Project", &id))?;

    let folder_config = match &state.folder_config {
        Some(cfg) => cfg,
        None => {
            return Err(ApiError::service_unavailable(
                "Document upload not configured (NC_SSH_HOST not set)",
            ));
        }
    };

    // Parse multipart fields
    let mut file_bytes: Option<axum::body::Bytes> = None;
    let mut original_filename: Option<String> = None;
    let mut subfolder_raw: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::bad_request(format!("Multipart parse error: {}", e)))?
    {
        match field.name() {
            Some("file") => {
                original_filename = field.file_name().map(|s| s.to_string());
                let data = field.bytes().await.map_err(|e| {
                    ApiError::bad_request(format!("Failed to read file bytes: {}", e))
                })?;
                file_bytes = Some(data);
            }
            Some("subfolder") => {
                let text = field.text().await.map_err(|e| {
                    ApiError::bad_request(format!("Failed to read subfolder field: {}", e))
                })?;
                subfolder_raw = Some(text);
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    let data = file_bytes.ok_or_else(|| ApiError::bad_request("Missing required 'file' field"))?;

    let raw_name = original_filename.unwrap_or_default();
    let filename = sanitize_filename(&raw_name)?;

    let subfolder_str = subfolder_raw.unwrap_or_else(|| DEFAULT_SUBFOLDER.to_string());
    let subfolder = validate_subfolder(&subfolder_str)?;

    // Build destination path
    let number = clean_number_for_path(&project.number.id);
    let name = &project.name;
    let relative_path = format!("01 RFPs/{} {}/{}/{}", number, name, subfolder, filename);
    let dest_dir = format!(
        "{}/01 RFPs/{} {}/{}",
        folder_config.nc_base_path, number, name, subfolder
    );
    let dest_file = format!("{}/{}", dest_dir, filename);

    let size_bytes = data.len() as u64;

    let ssh = SshOps::from_folder_config(folder_config);

    // Create destination directory
    ssh.mkdir_p(&dest_dir).await?;

    // Write file via stdin pipe
    ssh.write_file(&dest_file, &data).await?;

    // Trigger Nextcloud rescan
    let nc_scan_path = format!(
        "/emittiv/__groupfolders/1/01 Projects/01 RFPs/{} {}/{}",
        number, name, subfolder
    );
    ssh.nc_rescan(&nc_scan_path).await?;

    info!(
        "Uploaded document '{}' ({} bytes) to project {} ({})",
        filename, size_bytes, number, name
    );

    Ok(Json(json!({
        "status": "uploaded",
        "project_id": format!("projects:{}", id),
        "filename": filename,
        "path": relative_path,
        "size_bytes": size_bytes,
    })))
}

/// Strip path components from a filename and reject shell-unsafe characters.
///
/// Returns the sanitized filename, or an error if the name is unsafe.
fn sanitize_filename(name: &str) -> Result<String, ApiError> {
    // Strip path components — take the last segment after any / or \
    let basename = name
        .replace('\\', "/")
        .split('/')
        .filter(|s| !s.is_empty())
        .last()
        .unwrap_or("")
        .to_string();

    if basename.is_empty() || basename == "." || basename == ".." {
        return Err(ApiError::bad_request("Invalid filename"));
    }

    // Reject shell-unsafe characters
    const UNSAFE_CHARS: &[char] = &[';', '`', '$', '|', '&', '<', '>', '\'', '"', '\\'];
    if basename.chars().any(|c| UNSAFE_CHARS.contains(&c)) {
        return Err(ApiError::bad_request("Filename contains unsafe characters"));
    }

    Ok(basename)
}

/// Validate a subfolder path: must be relative and contain no `..` components.
fn validate_subfolder(path: &str) -> Result<String, ApiError> {
    if path.starts_with('/') {
        return Err(ApiError::bad_request("Subfolder must be a relative path"));
    }

    for component in path.split('/') {
        if component == ".." {
            return Err(ApiError::bad_request(
                "Subfolder must not contain '..' path components",
            ));
        }
    }

    Ok(path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- sanitize_filename tests ----

    #[test]
    fn test_sanitize_filename_valid() {
        assert_eq!(sanitize_filename("report.pdf").unwrap(), "report.pdf");
    }

    #[test]
    fn test_sanitize_filename_strips_path() {
        assert_eq!(sanitize_filename("path/to/file.pdf").unwrap(), "file.pdf");
    }

    #[test]
    fn test_sanitize_filename_strips_backslash_path() {
        assert_eq!(sanitize_filename("path\\to\\file.pdf").unwrap(), "file.pdf");
    }

    #[test]
    fn test_sanitize_filename_rejects_empty() {
        assert!(sanitize_filename("").is_err());
    }

    #[test]
    fn test_sanitize_filename_rejects_dot() {
        assert!(sanitize_filename(".").is_err());
    }

    #[test]
    fn test_sanitize_filename_rejects_dotdot() {
        assert!(sanitize_filename("..").is_err());
    }

    #[test]
    fn test_sanitize_filename_rejects_unsafe_semicolon() {
        assert!(sanitize_filename("file;rm -rf .pdf").is_err());
    }

    #[test]
    fn test_sanitize_filename_rejects_unsafe_dollar() {
        assert!(sanitize_filename("file$var.pdf").is_err());
    }

    #[test]
    fn test_sanitize_filename_rejects_unsafe_backtick() {
        assert!(sanitize_filename("file`cmd`.pdf").is_err());
    }

    #[test]
    fn test_sanitize_filename_rejects_unsafe_pipe() {
        assert!(sanitize_filename("file|cmd.pdf").is_err());
    }

    // ---- validate_subfolder tests ----

    #[test]
    fn test_validate_subfolder_valid() {
        assert_eq!(
            validate_subfolder("01 Client Info/01 Pre Award").unwrap(),
            "01 Client Info/01 Pre Award"
        );
    }

    #[test]
    fn test_validate_subfolder_simple() {
        assert_eq!(validate_subfolder("Documents").unwrap(), "Documents");
    }

    #[test]
    fn test_validate_subfolder_rejects_traversal() {
        assert!(validate_subfolder("../etc").is_err());
    }

    #[test]
    fn test_validate_subfolder_rejects_deep_traversal() {
        assert!(validate_subfolder("01 Docs/../../../etc/passwd").is_err());
    }

    #[test]
    fn test_validate_subfolder_rejects_absolute() {
        assert!(validate_subfolder("/absolute/path").is_err());
    }
}
