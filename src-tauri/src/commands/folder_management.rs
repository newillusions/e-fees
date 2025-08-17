use tauri::command;
use std::fs;
use std::path::{Path, PathBuf};
use std::env;

#[derive(Debug, serde::Serialize)]
pub struct FolderOperationResult {
    pub success: bool,
    pub message: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ProjectFolderInfo {
    pub project_number: String,
    pub current_location: String,
    pub full_path: String,
    pub exists: bool,
}

/// Get the base projects path from environment or default
fn get_projects_base_path() -> Result<PathBuf, String> {
    match env::var("PROJECT_BASE_PATH") {
        Ok(path) => Ok(PathBuf::from(path)),
        Err(_) => {
            // Use test path during development
            let test_path = "/Volumes/base/mms/DevTest";
            if Path::new(test_path).exists() {
                Ok(PathBuf::from(test_path))
            } else {
                // Fallback to main path if test path not available
                let dev_path = "/Volumes/base/emittiv/emittiv/01 Projects";
                if Path::new(dev_path).exists() {
                    Ok(PathBuf::from(dev_path))
                } else {
                    Err("PROJECT_BASE_PATH environment variable not set and no development paths found".to_string())
                }
            }
        }
    }
}

/// Get the folder name for a given status
fn get_folder_for_status(status: &str) -> Result<&str, String> {
    match status.to_lowercase().as_str() {
        "draft" | "rfp" | "proposal" | "submitted" => Ok("01 RFPs"),
        "active" | "current" | "awarded" | "ongoing" => Ok("11 Current"),
        "completed" | "finished" | "delivered" => Ok("99 Completed"),
        "cancelled" | "inactive" | "lost" | "on hold" => Ok("00 Inactive"),
        _ => Err(format!("Unknown status: {}", status))
    }
}

/// Find a project folder by number across all status directories
fn find_project_folder(project_number: &str) -> Result<ProjectFolderInfo, String> {
    let base_path = get_projects_base_path()?;
    
    let status_dirs = ["00 Inactive", "01 RFPs", "11 Current", "99 Completed"];
    
    for status_dir in &status_dirs {
        let search_path = base_path.join(status_dir);
        
        if !search_path.exists() {
            continue;
        }
        
        // Read directory contents
        match fs::read_dir(&search_path) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let folder_name = entry.file_name();
                        let folder_str = folder_name.to_string_lossy();
                        
                        // Check if folder starts with the project number
                        if folder_str.starts_with(project_number) {
                            return Ok(ProjectFolderInfo {
                                project_number: project_number.to_string(),
                                current_location: status_dir.to_string(),
                                full_path: entry.path().to_string_lossy().to_string(),
                                exists: true,
                            });
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read directory {}: {}", search_path.display(), e);
                continue;
            }
        }
    }
    
    Ok(ProjectFolderInfo {
        project_number: project_number.to_string(),
        current_location: "not_found".to_string(),
        full_path: String::new(),
        exists: false,
    })
}

#[command]
pub async fn get_project_folder_location(project_number: String) -> Result<ProjectFolderInfo, String> {
    find_project_folder(&project_number)
}

#[command]
pub async fn move_project_folder(
    project_number: String,
    new_status: String,
) -> Result<FolderOperationResult, String> {
    // Find current location
    let current_info = find_project_folder(&project_number)?;
    
    if !current_info.exists {
        return Ok(FolderOperationResult {
            success: false,
            message: format!("Project folder {} not found", project_number),
            old_path: None,
            new_path: None,
        });
    }
    
    // Get destination folder
    let dest_folder = get_folder_for_status(&new_status)?;
    
    // Check if already in correct location
    if current_info.current_location == dest_folder {
        return Ok(FolderOperationResult {
            success: true,
            message: format!("Project {} is already in {}", project_number, dest_folder),
            old_path: Some(current_info.full_path.clone()),
            new_path: Some(current_info.full_path),
        });
    }
    
    // Build destination path
    let base_path = get_projects_base_path()?;
    let dest_dir = base_path.join(dest_folder);
    
    // Ensure destination directory exists
    if !dest_dir.exists() {
        return Err(format!("Destination directory {} does not exist", dest_dir.display()));
    }
    
    // Get the folder name from current path
    let current_path = Path::new(&current_info.full_path);
    let folder_name = current_path.file_name()
        .ok_or("Could not get folder name")?;
    
    let new_path = dest_dir.join(folder_name);
    
    // Check if destination already exists
    if new_path.exists() {
        return Ok(FolderOperationResult {
            success: false,
            message: format!(
                "Destination folder already exists: {}", 
                new_path.display()
            ),
            old_path: Some(current_info.full_path),
            new_path: Some(new_path.to_string_lossy().to_string()),
        });
    }
    
    // Perform the move operation
    match fs::rename(&current_info.full_path, &new_path) {
        Ok(_) => {
            let mut success_message = format!(
                "Successfully moved {} from {} to {}",
                project_number,
                current_info.current_location,
                dest_folder
            );
            
            // If moving from RFP to Current, copy awarded project templates
            if current_info.current_location == "01 RFPs" && dest_folder == "11 Current" {
                match copy_awarded_templates(&new_path) {
                    Ok(_) => {
                        success_message.push_str(". Awarded project templates copied successfully.");
                    },
                    Err(e) => {
                        success_message.push_str(&format!(". Warning: Failed to copy some templates: {}", e));
                    }
                }
            }
            
            Ok(FolderOperationResult {
                success: true,
                message: success_message,
                old_path: Some(current_info.full_path),
                new_path: Some(new_path.to_string_lossy().to_string()),
            })
        },
        Err(e) => Ok(FolderOperationResult {
            success: false,
            message: format!("Failed to move folder: {}", e),
            old_path: Some(current_info.full_path),
            new_path: Some(new_path.to_string_lossy().to_string()),
        }),
    }
}

#[command]
pub async fn move_project_from_rfp(
    project_number: String,
    destination: String,
) -> Result<FolderOperationResult, String> {
    // Validate destination
    match destination.as_str() {
        "current" => move_project_folder(project_number, "active".to_string()).await,
        "archive" => move_project_folder(project_number, "completed".to_string()).await,
        "inactive" => move_project_folder(project_number, "cancelled".to_string()).await,
        _ => Err(format!("Invalid destination: {}. Use 'current', 'archive', or 'inactive'", destination))
    }
}

#[command]
pub async fn move_project_to_archive(project_number: String) -> Result<FolderOperationResult, String> {
    move_project_folder(project_number, "completed".to_string()).await
}

#[command]
pub async fn list_projects_in_folder(folder_path: String) -> Result<Vec<String>, String> {
    let base_path = get_projects_base_path()?;
    let full_path = base_path.join(&folder_path);
    
    if !full_path.exists() {
        return Err(format!("Folder does not exist: {}", full_path.display()));
    }
    
    let mut projects = Vec::new();
    
    match fs::read_dir(full_path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        let folder_name = entry.file_name().to_string_lossy().to_string();
                        // Filter for project folders (YY-CCCNN format)
                        if folder_name.len() >= 8 && folder_name.chars().nth(2) == Some('-') {
                            projects.push(folder_name);
                        }
                    }
                }
            }
        }
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    }
    
    projects.sort();
    Ok(projects)
}

/// Copy awarded project template folders when moving from RFP to Current
fn copy_awarded_templates(project_path: &Path) -> Result<(), String> {
    let base_path = get_projects_base_path()?;
    let template_path = base_path.join("11 Current").join("00 Additional Folders");
    
    if !template_path.exists() {
        return Err("Awarded project template folder not found".to_string());
    }
    
    // Copy each template folder to the project
    let template_folders = ["03 Contract", "04 Deliverables", "05 Submittals", "11 SubContractors", "98 Outgoing", "99 Temp"];
    
    for folder in &template_folders {
        let src = template_path.join(folder);
        let dest = project_path.join(folder);
        
        if src.exists() && !dest.exists() {
            if let Err(e) = copy_dir_recursively(&src, &dest) {
                eprintln!("Failed to copy template folder {}: {}", folder, e);
                // Continue with other folders even if one fails
            }
        }
    }
    
    Ok(())
}

/// Recursively copy a directory and its contents
fn copy_dir_recursively(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    if !src.exists() {
        return Ok(());
    }
    
    fs::create_dir_all(dest)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursively(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }
    
    Ok(())
}

#[command]
pub async fn validate_project_base_path() -> Result<String, String> {
    match get_projects_base_path() {
        Ok(path) => {
            if path.exists() {
                Ok(format!("Project base path is valid: {}", path.display()))
            } else {
                Err(format!("Project base path does not exist: {}", path.display()))
            }
        }
        Err(e) => Err(e)
    }
}