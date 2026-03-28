//! Reference data commands for E-Fees application
//!
//! This module handles country searches and location suggestions (areas, cities).

use log::{error, info};
use tauri::State;

use super::AppState;

/// Search countries with fuzzy matching across multiple fields.
///
/// This command performs comprehensive search across the countries reference
/// table, enabling users to find countries by name, code, or dial code.
/// It's used primarily by the NewProjectModal for country selection.
///
/// # Parameters
/// - `query`: Search string (minimum 2 characters recommended)
///
/// # Returns
/// - `Ok(Vec<serde_json::Value>)`: Matching country records
/// - `Err(String)`: Search error or database failure
///
/// # Search Fields
/// - `name`: Standard country name (e.g., "United Arab Emirates")
/// - `name_formal`: Formal country name
/// - `name_official`: Official country name
/// - `code`: ISO country code (e.g., "AE", "SA")
/// - `code_alt`: Alternative country codes
/// - `dial_code`: International dialing code (e.g., 971, 966)
///
/// # Search Strategy
/// Uses CONTAINS operator for substring matching across all fields
/// with NULL-safe queries to handle incomplete data gracefully.
///
/// # Frontend Usage
/// ```typescript
/// const countries = await invoke('search_countries', { query: 'UAE' });
/// // Returns: [{ name: "United Arab Emirates", dial_code: 971, ... }]
///
/// const byDialCode = await invoke('search_countries', { query: '971' });
/// // Same result - searches dial_code field too
/// ```
#[tauri::command]
pub async fn search_countries(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    info!("Searching countries with query: {}", query);

    let manager_clone = {
        let manager = state.read().await;
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

/// Get area suggestions for a specific country.
///
/// This command queries existing project data to provide autocomplete
/// suggestions for area fields when creating new projects. It helps
/// maintain consistency in location naming across projects.
///
/// # Parameters
/// - `country`: Country name to filter by (e.g., "United Arab Emirates")
///
/// # Returns
/// - `Ok(Vec<String>)`: List of unique area names from existing projects
/// - `Err(String)`: Query error or database failure
///
/// # Data Source
/// Suggestions are derived from the `area` field of existing projects
/// in the specified country, providing real-world location examples.
///
/// # Frontend Usage
/// ```typescript
/// const areas = await invoke('get_area_suggestions', {
///   country: 'United Arab Emirates'
/// });
/// // Returns: ["Dubai Marina", "Downtown Dubai", "Business Bay", ...]
/// ```
#[tauri::command]
pub async fn get_area_suggestions(
    country: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    info!("Getting area suggestions for country: {}", country);

    let manager_clone = {
        let manager = state.read().await;
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

/// Get all cities from existing projects.
///
/// This command retrieves all unique city names from projects in the database,
/// providing a complete list for autocomplete functionality.
///
/// # Returns
/// - `Ok(Vec<String>)`: List of all unique city names
/// - `Err(String)`: Query error or database failure
///
/// # Frontend Usage
/// ```typescript
/// const cities = await invoke('get_all_cities');
/// // Returns: ["Dubai", "Abu Dhabi", "Riyadh", ...]
/// ```
#[tauri::command]
pub async fn get_all_cities(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    info!("Getting all city suggestions");

    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };

    match manager_clone.get_all_cities().await {
        Ok(cities) => {
            info!("Successfully retrieved {} cities", cities.len());
            Ok(cities)
        }
        Err(e) => {
            error!("Failed to get all cities: {}", e);
            Err(format!("Failed to get all cities: {}", e))
        }
    }
}

/// Get city suggestions for a specific country.
///
/// This command provides autocomplete suggestions for city fields by
/// querying existing project data. Like area suggestions, it promotes
/// consistency in location naming.
///
/// # Parameters
/// - `country`: Country name to filter by (e.g., "United Arab Emirates")
///
/// # Returns
/// - `Ok(Vec<String>)`: List of unique city names from existing projects
/// - `Err(String)`: Query error or database failure
///
/// # Data Source
/// Suggestions come from the `city` field of existing projects in the
/// specified country, providing proven location examples.
///
/// # Frontend Usage
/// ```typescript
/// const cities = await invoke('get_city_suggestions', {
///   country: 'United Arab Emirates'
/// });
/// // Returns: ["Dubai", "Abu Dhabi", "Sharjah", ...]
/// ```
#[tauri::command]
pub async fn get_city_suggestions(
    country: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    info!("Getting city suggestions for country: {}", country);

    let manager_clone = {
        let manager = state.read().await;
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
