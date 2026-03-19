//! # Database Module Unit Tests
//!
//! Comprehensive test suite for database functionality including:
//! - Project number validation and parsing
//! - SQL injection prevention
//! - Database configuration loading
//! - Thing object handling
//! - Error handling
//! - Input validation (security module)

#[cfg(test)]
mod tests {
    use crate::db::DatabaseConfig;
    use crate::db::security::InputValidator;
    use serial_test::serial;
    use std::env;

    // ============================================================================
    // PROJECT NUMBER VALIDATION TESTS
    // ============================================================================

    #[test]
    fn test_project_number_validation_valid_formats() {
        // Valid 2-digit year, 3-digit country code, 2-digit sequence
        assert!(is_valid_project_number_format("25-97105"));
        assert!(is_valid_project_number_format("22-96601"));
        assert!(is_valid_project_number_format("24-97199"));
        assert!(is_valid_project_number_format("20-00001")); // Edge case: country code 000
    }

    #[test]
    fn test_project_number_validation_invalid_formats() {
        // Invalid formats
        assert!(!is_valid_project_number_format("invalid"));
        assert!(!is_valid_project_number_format("2025-971")); // 4-digit year
        assert!(!is_valid_project_number_format("25-971")); // Missing sequence
        assert!(!is_valid_project_number_format("25-9710")); // 4-digit country
        assert!(!is_valid_project_number_format("25-97")); // Missing sequence
        assert!(!is_valid_project_number_format("5-97105")); // 1-digit year
        assert!(!is_valid_project_number_format("25971-05")); // Wrong separator position
        assert!(!is_valid_project_number_format("")); // Empty string
        assert!(!is_valid_project_number_format("25_97105")); // Wrong separator
    }

    #[test]
    fn test_project_number_parsing_success() {
        let result = parse_project_number("25-97105");
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.year, 25);
        assert_eq!(parsed.country_code, 971);
        assert_eq!(parsed.sequence, 5);
    }

    #[test]
    fn test_project_number_parsing_edge_cases() {
        // Minimum values
        let min = parse_project_number("00-00001").unwrap();
        assert_eq!(min.year, 0);
        assert_eq!(min.country_code, 0);
        assert_eq!(min.sequence, 1);

        // Maximum values
        let max = parse_project_number("99-99999").unwrap();
        assert_eq!(max.year, 99);
        assert_eq!(max.country_code, 999);
        assert_eq!(max.sequence, 99);

        // Leading zeros preserved
        let zeros = parse_project_number("01-00101").unwrap();
        assert_eq!(zeros.year, 1);
        assert_eq!(zeros.country_code, 1);
        assert_eq!(zeros.sequence, 1);
    }

    #[test]
    fn test_project_number_parsing_errors() {
        assert!(parse_project_number("invalid").is_err());
        assert!(parse_project_number("25-971").is_err());
        assert!(parse_project_number("2025-97105").is_err());
        assert!(parse_project_number("").is_err());
    }

    #[test]
    fn test_project_number_range_validation() {
        // Sequence must be 1-99
        assert!(is_valid_sequence(1));
        assert!(is_valid_sequence(50));
        assert!(is_valid_sequence(99));
        assert!(!is_valid_sequence(0)); // Too low
        assert!(!is_valid_sequence(100)); // Too high
        assert!(!is_valid_sequence(200)); // Way too high
    }

    // ============================================================================
    // SQL INJECTION PREVENTION TESTS (CRITICAL SECURITY)
    // ============================================================================

    #[test]
    fn test_sql_escape_single_quotes() {
        let dangerous = "'; DROP TABLE projects; --";
        let escaped = escape_sql_string(dangerous);

        // Should escape single quotes (making them doubled)
        assert!(escaped.contains("''"));
        // Note: Escaping preserves the text content, just makes quotes safe
        // The actual SQL injection is prevented by quote doubling
        assert_eq!(escaped, "''; DROP TABLE projects; --");
    }

    #[test]
    fn test_sql_escape_special_characters() {
        let cases = vec![
            ("O'Brien", "O''Brien"),
            ("Test ' OR '1'='1", "Test '' OR ''1''=''1"),
            ("Robert'); DROP TABLE students;--", "Robert''); DROP TABLE students;--"),
            ("Normal text", "Normal text"),
            ("", ""),
        ];

        for (input, expected) in cases {
            let result = escape_sql_string(input);
            assert_eq!(result, expected, "Failed to escape: {}", input);
        }
    }

    #[test]
    fn test_sql_injection_attack_patterns() {
        let attack_patterns = vec![
            "' OR 1=1--",
            "'; DROP TABLE users; --",
            "' UNION SELECT * FROM passwords--",
            "admin'--",
            "' OR 'x'='x",
        ];

        for pattern in attack_patterns {
            let escaped = escape_sql_string(pattern);
            assert!(
                !escaped.contains("DROP") || escaped.contains("''"),
                "SQL injection pattern not properly escaped: {}",
                pattern
            );
        }
    }

    // ============================================================================
    // DATABASE CONFIGURATION TESTS
    // ============================================================================

    #[test]
    #[serial]
    fn test_database_config_from_env_success() {
        // Set up test environment variables
        env::set_var("SURREALDB_URL", "ws://localhost:8000");
        env::set_var("SURREALDB_NS", "test_namespace");
        env::set_var("SURREALDB_DB", "test_database");
        env::set_var("SURREALDB_USER", "test_user");
        env::set_var("SURREALDB_PASS", "test_password");

        let config = DatabaseConfig::from_env();
        assert!(config.is_ok());

        let cfg = config.unwrap();
        assert_eq!(cfg.url, "ws://localhost:8000");
        assert_eq!(cfg.namespace, "test_namespace");
        assert_eq!(cfg.database, "test_database");
        assert_eq!(cfg.username, "test_user");
        assert_eq!(cfg.password, "test_password");
        assert_eq!(cfg.verify_certificates, true); // Default
        assert_eq!(cfg.accept_invalid_hostnames, false); // Default

        // Clean up
        env::remove_var("SURREALDB_URL");
        env::remove_var("SURREALDB_NS");
        env::remove_var("SURREALDB_DB");
        env::remove_var("SURREALDB_USER");
        env::remove_var("SURREALDB_PASS");
    }

    #[test]
    #[serial]
    fn test_database_config_missing_required_vars() {
        // Ensure env vars are not set
        env::remove_var("SURREALDB_URL");
        env::remove_var("SURREALDB_NS");
        env::remove_var("SURREALDB_DB");
        env::remove_var("SURREALDB_USER");
        env::remove_var("SURREALDB_PASS");

        let config = DatabaseConfig::from_env();
        assert!(config.is_err());
        assert!(config.unwrap_err().contains("required"));
    }

    #[test]
    #[serial]
    fn test_database_config_tls_settings() {
        env::set_var("SURREALDB_URL", "wss://localhost:8000");
        env::set_var("SURREALDB_NS", "test");
        env::set_var("SURREALDB_DB", "test");
        env::set_var("SURREALDB_USER", "test");
        env::set_var("SURREALDB_PASS", "test");
        env::set_var("SURREALDB_VERIFY_CERTS", "false");
        env::set_var("SURREALDB_ACCEPT_INVALID_HOSTNAMES", "true");

        let config = DatabaseConfig::from_env().unwrap();
        assert_eq!(config.verify_certificates, false);
        assert_eq!(config.accept_invalid_hostnames, true);

        // Clean up
        env::remove_var("SURREALDB_URL");
        env::remove_var("SURREALDB_NS");
        env::remove_var("SURREALDB_DB");
        env::remove_var("SURREALDB_USER");
        env::remove_var("SURREALDB_PASS");
        env::remove_var("SURREALDB_VERIFY_CERTS");
        env::remove_var("SURREALDB_ACCEPT_INVALID_HOSTNAMES");
    }

    // ============================================================================
    // PROJECT NUMBER GENERATION TESTS
    // ============================================================================

    #[test]
    fn test_format_project_number() {
        assert_eq!(format_project_number(25, 971, 5), "25-97105");
        assert_eq!(format_project_number(24, 966, 1), "24-96601");
        assert_eq!(format_project_number(22, 971, 99), "22-97199");

        // Edge cases with leading zeros
        assert_eq!(format_project_number(1, 1, 1), "01-00101");
        assert_eq!(format_project_number(0, 0, 99), "00-00099");
    }

    #[test]
    fn test_increment_sequence() {
        assert_eq!(increment_sequence(1), Some(2));
        assert_eq!(increment_sequence(98), Some(99));
        assert_eq!(increment_sequence(99), None); // Max reached
        assert_eq!(increment_sequence(0), Some(1)); // Start from 0
    }

    // ============================================================================
    // THING OBJECT HANDLING TESTS
    // ============================================================================

    #[test]
    fn test_extract_thing_id() {
        use surrealdb::types::RecordId;

        // Create a RecordId using new method
        let record_id = RecordId::new("contacts", "john_doe");

        let extracted = extract_thing_id(&record_id);
        assert_eq!(extracted, Some("john_doe".to_string()));
    }

    #[test]
    fn test_extract_thing_id_from_string() {
        // Test string formats
        assert_eq!(extract_id_string("contacts:john_doe"), Some("john_doe".to_string()));
        assert_eq!(extract_id_string("company:ABC"), Some("ABC".to_string()));
        assert_eq!(extract_id_string("invalid"), None);
        assert_eq!(extract_id_string(""), None);
    }

    // ============================================================================
    // VALIDATION HELPER FUNCTIONS
    // ============================================================================

    #[test]
    fn test_validate_email_format() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag@example.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email(""));
    }

    #[test]
    fn test_validate_phone_format() {
        assert!(is_valid_phone("+971-50-123-4567"));
        assert!(is_valid_phone("+1-555-123-4567"));
        assert!(!is_valid_phone("123456")); // No country code
        assert!(!is_valid_phone("invalid"));
        assert!(!is_valid_phone(""));
    }

    #[test]
    fn test_validate_country_code() {
        assert!(is_valid_country_code(971)); // UAE
        assert!(is_valid_country_code(966)); // Saudi Arabia
        assert!(is_valid_country_code(1)); // USA
        assert!(is_valid_country_code(999)); // Max 3-digit
        assert!(!is_valid_country_code(0)); // Invalid
        assert!(!is_valid_country_code(1000)); // Too large
    }

    // ============================================================================
    // HELPER FUNCTIONS (Implementation)
    // ============================================================================

    fn is_valid_project_number_format(number: &str) -> bool {
        let pattern = regex::Regex::new(r"^\d{2}-\d{5}$").unwrap();
        pattern.is_match(number)
    }

    fn parse_project_number(number: &str) -> Result<ParsedProjectNumber, String> {
        if !is_valid_project_number_format(number) {
            return Err("Invalid project number format".to_string());
        }

        let parts: Vec<&str> = number.split('-').collect();
        if parts.len() != 2 {
            return Err("Invalid format".to_string());
        }

        let year = parts[0].parse::<u8>()
            .map_err(|_| "Invalid year".to_string())?;

        let country_and_seq = parts[1];
        if country_and_seq.len() != 5 {
            return Err("Invalid country/sequence length".to_string());
        }

        let country_code = country_and_seq[..3].parse::<u16>()
            .map_err(|_| "Invalid country code".to_string())?;

        let sequence = country_and_seq[3..].parse::<u8>()
            .map_err(|_| "Invalid sequence".to_string())?;

        Ok(ParsedProjectNumber {
            year,
            country_code,
            sequence,
        })
    }

    fn escape_sql_string(input: &str) -> String {
        input.replace("'", "''")
    }

    fn can_execute_sql_injection(sql: &str) -> bool {
        // Simple heuristic - check for dangerous SQL keywords after escaping
        let dangerous = vec!["DROP", "DELETE", "TRUNCATE", "ALTER"];
        let upper = sql.to_uppercase();

        for keyword in dangerous {
            if upper.contains(keyword) && !upper.contains("''") {
                return true;
            }
        }
        false
    }

    fn is_valid_sequence(seq: u8) -> bool {
        seq >= 1 && seq <= 99
    }

    fn format_project_number(year: u8, country: u16, seq: u8) -> String {
        format!("{:02}-{:03}{:02}", year, country, seq)
    }

    fn increment_sequence(current: u8) -> Option<u8> {
        if current >= 99 {
            None
        } else {
            Some(current + 1)
        }
    }

    fn extract_thing_id(record_id: &surrealdb::types::RecordId) -> Option<String> {
        use crate::db::types::record_key_string;
        Some(record_key_string(&record_id.key))
    }

    fn extract_id_string(id: &str) -> Option<String> {
        if id.contains(':') {
            id.split(':').nth(1).map(String::from)
        } else {
            None
        }
    }

    fn is_valid_email(email: &str) -> bool {
        let pattern = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        pattern.is_match(email)
    }

    fn is_valid_phone(phone: &str) -> bool {
        let pattern = regex::Regex::new(r"^\+\d{1,3}-\d{2,4}-\d{3,4}-\d{4}$").unwrap();
        pattern.is_match(phone)
    }

    fn is_valid_country_code(code: u16) -> bool {
        code > 0 && code < 1000
    }

    #[derive(Debug, PartialEq)]
    struct ParsedProjectNumber {
        year: u8,
        country_code: u16,
        sequence: u8,
    }

    // ============================================================================
    // INPUT VALIDATOR SECURITY TESTS (using actual InputValidator)
    // ============================================================================

    mod input_validator_tests {
        use super::*;

        // ----- Project Name Validation -----

        #[test]
        fn test_validate_project_name_valid() {
            // Normal project names
            assert!(InputValidator::validate_project_name("Test Project").is_ok());
            assert!(InputValidator::validate_project_name("Project-2025").is_ok());
            assert!(InputValidator::validate_project_name("Project_Name").is_ok());
            assert!(InputValidator::validate_project_name("Project (Phase 1)").is_ok());
            assert!(InputValidator::validate_project_name("Client & Partners").is_ok());
            assert!(InputValidator::validate_project_name("Name, Inc.").is_ok());
        }

        #[test]
        fn test_validate_project_name_empty() {
            let result = InputValidator::validate_project_name("");
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_project_name_too_short() {
            let result = InputValidator::validate_project_name("A");
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_project_name_too_long() {
            let long_name = "A".repeat(201);
            let result = InputValidator::validate_project_name(&long_name);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_project_name_sql_injection_attempts() {
            // SQL injection attempts should be rejected
            assert!(InputValidator::validate_project_name("'; DROP TABLE projects;--").is_err());
            assert!(InputValidator::validate_project_name("Project' OR '1'='1").is_err());
            assert!(InputValidator::validate_project_name("Test\"; DELETE FROM projects;").is_err());
            assert!(InputValidator::validate_project_name("Project`; DROP TABLE users;`").is_err());
        }

        #[test]
        fn test_validate_project_name_xss_attempts() {
            // XSS injection attempts should be rejected
            assert!(InputValidator::validate_project_name("<script>alert('XSS')</script>").is_err());
            assert!(InputValidator::validate_project_name("Project<img src=x onerror=alert(1)>").is_err());
        }

        // ----- Email Validation -----

        #[test]
        fn test_validate_email_valid() {
            assert!(InputValidator::validate_email("test@example.com").is_ok());
            assert!(InputValidator::validate_email("user.name@domain.co.uk").is_ok());
            assert!(InputValidator::validate_email("user+tag@example.org").is_ok());
        }

        #[test]
        fn test_validate_email_invalid() {
            assert!(InputValidator::validate_email("").is_err());
            assert!(InputValidator::validate_email("invalid").is_err());
            assert!(InputValidator::validate_email("@example.com").is_err());
            assert!(InputValidator::validate_email("user@").is_err());
            assert!(InputValidator::validate_email("user@.com").is_err());
        }

        #[test]
        fn test_validate_email_sql_injection() {
            // SQL injection in email fields
            assert!(InputValidator::validate_email("test'@example.com").is_err());
            assert!(InputValidator::validate_email("test'; DROP TABLE users;--@example.com").is_err());
        }

        // ----- Phone Validation -----

        #[test]
        fn test_validate_phone_valid() {
            assert!(InputValidator::validate_phone("+971501234567").is_ok());
            assert!(InputValidator::validate_phone("+1-555-123-4567").is_ok());
            assert!(InputValidator::validate_phone("(555) 123-4567").is_ok());
            assert!(InputValidator::validate_phone("555 123 4567").is_ok());
        }

        #[test]
        fn test_validate_phone_invalid() {
            assert!(InputValidator::validate_phone("").is_err());
            assert!(InputValidator::validate_phone("123").is_err()); // Too short
            assert!(InputValidator::validate_phone("123456789012345678901").is_err()); // Too long
        }

        #[test]
        fn test_validate_phone_sql_injection() {
            assert!(InputValidator::validate_phone("123'; DROP TABLE contacts;--").is_err());
            assert!(InputValidator::validate_phone("+1' OR '1'='1").is_err());
        }

        // ----- Project Number Validation -----

        #[test]
        fn test_validate_project_number_valid() {
            assert!(InputValidator::validate_project_number("25-97105").is_ok());
            assert!(InputValidator::validate_project_number("24-96601").is_ok());
            assert!(InputValidator::validate_project_number("00-00001").is_ok());
            assert!(InputValidator::validate_project_number("99-99999").is_ok());
        }

        #[test]
        fn test_validate_project_number_invalid_format() {
            assert!(InputValidator::validate_project_number("invalid").is_err());
            assert!(InputValidator::validate_project_number("2025-971").is_err());
            assert!(InputValidator::validate_project_number("25-971").is_err());
            assert!(InputValidator::validate_project_number("25971-05").is_err());
            assert!(InputValidator::validate_project_number("").is_err());
        }

        #[test]
        fn test_validate_project_number_sql_injection() {
            assert!(InputValidator::validate_project_number("25'; DROP TABLE projects;--").is_err());
            assert!(InputValidator::validate_project_number("' OR 1=1--").is_err());
        }

        // ----- Status Validation -----

        #[test]
        fn test_validate_status_valid() {
            let allowed = &["Lead", "RFP", "Submitted", "Awarded", "Design", "Construction", "Completed", "Lost", "No Response", "Cancelled", "On Hold", "Superseded"];
            assert!(InputValidator::validate_status("Lead", allowed).is_ok());
            assert!(InputValidator::validate_status("RFP", allowed).is_ok());
            assert!(InputValidator::validate_status("Submitted", allowed).is_ok());
            assert!(InputValidator::validate_status("Awarded", allowed).is_ok());
            assert!(InputValidator::validate_status("Design", allowed).is_ok());
            assert!(InputValidator::validate_status("Construction", allowed).is_ok());
            assert!(InputValidator::validate_status("Completed", allowed).is_ok());
            assert!(InputValidator::validate_status("Lost", allowed).is_ok());
            assert!(InputValidator::validate_status("No Response", allowed).is_ok());
            assert!(InputValidator::validate_status("Cancelled", allowed).is_ok());
            assert!(InputValidator::validate_status("On Hold", allowed).is_ok());
            assert!(InputValidator::validate_status("Superseded", allowed).is_ok());
        }

        #[test]
        fn test_validate_status_invalid() {
            let allowed = &["Lead", "RFP", "Submitted", "Awarded", "Design", "Construction", "Completed", "Lost", "No Response", "Cancelled", "On Hold", "Superseded"];
            assert!(InputValidator::validate_status("InvalidStatus", allowed).is_err());
            assert!(InputValidator::validate_status("", allowed).is_err());
            // Old statuses should no longer be valid
            assert!(InputValidator::validate_status("Active", allowed).is_err());
            assert!(InputValidator::validate_status("Draft", allowed).is_err());
            assert!(InputValidator::validate_status("Archived", allowed).is_err());
            assert!(InputValidator::validate_status("Tender", allowed).is_err());
        }

        #[test]
        fn test_validate_status_sql_injection() {
            let allowed = &["Lead", "RFP"];
            assert!(InputValidator::validate_status("Lead'; DROP TABLE--", allowed).is_err());
        }

        // ----- Text Field Validation -----

        #[test]
        fn test_validate_text_field_valid() {
            assert!(InputValidator::validate_text_field("name", "John", 1, 50).is_ok());
            assert!(InputValidator::validate_text_field("name", "A", 1, 50).is_ok()); // Min length
            assert!(InputValidator::validate_text_field("name", &"A".repeat(50), 1, 50).is_ok()); // Max length
        }

        #[test]
        fn test_validate_text_field_empty_when_required() {
            let result = InputValidator::validate_text_field("name", "", 1, 50);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_text_field_too_long() {
            let result = InputValidator::validate_text_field("name", &"A".repeat(51), 1, 50);
            assert!(result.is_err());
        }

        // ----- ID Validation -----

        #[test]
        fn test_validate_id_valid() {
            assert!(InputValidator::validate_id("john_doe").is_ok());
            assert!(InputValidator::validate_id("TEST123").is_ok());
            assert!(InputValidator::validate_id("company_ABC").is_ok());
        }

        #[test]
        fn test_validate_id_invalid() {
            assert!(InputValidator::validate_id("").is_err());
            assert!(InputValidator::validate_id("john-doe").is_err()); // Hyphen not allowed
            assert!(InputValidator::validate_id("john doe").is_err()); // Space not allowed
            assert!(InputValidator::validate_id("test@email").is_err()); // @ not allowed
        }

        #[test]
        fn test_validate_id_sql_injection() {
            assert!(InputValidator::validate_id("'; DROP TABLE contacts;--").is_err());
            assert!(InputValidator::validate_id("id' OR '1'='1").is_err());
        }

        // ----- Sanitization Functions -----

        #[test]
        fn test_sanitize_for_display() {
            assert_eq!(InputValidator::sanitize_for_display("Normal text"), "Normal text");
            assert_eq!(InputValidator::sanitize_for_display("Test-Project_123"), "Test-Project_123");
            assert_eq!(InputValidator::sanitize_for_display("Client & Co."), "Client & Co.");
            // Dangerous characters should be removed (quotes, angle brackets, semicolons)
            assert_eq!(InputValidator::sanitize_for_display("test'; DROP TABLE--"), "test DROP TABLE--");
            // Parentheses are allowed in sanitized output
            assert_eq!(InputValidator::sanitize_for_display("<script>alert(1)</script>"), "scriptalert(1)script");
        }

        #[test]
        fn test_escape_single_quotes() {
            assert_eq!(InputValidator::escape_single_quotes("test"), "test");
            assert_eq!(InputValidator::escape_single_quotes("O'Brien"), "O''Brien");
            assert_eq!(InputValidator::escape_single_quotes("'; DROP TABLE--"), "''; DROP TABLE--");
            assert_eq!(InputValidator::escape_single_quotes("' OR '1'='1"), "'' OR ''1''=''1");
        }
    }

    // ============================================================================
    // COMPREHENSIVE SQL INJECTION ATTACK PATTERN TESTS
    // ============================================================================

    mod sql_injection_attack_tests {
        use super::*;

        #[test]
        fn test_classic_sql_injection_patterns() {
            let attack_patterns = vec![
                "' OR '1'='1",
                "' OR 1=1--",
                "'; DROP TABLE users;--",
                "admin'--",
                "1'; DELETE FROM projects WHERE '1'='1",
                "'; TRUNCATE TABLE fees;--",
                "' UNION SELECT * FROM passwords--",
                "'); DROP TABLE contacts; --",
            ];

            for pattern in attack_patterns {
                // Project name validation should reject these
                let result = InputValidator::validate_project_name(pattern);
                assert!(result.is_err(), "Pattern should be rejected: {}", pattern);

                // ID validation should also reject
                let id_result = InputValidator::validate_id(pattern);
                assert!(id_result.is_err(), "ID validation should reject: {}", pattern);
            }
        }

        #[test]
        fn test_encoded_sql_injection_patterns() {
            // URL-encoded and hex-encoded attacks
            let encoded_patterns = vec![
                "test%27%20OR%201=1--", // URL encoded
                "test%00", // Null byte
            ];

            for pattern in encoded_patterns {
                let result = InputValidator::validate_project_name(pattern);
                assert!(result.is_err(), "Encoded pattern should be rejected: {}", pattern);
            }
        }

        #[test]
        fn test_comment_based_injection() {
            let comment_patterns = vec![
                "test/**/OR/**/1=1",
                "test--comment",
                "test#comment",
            ];

            for pattern in comment_patterns {
                // These contain special characters that should be rejected
                let result = InputValidator::validate_id(pattern);
                assert!(result.is_err(), "Comment pattern should be rejected: {}", pattern);
            }
        }

        #[test]
        fn test_escape_function_neutralizes_attacks() {
            // Even if these somehow passed validation, escaping should neutralize them
            let attacks = vec![
                ("'; DROP TABLE--", "''; DROP TABLE--"),
                ("test' OR '1'='1", "test'' OR ''1''=''1"),
                ("Robert'); DROP TABLE students;--", "Robert''); DROP TABLE students;--"),
            ];

            for (input, expected_escaped) in attacks {
                let escaped = InputValidator::escape_single_quotes(input);
                assert_eq!(escaped, expected_escaped);
                // The escaped version has doubled quotes which makes SQL injection ineffective
                assert!(escaped.contains("''"));
            }
        }
    }

    // ============================================================================
    // XSS PREVENTION TESTS
    // ============================================================================

    mod xss_prevention_tests {
        use super::*;

        #[test]
        fn test_xss_script_tags() {
            let xss_patterns = vec![
                "<script>alert('XSS')</script>",
                "<script src='evil.js'></script>",
                "<SCRIPT>alert(1)</SCRIPT>",
            ];

            for pattern in xss_patterns {
                let result = InputValidator::validate_project_name(pattern);
                assert!(result.is_err(), "XSS pattern should be rejected: {}", pattern);
            }
        }

        #[test]
        fn test_xss_event_handlers() {
            let xss_patterns = vec![
                "<img src=x onerror=alert(1)>",
                "<body onload=alert('XSS')>",
                "<div onclick=alert(1)>",
            ];

            for pattern in xss_patterns {
                let result = InputValidator::validate_project_name(pattern);
                assert!(result.is_err(), "XSS event handler should be rejected: {}", pattern);
            }
        }

        #[test]
        fn test_sanitize_removes_xss() {
            let xss = "<script>alert('XSS')</script>";
            let sanitized = InputValidator::sanitize_for_display(xss);
            assert!(!sanitized.contains("<"));
            assert!(!sanitized.contains(">"));
            assert!(!sanitized.contains("'"));
        }
    }

    // ============================================================================
    // BOUNDARY VALUE TESTS
    // ============================================================================

    mod boundary_value_tests {
        use super::*;

        #[test]
        fn test_text_field_boundary_values() {
            // Minimum boundary
            assert!(InputValidator::validate_text_field("test", "A", 1, 100).is_ok());
            assert!(InputValidator::validate_text_field("test", "", 1, 100).is_err());

            // Maximum boundary
            let at_max = "A".repeat(100);
            let over_max = "A".repeat(101);
            assert!(InputValidator::validate_text_field("test", &at_max, 1, 100).is_ok());
            assert!(InputValidator::validate_text_field("test", &over_max, 1, 100).is_err());
        }

        #[test]
        fn test_project_name_boundary_values() {
            // Min: 2 characters
            assert!(InputValidator::validate_project_name("AB").is_ok());
            assert!(InputValidator::validate_project_name("A").is_err());

            // Max: 200 characters
            let at_max = "A".repeat(200);
            let over_max = "A".repeat(201);
            assert!(InputValidator::validate_project_name(&at_max).is_ok());
            assert!(InputValidator::validate_project_name(&over_max).is_err());
        }

    // ============================================================================
    // SURREALDB VALUE SERIALIZATION TEST
    // ============================================================================

    #[test]
    fn test_dbvalue_serialization_format() {
        use surrealdb_types::Value as DbValue;
        use surrealdb_types::Number;

        // Check how DbValue serializes to JSON string
        let int_val = DbValue::Number(Number::Int(150000));
        let json = serde_json::to_string(&int_val).unwrap();
        println!("surrealdb_types Int serializes as: {}", json);

        // Check surrealdb::types::Value
        use surrealdb::types::Value as SdkValue;
        let sdk_int = SdkValue::Number(Number::Int(150000));
        let json = serde_json::to_string(&sdk_int).unwrap();
        println!("surrealdb::types Int serializes as: {}", json);

        // Check if they're the same type
        let same: bool = std::any::TypeId::of::<DbValue>() == std::any::TypeId::of::<SdkValue>();
        println!("Same type: {}", same);
    }
    }

    // ============================================================================
    // WEBSOCKET URL NORMALIZATION TESTS
    // ============================================================================

    #[test]
    fn test_normalize_ws_url_appends_rpc_when_missing() {
        use crate::db::client::normalize_ws_url;
        assert_eq!(normalize_ws_url("ws://10.0.23.11:8000"), "ws://10.0.23.11:8000/rpc");
    }

    #[test]
    fn test_normalize_ws_url_preserves_existing_rpc() {
        use crate::db::client::normalize_ws_url;
        assert_eq!(normalize_ws_url("ws://10.0.23.11:8000/rpc"), "ws://10.0.23.11:8000/rpc");
    }

    #[test]
    fn test_normalize_ws_url_handles_trailing_slash() {
        use crate::db::client::normalize_ws_url;
        assert_eq!(normalize_ws_url("ws://10.0.23.11:8000/"), "ws://10.0.23.11:8000/rpc");
    }

    #[test]
    fn test_normalize_ws_url_handles_wss() {
        use crate::db::client::normalize_ws_url;
        assert_eq!(normalize_ws_url("wss://prod.example.com:8000"), "wss://prod.example.com:8000/rpc");
        assert_eq!(normalize_ws_url("wss://prod.example.com:8000/rpc"), "wss://prod.example.com:8000/rpc");
    }

    // ============================================================================
    // PROD INTEGRATION TEST - run manually with:
    // cargo test test_prod_fee_deserialization -- --ignored --nocapture
    // ============================================================================

    #[tokio::test]
    #[ignore]
    async fn test_prod_fee_deserialization() {
        use surrealdb::engine::remote::ws::{Ws, Client};
        use surrealdb::Surreal;
        use surrealdb::opt::auth::Root;
        use crate::db::types::Fee;

        // Connect to PROD
        let db: Surreal<Client> = Surreal::new::<Ws>("10.0.23.11:8000").await
            .expect("Failed to connect to PROD WS");

        db.signin(Root {
            username: "martin".to_string(),
            password: "th38ret3ch".to_string(),
        }).await.expect("Failed to sign in");

        db.use_ns("emittiv").use_db("projects").await
            .expect("Failed to set ns/db");

        // Run the same query as get_fees()
        let mut response = db.query("SELECT * OMIT import_source FROM fee ORDER BY time.created_at DESC")
            .await.expect("Query failed");

        let result: Result<Vec<Fee>, _> = response.take(0);
        match result {
            Ok(fees) => {
                println!("SUCCESS: Deserialized {} fees from PROD", fees.len());
                assert!(fees.len() > 0, "Expected at least 1 fee from PROD");

                // Check first fee with pricing
                for fee in &fees {
                    if fee.pricing.is_some() {
                        let pricing_json = fee.pricing_typed();
                        println!("Fee {:?}: pricing_typed() = {:?}",
                            fee.id, pricing_json.is_some());

                        // Verify serde serialization works (this is what Tauri IPC sends to frontend)
                        let serialized = serde_json::to_string(&fee).unwrap();
                        println!("Fee serializes to JSON OK ({} bytes)", serialized.len());

                        // Verify pricing appears as plain JSON, not tagged enum
                        if let Some(ref p) = fee.pricing {
                            let pricing_json = crate::db::types::dbvalue_to_json(p);
                            println!("Pricing as plain JSON: {}", serde_json::to_string(&pricing_json).unwrap());
                            // Should NOT contain "Number" or "Int" wrapper keys
                            let s = serde_json::to_string(&pricing_json).unwrap();
                            assert!(!s.contains("\"Number\""), "Pricing JSON should be plain, not tagged");
                            assert!(!s.contains("\"Int\""), "Pricing JSON should be plain, not tagged");
                        }
                        break;
                    }
                }
                println!("ALL CHECKS PASSED");
            }
            Err(e) => {
                panic!("Fee deserialization FAILED: {}", e);
            }
        }
    }

    // ============================================================================
    // PROD PAGINATED TEST - tests the exact code path used by Proposals page
    // cargo test test_prod_fee_paginated -- --ignored --nocapture
    // ============================================================================

    #[tokio::test]
    #[ignore]
    async fn test_prod_fee_paginated() {
        use surrealdb::engine::remote::ws::{Ws, Client};
        use surrealdb::Surreal;
        use surrealdb::opt::auth::Root;
        use crate::db::types::Fee;

        // Connect to PROD
        let db: Surreal<Client> = Surreal::new::<Ws>("10.0.23.11:8000").await
            .expect("Failed to connect to PROD WS");

        db.signin(Root {
            username: "martin".to_string(),
            password: "th38ret3ch".to_string(),
        }).await.expect("Failed to sign in");

        db.use_ns("emittiv").use_db("projects").await
            .expect("Failed to set ns/db");

        // Run the EXACT same query as get_fees_page() — this is what was failing
        let page_size = 20;
        let offset = 0;
        let query = format!(
            "SELECT count() FROM fee GROUP ALL; SELECT * OMIT import_source FROM fee ORDER BY time.created_at DESC LIMIT {} START {}",
            page_size, offset
        );
        let mut response = db.query(&query).await.expect("Query failed");

        // Statement 0: count
        let count_result: Option<serde_json::Value> = response.take(0)
            .expect("Failed to take count result");
        let total = count_result
            .and_then(|v| v.get("count").and_then(|c| c.as_u64()))
            .unwrap_or(0);
        println!("Total fee count: {}", total);
        assert!(total > 0, "Expected at least 1 fee");

        // Statement 1: paginated fees — THIS WAS THE FAILING LINE
        let items: Vec<Fee> = response.take(1)
            .expect("Failed to deserialize paginated fees — this was the bug!");
        println!("SUCCESS: Deserialized {} paginated fees from PROD", items.len());
        assert!(!items.is_empty(), "Expected paginated results");

        // Verify rev field deserializes correctly
        for fee in &items {
            println!("Fee rev={}, status={}, name={}", fee.rev, fee.status, fee.name);
        }

        // CRITICAL: Test Tauri IPC serialization path
        // Tauri serializes the return value to JSON. If this fails, the frontend gets an error.
        let paginated = crate::db::types::PaginatedResponse::new(items.clone(), total as usize, 1, page_size);
        match serde_json::to_string(&paginated) {
            Ok(json) => {
                println!("SUCCESS: PaginatedResponse<Fee> serialized to JSON ({} bytes)", json.len());
                // Print first fee's JSON to verify format
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json) {
                    if let Some(first_item) = parsed.get("items").and_then(|i| i.as_array()).and_then(|a| a.first()) {
                        println!("First fee JSON keys: {:?}", first_item.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                        // Check the critical fields
                        println!("  id: {:?}", first_item.get("id"));
                        println!("  project_id: {:?}", first_item.get("project_id"));
                        println!("  rev: {:?}", first_item.get("rev"));
                        println!("  pricing: {:?}", first_item.get("pricing").map(|p| if p.is_null() { "null" } else { "present" }));
                        println!("  time.created_at: {:?}", first_item.get("time").and_then(|t| t.get("created_at")));
                    }
                }
            }
            Err(e) => {
                // This is likely the bug — serialization fails
                println!("SERIALIZATION FAILED: {}", e);
                // Try serializing each fee individually to find which one fails
                for (i, fee) in items.iter().enumerate() {
                    match serde_json::to_string(fee) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Fee #{} serialization failed: {} (name={}, rev={})", i, e, fee.name, fee.rev);
                            // Try each field
                            println!("  pricing is_some: {}", fee.pricing.is_some());
                            println!("  payment_schedule is_some: {}", fee.payment_schedule.is_some());
                            println!("  import_source is_some: {}", fee.import_source.is_some());
                        }
                    }
                }
                panic!("Tauri IPC serialization would fail: {}", e);
            }
        }

        // Also test all Projects (ProjectNumber has i64 fields)
        let mut proj_response = db.query("SELECT * FROM projects LIMIT 5").await
            .expect("Projects query failed");
        let projects: Vec<crate::db::types::Project> = proj_response.take(0)
            .expect("Failed to deserialize projects");
        println!("Deserialized {} projects", projects.len());
        for p in &projects {
            println!("Project: year={}, country={}, seq={}, id={}",
                p.number.year, p.number.country, p.number.seq, p.number.id);
        }

        // Test ALL fees (non-paginated) — this is the get_fees() path used by dashboard
        let mut all_fees_response = db.query("SELECT * OMIT import_source FROM fee ORDER BY time.created_at DESC")
            .await.expect("All fees query failed");
        let all_fees: Vec<Fee> = all_fees_response.take(0)
            .expect("Failed to deserialize ALL fees — non-paginated path broken!");
        println!("SUCCESS: Deserialized ALL {} fees (non-paginated)", all_fees.len());

        // Verify serialization for ALL fees
        for (i, fee) in all_fees.iter().enumerate() {
            if let Err(e) = serde_json::to_string(fee) {
                panic!("Fee #{} serialization failed: {} (id={:?}, rev={})", i, e, fee.id, fee.rev);
            }
        }
        println!("SUCCESS: All {} fees serialize to JSON", all_fees.len());

        println!("ALL PAGINATED CHECKS PASSED");
    }
}
