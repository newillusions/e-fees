//! Input validation for write endpoints.
//!
//! Mirrors `e-fees-api/src/validation.rs::validate_id` - e-fees-scope has its
//! own axum app/error type so the helper is duplicated rather than shared
//! across a crate boundary that doesn't otherwise exist between the two
//! services (see e-fees CI review, security-pass 2026-09-15).

use crate::error::ApiError;

/// Validate a SurrealDB record ID path/body parameter.
/// Allows alphanumeric, underscore, hyphen. Max 100 chars.
///
/// Several handlers in this service (clauses, deliverables, scope) build
/// raw SurrealQL text with a record key interpolated directly into it
/// (`UPDATE clause:{key}`, `` fee:`{fee_key}` ``, ...) rather than binding it
/// as a parameter. Call this on every such key BEFORE it reaches a `format!`
/// that feeds `.query()`, so a value carrying a quote/backtick/semicolon is
/// rejected as 400 instead of being interpolated into query text.
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
    fn validate_id_accepts_normal_keys() {
        assert!(validate_id("some_clause_1").is_ok());
        assert!(validate_id("25_97101_1").is_ok());
    }

    #[test]
    fn validate_id_rejects_empty_and_oversized() {
        assert!(validate_id("").is_err());
        assert!(validate_id(&"a".repeat(101)).is_err());
    }

    /// scope.rs generate_scope/regenerate_scope/fetch_corpus_examples build
    /// `` SELECT ... FROM fee:`{fee_key}` `` with a backtick-quoted record id -
    /// a fee_key containing a backtick breaks out of that quoting.
    #[test]
    fn validate_id_rejects_backtick_breakout_in_fee_key() {
        let malicious = "1`; DELETE scope_assembly WHERE true;--";
        assert!(validate_id(malicious).is_err());
    }

    /// clauses.rs/deliverables.rs update_* and delete_* build
    /// `UPDATE clause:{key} SET ...; SELECT * FROM clause:{key};` with a bare
    /// (unquoted) record id - a key containing a semicolon terminates the
    /// UPDATE statement early and appends an attacker-controlled statement.
    #[test]
    fn validate_id_rejects_statement_injection_in_bare_record_key() {
        let malicious = "1; DELETE clause WHERE true;--";
        assert!(validate_id(malicious).is_err());
    }
}
