//! Project entity types.

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

use super::common::TimeStamps;

/// Project entity representing core business projects.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Project {
    pub id: Option<RecordId>,
    pub name: String,
    pub name_short: String,
    pub status: String,
    pub area: String,
    pub city: String,
    pub country: String,
    pub folder: String,
    pub number: ProjectNumber,
    pub time: TimeStamps,
}

/// Project number structure implementing the YY-CCCNN numbering system.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProjectNumber {
    pub year: i64,
    pub country: i64,
    pub seq: i64,
    pub id: String,
}

/// Project creation struct without auto-managed fields.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NewProject {
    pub name: String,
    pub name_short: String,
    pub status: String,
    pub area: String,
    pub city: String,
    pub country: String,
    pub folder: String,
    pub number: ProjectNumber,
}

/// API-specific project creation struct that supports optional auto-numbering.
/// When `number` is omitted, the API auto-assigns the next available number
/// based on the `country` field and the current year.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NewProjectApi {
    pub name: String,
    pub name_short: String,
    pub status: String,
    pub area: String,
    pub city: String,
    /// Country name (e.g. "UAE", "Saudi Arabia"). Used for number auto-assignment.
    pub country: String,
    pub folder: String,
    /// Project number. If omitted, auto-assigned from country dial code + next sequence.
    pub number: Option<ProjectNumber>,
}
