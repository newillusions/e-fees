#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test ID matching logic used in write_fee_to_json
    #[test]
    fn test_fee_id_matching() {
        // Test cases based on the actual formats we see in the system
        let test_cases = vec![
            // (database_id, frontend_input, should_match)
            ("⟨22_96601_1⟩", "22_96601_1", true),
            ("⟨22_96601_1⟩", "fee:22_96601_1", true),
            ("⟨25_97107_1⟩", "25_97107_1", true),
            ("⟨25_97107_1⟩", "fee:25_97107_1", true),
            ("⟨22_96601_1⟩", "25_97107_1", false),
            ("⟨25_97107_1⟩", "22_96601_1", false),
        ];
        
        for (db_id, input_id, expected) in test_cases {
            let db_id_clean = db_id.trim_start_matches('⟨').trim_end_matches('⟩').to_string();
            let input_id_clean = input_id.trim_start_matches("fee:").to_string();
            
            let matches = db_id_clean == input_id_clean;
            
            assert_eq!(matches, expected, 
                "ID matching failed: db='{}' clean='{}' vs input='{}' clean='{}' -> expected {} got {}", 
                db_id, db_id_clean, input_id, input_id_clean, expected, matches
            );
        }
    }
    
    /// Test the angle bracket stripping logic
    #[test]
    fn test_angle_bracket_stripping() {
        let test_cases = vec![
            ("⟨22_96601_1⟩", "22_96601_1"),
            ("⟨25_97107_1⟩", "25_97107_1"),
            ("22_96601_1", "22_96601_1"), // No brackets
            ("", ""), // Empty string
        ];
        
        for (input, expected) in test_cases {
            let result = input.trim_start_matches('⟨').trim_end_matches('⟩');
            assert_eq!(result, expected, "Bracket stripping failed for '{}'", input);
        }
    }
    
    /// Test the fee prefix stripping logic  
    #[test]
    fn test_fee_prefix_stripping() {
        let test_cases = vec![
            ("fee:22_96601_1", "22_96601_1"),
            ("fee:25_97107_1", "25_97107_1"),
            ("22_96601_1", "22_96601_1"), // No prefix
            ("", ""), // Empty string
        ];
        
        for (input, expected) in test_cases {
            let result = input.trim_start_matches("fee:");
            assert_eq!(result, expected, "Fee prefix stripping failed for '{}'", input);
        }
    }
}