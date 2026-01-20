//! Database connection commands for E-Fees application
//!
//! This module handles database connection status checking and diagnostics.

use crate::db::ConnectionStatus;
use log::info;
use tauri::State;

use super::AppState;

/// Check database connection status (simple boolean result).
///
/// This command provides a quick health check for the database connection.
/// It's used by the frontend to determine if the application can communicate
/// with SurrealDB and display appropriate connection indicators.
///
/// # Returns
/// - `Ok(true)`: Database is connected and responsive
/// - `Ok(false)`: Database is not connected or not responding
/// - `Err(String)`: Error occurred during connection check
///
/// # Frontend Usage
/// ```typescript
/// const isConnected = await invoke('check_db_connection');
/// ```
///
/// # Performance
/// This is a lightweight operation that should complete within 1-2 seconds
/// under normal conditions. Timeouts are handled at the database layer.
#[tauri::command]
pub async fn check_db_connection(state: State<'_, AppState>) -> Result<bool, String> {
    info!("Checking database connection");

    // Clone the database manager to avoid holding the lock during async operation
    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    }; // Read lock is automatically dropped here when manager goes out of scope

    let is_connected = manager_clone.check_connection().await;
    Ok(is_connected)
}

/// Get detailed connection status with diagnostic information.
///
/// This command provides comprehensive connection information including
/// timestamps, error messages, and configuration details. It's used for
/// debugging connection issues and displaying detailed status information.
///
/// # Returns
/// - `Ok(ConnectionStatus)`: Detailed connection information
/// - `Err(String)`: Error occurred while retrieving status
///
/// # ConnectionStatus Fields
/// - `is_connected`: Boolean connection state
/// - `last_check`: Timestamp of last connection attempt
/// - `error_message`: Last error message if connection failed
///
/// # Frontend Usage
/// ```typescript
/// const status = await invoke('get_connection_status');
/// console.log(`Connected: ${status.is_connected}, Last Check: ${status.last_check}`);
/// ```
#[tauri::command]
pub async fn get_connection_status(state: State<'_, AppState>) -> Result<ConnectionStatus, String> {
    info!("Getting detailed connection status");

    let manager = state.read().await;
    let status = manager.get_status().await;
    Ok(status)
}
