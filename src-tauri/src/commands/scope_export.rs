use chrono::Local;
use std::fs;
use std::path::PathBuf;

/// Export scope as markdown to the project folder.
/// Returns the file path on success, or None if folder not found.
#[tauri::command]
pub async fn export_scope_markdown(
    fee_ref: String,
    project_name: String,
    project_folder: String,
    revision: i64,
    stages: Vec<String>,
    scope_text: String,
) -> Result<Option<String>, String> {
    let folder = PathBuf::from(&project_folder);

    if !folder.exists() {
        return Ok(None);
    }

    let scope_dir = folder.join("scope");
    fs::create_dir_all(&scope_dir).map_err(|e| e.to_string())?;

    let stages_yaml = stages.join(", ");
    let date = Local::now().format("%Y-%m-%d").to_string();

    let content = format!(
        "---\nfee: {}\nproject: {}\nrevision: {}\ndate: {}\nstages: [{}]\n---\n\n{}",
        fee_ref, project_name, revision, date, stages_yaml, scope_text
    );

    // Write revision file
    let rev_filename = format!("scope-rev-{:02}.md", revision);
    let rev_path = scope_dir.join(&rev_filename);
    fs::write(&rev_path, &content).map_err(|e| e.to_string())?;

    // Write/overwrite current file
    let current_path = scope_dir.join("scope-current.md");
    fs::write(&current_path, &content).map_err(|e| e.to_string())?;

    Ok(Some(rev_path.to_string_lossy().to_string()))
}
