//! System and utility commands for E-Fees application
//!
//! This module handles health checks, statistics, database info,
//! window positioning, logging, and debugging utilities.

use chrono::Utc;
use log::{error, info};
use tauri::State;

use super::AppState;

/// Application health check command.
///
/// This command provides a simple way to verify that the Tauri backend
/// is running and responsive. It's used for system monitoring and debugging.
///
/// # Returns
/// - `Ok(String)`: Always returns "Application is running"
/// - `Err(String)`: Should never occur unless system is critically failing
///
/// # Frontend Usage
/// ```typescript
/// const status = await invoke('health_check');
/// console.log(status); // "Application is running"
/// ```
#[tauri::command]
pub async fn health_check() -> Result<String, String> {
    info!("Application health check");
    Ok("Application is running".to_string())
}

/// Get comprehensive database connection information for debugging.
///
/// This command provides detailed diagnostic information about the database
/// connection, configuration, and troubleshooting guidance. It's primarily
/// used for development and support purposes.
///
/// # Returns
/// - `Ok(serde_json::Value)`: Detailed connection information object
/// - `Err(String)`: Error accessing connection information
///
/// # Information Included
/// - Database configuration (URL, namespace, database, username)
/// - Current connection status with timestamp
/// - Troubleshooting guidance for common issues
/// - Environment configuration source information
///
/// # Security Note
/// Passwords are never included in the response for security reasons.
///
/// # Frontend Usage
/// ```typescript
/// const info = await invoke('get_db_info');
/// console.log(`Connected to: ${info.url}/${info.namespace}/${info.database}`);
/// console.log(`Status: ${info.connection_status.is_connected}`);
/// ```
#[tauri::command]
pub async fn get_db_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Getting database connection information");

    let (connection_status, config) = {
        let manager = state.read().await;
        (manager.get_status().await, manager.config.clone())
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

/// Get application statistics and metrics.
///
/// This command calculates and returns key application metrics for
/// dashboard displays and monitoring purposes. Statistics are calculated
/// in real-time from the database.
///
/// # Returns
/// - `Ok(serde_json::Value)`: Statistics object with counts and metrics
/// - `Err(String)`: Error calculating statistics
///
/// # Statistics Included
/// - `totalProjects`: Total count of all projects
/// - `activeRfps`: Count of RFPs not in 'Lost' or 'Cancelled' status
/// - `totalCompanies`: Total count of all companies
/// - `totalContacts`: Total count of all contacts
/// - `totalRfps`: Total count of all RFPs
///
/// # Performance
/// This command queries multiple tables and may take 1-2 seconds for
/// large datasets. Results should be cached on the frontend.
///
/// # Frontend Usage
/// ```typescript
/// const stats = await invoke('get_stats');
/// console.log(`Active RFPs: ${stats.activeRfps}/${stats.totalRfps}`);
/// console.log(`Total Projects: ${stats.totalProjects}`);
/// ```
#[tauri::command]
pub async fn get_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Fetching application statistics using efficient COUNT queries");

    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope

    // Use efficient COUNT queries instead of loading all records
    let counts = manager_clone.get_entity_counts().await
        .map_err(|e| format!("Failed to get entity counts: {}", e))?;

    let stats = serde_json::json!({
        "totalProjects": counts.total_projects,
        "activeFees": counts.active_fees,
        "totalCompanies": counts.total_companies,
        "totalContacts": counts.total_contacts,
        "totalFees": counts.total_fees
    });

    info!("Successfully calculated statistics: {:?}", stats);
    Ok(stats)
}

/// Get database table schema information for development.
///
/// This command retrieves the schema definition for a specified table,
/// including field definitions, constraints, and indexes. It's used for
/// development, debugging, and documentation purposes.
///
/// # Parameters
/// - `table_name`: Name of the table to inspect (e.g., "projects", "company")
///
/// # Returns
/// - `Ok(serde_json::Value)`: Table schema information
/// - `Err(String)`: Table not found or access error
///
/// # Schema Information
/// - Field definitions with types and constraints
/// - Index information for performance optimization
/// - Foreign key relationships
/// - Validation rules and default values
///
/// # Frontend Usage
/// ```typescript
/// const schema = await invoke('get_table_schema', { table_name: 'projects' });
/// console.log('Project fields:', schema.fields);
/// ```
#[tauri::command]
pub async fn get_table_schema(table_name: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Getting schema for table: {}", table_name);

    let manager_clone = {
        let manager = state.read().await;
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

/// Position application window on right half of 4K monitor.
///
/// This command provides a convenient way to position the application window
/// for optimal use with 4K monitors. It places the window on the right half
/// of the screen at full height.
///
/// # Parameters
/// - `window`: Tauri window handle (automatically provided)
///
/// # Returns
/// - `Ok(String)`: Success message
/// - `Err(String)`: Window positioning error
///
/// # Window Configuration
/// - **Position**: (1920, 0) - Right half of 4K screen
/// - **Size**: 1920x2160 - Half width, full height
/// - **Use Case**: Dual-monitor setup or large 4K display
///
/// # Frontend Usage
/// ```typescript
/// await invoke('position_window_4k');
/// // Window now positioned on right half of 4K monitor
/// ```
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

/// Investigate specific database record for debugging.
///
/// This command provides detailed information about a specific database
/// record, including its complete data structure, relationships, and
/// metadata. It's primarily used for debugging and development.
///
/// # Parameters
/// - `record_id`: Full record ID (e.g., "projects:25_97105", "company:CHE")
///
/// # Returns
/// - `Ok(serde_json::Value)`: Complete record data with metadata
/// - `Err(String)`: Record not found or access error
///
/// # Information Included
/// - Complete record data with all fields
/// - Related record information (if applicable)
/// - Database metadata (ID, timestamps, etc.)
/// - Field validation status
///
/// # Frontend Usage
/// ```typescript
/// const record = await invoke('investigate_record', {
///   record_id: 'projects:25_97105'
/// });
/// console.log('Record data:', record);
/// ```
#[tauri::command]
pub async fn investigate_record(record_id: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Investigating record: {}", record_id);

    let manager_clone = {
        let manager = state.read().await;
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

/// Frontend logging command for unified logging across frontend and backend.
///
/// This command allows the Svelte frontend to send log messages to the Rust
/// logging system, providing centralized logging with proper levels and targets.
///
/// # Parameters
/// - `level`: Log level (error, warn, info, debug, trace)
/// - `target`: Log target (component name, module, etc.)
/// - `message`: The log message
/// - `context`: Optional JSON context for structured logging
///
/// # Frontend Usage
/// ```typescript
/// await invoke('log_message', {
///   level: 'error',
///   target: 'ProjectModal',
///   message: 'Failed to create project',
///   context: JSON.stringify({ userId: '123', action: 'create' })
/// });
/// ```
#[tauri::command]
pub async fn log_message(
    level: String,
    target: String,
    message: String,
    context: Option<String>
) -> Result<(), String> {
    // Parse log level
    let log_level = match level.to_lowercase().as_str() {
        "error" => log::Level::Error,
        "warn" => log::Level::Warn,
        "info" => log::Level::Info,
        "debug" => log::Level::Debug,
        "trace" => log::Level::Trace,
        _ => log::Level::Info, // Default to info for unknown levels
    };

    // Format message with context if provided
    let formatted_message = if let Some(ctx) = context {
        if ctx.trim().is_empty() || ctx == "null" {
            message
        } else {
            format!("{} | Context: {}", message, ctx)
        }
    } else {
        message
    };

    // Log using the appropriate level
    match log_level {
        log::Level::Error => log::error!(target: &target, "{}", formatted_message),
        log::Level::Warn => log::warn!(target: &target, "{}", formatted_message),
        log::Level::Info => log::info!(target: &target, "{}", formatted_message),
        log::Level::Debug => log::debug!(target: &target, "{}", formatted_message),
        log::Level::Trace => log::trace!(target: &target, "{}", formatted_message),
    }

    // For updater logs, also write to dedicated log file
    if target == "updater" {
        if let Err(e) = write_updater_log(&level, &formatted_message) {
            log::warn!("Failed to write to updater log file: {}", e);
        }
    }

    Ok(())
}

/// Write a message to the updater log file.
/// This provides persistent logging for debugging update issues in production.
fn write_updater_log(level: &str, message: &str) -> Result<(), std::io::Error> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let log_path = std::env::temp_dir().join("e-fees-updater.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    writeln!(file, "[{}] [{}] {}", timestamp, level.to_uppercase(), message)?;
    file.flush()?;

    Ok(())
}
