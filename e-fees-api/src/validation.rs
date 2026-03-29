//! Input validation for write endpoints.

use crate::error::ApiError;

/// Valid project statuses (from src/types/database.ts).
pub const PROJECT_STATUSES: &[&str] = &[
    "Lead",
    "RFP",
    "Submitted",
    "Awarded",
    "Design",
    "Construction",
    "Completed",
    "Lost",
    "No Response",
    "Cancelled",
    "On Hold",
    "Superseded",
];

/// Valid fee statuses (from src/types/database.ts).
pub const FEE_STATUSES: &[&str] = &[
    "Draft",
    "Sent",
    "Negotiation",
    "Accepted",
    "Rejected",
    "No Response",
    "Superseded",
];

/// Validate that a status string is in the allowed list.
pub fn validate_status(status: &str, allowed: &[&str], entity: &str) -> Result<(), ApiError> {
    if allowed.contains(&status) {
        Ok(())
    } else {
        Err(ApiError::bad_request(format!(
            "Invalid {} status '{}'. Must be one of: {}",
            entity,
            status,
            allowed.join(", ")
        )))
    }
}

/// Validate that a required string field is non-empty.
pub fn require_non_empty(value: &str, field: &str) -> Result<(), ApiError> {
    if value.trim().is_empty() {
        Err(ApiError::bad_request(format!(
            "'{}' must not be empty",
            field
        )))
    } else {
        Ok(())
    }
}

/// Validate a SurrealDB record ID path parameter.
/// Allows alphanumeric, underscore, hyphen. Max 100 chars.
pub fn validate_id(id: &str) -> Result<(), ApiError> {
    if id.is_empty()
        || id.len() > 100
        || !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        Err(ApiError::bad_request(format!("Invalid ID '{}'", id)))
    } else {
        Ok(())
    }
}
