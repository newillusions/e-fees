//! Activity log commands for E-Fees application
//!
//! This module handles creation and retrieval of activity log entries
//! for tracking user actions across the system.

use crate::db::{ActivityLog, ActivityLogCreate};
use log::info;
use tauri::State;

use super::AppState;

/// Create a new activity log entry.
///
/// This command records user actions for the activity feed. Logs are
/// automatically timestamped by the database and synced across all machines.
///
/// # Parameters
/// - `log`: Activity log entry containing action details
///
/// # Returns
/// - `Ok(ActivityLog)`: Created log entry with database-assigned ID
/// - `Err(String)`: Database error
///
/// # Action Types
/// - `create`: New entity created
/// - `update`: Existing entity modified
/// - `delete`: Entity removed
/// - `status_change`: Entity status changed (e.g., project completed)
///
/// # Frontend Usage
/// ```typescript
/// await invoke('create_activity_log', {
///   log: {
///     action: 'status_change',
///     entity_type: 'project',
///     entity_id: 'projects:25-97105',
///     entity_name: 'Conrad Hilton Hotel',
///     description: 'Project marked as completed',
///     old_value: 'Active',
///     new_value: 'Completed'
///   }
/// });
/// ```
#[tauri::command]
pub async fn create_activity_log(
    log: ActivityLogCreate,
    state: State<'_, AppState>,
) -> Result<ActivityLog, String> {
    info!(
        "Creating activity log: {} on {}",
        log.action, log.entity_name
    );

    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };

    manager_clone
        .create_activity_log(log)
        .await
        .map_err(|e| format!("Failed to create activity log: {}", e))
}

/// Get recent activity logs with optional filtering.
///
/// This command retrieves activity logs for display in the dashboard's
/// Recent Activity panel. Logs are returned in reverse chronological order.
///
/// # Parameters
/// - `limit`: Maximum number of logs to return (default: 50)
/// - `entity_type`: Optional filter by entity type (project, fee, company, contact)
///
/// # Returns
/// - `Ok(Vec<ActivityLog>)`: List of activity logs
/// - `Err(String)`: Database error
///
/// # Frontend Usage
/// ```typescript
/// // Get last 20 activities
/// const logs = await invoke('get_activity_logs', { limit: 20 });
///
/// // Get only project-related activities
/// const projectLogs = await invoke('get_activity_logs', {
///   limit: 50,
///   entityType: 'project'
/// });
/// ```
#[tauri::command]
pub async fn get_activity_logs(
    limit: Option<usize>,
    entity_type: Option<String>,
    offset: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<ActivityLog>, String> {
    info!(
        "Fetching activity logs (limit: {:?}, entity_type: {:?}, offset: {:?})",
        limit, entity_type, offset
    );

    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };

    manager_clone
        .get_activity_logs(limit, entity_type, offset)
        .await
        .map_err(|e| format!("Failed to get activity logs: {}", e))
}
