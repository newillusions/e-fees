//! Contact entity types.

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

use super::common::TimeStamps;

/// Contact entity.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Contact {
    pub id: Option<RecordId>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub position: Option<String>,
    #[serde(default)]
    pub company: Option<RecordId>,
    #[serde(default)]
    pub time: Option<TimeStamps>,
}

/// Contact creation struct.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ContactCreate {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub position: String,
    pub company: String,
}
