//! Shared fee export utilities used by both the Tauri desktop app and the standalone API.
//!
//! This module contains pure functions for formatting and building fee JSON data
//! for InDesign variable data export. No I/O, no DB access — pure logic only.

pub mod indesign_workbook;

use chrono::Utc;
use serde_json::json;

use crate::models::{Company, Contact, Fee, Project};

// ============================================================================
// PLACEHOLDER CONSTANTS
// ============================================================================

/// Known template placeholder values inserted by InDesign variable data merge.
/// Any field matching these values is considered "not set" by the user.
pub const PLACEHOLDER_VALUES: &[&str] = &[
    "<variable not set>",
    "«variable not set»",
    "<none>",
    "«none»",
    "#error",
];

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Strip Unicode angle bracket wrappers (⟨…⟩) from SurrealDB v3 RecordId key strings.
///
/// SurrealDB v3 serializes certain RecordId keys with Unicode left/right angle brackets
/// (U+27E8 and U+27E9). This is a display artifact — strip them to get the bare key.
///
/// # Examples
/// ```
/// use e_fees_core::export::clean_record_key;
/// assert_eq!(clean_record_key("⟨25_97101⟩"), "25_97101");
/// assert_eq!(clean_record_key("25_97101"), "25_97101");
/// ```
pub fn clean_record_key(key: &str) -> String {
    key.trim_start_matches('\u{27E8}')
        .trim_end_matches('\u{27E9}')
        .to_string()
}

/// Clean a DB record key and convert underscores to dashes for filesystem use.
///
/// DB stores project numbers as `25_97101`; filesystem paths use `25-97101`.
///
/// # Examples
/// ```
/// use e_fees_core::export::clean_number_for_path;
/// assert_eq!(clean_number_for_path("25_97101"), "25-97101");
/// assert_eq!(clean_number_for_path("⟨25_97101⟩"), "25-97101");
/// ```
pub fn clean_number_for_path(key: &str) -> String {
    clean_record_key(key).replace('_', "-")
}

/// Returns true if the value is empty or matches a known InDesign template placeholder.
///
/// # Examples
/// ```
/// use e_fees_core::export::is_placeholder;
/// assert!(is_placeholder(""));
/// assert!(is_placeholder("<variable not set>"));
/// assert!(!is_placeholder("John Doe"));
/// ```
pub fn is_placeholder(value: &str) -> bool {
    value.is_empty() || PLACEHOLDER_VALUES.contains(&value)
}

// ============================================================================
// DATE FORMATTING
// ============================================================================

/// Format a fee issue date from YYMMDD (6-char string) to "dd MMM yyyy".
///
/// Year interpretation: < 50 → 2000s, >= 50 → 1900s.
/// Falls back to current UTC date if input is invalid.
///
/// # Examples
/// ```
/// use e_fees_core::export::format_issue_date;
/// assert_eq!(format_issue_date("250115"), "15 Jan 2025");
/// assert_eq!(format_issue_date("991231"), "31 Dec 1999");
/// assert_eq!(format_issue_date("000101"), "01 Jan 2000");
/// // invalid → non-empty fallback
/// assert!(!format_issue_date("badval").is_empty());
/// ```
pub fn format_issue_date(date_str: &str) -> String {
    if date_str.len() == 6 {
        if let (Ok(year), Ok(month), Ok(day)) = (
            date_str[0..2].parse::<i32>(),
            date_str[2..4].parse::<u32>(),
            date_str[4..6].parse::<u32>(),
        ) {
            let full_year = if year >= 50 { 1900 + year } else { 2000 + year };
            if let Some(date) = chrono::NaiveDate::from_ymd_opt(full_year, month, day) {
                return date.format("%d %b %Y").to_string();
            }
        }
    }
    Utc::now().format("%d %b %Y").to_string()
}

// ============================================================================
// JSON BUILDER
// ============================================================================

/// Build the 21-field InDesign variable data JSON for a fee proposal export.
///
/// All string fields are cloned from references; no ownership transfer required.
///
/// # Arguments
/// * `fee` — The fee proposal record
/// * `project` — The linked project record
/// * `company` — The linked company record
/// * `contact` — The linked contact record
pub fn build_fee_json(
    fee: &Fee,
    project: &Project,
    company: &Company,
    contact: &Contact,
) -> serde_json::Value {
    let issue_date = format_issue_date(&fee.issue_date);
    let contact_name = contact.full_name.clone().unwrap_or_else(|| {
        let first = contact.first_name.clone().unwrap_or_default();
        let last = contact.last_name.clone().unwrap_or_default();
        format!("{} {}", first, last)
    });

    json!({
        "01 Document Name": fee.name.clone(),
        "02 Document Number": fee.number.clone(),
        "03 Document Release": format!("{:02}", fee.rev),
        "04 Document Issue Date": issue_date,
        "06 Project Name": project.name.clone(),
        "07 Project Activity": fee.activity.clone(),
        "08 Project Package": fee.package.clone(),
        "09 Project Stage": project.status.clone(),
        "11 Project Area": project.area.clone(),
        "12 Project City": project.city.clone(),
        "13 Project Country": project.country.clone(),
        "21 Client Company": company.name.clone(),
        "22 Client City": company.city.clone(),
        "23 Client Country": company.country.clone(),
        "26 Contact Name": contact_name,
        "27 Contact Position": contact.position.clone().unwrap_or_default(),
        "28 Contact Phone": contact.phone.clone().unwrap_or_default(),
        "29 Contact Email": contact.email.clone().unwrap_or_default(),
        "91 Staff Name": fee.staff_name.clone(),
        "92 Staff Position": fee.staff_position.clone(),
        "93 Staff Phone": fee.staff_phone.clone(),
        "94 Staff Email": fee.staff_email.clone(),
        "99 Strap Line": fee.strap_line.clone()
    })
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use surrealdb::types::RecordId;

    use crate::models::{
        common::TimeStamps,
        company::Company,
        contact::Contact,
        fee::Fee,
        project::{Project, ProjectNumber},
    };
    use surrealdb_types::Datetime;

    // ---- helpers ----

    fn make_timestamps() -> TimeStamps {
        TimeStamps {
            created_at: Datetime::default(),
            updated_at: Datetime::default(),
        }
    }

    fn make_record_id(table: &str, key: &str) -> RecordId {
        RecordId::new(table, key)
    }

    fn make_fee() -> Fee {
        Fee {
            id: None,
            name: "Test Fee".to_string(),
            number: "TF-001".to_string(),
            rev: 3,
            status: "Draft".to_string(),
            issue_date: "260315".to_string(),
            activity: "Lighting Design".to_string(),
            package: "Full".to_string(),
            project_id: make_record_id("projects", "26_97101"),
            company_id: make_record_id("company", "acme"),
            contact_id: make_record_id("contacts", "john"),
            staff_name: "Alice Smith".to_string(),
            staff_email: "alice@emittiv.com".to_string(),
            staff_phone: "+971501234567".to_string(),
            staff_position: "Senior Designer".to_string(),
            strap_line: "Excellence in light".to_string(),
            revisions: vec![],
            time: make_timestamps(),
            pricing: None,
            post_contract_items: None,
            reimbursable_costs: None,
            payment_schedule: None,
            pricing_revisions: None,
            current_revision_number: None,
            current_release_number: None,
            import_source: None,
        }
    }

    fn make_project() -> Project {
        Project {
            id: None,
            name: "Burj Al Arab Renovation".to_string(),
            name_short: "BAA Reno".to_string(),
            status: "Design".to_string(),
            area: "120000".to_string(),
            city: "Dubai".to_string(),
            country: "UAE".to_string(),
            folder: "/projects/26-97101".to_string(),
            number: ProjectNumber {
                year: 26,
                country: 971,
                seq: 1,
                id: "26_97101".to_string(),
            },
            time: make_timestamps(),
        }
    }

    fn make_company() -> Company {
        Company {
            id: None,
            name: "Acme Holdings LLC".to_string(),
            name_short: "Acme".to_string(),
            abbreviation: "AH".to_string(),
            city: "Dubai".to_string(),
            country: "UAE".to_string(),
            reg_no: None,
            tax_no: None,
            time: make_timestamps(),
        }
    }

    fn make_contact_full_name() -> Contact {
        Contact {
            id: None,
            first_name: None,
            last_name: None,
            full_name: Some("John Doe".to_string()),
            email: Some("john@acme.com".to_string()),
            phone: Some("+971507654321".to_string()),
            position: Some("Director".to_string()),
            company: None,
            time: None,
        }
    }

    fn make_contact_parts() -> Contact {
        Contact {
            id: None,
            first_name: Some("Jane".to_string()),
            last_name: Some("Smith".to_string()),
            full_name: None,
            email: None,
            phone: None,
            position: None,
            company: None,
            time: None,
        }
    }

    // ---- format_issue_date tests ----

    #[test]
    fn test_format_issue_date_2000s() {
        assert_eq!(format_issue_date("250115"), "15 Jan 2025");
        assert_eq!(format_issue_date("000101"), "01 Jan 2000");
        assert_eq!(format_issue_date("261231"), "31 Dec 2026");
    }

    #[test]
    fn test_format_issue_date_1900s() {
        assert_eq!(format_issue_date("991231"), "31 Dec 1999");
        assert_eq!(format_issue_date("500601"), "01 Jun 1950");
    }

    #[test]
    fn test_format_issue_date_boundary() {
        // year 49 → 2049, year 50 → 1950
        assert_eq!(format_issue_date("490101"), "01 Jan 2049");
        assert_eq!(format_issue_date("500101"), "01 Jan 1950");
    }

    #[test]
    fn test_format_issue_date_invalid_returns_nonempty() {
        assert!(!format_issue_date("invalid").is_empty());
        assert!(!format_issue_date("12345").is_empty()); // too short
        assert!(!format_issue_date("").is_empty());
        assert!(!format_issue_date("261399").is_empty()); // month 13 invalid
    }

    // ---- clean_record_key tests ----

    #[test]
    fn test_clean_record_key_strips_brackets() {
        assert_eq!(clean_record_key("⟨25_97101⟩"), "25_97101");
    }

    #[test]
    fn test_clean_record_key_passthrough() {
        assert_eq!(clean_record_key("25_97101"), "25_97101");
        assert_eq!(clean_record_key("acme"), "acme");
    }

    #[test]
    fn test_clean_record_key_empty() {
        assert_eq!(clean_record_key(""), "");
    }

    // ---- clean_number_for_path tests ----

    #[test]
    fn test_clean_number_for_path_basic() {
        assert_eq!(clean_number_for_path("25_97101"), "25-97101");
    }

    #[test]
    fn test_clean_number_for_path_with_brackets() {
        assert_eq!(clean_number_for_path("⟨25_97101⟩"), "25-97101");
    }

    #[test]
    fn test_clean_number_for_path_no_underscores() {
        assert_eq!(clean_number_for_path("acme"), "acme");
    }

    // ---- is_placeholder tests ----

    #[test]
    fn test_is_placeholder_empty() {
        assert!(is_placeholder(""));
    }

    #[test]
    fn test_is_placeholder_known_values() {
        assert!(is_placeholder("<variable not set>"));
        assert!(is_placeholder("«variable not set»"));
        assert!(is_placeholder("<none>"));
        assert!(is_placeholder("«none»"));
        assert!(is_placeholder("#error"));
    }

    #[test]
    fn test_is_placeholder_real_values() {
        assert!(!is_placeholder("John Doe"));
        assert!(!is_placeholder("0"));
        assert!(!is_placeholder(" ")); // whitespace-only is NOT a placeholder
    }

    // ---- build_fee_json tests ----

    #[test]
    fn test_build_fee_json_has_all_21_fields() {
        let fee = make_fee();
        let project = make_project();
        let company = make_company();
        let contact = make_contact_full_name();

        let result = build_fee_json(&fee, &project, &company, &contact);
        let obj = result.as_object().expect("result must be a JSON object");

        let expected_keys = [
            "01 Document Name",
            "02 Document Number",
            "03 Document Release",
            "04 Document Issue Date",
            "06 Project Name",
            "07 Project Activity",
            "08 Project Package",
            "09 Project Stage",
            "11 Project Area",
            "12 Project City",
            "13 Project Country",
            "21 Client Company",
            "22 Client City",
            "23 Client Country",
            "26 Contact Name",
            "27 Contact Position",
            "28 Contact Phone",
            "29 Contact Email",
            "91 Staff Name",
            "92 Staff Position",
            "93 Staff Phone",
            "94 Staff Email",
            "99 Strap Line",
        ];
        assert_eq!(obj.len(), 23, "expected 23 fields (01-99 with gaps)");
        for key in &expected_keys {
            assert!(obj.contains_key(*key), "missing field: {}", key);
        }
    }

    #[test]
    fn test_build_fee_json_values() {
        let fee = make_fee();
        let project = make_project();
        let company = make_company();
        let contact = make_contact_full_name();

        let result = build_fee_json(&fee, &project, &company, &contact);

        assert_eq!(result["01 Document Name"], "Test Fee");
        assert_eq!(result["02 Document Number"], "TF-001");
        assert_eq!(result["03 Document Release"], "03"); // zero-padded
        assert_eq!(result["04 Document Issue Date"], "15 Mar 2026");
        assert_eq!(result["06 Project Name"], "Burj Al Arab Renovation");
        assert_eq!(result["07 Project Activity"], "Lighting Design");
        assert_eq!(result["08 Project Package"], "Full");
        assert_eq!(result["09 Project Stage"], "Design");
        assert_eq!(result["11 Project Area"], "120000");
        assert_eq!(result["12 Project City"], "Dubai");
        assert_eq!(result["13 Project Country"], "UAE");
        assert_eq!(result["21 Client Company"], "Acme Holdings LLC");
        assert_eq!(result["22 Client City"], "Dubai");
        assert_eq!(result["23 Client Country"], "UAE");
        assert_eq!(result["26 Contact Name"], "John Doe");
        assert_eq!(result["27 Contact Position"], "Director");
        assert_eq!(result["28 Contact Phone"], "+971507654321");
        assert_eq!(result["29 Contact Email"], "john@acme.com");
        assert_eq!(result["91 Staff Name"], "Alice Smith");
        assert_eq!(result["92 Staff Position"], "Senior Designer");
        assert_eq!(result["93 Staff Phone"], "+971501234567");
        assert_eq!(result["94 Staff Email"], "alice@emittiv.com");
        assert_eq!(result["99 Strap Line"], "Excellence in light");
    }

    #[test]
    fn test_build_fee_json_contact_name_from_parts() {
        let fee = make_fee();
        let project = make_project();
        let company = make_company();
        let contact = make_contact_parts(); // no full_name, has first + last

        let result = build_fee_json(&fee, &project, &company, &contact);
        assert_eq!(result["26 Contact Name"], "Jane Smith");
    }

    #[test]
    fn test_build_fee_json_contact_optional_fields_default_empty() {
        let fee = make_fee();
        let project = make_project();
        let company = make_company();
        let contact = make_contact_parts(); // no email/phone/position

        let result = build_fee_json(&fee, &project, &company, &contact);
        assert_eq!(result["27 Contact Position"], "");
        assert_eq!(result["28 Contact Phone"], "");
        assert_eq!(result["29 Contact Email"], "");
    }

    #[test]
    fn test_build_fee_json_rev_zero_padded() {
        let mut fee = make_fee();
        fee.rev = 1;
        let result = build_fee_json(
            &fee,
            &make_project(),
            &make_company(),
            &make_contact_full_name(),
        );
        assert_eq!(result["03 Document Release"], "01");

        fee.rev = 10;
        let result = build_fee_json(
            &fee,
            &make_project(),
            &make_company(),
            &make_contact_full_name(),
        );
        assert_eq!(result["03 Document Release"], "10");
    }
}
