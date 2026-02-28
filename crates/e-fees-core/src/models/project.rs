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
