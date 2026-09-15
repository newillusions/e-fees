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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_id_accepts_normal_record_keys() {
        assert!(validate_id("26_97104").is_ok());
        assert!(validate_id("fee-revision-2").is_ok());
    }

    #[test]
    fn validate_id_rejects_empty_and_oversized() {
        assert!(validate_id("").is_err());
        assert!(validate_id(&"a".repeat(101)).is_err());
    }

    /// projects.rs create_project builds `CREATE projects:{key}` from a
    /// client-suppliable `body.number.id` (hyphens normalised to underscores
    /// upstream, but nothing else is normalised) - a record id carrying an
    /// injected statement terminator must be rejected before it ever reaches
    /// query text.
    #[test]
    fn validate_id_rejects_sqli_attempt_in_project_record_key() {
        let malicious = "26_97104; DROP TABLE fee;--";
        assert!(validate_id(malicious).is_err());
    }

    /// fees.rs create_fee interpolates `project_key` both as a bare record-id
    /// segment and inside a single-quoted `type::record('projects', '...')`
    /// literal - a value containing a quote/backtick must be rejected.
    #[test]
    fn validate_id_rejects_sqli_attempt_in_fee_project_key() {
        let malicious = "97104') OR '1'='1";
        assert!(validate_id(malicious).is_err());
        let backtick_attempt = "97104`; SELECT * FROM app_state;";
        assert!(validate_id(backtick_attempt).is_err());
    }
}
