//! Venue management commands for E-Fees application.
//!
//! This module handles CRUD operations for venues (physical places
//! where projects are located).

use crate::crud_command;
use crate::db::{Venue, VenueCreate, VenueLocation, PaginatedResponse, Project};
use crate::commands::utils::execute_with_manager;
use tauri::State;

use super::AppState;

/// Retrieve all venues from the database, ordered by name.
crud_command!(
    get_venues,
    Vec<Venue>,
    get_venues,
    "fetch",
    "venues"
);

/// Retrieve a paginated page of venues.
crud_command!(
    get_venues_page,
    PaginatedResponse<Venue>,
    get_venues_page,
    "fetch page",
    "venues",
    paginated
);

/// Fetch a single venue by ID.
crud_command!(
    get_venue_by_id,
    Option<Venue>,
    get_venue_by_id,
    "fetch",
    "venue",
    id: String
);

/// Create a new venue in the database.
///
/// # Parameters
/// - `name`: Venue name (required)
/// - `name_short`: Short display name (optional)
/// - `city`, `country`, `area`: Location fields (optional)
/// - `tags`: Categorisation tags (optional)
/// - `notes`: Free-text notes (optional)
///
/// # Returns
/// - `Ok(Venue)`: Created venue with database-assigned ID and timestamps
/// - `Err(String)`: Validation error or database failure
#[tauri::command]
pub async fn create_venue(
    name: String,
    name_short: Option<String>,
    city: Option<String>,
    country: Option<String>,
    area: Option<String>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
    state: State<'_, AppState>,
) -> Result<Venue, String> {
    let venue = VenueCreate {
        name: name.clone(),
        name_short: name_short.unwrap_or_default(),
        location: VenueLocation {
            city: city.unwrap_or_default(),
            country: country.unwrap_or_default(),
            area: area.unwrap_or_default(),
        },
        tags: tags.unwrap_or_default(),
        notes: notes.unwrap_or_default(),
    };
    execute_with_manager(
        &state,
        |manager| Box::pin(async move {
            manager.create_venue(venue).await
        }),
        "create",
        &format!("venue '{}'", name),
    ).await
}

/// Update an existing venue.
///
/// # Parameters
/// - `id`: Venue record ID
/// - `name`: Updated name
/// - `name_short`, `city`, `country`, `area`, `tags`, `notes`: Updated fields
///
/// # Returns
/// - `Ok(Venue)`: Updated venue
/// - `Err(String)`: Venue not found or database failure
#[tauri::command]
pub async fn update_venue(
    id: String,
    name: String,
    name_short: Option<String>,
    city: Option<String>,
    country: Option<String>,
    area: Option<String>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
    state: State<'_, AppState>,
) -> Result<Venue, String> {
    let venue = VenueCreate {
        name,
        name_short: name_short.unwrap_or_default(),
        location: VenueLocation {
            city: city.unwrap_or_default(),
            country: country.unwrap_or_default(),
            area: area.unwrap_or_default(),
        },
        tags: tags.unwrap_or_default(),
        notes: notes.unwrap_or_default(),
    };
    let venue_label = format!("venue '{}'", id);
    execute_with_manager(
        &state,
        |manager| {
            let id_clone = id.clone();
            Box::pin(async move {
                manager.update_venue(&id_clone, venue).await
            })
        },
        "update",
        &venue_label,
    ).await
}

/// Delete a venue from the database.
///
/// This will fail if there are projects linked to this venue.
///
/// # Parameters
/// - `id`: Venue record ID to delete
///
/// # Returns
/// - `Ok(Venue)`: The deleted venue data
/// - `Err(String)`: Venue not found or has linked projects
crud_command!(
    delete_venue,
    Venue,
    delete_venue,
    "delete",
    "venue",
    id: String
);

/// Get all projects linked to a specific venue.
///
/// # Parameters
/// - `venue_id`: The venue ID to look up projects for
///
/// # Returns
/// - `Ok(Vec<Project>)`: Projects linked to the venue
/// - `Err(String)`: Database error
#[tauri::command]
pub async fn get_projects_for_venue(
    venue_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Project>, String> {
    let label = format!("projects for venue '{}'", venue_id);
    execute_with_manager(
        &state,
        |manager| {
            let id_clone = venue_id.clone();
            Box::pin(async move {
                manager.get_projects_for_venue(&id_clone).await
            })
        },
        "fetch",
        &label,
    ).await
}
