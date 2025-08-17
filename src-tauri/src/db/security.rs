//! # Database Security Module
//! 
//! This module provides input validation to prevent SQL injection attacks.

use serde::{Deserialize, Serialize};
use regex::Regex;

/// Input validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    InvalidLength { field: String, min: usize, max: usize, actual: usize },
    InvalidCharacters { field: String, pattern: String },
    RequiredField { field: String },
    InvalidFormat { field: String, expected: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidLength { field, min, max, actual } => {
                write!(f, "Field '{}' length {} is outside allowed range {}-{}", field, actual, min, max)
            }
            ValidationError::InvalidCharacters { field, pattern } => {
                write!(f, "Field '{}' contains invalid characters. Expected pattern: {}", field, pattern)
            }
            ValidationError::RequiredField { field } => {
                write!(f, "Required field '{}' is missing or empty", field)
            }
            ValidationError::InvalidFormat { field, expected } => {
                write!(f, "Field '{}' has invalid format. Expected: {}", field, expected)
            }
        }
    }
}

/// Input validation functions
pub struct InputValidator;

impl InputValidator {
    /// Validate project name (alphanumeric, spaces, hyphens, underscores)
    pub fn validate_project_name(name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::RequiredField { field: "name".to_string() });
        }
        
        if name.len() < 2 || name.len() > 200 {
            return Err(ValidationError::InvalidLength {
                field: "name".to_string(),
                min: 2,
                max: 200,
                actual: name.len(),
            });
        }

        // Allow alphanumeric, spaces, hyphens, underscores, parentheses, and common punctuation
        let pattern = Regex::new(r"^[a-zA-Z0-9\s\-_().&,]+$").unwrap();
        if !pattern.is_match(name) {
            return Err(ValidationError::InvalidCharacters {
                field: "name".to_string(),
                pattern: "alphanumeric characters, spaces, hyphens, underscores, parentheses".to_string(),
            });
        }

        Ok(())
    }

    /// Validate email address
    pub fn validate_email(email: &str) -> Result<(), ValidationError> {
        if email.is_empty() {
            return Err(ValidationError::RequiredField { field: "email".to_string() });
        }

        let email_pattern = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if !email_pattern.is_match(email) {
            return Err(ValidationError::InvalidFormat {
                field: "email".to_string(),
                expected: "valid email address".to_string(),
            });
        }

        Ok(())
    }

    /// Validate phone number
    pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
        if phone.is_empty() {
            return Err(ValidationError::RequiredField { field: "phone".to_string() });
        }

        // Allow international phone formats
        let phone_pattern = Regex::new(r"^[\+]?[0-9\s\-\(\)]{7,20}$").unwrap();
        if !phone_pattern.is_match(phone) {
            return Err(ValidationError::InvalidFormat {
                field: "phone".to_string(),
                expected: "valid phone number".to_string(),
            });
        }

        Ok(())
    }

    /// Validate project number format (YY-CCCNN)
    pub fn validate_project_number(number: &str) -> Result<(), ValidationError> {
        let pattern = Regex::new(r"^\d{2}-\d{3}\d{2}$").unwrap();
        if !pattern.is_match(number) {
            return Err(ValidationError::InvalidFormat {
                field: "project_number".to_string(),
                expected: "YY-CCCNN format (e.g., 25-97105)".to_string(),
            });
        }

        Ok(())
    }

    /// Validate status values
    pub fn validate_status(status: &str, allowed_values: &[&str]) -> Result<(), ValidationError> {
        if !allowed_values.contains(&status) {
            return Err(ValidationError::InvalidFormat {
                field: "status".to_string(),
                expected: format!("one of: {}", allowed_values.join(", ")),
            });
        }

        Ok(())
    }

    /// Validate text field with length limits
    pub fn validate_text_field(field_name: &str, value: &str, min_len: usize, max_len: usize) -> Result<(), ValidationError> {
        if value.is_empty() && min_len > 0 {
            return Err(ValidationError::RequiredField { field: field_name.to_string() });
        }

        if value.len() < min_len || value.len() > max_len {
            return Err(ValidationError::InvalidLength {
                field: field_name.to_string(),
                min: min_len,
                max: max_len,
                actual: value.len(),
            });
        }

        Ok(())
    }

    /// Validate ID format (alphanumeric and underscores only)
    pub fn validate_id(id: &str) -> Result<(), ValidationError> {
        if id.is_empty() {
            return Err(ValidationError::RequiredField { field: "id".to_string() });
        }

        let id_pattern = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        if !id_pattern.is_match(id) {
            return Err(ValidationError::InvalidCharacters {
                field: "id".to_string(),
                pattern: "alphanumeric characters and underscores only".to_string(),
            });
        }

        Ok(())
    }

    /// Comprehensive input sanitization for dangerous characters
    pub fn sanitize_for_display(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || " -_.,()&@".contains(*c))
            .collect()
    }

    /// Escape single quotes for SurrealDB queries
    pub fn escape_single_quotes(input: &str) -> String {
        input.replace("'", "''")
    }
}