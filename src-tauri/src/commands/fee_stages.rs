//! Fee stage access commands.
//!
//! This module provides Tauri commands for reading and writing pricing stages
//! on fee records. Used by the scope module to link scope sections to pricing stages.

use serde::{Deserialize, Serialize};
use tauri::State;
use log::info;

use super::AppState;

// ============================================================================
// TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStage {
    pub id: String,
    pub name: String,
    pub code: String,
    pub percentage: f64,
    pub order: i64,
    pub is_post_contract: bool,
}

// ============================================================================
// COMMANDS
// ============================================================================

/// Get the pricing stages for a fee. Returns empty vec if no pricing configured.
#[tauri::command]
pub async fn get_fee_stages(
    fee_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<FeeStage>, String> {
    info!("Fetching fee stages for fee '{}'", fee_id);

    let manager = state.read().await;
    let client = manager.client.as_ref()
        .ok_or_else(|| "DB not connected".to_string())?;

    let mut bindings = serde_json::Map::new();
    bindings.insert("fee_key".to_string(), serde_json::Value::String(fee_id.clone()));

    let mut res = client
        .query_bind_map(
            "SELECT pricing FROM type::record('fee', $fee_key) LIMIT 1;",
            bindings,
        )
        .await
        .map_err(|e| format!("Failed to fetch fee stages: {}", e))?;

    let rows: Vec<serde_json::Value> = res
        .take(0)
        .map_err(|e| format!("Failed to parse fee stages result: {}", e))?;

    let stages = rows
        .into_iter()
        .next()
        .and_then(|r| r.get("pricing").cloned())
        .and_then(|p| p.get("stages").cloned())
        .and_then(|s| serde_json::from_value::<Vec<FeeStage>>(s).ok())
        .unwrap_or_default();

    info!("Found {} stages for fee '{}'", stages.len(), fee_id);
    Ok(stages)
}

/// Add a stage to a fee's pricing breakdown. Creates minimal pricing if none exists.
///
/// Idempotent: if a stage with the same id already exists it will not be duplicated.
#[tauri::command]
pub async fn add_stage_to_fee(
    fee_id: String,
    stage: FeeStage,
    state: State<'_, AppState>,
) -> Result<Vec<FeeStage>, String> {
    info!("Adding stage '{}' to fee '{}'", stage.id, fee_id);

    let manager = state.read().await;
    let client = manager.client.as_ref()
        .ok_or_else(|| "DB not connected".to_string())?;

    // Fetch current pricing blob
    let mut fetch_bindings = serde_json::Map::new();
    fetch_bindings.insert("fee_key".to_string(), serde_json::Value::String(fee_id.clone()));

    let mut res = client
        .query_bind_map(
            "SELECT pricing FROM type::record('fee', $fee_key) LIMIT 1;",
            fetch_bindings,
        )
        .await
        .map_err(|e| format!("Failed to fetch fee pricing: {}", e))?;

    let rows: Vec<serde_json::Value> = res
        .take(0)
        .map_err(|e| format!("Failed to parse fee pricing result: {}", e))?;

    let current_pricing = rows
        .into_iter()
        .next()
        .and_then(|r| r.get("pricing").cloned());

    // Build updated stages list (idempotent — skip if id already present)
    let mut stages: Vec<FeeStage> = current_pricing
        .as_ref()
        .and_then(|p| p.get("stages").cloned())
        .and_then(|s| serde_json::from_value(s).ok())
        .unwrap_or_default();

    if !stages.iter().any(|s| s.id == stage.id) {
        stages.push(stage);
    }

    // Merge stages back into the pricing blob
    let mut pricing: serde_json::Value = current_pricing.unwrap_or_else(|| serde_json::json!({}));
    pricing["stages"] = serde_json::to_value(&stages)
        .map_err(|e| format!("Failed to serialize stages: {}", e))?;

    // Write back
    let mut update_bindings = serde_json::Map::new();
    update_bindings.insert("fee_key".to_string(), serde_json::Value::String(fee_id.clone()));
    update_bindings.insert("pricing".to_string(), pricing);

    client
        .query_bind_map(
            "UPDATE type::record('fee', $fee_key) SET pricing = $pricing;",
            update_bindings,
        )
        .await
        .map_err(|e| format!("Failed to update fee stages: {}", e))?;

    info!("Successfully updated stages for fee '{}' ({} total)", fee_id, stages.len());
    Ok(stages)
}
