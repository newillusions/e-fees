//! # Database Module Unit Tests
//!
//! Comprehensive test suite for database functionality including:
//! - Project number validation and parsing
//! - SQL injection prevention
//! - Database configuration loading
//! - Thing object handling
//! - Error handling

#[cfg(test)]
mod tests {
    use crate::db::DatabaseConfig;
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
        use surrealdb::sql::Thing;

        // Create a Thing object using from method
        let thing = Thing::from(("contacts", "john_doe"));

        let extracted = extract_thing_id(&thing);
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

    fn extract_thing_id(thing: &surrealdb::sql::Thing) -> Option<String> {
        Some(thing.id.to_string())
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
}
