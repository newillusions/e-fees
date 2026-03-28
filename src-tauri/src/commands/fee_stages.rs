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

/// A stage from the scope service dictionary. Used for autocomplete suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDictEntry {
    pub canonical_name: String,
    pub default_label: String,
    pub aliases: Vec<String>,
    pub sort_order: i64,
}

/// Scope service response wrapper.
#[derive(Debug, Deserialize)]
struct StagesApiResponse {
    data: Vec<StageDictEntryRaw>,
}

/// Raw entry from scope API (aliases may be absent).
#[derive(Debug, Deserialize)]
struct StageDictEntryRaw {
    canonical_name: String,
    default_label: String,
    #[serde(default)]
    aliases: Vec<String>,
    sort_order: i64,
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

/// Fetch the stage dictionary from the scope service.
/// Returns a list of canonical stages with labels and aliases for autocomplete.
#[tauri::command]
pub async fn get_stage_dictionary(
    app_handle: tauri::AppHandle,
) -> Result<Vec<StageDictEntry>, String> {
    let settings = super::settings::get_settings_internal(&app_handle)
        .await
        .map_err(|e| format!("Failed to read settings: {}", e))?;

    let base_url = settings.scope_api_url
        .ok_or_else(|| "SCOPE_API_URL not configured in settings".to_string())?;
    let api_key = settings.scope_api_key
        .ok_or_else(|| "SCOPE_API_KEY not configured in settings".to_string())?;

    let url = format!("{}/stages", base_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("X-API-Key", &api_key)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Failed to reach scope service: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Scope service returned {}", response.status()));
    }

    let body: StagesApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse scope response: {}", e))?;

    let entries: Vec<StageDictEntry> = body.data.into_iter().map(|raw| StageDictEntry {
        canonical_name: raw.canonical_name,
        default_label: raw.default_label,
        aliases: raw.aliases,
        sort_order: raw.sort_order,
    }).collect();

    info!("Fetched {} stage dictionary entries from scope service", entries.len());
    Ok(entries)
}
