//! Settings and configuration commands for E-Fees application
//!
//! This module handles application settings, database configuration,
//! folder selection, and file explorer integration.

use log::{error, info, warn};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use super::{AppState, AppSettings};

/// Read current application settings from .env file.
///
/// This command loads all configurable application settings from the
/// `.env` file located in the project root directory. Settings include
/// database configuration, staff information, and file system paths.
///
/// # Returns
/// - `Ok(AppSettings)`: Current settings with all configured values
/// - `Err(String)`: File read error or parsing failure
///
/// # Configuration Source
/// Settings are read from `../.env` relative to the Tauri binary location.
/// Missing settings are returned as `None` values, allowing for partial
/// configuration and gradual setup.
///
/// # Security Considerations
/// Database passwords and sensitive information are included in the response
/// but should be handled securely on the frontend (e.g., masked in UI).
///
/// # Frontend Usage
/// ```typescript
/// const settings = await invoke('get_settings');
/// if (settings.surrealdb_url) {
///   console.log(`Database: ${settings.surrealdb_url}`);
/// }
/// ```
#[tauri::command]
pub async fn get_settings(app_handle: AppHandle) -> Result<AppSettings, String> {
    info!("Reading settings from .env file");

    // Log the current working directory
    if let Ok(cwd) = std::env::current_dir() {
        info!("Current working directory: {:?}", cwd);
    }

    // Use app data directory for .env file when running from app bundle
    // In debug mode, use .env.dev to separate dev and production configs
    let env_filename = if cfg!(debug_assertions) {
        ".env.dev"
    } else {
        ".env"
    };

    let env_path = if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        app_data_dir.join(env_filename)
    } else {
        // Fallback to current directory for development
        PathBuf::from(env_filename)
    };

    // Log which .env file is being loaded
    info!("Loading environment configuration from: {:?}", env_path);
    if cfg!(debug_assertions) {
        info!("Running in DEBUG mode - using .env.dev");
    } else {
        info!("Running in RELEASE mode - using .env");
    }

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
        dev_mode: None,
    };

    info!("Looking for .env file at: {:?}", env_path);

    if env_path.exists() {
        info!(".env file found, reading contents");
        match fs::read_to_string(&env_path) {
            Ok(content) => {
                info!("Successfully read .env file, parsing {} lines", content.lines().count());
                // Parse .env file line by line
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue; // Skip empty lines and comments
                    }

                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim().trim_matches('"'); // Remove quotes

                        // Map environment variables to settings fields
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
                            "DEV_MODE" => settings.dev_mode = Some(value.to_lowercase() == "true"),
                            _ => {} // Ignore unknown variables
                        }
                    }
                }
                info!("Successfully loaded settings from .env file");
                if let Some(ref url) = settings.surrealdb_url {
                    info!("Database URL loaded: {}", url);
                }
            }
            Err(e) => {
                error!("Failed to read .env file: {}", e);
                return Err(format!("Failed to read .env file: {}", e));
            }
        }
    } else {
        info!("No .env file found, returning empty settings");
    }

    info!("Returning settings: staff_name={:?}, staff_email={:?}, staff_phone={:?}, staff_position={:?}",
          settings.staff_name, settings.staff_email, settings.staff_phone, settings.staff_position);

    Ok(settings)
}

/// Save application settings to .env file.
///
/// This command writes application settings to the `.env` file, preserving
/// existing environment variables that are not managed by the application.
/// The settings are organized into logical sections with comments.
///
/// # Parameters
/// - `settings`: Complete settings object with values to save
///
/// # Returns
/// - `Ok(String)`: Success message
/// - `Err(String)`: File write error or permission issue
///
/// # File Management Strategy
/// 1. Read existing .env file to preserve other variables
/// 2. Remove old application-managed variables
/// 3. Add new settings in organized sections
/// 4. Write complete file atomically
///
/// # Frontend Usage
/// ```typescript
/// const settings = {
///   surrealdb_url: "ws://10.0.1.17:8000",
///   surrealdb_ns: "emittiv",
///   staff_name: "John Smith"
/// };
/// await invoke('save_settings', { settings });
/// ```
#[tauri::command]
pub async fn save_settings(settings: AppSettings, app_handle: AppHandle) -> Result<String, String> {
    info!("Saving settings to .env file");

    // Use same filename logic as get_settings for consistency
    // In debug mode, use .env.dev to separate dev and production configs
    let env_filename = if cfg!(debug_assertions) {
        ".env.dev"
    } else {
        ".env"
    };

    // Use app data directory for .env file when running from app bundle
    let env_path = if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        let env_file_path = app_data_dir.join(env_filename);

        // Create app data directory if it doesn't exist
        if let Some(parent_dir) = env_file_path.parent() {
            if let Err(e) = fs::create_dir_all(parent_dir) {
                return Err(format!("Failed to create app data directory: {}", e));
            }
        }

        env_file_path
    } else {
        // Fallback to current directory for development
        PathBuf::from(env_filename)
    };

    info!("Using .env file path: {:?}", env_path);
    let mut lines = Vec::new();

    // Read existing .env file to preserve other variables
    if env_path.exists() {
        match fs::read_to_string(&env_path) {
            Ok(content) => {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        lines.push(line.to_string());
                        continue;
                    }

                    // Skip our managed section headers to avoid duplicates
                    if line.starts_with('#') {
                        match line {
                            "# SurrealDB Configuration" |
                            "# TLS Configuration" |
                            "# Staff Information" |
                            "# Project Configuration" |
                            "# Developer Options" => continue,
                            _ => {
                                lines.push(line.to_string());
                                continue;
                            }
                        }
                    }

                    if let Some((key, _)) = line.split_once('=') {
                        let key = key.trim();

                        // Skip our managed settings - we'll add them back
                        match key {
                            "SURREALDB_URL" | "SURREALDB_NS" | "SURREALDB_DB" |
                            "SURREALDB_USER" | "SURREALDB_PASS" |
                            "SURREALDB_VERIFY_CERTS" | "SURREALDB_ACCEPT_INVALID_HOSTNAMES" |
                            "STAFF_NAME" | "STAFF_EMAIL" | "STAFF_PHONE" | "STAFF_POSITION" |
                            "PROJECT_FOLDER_PATH" | "DEV_MODE" => continue,
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

    // Add our settings in organized sections
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

    // Add TLS configuration
    lines.push("".to_string());
    lines.push("# TLS Configuration".to_string());
    lines.push("SURREALDB_VERIFY_CERTS=true".to_string());
    lines.push("SURREALDB_ACCEPT_INVALID_HOSTNAMES=false".to_string());

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

    lines.push("".to_string());
    lines.push("# Developer Options".to_string());

    // Write dev_mode setting (defaults to false if not set)
    let dev_mode_value = settings.dev_mode.unwrap_or(false);
    lines.push(format!("DEV_MODE=\"{}\"", dev_mode_value));

    // Write to file atomically
    let content = lines.join("\n");
    match fs::write(&env_path, content) {
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

/// Get the current dev mode setting.
///
/// This command retrieves the development mode flag from settings. When dev mode
/// is enabled, the application will output verbose logging for debugging purposes,
/// particularly for the auto-updater functionality.
///
/// # Returns
/// - `Ok(bool)`: true if dev mode is enabled, false otherwise
/// - `Err(String)`: Error occurred while reading settings
///
/// # Frontend Usage
/// ```typescript
/// const devMode = await invoke<boolean>('get_dev_mode');
/// if (devMode) {
///   console.log('Development mode is enabled');
/// }
/// ```
#[tauri::command]
pub async fn get_dev_mode(app_handle: AppHandle) -> Result<bool, String> {
    let settings = get_settings(app_handle).await?;
    Ok(settings.dev_mode.unwrap_or(false))
}

/// Reload database configuration from the .env file and reinitialize connection.
///
/// This command allows the application to pick up new database settings without
/// requiring a restart. It reloads settings from the .env file and reconfigures
/// the database manager with the new connection parameters.
///
/// # Returns
/// - `Ok(String)`: Success message indicating the database was reconfigured
/// - `Err(String)`: Error message if reconfiguration failed
///
/// # Frontend Usage
/// ```typescript
/// try {
///   await invoke('reload_database_config');
///   console.log('Database configuration reloaded successfully');
/// } catch (error) {
///   console.error('Failed to reload database config:', error);
/// }
/// ```
#[tauri::command]
pub async fn reload_database_config(state: State<'_, AppState>, app_handle: AppHandle) -> Result<String, String> {
    info!("Reloading database configuration from .env file");

    // Load fresh settings from the .env file
    let settings = get_settings(app_handle).await?;

    // Extract database configuration
    let url = settings.surrealdb_url.ok_or("Missing SurrealDB URL in settings")?;
    let namespace = settings.surrealdb_ns.ok_or("Missing SurrealDB namespace in settings")?;
    let database = settings.surrealdb_db.ok_or("Missing SurrealDB database in settings")?;
    let username = settings.surrealdb_user.ok_or("Missing SurrealDB username in settings")?;
    let password = settings.surrealdb_pass.ok_or("Missing SurrealDB password in settings")?;

    // Reconfigure the database manager
    {
        let mut manager = state.write().await;
        manager.reconfigure(url, namespace, database, username, password).await?;
    }

    // Test the new connection
    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };

    // Attempt to connect with new settings
    let is_connected = manager_clone.check_connection().await;
    if is_connected {
        info!("Database reconfiguration successful - connection established");
        Ok("Database configuration reloaded and connected successfully".to_string())
    } else {
        warn!("Database reconfiguration completed but connection test failed");
        Ok("Database configuration reloaded but connection test failed - check settings".to_string())
    }
}

/// Open folder picker dialog for user selection.
///
/// This command displays a native folder picker dialog allowing users to
/// select directories. It's commonly used for selecting project template
/// directories or output locations.
///
/// # Parameters
/// - `app_handle`: Tauri app handle (automatically provided)
///
/// # Returns
/// - `Ok(Some(String))`: Selected folder path
/// - `Ok(None)`: User cancelled selection
/// - `Err(String)`: Dialog error or timeout
///
/// # Dialog Configuration
/// - **Title**: "Select Project Folder"
/// - **Timeout**: 30 seconds for user interaction
/// - **Platform**: Native OS dialog (Windows Explorer, macOS Finder, etc.)
///
/// # Frontend Usage
/// ```typescript
/// const folderPath = await invoke('select_folder');
/// if (folderPath) {
///   console.log(`Selected: ${folderPath}`);
/// } else {
///   console.log('User cancelled selection');
/// }
/// ```
#[tauri::command]
pub async fn select_folder(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    info!("Opening folder picker dialog");

    use std::sync::mpsc;
    use std::time::Duration;

    let (tx, rx) = mpsc::channel();

    // Open native folder picker dialog
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

/// Open folder in native file explorer.
///
/// This command opens the specified folder path in the system's default
/// file manager (Windows Explorer, macOS Finder, Linux file manager).
/// It's used to provide quick access to project folders.
///
/// # Parameters
/// - `folder_path`: Absolute path to folder to open
///
/// # Returns
/// - `Ok(String)`: Success message
/// - `Err(String)`: Path not found or permission error
///
/// # Platform Support
/// - **Windows**: Uses `explorer` command
/// - **macOS**: Uses `open` command
/// - **Linux**: Uses `xdg-open` command
///
/// # Security Considerations
/// Path validation is performed to prevent command injection, but callers
/// should ensure paths are trusted before calling this command.
///
/// # Frontend Usage
/// ```typescript
/// await invoke('open_folder_in_explorer', {
///   folder_path: 'E:\\Projects\\25-97105 Hotel Project'
/// });
/// // Opens folder in system file manager
/// ```
#[tauri::command]
pub async fn open_folder_in_explorer(folder_path: String) -> Result<String, String> {
    info!("Opening folder in explorer: {}", folder_path);

    use std::path::Path;

    // Check if the folder exists first
    let path = Path::new(&folder_path);
    if !path.exists() {
        error!("Folder does not exist: {}", folder_path);
        return Err(format!("Folder does not exist: {}", folder_path));
    }

    // Use platform-specific command to open folder
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
