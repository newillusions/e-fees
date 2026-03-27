//! Stage dictionary commands for autocomplete support.
//!
//! Provides search and auto-insert for the `stage_config` table,
//! enabling autocomplete suggestions when users create stages.

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

use crate::commands::AppState;
use crate::commands::utils::execute_with_manager;

/// A single entry from the stage dictionary.
#[derive(Debug, Serialize, Deserialize)]
pub struct StageDictionaryEntry {
    pub canonical_name: String,
    pub default_label: String,
    pub code: Option<String>,
    pub is_post_contract: bool,
    pub usage_count: i64,
}

/// Search stage dictionary by name prefix/substring.
///
/// Returns up to 20 entries ordered by usage frequency (most-used first).
#[tauri::command]
pub async fn search_stage_dictionary(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<StageDictionaryEntry>, String> {
    execute_with_manager(
        &state,
        |manager| {
            let q = query.clone();
            Box::pin(async move {
                let client = manager.client.as_ref()
                    .ok_or_else(|| surrealdb::Error::thrown("DB not connected".to_string()))?;

                let mut bindings = serde_json::Map::new();
                bindings.insert("query".into(), json!(q));

                let mut res = client.query_bind_map(
                    "SELECT canonical_name, default_label, code, is_post_contract, usage_count \
                     FROM stage_config \
                     WHERE status = 'active' \
                     AND (string::lowercase(canonical_name) CONTAINS string::lowercase($query) \
                          OR string::lowercase(default_label) CONTAINS string::lowercase($query)) \
                     ORDER BY usage_count DESC \
                     LIMIT 20;",
                    bindings,
                )
                .await?;

                let raw: Vec<serde_json::Value> = res.take(0)?;
                let entries: Vec<StageDictionaryEntry> = raw.into_iter()
                    .filter_map(|v| serde_json::from_value(v).ok())
                    .collect();
                Ok(entries)
            })
        },
        "search",
        "stage dictionary",
    )
    .await
}

/// Add or update an entry in the stage dictionary.
///
/// If an entry with the given canonical name already exists, increments its
/// usage count. Otherwise creates a new entry with usage_count = 1.
#[tauri::command]
pub async fn add_stage_to_dictionary(
    name: String,
    code: Option<String>,
    is_post_contract: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    execute_with_manager(
        &state,
        |manager| {
            let n = name.clone();
            let c = code.clone();
            Box::pin(async move {
                let client = manager.client.as_ref()
                    .ok_or_else(|| surrealdb::Error::thrown("DB not connected".to_string()))?;

                // Check whether the entry exists first.
                let mut check_bindings = serde_json::Map::new();
                check_bindings.insert("name".into(), json!(n.clone()));

                let mut check_res = client.query_bind_map(
                    "SELECT count() FROM stage_config WHERE canonical_name = $name GROUP ALL;",
                    check_bindings,
                )
                .await?;

                let counts: Vec<serde_json::Value> = check_res.take(0)?;
                let exists = counts.first()
                    .and_then(|v| v.get("count"))
                    .and_then(|c| c.as_i64())
                    .unwrap_or(0) > 0;

                if exists {
                    let mut upd_bindings = serde_json::Map::new();
                    upd_bindings.insert("name".into(), json!(n));
                    client.query_bind_map(
                        "UPDATE stage_config SET usage_count += 1 WHERE canonical_name = $name;",
                        upd_bindings,
                    )
                    .await?;
                } else {
                    let mut ins_bindings = serde_json::Map::new();
                    ins_bindings.insert("name".into(), json!(n));
                    ins_bindings.insert("code".into(), json!(c));
                    ins_bindings.insert("is_post_contract".into(), json!(is_post_contract));
                    client.query_bind_map(
                        "CREATE stage_config SET \
                           canonical_name = $name, \
                           default_label  = $name, \
                           code           = $code, \
                           is_post_contract = $is_post_contract, \
                           sort_order     = 99, \
                           status         = 'active', \
                           usage_count    = 1;",
                        ins_bindings,
                    )
                    .await?;
                }

                Ok(())
            })
        },
        "upsert",
        "stage dictionary entry",
    )
    .await
}
