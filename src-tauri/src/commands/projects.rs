//! Project management commands.
//!
//! This module provides Tauri commands for managing projects including
//! CRUD operations, search, pagination, and project number generation.

use super::AppState;
use crate::commands::types::ProjectUpdate;
use crate::crud_command;
use crate::db::{
    PaginatedResponse, Project, ProjectDeleteOutcome, ProjectDeletePreview, ProjectMergeOutcome,
    ProjectMergePreview,
};

use log::{error, info};
use tauri::State;

// ============================================================================
// PROJECT CRUD COMMANDS
// ============================================================================

/// Retrieve all projects from the database.
crud_command!(
    get_projects,
    Vec<Project>,
    get_projects,
    "fetch",
    "projects"
);

/// Create a new project in the database (without template).
crud_command!(
    create_project,
    Project,
    Project,
    create_project,
    "create",
    "project",
    data: project
);

/// Retrieve a paginated page of projects.
crud_command!(
    get_projects_page,
    PaginatedResponse<Project>,
    get_projects_page,
    "fetch page",
    "projects",
    paginated
);

/// Fetch a single project by ID.
crud_command!(
    get_project_by_id,
    Option<Project>,
    get_project_by_id,
    "fetch",
    "project",
    id: String
);

/// Delete a project from the database.
crud_command!(
    delete_project,
    Project,
    delete_project,
    "delete",
    "project",
    id: String
);

// ============================================================================
// PROJECT MERGE / CASCADE-DELETE COMMANDS
// ============================================================================
//
// These recover from a project created in error or a duplicate created from
// PA RFP intake, at the project level: merge folds one project's fee
// proposals into another; cascade-delete removes a project together with
// its proposals instead of orphaning them. See
// `src-tauri/src/db/project_lifecycle.rs` for the transactional
// implementation and its design notes.

/// Preview a project merge: which fees would move from `source_id` into
/// `target_id`, and how their revision numbers would change. Read-only -
/// call this to render a confirmation dialog before `merge_projects`.
#[tauri::command]
pub async fn preview_project_merge(
    source_id: String,
    target_id: String,
    state: State<'_, AppState>,
) -> Result<ProjectMergePreview, String> {
    info!(
        "Previewing merge of project '{}' into '{}'",
        source_id, target_id
    );
    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };
    manager_clone
        .preview_project_merge(&source_id, &target_id)
        .await
        .map_err(|e| {
            error!("Failed to preview project merge: {}", e);
            format!("Failed to preview project merge: {}", e)
        })
}

/// Merge `source_id` into `target_id`: reparent every fee proposal from the
/// source project onto the target, renumbering revisions only where needed
/// to avoid a collision, then delete the (now empty) source project.
#[tauri::command]
pub async fn merge_projects(
    source_id: String,
    target_id: String,
    state: State<'_, AppState>,
) -> Result<ProjectMergeOutcome, String> {
    info!("Merging project '{}' into '{}'", source_id, target_id);
    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };
    match manager_clone.merge_projects(&source_id, &target_id).await {
        Ok(outcome) => {
            info!(
                "Merged project '{}' into '{}': {} fee(s) moved",
                source_id, target_id, outcome.fees_moved
            );
            Ok(outcome)
        }
        Err(e) => {
            error!("Failed to merge projects: {}", e);
            Err(format!("Failed to merge projects: {}", e))
        }
    }
}

/// Preview a project delete: the project and every fee proposal that would
/// be cascade-deleted with it. Read-only - call this to render a
/// confirmation dialog before `delete_project_cascade`.
#[tauri::command]
pub async fn preview_project_delete(
    id: String,
    state: State<'_, AppState>,
) -> Result<ProjectDeletePreview, String> {
    info!("Previewing delete of project '{}'", id);
    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };
    manager_clone
        .preview_project_delete(&id)
        .await
        .map_err(|e| {
            error!("Failed to preview project delete: {}", e);
            format!("Failed to preview project delete: {}", e)
        })
}

/// Delete a project, cascading to its fee proposals. If the project has
/// fee proposals, `cascade` must be `true` or the call is refused - callers
/// should show `preview_project_delete`'s `dependent_fees` in a
/// confirmation dialog first.
#[tauri::command]
pub async fn delete_project_cascade(
    id: String,
    cascade: bool,
    state: State<'_, AppState>,
) -> Result<ProjectDeleteOutcome, String> {
    info!("Deleting project '{}' (cascade: {})", id, cascade);
    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };
    match manager_clone.delete_project_cascade(&id, cascade).await {
        Ok(outcome) => {
            info!(
                "Deleted project '{}' with {} cascaded fee(s)",
                id,
                outcome.deleted_fees.len()
            );
            Ok(outcome)
        }
        Err(e) => {
            error!("Failed to delete project: {}", e);
            Err(format!("Failed to delete project: {}", e))
        }
    }
}

/// Search projects using fuzzy matching across multiple fields.
#[tauri::command]
pub async fn search_projects(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<Project>, String> {
    info!("Searching projects with query: {}", query);

    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };

    match manager_clone.search_projects(&query).await {
        Ok(projects) => {
            info!("Search returned {} projects", projects.len());
            Ok(projects)
        }
        Err(e) => {
            error!("Failed to search projects: {}", e);
            Err(format!("Failed to search projects: {}", e))
        }
    }
}

/// Update an existing project in the database.
#[tauri::command]
pub async fn update_project(
    id: String,
    project_update: ProjectUpdate,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    info!("Updating project with ID: {}", id);

    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };

    match manager_clone.update_project(&id, project_update).await {
        Ok(project) => {
            info!("Successfully updated project: {}", id);
            Ok(project)
        }
        Err(e) => {
            error!("Failed to update project: {}", e);
            Err(format!("Failed to update project: {}", e))
        }
    }
}

// ============================================================================
// PROJECT NUMBER COMMANDS
// ============================================================================

/// Generate the next project number for a given country.
///
/// Project numbers follow the format: YY-CCCNN where:
/// - YY: 2-digit year
/// - CCC: 3-digit country dial code (e.g., 971 for UAE)
/// - NN: 2-digit sequence number (01-99)
#[tauri::command]
pub async fn generate_next_project_number(
    country_name: String,
    year: Option<u8>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!(
        "Generating next project number for country: {}, year: {:?}",
        country_name, year
    );

    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };

    match manager_clone
        .generate_next_project_number(&country_name, year)
        .await
    {
        Ok(number) => {
            info!("Generated project number: {}", number);
            Ok(number)
        }
        Err(e) => {
            error!("Failed to generate project number: {}", e);
            Err(format!("Failed to generate project number: {}", e))
        }
    }
}

/// Validate a project number format and check for uniqueness.
#[tauri::command]
pub async fn validate_project_number(
    project_number: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    info!("Validating project number: {}", project_number);

    let manager_clone = {
        let manager = state.read().await;
        manager.clone()
    };

    match manager_clone.validate_project_number(&project_number).await {
        Ok(is_valid) => {
            info!("Project number validation result: {}", is_valid);
            Ok(is_valid)
        }
        Err(e) => {
            error!("Failed to validate project number: {}", e);
            Err(format!("Failed to validate project number: {}", e))
        }
    }
}
