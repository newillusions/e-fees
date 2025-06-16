use crate::db::{DatabaseManager, ConnectionStatus, Project, NewProject, Company, Contact, Rfp};
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::Path;
use tauri::State;
use log::{error, info};
use serde::{Serialize, Deserialize};
use tauri_plugin_dialog::DialogExt;

// Application state that holds the database manager
pub type AppState = Arc<Mutex<DatabaseManager>>;

// Company update structure (partial fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyUpdate {
    pub name: Option<String>,
    pub name_short: Option<String>,
    pub abbreviation: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub reg_no: Option<String>,
    pub tax_no: Option<String>,
}

// Settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub surrealdb_url: Option<String>,
    pub surrealdb_ns: Option<String>,
    pub surrealdb_db: Option<String>,
    pub surrealdb_user: Option<String>,
    pub surrealdb_pass: Option<String>,
    pub staff_name: Option<String>,
    pub staff_email: Option<String>,
    pub staff_phone: Option<String>,
    pub staff_position: Option<String>,
    pub project_folder_path: Option<String>,
}

// Check database connection status
#[tauri::command]
pub async fn check_db_connection(state: State<'_, AppState>) -> Result<bool, String> {
    info!("Checking database connection");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    let is_connected = manager_clone.check_connection().await;
    Ok(is_connected)
}

// Get connection status with details
#[tauri::command]
pub async fn get_connection_status(state: State<'_, AppState>) -> Result<ConnectionStatus, String> {
    info!("Getting detailed connection status");
    
    let manager = state.lock().map_err(|e| e.to_string())?;
    let status = manager.get_status();
    Ok(status)
}

// Get all projects from database
#[tauri::command]
pub async fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    info!("Fetching projects from database");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.get_projects().await {
        Ok(projects) => {
            info!("Successfully fetched {} projects", projects.len());
            Ok(projects)
        }
        Err(e) => {
            error!("Failed to fetch projects: {}", e);
            Err(format!("Failed to fetch projects: {}", e))
        }
    }
}

// Search projects with fuzzy-like matching
#[tauri::command]
pub async fn search_projects(query: String, state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    info!("Searching projects with query: {}", query);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.search_projects(&query).await {
        Ok(projects) => {
            info!("Search returned {} projects", projects.len());
            Ok(projects)
        }
        Err(e) => {
            error!("Failed to search projects: {}", e);
            Err(format!("Failed to search projects: {}", e))
        }
    }
}

// Get all companies from database
#[tauri::command]
pub async fn get_companies(state: State<'_, AppState>) -> Result<Vec<Company>, String> {
    info!("Fetching companies from database");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.get_companies().await {
        Ok(companies) => {
            info!("Successfully fetched {} companies", companies.len());
            Ok(companies)
        }
        Err(e) => {
            error!("Failed to fetch companies: {}", e);
            Err(format!("Failed to fetch companies: {}", e))
        }
    }
}

// Get all contacts from database
#[tauri::command]
pub async fn get_contacts(state: State<'_, AppState>) -> Result<Vec<Contact>, String> {
    info!("Fetching contacts from database");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.get_contacts().await {
        Ok(contacts) => {
            info!("Successfully fetched {} contacts", contacts.len());
            Ok(contacts)
        }
        Err(e) => {
            error!("Failed to fetch contacts: {}", e);
            Err(format!("Failed to fetch contacts: {}", e))
        }
    }
}

// Get all rfps from database
#[tauri::command]
pub async fn get_rfps(state: State<'_, AppState>) -> Result<Vec<Rfp>, String> {
    info!("Fetching rfps from database");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.get_rfps().await {
        Ok(rfps) => {
            info!("Successfully fetched {} rfps", rfps.len());
            Ok(rfps)
        }
        Err(e) => {
            error!("Failed to fetch rfps: {}", e);
            Err(format!("Failed to fetch rfps: {}", e))
        }
    }
}

// Create a new project
#[tauri::command]
pub async fn create_project(project: Project, state: State<'_, AppState>) -> Result<Project, String> {
    info!("Creating new project: {}", project.name);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.create_project(project).await {
        Ok(created_project) => {
            info!("Successfully created project with id: {:?}", created_project.id);
            Ok(created_project)
        }
        Err(e) => {
            error!("Failed to create project: {}", e);
            Err(format!("Failed to create project: {}", e))
        }
    }
}

// Create a new company
#[tauri::command]
pub async fn create_company(company: Company, state: State<'_, AppState>) -> Result<Company, String> {
    info!("Creating new company: {}", company.name);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.create_company(company).await {
        Ok(created_company) => {
            info!("Successfully created company with id: {:?}", created_company.id);
            Ok(created_company)
        }
        Err(e) => {
            error!("Failed to create company: {}", e);
            Err(format!("Failed to create company: {}", e))
        }
    }
}

// Update an existing company
#[tauri::command]
pub async fn update_company(id: String, company_update: CompanyUpdate, state: State<'_, AppState>) -> Result<Company, String> {
    info!("Updating company with id: {} with partial data: {:?}", id, company_update);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.update_company_partial(&id, company_update).await {
        Ok(updated_company) => {
            info!("Successfully updated company with id: {}", id);
            Ok(updated_company)
        }
        Err(e) => {
            error!("Failed to update company: {}", e);
            Err(format!("Failed to update company: {}", e))
        }
    }
}

// Delete a company
#[tauri::command]
pub async fn delete_company(id: String, state: State<'_, AppState>) -> Result<Company, String> {
    info!("Deleting company with id: {}", id);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.delete_company(&id).await {
        Ok(deleted_company) => {
            info!("Successfully deleted company with id: {}", id);
            Ok(deleted_company)
        }
        Err(e) => {
            error!("Failed to delete company: {}", e);
            Err(format!("Failed to delete company: {}", e))
        }
    }
}

// Create a new contact
#[tauri::command]
pub async fn create_contact(contact: Contact, state: State<'_, AppState>) -> Result<Contact, String> {
    info!("Creating new contact: {} {}", contact.first_name, contact.last_name);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.create_contact(contact).await {
        Ok(created_contact) => {
            info!("Successfully created contact with id: {:?}", created_contact.id);
            Ok(created_contact)
        }
        Err(e) => {
            error!("Failed to create contact: {}", e);
            Err(format!("Failed to create contact: {}", e))
        }
    }
}

// Create a new rfp
#[tauri::command]
pub async fn create_rfp(rfp: Rfp, state: State<'_, AppState>) -> Result<Rfp, String> {
    info!("Creating new rfp: {}", rfp.name);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.create_rfp(rfp).await {
        Ok(created_rfp) => {
            info!("Successfully created rfp with id: {:?}", created_rfp.id);
            Ok(created_rfp)
        }
        Err(e) => {
            error!("Failed to create rfp: {}", e);
            Err(format!("Failed to create rfp: {}", e))
        }
    }
}

// Health check command
#[tauri::command]
pub async fn health_check() -> Result<String, String> {
    info!("Application health check");
    Ok("Application is running".to_string())
}

// Get detailed database connection info for debugging
#[tauri::command]
pub async fn get_db_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Getting database connection information");
    
    let (connection_status, config) = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        (manager.get_status(), manager.config.clone())
    };
    
    let info = serde_json::json!({
        "url": config.url,
        "namespace": config.namespace,
        "database": config.database,
        "username": config.username,
        "connection_status": connection_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "troubleshooting": {
            "dns_issue": format!("If you see 'No such host is known', the server {} cannot be resolved", config.url),
            "network_check": format!("Try pinging the server: ping {}", config.url.replace("ws://", "").replace(":8000", "")),
            "surrealdb_check": format!("Verify SurrealDB is running: telnet {} 8000", config.url.replace("ws://", "").replace(":8000", "")),
            "config_source": "Configuration loaded from .env file or environment variables"
        }
    });
    
    Ok(info)
}

// Get application statistics
#[tauri::command]
pub async fn get_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Fetching application statistics");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    let projects = manager_clone.get_projects().await.unwrap_or_default();
    let companies = manager_clone.get_companies().await.unwrap_or_default();
    let contacts = manager_clone.get_contacts().await.unwrap_or_default();
    let rfps = manager_clone.get_rfps().await.unwrap_or_default();
    
    let active_rfps = rfps.iter()
        .filter(|r| r.status != "Lost" && r.status != "Cancelled")
        .count();
    
    let stats = serde_json::json!({
        "totalProjects": projects.len(),
        "activeRfps": active_rfps,
        "totalCompanies": companies.len(),
        "totalContacts": contacts.len(),
        "totalRfps": rfps.len()
    });
    
    info!("Successfully calculated statistics");
    Ok(stats)
}

// Get table schema information
#[tauri::command]
pub async fn get_table_schema(table_name: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Getting schema for table: {}", table_name);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.get_table_schema(&table_name).await {
        Ok(schema) => {
            info!("Successfully retrieved schema for table: {}", table_name);
            Ok(schema)
        }
        Err(e) => {
            error!("Failed to get schema for table {}: {}", table_name, e);
            Err(format!("Failed to get schema for table {}: {}", table_name, e))
        }
    }
}

// Position window on right half of 4K monitor
#[tauri::command]
pub async fn position_window_4k(window: tauri::Window) -> Result<String, String> {
    info!("Positioning window for 4K monitor");
    
    // Set position to right half of 4K screen
    window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: 1920, y: 0 }))
        .map_err(|e| format!("Failed to set position: {}", e))?;
    
    // Set size to half width, full height
    window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: 1920, height: 2160 }))
        .map_err(|e| format!("Failed to set size: {}", e))?;
    
    info!("Window positioned successfully on right half of 4K monitor");
    Ok("Window positioned successfully".to_string())
}

// Get current settings from .env file
#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    info!("Reading settings from .env file");
    
    let env_path = "../.env"; // Go up one level from src-tauri to project root
    let mut settings = AppSettings {
        surrealdb_url: None,
        surrealdb_ns: None,
        surrealdb_db: None,
        surrealdb_user: None,
        surrealdb_pass: None,
        staff_name: None,
        staff_email: None,
        staff_phone: None,
        staff_position: None,
        project_folder_path: None,
    };
    
    if Path::new(env_path).exists() {
        match fs::read_to_string(env_path) {
            Ok(content) => {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim().trim_matches('"');
                        
                        match key {
                            "SURREALDB_URL" => settings.surrealdb_url = Some(value.to_string()),
                            "SURREALDB_NS" => settings.surrealdb_ns = Some(value.to_string()),
                            "SURREALDB_DB" => settings.surrealdb_db = Some(value.to_string()),
                            "SURREALDB_USER" => settings.surrealdb_user = Some(value.to_string()),
                            "SURREALDB_PASS" => settings.surrealdb_pass = Some(value.to_string()),
                            "STAFF_NAME" => settings.staff_name = Some(value.to_string()),
                            "STAFF_EMAIL" => settings.staff_email = Some(value.to_string()),
                            "STAFF_PHONE" => settings.staff_phone = Some(value.to_string()),
                            "STAFF_POSITION" => settings.staff_position = Some(value.to_string()),
                            "PROJECT_FOLDER_PATH" => settings.project_folder_path = Some(value.to_string()),
                            _ => {}
                        }
                    }
                }
                info!("Successfully loaded settings from .env file");
            }
            Err(e) => {
                error!("Failed to read .env file: {}", e);
                return Err(format!("Failed to read .env file: {}", e));
            }
        }
    } else {
        info!("No .env file found, returning empty settings");
    }
    
    Ok(settings)
}

// Save settings to .env file
#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<String, String> {
    info!("Saving settings to .env file");
    
    let env_path = "../.env"; // Go up one level from src-tauri to project root
    let mut lines = Vec::new();
    
    // Read existing .env file to preserve other variables
    if Path::new(env_path).exists() {
        match fs::read_to_string(env_path) {
            Ok(content) => {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        lines.push(line.to_string());
                        continue;
                    }
                    
                    if let Some((key, _)) = line.split_once('=') {
                        let key = key.trim();
                        
                        // Skip our managed settings - we'll add them back
                        match key {
                            "SURREALDB_URL" | "SURREALDB_NS" | "SURREALDB_DB" | 
                            "SURREALDB_USER" | "SURREALDB_PASS" |
                            "STAFF_NAME" | "STAFF_EMAIL" | "STAFF_PHONE" | "STAFF_POSITION" |
                            "PROJECT_FOLDER_PATH" => continue,
                            _ => lines.push(line.to_string()),
                        }
                    } else {
                        lines.push(line.to_string());
                    }
                }
            }
            Err(e) => {
                error!("Failed to read existing .env file: {}", e);
                return Err(format!("Failed to read existing .env file: {}", e));
            }
        }
    }
    
    // Add our settings
    lines.push("".to_string()); // Empty line for separation
    lines.push("# SurrealDB Configuration".to_string());
    
    if let Some(url) = &settings.surrealdb_url {
        lines.push(format!("SURREALDB_URL=\"{}\"", url));
    }
    if let Some(ns) = &settings.surrealdb_ns {
        lines.push(format!("SURREALDB_NS=\"{}\"", ns));
    }
    if let Some(db) = &settings.surrealdb_db {
        lines.push(format!("SURREALDB_DB=\"{}\"", db));
    }
    if let Some(user) = &settings.surrealdb_user {
        lines.push(format!("SURREALDB_USER=\"{}\"", user));
    }
    if let Some(pass) = &settings.surrealdb_pass {
        lines.push(format!("SURREALDB_PASS=\"{}\"", pass));
    }
    
    lines.push("".to_string());
    lines.push("# Staff Information".to_string());
    
    if let Some(name) = &settings.staff_name {
        lines.push(format!("STAFF_NAME=\"{}\"", name));
    }
    if let Some(email) = &settings.staff_email {
        lines.push(format!("STAFF_EMAIL=\"{}\"", email));
    }
    if let Some(phone) = &settings.staff_phone {
        lines.push(format!("STAFF_PHONE=\"{}\"", phone));
    }
    if let Some(position) = &settings.staff_position {
        lines.push(format!("STAFF_POSITION=\"{}\"", position));
    }
    
    lines.push("".to_string());
    lines.push("# Project Configuration".to_string());
    
    if let Some(folder_path) = &settings.project_folder_path {
        lines.push(format!("PROJECT_FOLDER_PATH=\"{}\"", folder_path));
    }
    
    // Write to file
    let content = lines.join("\n");
    match fs::write(env_path, content) {
        Ok(_) => {
            info!("Successfully saved settings to .env file");
            Ok("Settings saved successfully".to_string())
        }
        Err(e) => {
            error!("Failed to write .env file: {}", e);
            Err(format!("Failed to write .env file: {}", e))
        }
    }
}

// Open folder picker dialog
#[tauri::command]
pub async fn select_folder(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    info!("Opening folder picker dialog");
    
    use std::sync::mpsc;
    use std::time::Duration;
    
    let (tx, rx) = mpsc::channel();
    
    app_handle.dialog()
        .file()
        .set_title("Select Project Folder")
        .pick_folder(move |folder_path| {
            let _ = tx.send(folder_path);
        });
    
    // Wait for the dialog result with a timeout
    match rx.recv_timeout(Duration::from_secs(30)) {
        Ok(Some(path)) => {
            let path_str = path.to_string();
            info!("Selected folder: {}", path_str);
            Ok(Some(path_str))
        }
        Ok(None) => {
            info!("No folder selected");
            Ok(None)
        }
        Err(_) => {
            error!("Folder selection timed out");
            Err("Folder selection timed out".to_string())
        }
    }
}

// Open folder in native file explorer
#[tauri::command]
pub async fn open_folder_in_explorer(folder_path: String) -> Result<String, String> {
    info!("Opening folder in explorer: {}", folder_path);
    
    use std::process::Command;
    
    let result = if cfg!(target_os = "windows") {
        Command::new("explorer")
            .arg(&folder_path)
            .spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(&folder_path)
            .spawn()
    } else {
        // Linux and other Unix-like systems
        Command::new("xdg-open")
            .arg(&folder_path)
            .spawn()
    };
    
    match result {
        Ok(_) => {
            info!("Successfully opened folder: {}", folder_path);
            Ok("Folder opened successfully".to_string())
        }
        Err(e) => {
            error!("Failed to open folder {}: {}", folder_path, e);
            Err(format!("Failed to open folder: {}", e))
        }
    }
}

// Investigate specific database record
#[tauri::command]
pub async fn investigate_record(record_id: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Investigating record: {}", record_id);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.investigate_record(&record_id).await {
        Ok(result) => {
            info!("Successfully investigated record: {}", record_id);
            Ok(result)
        }
        Err(e) => {
            error!("Failed to investigate record {}: {}", record_id, e);
            Err(format!("Failed to investigate record: {}", e))
        }
    }
}

// Search countries with fuzzy matching
#[tauri::command]
pub async fn search_countries(query: String, state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    info!("Searching countries with query: {}", query);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.search_countries(&query).await {
        Ok(countries) => {
            info!("Search returned {} countries", countries.len());
            Ok(countries)
        }
        Err(e) => {
            error!("Failed to search countries: {}", e);
            Err(format!("Failed to search countries: {}", e))
        }
    }
}

// Generate next project number for given country name and year
#[tauri::command]
pub async fn generate_next_project_number(country_name: String, year: Option<u8>, state: State<'_, AppState>) -> Result<String, String> {
    info!("Generating next project number for country: {}, year: {:?}", country_name, year);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.generate_next_project_number(&country_name, year).await {
        Ok(number) => {
            info!("Generated project number: {}", number);
            Ok(number)
        }
        Err(e) => {
            error!("Failed to generate project number: {}", e);
            Err(format!("Failed to generate project number: {}", e))
        }
    }
}

// Validate project number doesn't already exist
#[tauri::command]
pub async fn validate_project_number(project_number: String, state: State<'_, AppState>) -> Result<bool, String> {
    info!("Validating project number: {}", project_number);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.validate_project_number(&project_number).await {
        Ok(is_valid) => {
            info!("Project number {} is valid: {}", project_number, is_valid);
            Ok(is_valid)
        }
        Err(e) => {
            error!("Failed to validate project number: {}", e);
            Err(format!("Failed to validate project number: {}", e))
        }
    }
}

// Create project with template folder copy
#[tauri::command]
pub async fn create_project_with_template(project: NewProject, state: State<'_, AppState>) -> Result<Project, String> {
    info!("Creating project with template: {}", project.name);
    info!("Project data: {:?}", project);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    // First create the project in database
    info!("About to create project in database...");
    match manager_clone.create_new_project(project.clone()).await {
        Ok(created_project) => {
            info!("Successfully created project in database: {:?}", created_project.id);
            
            // Get project folder path from settings
            info!("Getting settings for project folder path...");
            let settings = get_settings().await.map_err(|e| format!("Failed to get settings: {}", e))?;
            
            info!("Settings loaded - project_folder_path: {:?}", settings.project_folder_path);
            
            if let Some(base_path) = settings.project_folder_path {
                // Copy template folder
                let template_path = format!("{}\\01 RFPs\\_yy-cccnn Project Name", base_path);
                let project_number = created_project.number.id.clone();
                let dest_folder_name = format!("{} {}", project_number, created_project.name_short);
                let dest_path = format!("{}\\01 RFPs\\{}", base_path, dest_folder_name);
                
                info!("Copying template from {} to {}", template_path, dest_path);
                
                // Use Windows xcopy for robust folder copying
                use std::process::Command;
                let result = Command::new("xcopy")
                    .args(&[&template_path, &dest_path, "/E", "/I", "/Q"])
                    .output();
                
                match result {
                    Ok(output) => {
                        if output.status.success() {
                            info!("Successfully copied template folder");
                            
                            // Rename files within the copied folder
                            if let Err(e) = rename_template_files(&dest_path, "yy-cccnn", &project_number) {
                                error!("Failed to rename template files: {}", e);
                                // Don't fail the entire operation just because rename failed
                            }
                        } else {
                            error!("Failed to copy template folder: {}", String::from_utf8_lossy(&output.stderr));
                        }
                    }
                    Err(e) => {
                        error!("Failed to execute xcopy: {}", e);
                    }
                }
            } else {
                info!("No project_folder_path configured in settings - skipping template folder creation");
            }
            
            Ok(created_project)
        }
        Err(e) => {
            error!("Failed to create project: {}", e);
            Err(format!("Failed to create project: {}", e))
        }
    }
}

// Helper function to rename template files
fn rename_template_files(dir_path: &str, old_pattern: &str, new_pattern: &str) -> Result<(), String> {
    use std::path::Path;
    
    let path = Path::new(dir_path);
    
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", dir_path));
    }
    
    // Walk through all files and folders recursively
    visit_dirs(path, old_pattern, new_pattern)?;
    
    Ok(())
}

fn visit_dirs(dir: &Path, old_pattern: &str, new_pattern: &str) -> Result<(), String> {
    use std::fs;
    
    if dir.is_dir() {
        for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recurse into subdirectories
                visit_dirs(&path, old_pattern, new_pattern)?;
            }
            
            // Check if filename contains the pattern
            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    if file_name_str.contains(old_pattern) {
                        let new_name = file_name_str.replace(old_pattern, new_pattern);
                        let new_path = path.with_file_name(new_name);
                        
                        info!("Renaming {:?} to {:?}", path, new_path);
                        fs::rename(&path, &new_path)
                            .map_err(|e| format!("Failed to rename file: {}", e))?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

// Get area suggestions for a country
#[tauri::command]
pub async fn get_area_suggestions(country: String, state: State<'_, AppState>) -> Result<Vec<String>, String> {
    info!("Getting area suggestions for country: {}", country);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.get_area_suggestions(&country).await {
        Ok(areas) => {
            info!("Found {} area suggestions", areas.len());
            Ok(areas)
        }
        Err(e) => {
            error!("Failed to get area suggestions: {}", e);
            Err(e.to_string())
        }
    }
}

// Get city suggestions for a country  
#[tauri::command]
pub async fn get_city_suggestions(country: String, state: State<'_, AppState>) -> Result<Vec<String>, String> {
    info!("Getting city suggestions for country: {}", country);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.get_city_suggestions(&country).await {
        Ok(cities) => {
            info!("Found {} city suggestions", cities.len());
            Ok(cities)
        }
        Err(e) => {
            error!("Failed to get city suggestions: {}", e);
            Err(e.to_string())
        }
    }
}

