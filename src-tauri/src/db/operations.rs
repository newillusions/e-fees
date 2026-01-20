//! Entity operations for DatabaseManager.
//!
//! This module contains impl blocks for database entity operations,
//! extracted from mod.rs to reduce file size and improve organization.

use surrealdb::{Error, Value};
use log::{error, info, warn};
use chrono::Datelike;

use super::{
    DatabaseManager, PaginatedResponse,
    Project, NewProject, Company, CompanyCreate,
    Contact, ContactCreate, Fee, FeeCreate, FeeUpdate,
    EntityCounts, ActivityLog, ActivityLogCreate,
};
use crate::commands::{CompanyUpdate, ProjectUpdate};

// ==================== Project Operations ====================

impl DatabaseManager {
    pub async fn get_projects(&self) -> Result<Vec<Project>, Error> {
        let client = self.get_client()?;
        info!("Attempting to query projects table");
        client.select("projects").await.or_else(|_| Ok(vec![]))
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
        info!("Attempting to query company table");
        client.select("company").await.or_else(|_| Ok(vec![]))
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
    pub async fn get_contacts(&self) -> Result<Vec<Contact>, Error> {
        let client = self.get_client()?;
        info!("Attempting to query contacts table");

        let all_contacts: Vec<Contact> = client.select("contacts").await.unwrap_or_default();
        info!("Raw fetched {} contacts", all_contacts.len());

        // Filter out incomplete contacts
        let valid_contacts: Vec<Contact> = all_contacts.into_iter()
            .filter(|contact| {
                let has_first_name = contact.first_name.as_ref().map_or(false, |s| !s.is_empty());
                let has_last_name = contact.last_name.as_ref().map_or(false, |s| !s.is_empty());
                let has_email = contact.email.as_ref().map_or(false, |s| !s.is_empty());
                let has_phone = contact.phone.as_ref().map_or(false, |s| !s.is_empty());
                let has_position = contact.position.as_ref().map_or(false, |s| !s.is_empty());
                let has_company = contact.company.is_some();

                has_first_name && has_last_name && has_email && has_phone && has_position && has_company
            })
            .collect();

        info!("Successfully fetched {} valid contacts", valid_contacts.len());
        Ok(valid_contacts)
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
        info!("Attempting to query fee table");

        match client.select("fee").await {
            Ok(fees) => {
                info!("Successfully fetched {} fee records", fees.len());
                Ok(fees)
            }
            Err(e) => {
                error!("Failed to select from fee table: {}", e);
                Ok(Vec::new())
            }
        }
    }

    pub async fn get_fees_page(&self, page: usize, page_size: usize) -> Result<PaginatedResponse<Fee>, Error> {
        self.paginate("fee", page, page_size).await
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

        let query = format!(
            "SELECT area FROM projects WHERE country = '{}' AND area IS NOT NONE GROUP BY area ORDER BY area ASC LIMIT 20",
            country.replace("'", "''")
        );

        let mut response = client.query(&query).await?;
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

        let query = format!(
            "SELECT city FROM projects WHERE country = '{}' AND city IS NOT NONE GROUP BY city ORDER BY city ASC LIMIT 20",
            country.replace("'", "''")
        );

        let mut response = client.query(&query).await?;
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
                        dial_code as u16
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

        let year = year.unwrap_or_else(|| (chrono::Utc::now().year() % 100) as u8);

        // Find max sequence using parameterized query
        let query = "SELECT number.seq FROM projects WHERE number.year = $year AND number.country = $country AND number.seq >= 1 AND number.seq <= 99 ORDER BY number.seq DESC LIMIT 1";
        let mut response = client.query_bind(query, (("year", year), ("country", country_code))).await?;
        let result: Result<Vec<serde_json::Value>, _> = response.take(0);

        let next_seq = match result {
            Ok(records) => {
                if let Some(first) = records.first() {
                    if let Some(seq) = first.get("number").and_then(|n| n.get("seq")).and_then(|s| s.as_u64()) {
                        (seq + 1) as u8
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

        let query = format!(
            "SELECT count() FROM projects WHERE number.year = {} AND number.country = {} AND number.seq = {}",
            year, country, seq
        );

        let mut response = client.query(&query).await?;
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
            SELECT count() as count FROM rfp GROUP ALL;
            SELECT count() as count FROM rfp WHERE status IN ['Draft', 'Sent', 'Negotiation'] GROUP ALL;
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

        let query = if let Some(et) = &entity_type {
            format!(
                "SELECT * FROM activity_log WHERE entity_type = '{}' ORDER BY timestamp DESC LIMIT {} START {}",
                et.replace("'", "''"), limit_val, offset_val
            )
        } else {
            format!(
                "SELECT * FROM activity_log ORDER BY timestamp DESC LIMIT {} START {}",
                limit_val, offset_val
            )
        };

        let mut response = client.query(&query).await?;
        let logs: Vec<ActivityLog> = response.take(0)?;
        info!("Retrieved {} activity logs", logs.len());
        Ok(logs)
    }
}

// ==================== Utility Operations ====================

impl DatabaseManager {
    pub async fn execute_raw_query(&self, query: &str) -> Result<(), Error> {
        let client = self.get_client()?;
        client.query(query).await?;
        Ok(())
    }

    pub async fn get_table_schema(&self, table_name: &str) -> Result<serde_json::Value, Error> {
        let client = self.get_client()?;

        let query = format!("INFO FOR TABLE {};", table_name);
        let mut result = client.query(&query).await?;
        let schema: Option<serde_json::Value> = result.take(0)?;
        Ok(schema.unwrap_or(serde_json::json!({})))
    }

    pub async fn investigate_record(&self, record_id: &str) -> Result<serde_json::Value, Error> {
        let client = self.get_client()?;
        info!("Investigating record: {}", record_id);

        let mut queries = Vec::new();
        let mut results = serde_json::json!({
            "record_id": record_id,
            "investigation": {}
        });

        if record_id.contains(":") {
            queries.push(format!("SELECT * FROM {};", record_id));
        }

        if !record_id.contains(":") {
            queries.push(format!("SELECT * FROM {} LIMIT 5;", record_id));
        }

        let table_part = if record_id.contains(":") {
            record_id.split(":").next().unwrap_or("")
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
