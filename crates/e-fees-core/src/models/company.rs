//! Company entity types.

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

use super::common::TimeStamps;

/// Company entity representing client organizations.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Company {
    pub id: Option<RecordId>,
    pub name: String,
    pub name_short: String,
    pub abbreviation: String,
    pub city: String,
    pub country: String,
    pub reg_no: Option<String>,
    pub tax_no: Option<String>,
    pub time: TimeStamps,
}

/// CompanyCreate represents a new company being created.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CompanyCreate {
    pub name: String,
    pub name_short: String,
    pub abbreviation: String,
    pub city: String,
    pub country: String,
    pub reg_no: Option<String>,
    pub tax_no: Option<String>,
}
