//! Entity operations for DatabaseManager.
//!
//! This module contains impl blocks for database entity operations,
//! extracted from mod.rs to reduce file size and improve organization.

use surrealdb::{Error, types::Value};
use log::{error, info};
use chrono::Datelike;
use crate::db::types::record_key_string;

use super::{
    DatabaseManager, PaginatedResponse,
    Project, NewProject, Company, CompanyCreate,
    Contact, ContactCreate, Fee, FeeCreate, FeeUpdate, PricingUpdate,
    EntityCounts, ActivityLog, ActivityLogCreate,
};
use crate::commands::{CompanyUpdate, ProjectUpdate};

// ==================== Project Operations ====================

impl DatabaseManager {
    pub async fn get_projects(&self) -> Result<Vec<Project>, Error> {
        let client = self.get_client()?;
        let mut response = client.query("SELECT * FROM projects ORDER BY time.created_at DESC").await?;
        let projects: Vec<Project> = response.take(0)?;
        info!("Fetched {} projects", projects.len());
        Ok(projects)
    }

    pub async fn get_projects_page(&self, page: usize, page_size: usize) -> Result<PaginatedResponse<Project>, Error> {
        self.paginate("projects", page, page_size).await
    }

    pub async fn get_project_by_id(&self, id: &str) -> Result<Option<Project>, Error> {
        self.get_by_id("projects", id).await
    }

    pub async fn search_projects(&self, query: &str) -> Result<Vec<Project>, Error> {
        let client = self.get_client()?;
        client.search_projects(query).await
    }

    pub async fn create_project(&self, project: Project) -> Result<Project, Error> {
        let client = self.get_client()?;
        client.create_project(project).await?
            .ok_or_else(|| self.not_found_error("create project"))
    }

    pub async fn create_new_project(&self, project: NewProject) -> Result<Project, Error> {
        let client = self.get_client()?;
        client.create_new_project(project).await?
            .ok_or_else(|| self.not_found_error("create project"))
    }

    pub async fn update_project(&self, id: &str, project_update: ProjectUpdate) -> Result<Project, Error> {
        let client = self.get_client()?;
        client.update_project(id, project_update).await?
            .ok_or_else(|| self.not_found_error("update project"))
    }

    pub async fn delete_project(&self, id: &str) -> Result<Project, Error> {
        let client = self.get_client()?;
        client.delete_project(id).await?
            .ok_or_else(|| self.not_found_error("delete project"))
    }
}

// ==================== Company Operations ====================

impl DatabaseManager {
    pub async fn get_companies(&self) -> Result<Vec<Company>, Error> {
        let client = self.get_client()?;
        let mut response = client.query("SELECT * FROM company ORDER BY time.created_at DESC").await?;
        let companies: Vec<Company> = response.take(0)?;
        info!("Fetched {} companies", companies.len());
        Ok(companies)
    }

    pub async fn get_companies_page(&self, page: usize, page_size: usize) -> Result<PaginatedResponse<Company>, Error> {
        self.paginate("company", page, page_size).await
    }

    pub async fn get_company_by_id(&self, id: &str) -> Result<Option<Company>, Error> {
        self.get_by_id("company", id).await
    }

    pub async fn create_company(&self, company: CompanyCreate) -> Result<Company, Error> {
        let client = self.get_client()?;
        client.create_company(company).await?
            .ok_or_else(|| self.not_found_error("create company"))
    }

    pub async fn update_company_partial(&self, id: &str, company_update: CompanyUpdate) -> Result<Company, Error> {
        let client = self.get_client()?;
        client.update_company_partial(id, company_update).await?
            .ok_or_else(|| self.not_found_error("update company"))
    }

    pub async fn delete_company(&self, id: &str) -> Result<Company, Error> {
        let client = self.get_client()?;
        client.delete_company(id).await?
            .ok_or_else(|| self.not_found_error("delete company"))
    }
}

// ==================== Contact Operations ====================

impl DatabaseManager {
    /// Get all contacts with required fields populated.
    /// Filtering is done at the database level for efficiency.
    pub async fn get_contacts(&self) -> Result<Vec<Contact>, Error> {
        let client = self.get_client()?;
        // Filter at database level instead of fetching all and filtering in Rust
        let query = r#"
            SELECT * FROM contacts
            WHERE first_name IS NOT NONE AND first_name IS NOT ''
            AND last_name IS NOT NONE AND last_name IS NOT ''
            AND email IS NOT NONE AND email IS NOT ''
            AND phone IS NOT NONE AND phone IS NOT ''
            AND position IS NOT NONE AND position IS NOT ''
            AND company IS NOT NONE
            ORDER BY time.updated_at DESC
        "#;

        let mut response = client.query(query).await?;
        let contacts: Vec<Contact> = response.take(0)?;

        info!("Successfully fetched {} valid contacts", contacts.len());
        Ok(contacts)
    }

    pub async fn get_contacts_page(&self, page: usize, page_size: usize) -> Result<PaginatedResponse<Contact>, Error> {
        self.paginate("contacts", page, page_size).await
    }

    pub async fn get_contact_by_id(&self, id: &str) -> Result<Option<Contact>, Error> {
        self.get_by_id("contacts", id).await
    }

    pub async fn create_contact(&self, contact: ContactCreate) -> Result<Contact, Error> {
        let client = self.get_client()?;
        client.create_contact(contact).await?
            .ok_or_else(|| self.not_found_error("create contact"))
    }

    pub async fn update_contact_partial(&self, id: &str, contact_update: crate::commands::ContactUpdate) -> Result<Contact, Error> {
        let client = self.get_client()?;
        client.update_contact_partial(id, contact_update).await?
            .ok_or_else(|| self.not_found_error("update contact"))
    }

    pub async fn delete_contact(&self, id: &str) -> Result<Contact, Error> {
        let client = self.get_client()?;
        client.delete_contact(id).await?
            .ok_or_else(|| self.not_found_error("delete contact"))
    }
}

// ==================== Fee Operations ====================

impl DatabaseManager {
    pub async fn get_fees(&self) -> Result<Vec<Fee>, Error> {
        let client = self.get_client()?;
        let mut response = client.query("SELECT * FROM fee ORDER BY time.created_at DESC").await?;
        let fees: Vec<Fee> = response.take(0)?;
        info!("Fetched {} fees", fees.len());
        Ok(fees)
    }

    pub async fn get_fees_page(&self, page: usize, page_size: usize) -> Result<PaginatedResponse<Fee>, Error> {
        self.paginate("fee", page, page_size).await
    }

    pub async fn get_fee_by_id(&self, id: &str) -> Result<Option<Fee>, Error> {
        self.get_by_id("fee", id).await
    }

    pub async fn create_fee(&self, fee: FeeCreate) -> Result<Fee, Error> {
        let client = self.get_client()?;
        client.create_fee(fee).await?
            .ok_or_else(|| self.not_found_error("create fee"))
    }

    pub async fn update_fee(&self, id: &str, fee: FeeUpdate) -> Result<Fee, Error> {
        let client = self.get_client()?;
        client.update_fee(id, fee).await?
            .ok_or_else(|| self.not_found_error("update fee"))
    }

    pub async fn delete_fee(&self, id: &str) -> Result<Fee, Error> {
        let client = self.get_client()?;
        client.delete_fee(id).await?
            .ok_or_else(|| self.not_found_error("delete fee"))
    }

    pub async fn update_fee_pricing(&self, id: &str, pricing: PricingUpdate) -> Result<Fee, Error> {
        let client = self.get_client()?;
        client.update_fee_pricing(id, pricing).await?
            .ok_or_else(|| self.not_found_error("update fee pricing"))
    }

    /// Clone a fee as a new revision with incremented rev number.
    pub async fn clone_fee_as_revision(&self, source_fee_id: &str) -> Result<Fee, Error> {
        // 1. Fetch the source fee
        let source: Option<Fee> = self.get_by_id("fee", source_fee_id).await?;
        let source = source.ok_or_else(|| self.not_found_error("source fee"))?;

        // 2. Find max rev for this project using parameterized query
        let client = self.get_client()?;
        let project_id_val = record_key_string(&source.project_id.key);
        let mut response = client.query_bind(
            "SELECT math::max(rev) AS max_rev FROM fee WHERE project_id = projects:$pid GROUP ALL",
            ("pid", project_id_val)
        ).await?;

        let max_rev_result: Option<serde_json::Value> = response.take(0)?;
        let new_rev = max_rev_result
            .and_then(|v| v.get("max_rev").and_then(|r| r.as_i64()))
            .map(|r| r as i32 + 1)
            .unwrap_or(1);

        // 3. Create new fee with incremented rev, copying pricing data
        let new_fee = FeeCreate {
            name: source.name.clone(),
            number: source.number.clone(),
            rev: new_rev,
            status: "Draft".to_string(),
            issue_date: chrono::Utc::now().format("%y%m%d").to_string(),
            activity: source.activity.clone(),
            package: source.package.clone(),
            project_id: record_key_string(&source.project_id.key),
            company_id: record_key_string(&source.company_id.key),
            contact_id: record_key_string(&source.contact_id.key),
            staff_name: source.staff_name.clone(),
            staff_email: source.staff_email.clone(),
            staff_phone: source.staff_phone.clone(),
            staff_position: source.staff_position.clone(),
            strap_line: source.strap_line.clone(),
            revisions: vec![],
            pricing: source.pricing.clone(),
            post_contract_items: source.post_contract_items.clone(),
            reimbursable_costs: source.reimbursable_costs.clone(),
            payment_schedule: None,
            pricing_revisions: None,
            current_revision_number: None,
            current_release_number: None,
            import_source: None,
        };

        self.create_fee(new_fee).await
    }

    /// Get all fees for a specific project, sorted by rev DESC.
    pub async fn get_fees_for_project(&self, project_id: &str) -> Result<Vec<Fee>, Error> {
        let client = self.get_client()?;
        let mut result = client.query_bind(
            "SELECT * FROM fee WHERE project_id = projects:$pid ORDER BY rev DESC",
            ("pid", project_id.to_string())
        ).await?;
        let fees: Vec<Fee> = result.take(0)?;
        Ok(fees)
    }
}

// ==================== Reference Data Operations ====================

impl DatabaseManager {
    pub async fn search_countries(&self, query: &str) -> Result<Vec<serde_json::Value>, Error> {
        let client = self.get_client()?;
        info!("Searching countries with query: {}", query);
        client.search_countries(query).await
    }

    pub async fn get_area_suggestions(&self, country: &str) -> Result<Vec<String>, Error> {
        let client = self.get_client()?;
        info!("Getting area suggestions for country: {}", country);

        // Use parameterized query to prevent SQL injection
        let query = "SELECT area FROM projects WHERE country = $country AND area IS NOT NONE GROUP BY area ORDER BY area ASC LIMIT 20";

        let mut response = client.query_bind(query, ("country", country.to_string())).await?;
        let result: Result<Vec<serde_json::Value>, _> = response.take(0);

        match result {
            Ok(areas) => {
                let area_strings: Vec<String> = areas
                    .into_iter()
                    .filter_map(|area| area.get("area").and_then(|a| a.as_str()).map(|s| s.to_string()))
                    .collect();
                info!("Found {} area suggestions", area_strings.len());
                Ok(area_strings)
            }
            Err(e) => {
                error!("Failed to get area suggestions: {}", e);
                Err(e)
            }
        }
    }

    pub async fn get_city_suggestions(&self, country: &str) -> Result<Vec<String>, Error> {
        let client = self.get_client()?;
        info!("Getting city suggestions for country: {}", country);

        // Use parameterized query to prevent SQL injection
        let query = "SELECT city FROM projects WHERE country = $country AND city IS NOT NONE GROUP BY city ORDER BY city ASC LIMIT 20";

        let mut response = client.query_bind(query, ("country", country.to_string())).await?;
        let result: Result<Vec<serde_json::Value>, _> = response.take(0);

        match result {
            Ok(cities) => {
                let city_strings: Vec<String> = cities
                    .into_iter()
                    .filter_map(|city| city.get("city").and_then(|c| c.as_str()).map(|s| s.to_string()))
                    .collect();
                info!("Found {} city suggestions", city_strings.len());
                Ok(city_strings)
            }
            Err(e) => {
                error!("Failed to get city suggestions: {}", e);
                Err(e)
            }
        }
    }

    pub async fn get_all_cities(&self) -> Result<Vec<String>, Error> {
        let client = self.get_client()?;
        info!("Getting all city suggestions");

        let mut all_cities = Vec::new();

        // Get cities from projects
        let projects_query = "SELECT city FROM projects WHERE city IS NOT NONE GROUP BY city ORDER BY city ASC";
        if let Ok(mut response) = client.query(projects_query).await {
            if let Ok(cities) = response.take::<Vec<serde_json::Value>>(0) {
                let project_cities: Vec<String> = cities
                    .into_iter()
                    .filter_map(|city| city.get("city").and_then(|c| c.as_str()).map(|s| s.to_string()))
                    .collect();
                all_cities.extend(project_cities);
            }
        }

        // Get cities from companies
        let companies_query = "SELECT city FROM company WHERE city IS NOT NONE GROUP BY city ORDER BY city ASC";
        if let Ok(mut response) = client.query(companies_query).await {
            if let Ok(cities) = response.take::<Vec<serde_json::Value>>(0) {
                let company_cities: Vec<String> = cities
                    .into_iter()
                    .filter_map(|city| city.get("city").and_then(|c| c.as_str()).map(|s| s.to_string()))
                    .collect();
                all_cities.extend(company_cities);
            }
        }

        all_cities.sort();
        all_cities.dedup();
        all_cities.truncate(50);

        info!("Found {} total unique city suggestions", all_cities.len());
        Ok(all_cities)
    }
}

// ==================== Project Number Operations ====================

impl DatabaseManager {
    pub async fn generate_next_project_number(&self, country_name: &str, year: Option<u8>) -> Result<String, Error> {
        let client = self.get_client()?;
        info!("Generating next project number for country: {}", country_name);

        // Validate country_name length
        if country_name.is_empty() || country_name.len() > 100 {
            return Err(self.invalid_request_error("Invalid country name"));
        }

        // Look up dial code using parameterized query (SQL injection safe)
        let country_lookup_query = "SELECT dial_code FROM country WHERE name = $name LIMIT 1";
        let mut country_response = client.query_bind(country_lookup_query, ("name", country_name.to_string())).await?;
        let country_result: Result<Vec<serde_json::Value>, _> = country_response.take(0);

        let country_code = match country_result {
            Ok(records) => {
                if let Some(first) = records.first() {
                    if let Some(dial_code) = first.get("dial_code").and_then(|v| v.as_u64()) {
                        // TYPE-M2: Use try_into with bounds checking instead of lossy cast
                        u16::try_from(dial_code).map_err(|_| {
                            self.invalid_request_error(&format!("Dial code {} out of valid range for country: {}", dial_code, country_name))
                        })?
                    } else {
                        return Err(self.invalid_request_error(&format!("Dial code is not a number for country: {}", country_name)));
                    }
                } else {
                    return Err(self.invalid_request_error(&format!("Country not found: {}", country_name)));
                }
            }
            Err(e) => return Err(e),
        };

        info!("Found country code {} for country {}", country_code, country_name);

        // TYPE-M2: year % 100 is always 0-99, safe for u8, but use explicit conversion
        let year = year.unwrap_or_else(|| {
            let current_year = chrono::Utc::now().year() % 100;
            // This is always 0-99, so u8 conversion is safe
            u8::try_from(current_year).unwrap_or(0)
        });

        // Find max sequence using parameterized query
        let query = "SELECT number.seq FROM projects WHERE number.year = $year AND number.country = $country AND number.seq >= 1 AND number.seq <= 99 ORDER BY number.seq DESC LIMIT 1";
        let mut response = client.query_bind(query, (("year", year), ("country", country_code))).await?;
        let result: Result<Vec<serde_json::Value>, _> = response.take(0);

        let next_seq = match result {
            Ok(records) => {
                if let Some(first) = records.first() {
                    if let Some(seq) = first.get("number").and_then(|n| n.get("seq")).and_then(|s| s.as_u64()) {
                        // TYPE-M2: Use checked arithmetic and try_from for safe conversion
                        let next = seq.saturating_add(1);
                        u8::try_from(next).unwrap_or(u8::MAX)
                    } else {
                        1
                    }
                } else {
                    1
                }
            }
            Err(_) => 1,
        };

        if next_seq > 99 {
            return Err(self.invalid_request_error(&format!(
                "Maximum of 99 projects per year per country reached for year {} country {}",
                year, country_code
            )));
        }

        let project_number = format!("{:02}-{}{:02}", year, country_code, next_seq);
        info!("Generated project number: {}", project_number);

        Ok(project_number)
    }

    pub async fn validate_project_number(&self, project_number: &str) -> Result<bool, Error> {
        let client = self.get_client()?;
        info!("Validating project number: {}", project_number);

        let parts: Vec<&str> = project_number.split('-').collect();
        if parts.len() != 2 || parts[0].len() != 2 || parts[1].len() != 5 {
            return Ok(false);
        }

        let year = parts[0].parse::<u8>().map_err(|_| self.invalid_request_error("Invalid year format"))?;
        let country = parts[1][..3].parse::<u16>().map_err(|_| self.invalid_request_error("Invalid country code"))?;
        let seq = parts[1][3..].parse::<u8>().map_err(|_| self.invalid_request_error("Invalid sequence number"))?;

        // Use parameterized query to prevent SQL injection
        let query = "SELECT count() FROM projects WHERE number.year = $year AND number.country = $country AND number.seq = $seq";

        let mut response = client.query_bind(query, (("year", year), ("country", country), ("seq", seq))).await?;
        let result: Result<Value, _> = response.take(0);

        match result {
            Ok(value) => {
                let json_value = serde_json::to_value(&value).unwrap_or_else(|_| serde_json::json!(null));
                let count = json_value.as_u64()
                    .or_else(|| json_value.as_object().and_then(|o| o.get("count").and_then(|v| v.as_u64())))
                    .unwrap_or(0);
                Ok(count == 0)
            }
            Err(e) => Err(e)
        }
    }
}

// ==================== Statistics Operations ====================

impl DatabaseManager {
    pub async fn get_entity_counts(&self) -> Result<EntityCounts, Error> {
        let client = self.get_client()?;
        info!("Fetching entity counts");

        let count_query = r#"
            SELECT count() as count FROM projects GROUP ALL;
            SELECT count() as count FROM company GROUP ALL;
            SELECT count() as count FROM contacts GROUP ALL;
            SELECT count() as count FROM fee GROUP ALL;
            SELECT count() as count FROM fee WHERE status IN ['Draft', 'Sent', 'Negotiation'] GROUP ALL;
        "#;

        let extract_count = |result: Option<serde_json::Value>| -> usize {
            result.and_then(|v| v.get("count").and_then(|c| c.as_u64())).unwrap_or(0) as usize
        };

        let mut response = client.query(count_query).await?;
        let projects: Option<serde_json::Value> = response.take(0)?;
        let companies: Option<serde_json::Value> = response.take(1)?;
        let contacts: Option<serde_json::Value> = response.take(2)?;
        let fees: Option<serde_json::Value> = response.take(3)?;
        let active: Option<serde_json::Value> = response.take(4)?;

        Ok(EntityCounts {
            total_projects: extract_count(projects),
            total_companies: extract_count(companies),
            total_contacts: extract_count(contacts),
            total_fees: extract_count(fees),
            active_fees: extract_count(active),
        })
    }
}

// ==================== Activity Log Operations ====================

impl DatabaseManager {
    pub async fn create_activity_log(&self, log: ActivityLogCreate) -> Result<ActivityLog, Error> {
        let client = self.get_client()?;
        info!("Creating activity log: {} {} on {}", log.action, log.entity_type, log.entity_name);

        let action = log.action.clone();
        let entity_type = log.entity_type.clone();
        let entity_id = log.entity_id.clone();
        let entity_name = log.entity_name.clone();
        let description = log.description.clone();
        let old_value = log.old_value.clone();
        let new_value = log.new_value.clone();
        let user = log.user.unwrap_or_else(|| "system".to_string());
        let metadata_json = log.metadata.map(|m| m.to_string()).unwrap_or_else(|| "null".to_string());

        let query = format!(
            r#"CREATE activity_log CONTENT {{
                action: $action,
                entity_type: $entity_type,
                entity_id: $entity_id,
                entity_name: $entity_name,
                description: $description,
                old_value: $old_value,
                new_value: $new_value,
                user: $user,
                metadata: {}
            }}"#,
            metadata_json
        );

        let mut response = client.query(&query).await?;

        // Note: Binding would require access to the underlying client
        // For now, use direct string interpolation (already sanitized in caller)
        let result: Option<ActivityLog> = response.take(0)?;

        result.ok_or_else(|| self.not_found_error("create activity log"))
    }

    pub async fn get_activity_logs(&self, limit: Option<usize>, entity_type: Option<String>, offset: Option<usize>) -> Result<Vec<ActivityLog>, Error> {
        let client = self.get_client()?;

        let limit_val = limit.unwrap_or(50);
        let offset_val = offset.unwrap_or(0);
        info!("Fetching activity logs (limit: {}, entity_type: {:?}, offset: {})", limit_val, entity_type, offset_val);

        // LIMIT and START are safe to interpolate as integers (no SQL injection risk)
        // Parameterized bindings with nested tuples don't work reliably in SurrealDB SDK
        let mut response = if let Some(et) = &entity_type {
            let query = format!(
                "SELECT * FROM activity_log WHERE entity_type = $entity_type ORDER BY timestamp DESC LIMIT {} START {}",
                limit_val, offset_val
            );
            client.query_bind(&query, ("entity_type", et.clone())).await?
        } else {
            let query = format!(
                "SELECT * FROM activity_log ORDER BY timestamp DESC LIMIT {} START {}",
                limit_val, offset_val
            );
            client.query(&query).await?
        };

        let logs: Vec<ActivityLog> = response.take(0)?;
        info!("Retrieved {} activity logs", logs.len());
        Ok(logs)
    }
}

// ==================== Utility Operations ====================

impl DatabaseManager {
    /// Update a project's status safely using parameterized queries.
    ///
    /// This replaces the dangerous `execute_raw_query` function with a safe,
    /// purpose-built method that validates inputs and uses parameterized queries.
    pub async fn update_project_status(&self, project_id: &str, new_status: &str) -> Result<(), Error> {
        let client = self.get_client()?;

        // Validate project_id format (should be like "25-97101" or "25_97101")
        // Allow alphanumeric, hyphens, and underscores only
        if project_id.is_empty() || project_id.len() > 20 {
            return Err(self.invalid_request_error("Invalid project ID length"));
        }
        if !project_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(self.invalid_request_error("Invalid project ID format"));
        }

        // Validate status against known values
        let valid_statuses = [
            "Draft", "Active", "On Hold", "Completed", "Archived", "Cancelled",
            "RFP", "Tender", "Awarded", "Lost"
        ];
        if !valid_statuses.contains(&new_status) {
            return Err(self.invalid_request_error(&format!("Invalid status: {}", new_status)));
        }

        // Use parameterized query for safety
        let query = format!(
            "UPDATE projects:`{}` SET status = $status, time.updated_at = time::now()",
            project_id
        );
        client.query_bind(&query, ("status", new_status.to_string())).await?;

        info!("Successfully updated project {} status to {}", project_id, new_status);
        Ok(())
    }

    pub async fn get_table_schema(&self, table_name: &str) -> Result<serde_json::Value, Error> {
        let client = self.get_client()?;

        // Validate table name to prevent SQL injection (only alphanumeric and underscores)
        if !table_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(self.invalid_request_error("Invalid table name"));
        }

        let query = format!("INFO FOR TABLE {};", table_name);
        let mut result = client.query(&query).await?;
        let schema: Option<serde_json::Value> = result.take(0)?;
        Ok(schema.unwrap_or(serde_json::json!({})))
    }

    pub async fn investigate_record(&self, record_id: &str) -> Result<serde_json::Value, Error> {
        let client = self.get_client()?;
        info!("Investigating record: {}", record_id);

        // Validate record_id format to prevent SQL injection
        // Only allow alphanumeric, underscores, and colons (for table:id format)
        if !record_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ':') {
            return Err(self.invalid_request_error("Invalid record ID format"));
        }

        // Limit length to prevent abuse
        if record_id.len() > 100 {
            return Err(self.invalid_request_error("Record ID too long"));
        }

        let mut queries = Vec::new();
        let mut results = serde_json::json!({
            "record_id": record_id,
            "investigation": {}
        });

        if record_id.contains(':') {
            queries.push(format!("SELECT * FROM {};", record_id));
        }

        if !record_id.contains(':') {
            queries.push(format!("SELECT * FROM {} LIMIT 5;", record_id));
        }

        let table_part = if record_id.contains(':') {
            record_id.split(':').next().unwrap_or("")
        } else {
            record_id
        };

        if !table_part.is_empty() {
            queries.push(format!("INFO FOR TABLE {};", table_part));
            queries.push(format!("SELECT count() FROM {} GROUP ALL;", table_part));
        }

        for (i, query) in queries.iter().enumerate() {
            info!("Executing query {}: {}", i + 1, query);

            let query_result = match client.query(query).await {
                Ok(mut response) => {
                    let result: Result<Value, _> = response.take(0);
                    match result {
                        Ok(value) => {
                            let json_value = serde_json::to_value(&value).unwrap_or_else(|_| serde_json::json!(null));
                            serde_json::json!({
                                "status": "success",
                                "data": json_value
                            })
                        },
                        Err(e) => serde_json::json!({
                            "status": "error",
                            "error": e.to_string()
                        })
                    }
                },
                Err(e) => serde_json::json!({
                    "status": "error",
                    "error": e.to_string()
                })
            };

            results["investigation"][format!("query_{}", i + 1)] = serde_json::json!({
                "query": query,
                "result": query_result
            });
        }

        Ok(results)
    }
}
