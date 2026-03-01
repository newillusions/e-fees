//! Batch operation commands for multi-select actions.

use crate::commands::utils::execute_with_manager;
use crate::commands::AppState;
use tauri::State;

/// Delete multiple entities from a table by their IDs.
/// Returns the records that were deleted (as JSON values).
#[tauri::command]
pub async fn batch_delete_entities(
    table: String,
    ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let entity_label = format!("{} entities", table);
    execute_with_manager(
        &state,
        |manager| Box::pin(async move {
            manager.batch_delete(&table, &ids).await
        }),
        "batch delete",
        &entity_label,
    ).await
}

/// Update the status of multiple entities in a table.
/// Returns the count of updated records.
#[tauri::command]
pub async fn batch_update_status(
    table: String,
    ids: Vec<String>,
    status: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let entity_label = format!("{} entities", table);
    execute_with_manager(
        &state,
        |manager| Box::pin(async move {
            manager.batch_update_status(&table, &ids, &status).await
        }),
        "batch update status",
        &entity_label,
    ).await
}
