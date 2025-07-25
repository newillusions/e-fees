//! # Database Layer Refactoring Examples
//! 
//! This file demonstrates how the database operations can be refactored using
//! the new utility functions to eliminate the repetitive client validation
//! and error handling patterns.

use crate::db::entities::*;
use crate::db::utils::{execute_with_client, create_db_error, validate_entity_id};
use surrealdb::{Error, Surreal};
use surrealdb::engine::remote::ws::Client as DatabaseClient;

/// Example of refactored DatabaseManager with utility functions
impl DatabaseManager {
    
    // =============================================================================
    // BEFORE: Traditional implementation (15+ lines per method)
    // =============================================================================
    
    /*
    pub async fn get_companies_old(&self) -> Result<Vec<Company>, Error> {
        if let Some(client) = &self.client {
            let companies: Vec<Company> = client
                .select("company")
                .await?;
            Ok(companies)
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest(
                "No database connection".to_string()
            )))
        }
    }
    */
    
    // =============================================================================
    // AFTER: Refactored implementation (5 lines per method)
    // =============================================================================
    
    /// Get all companies - refactored version
    pub async fn get_companies_refactored(&self) -> Result<Vec<Company>, Error> {
        execute_with_client(&self.client, |client| {
            Box::pin(async move {
                let companies: Vec<Company> = client.select("company").await?;
                Ok(companies)
            })
        }).await
    }
    
    /// Get company by ID - refactored version
    pub async fn get_company_refactored(&self, id: &str) -> Result<Company, Error> {
        validate_entity_id(id, "company")?;
        
        execute_with_client(&self.client, |client| {
            let table_id = format!("company:{}", id);
            Box::pin(async move {
                let company: Option<Company> = client.select(&table_id).await?;
                company.ok_or_else(|| create_db_error("get", "company", Some("not found")))
            })
        }).await
    }
    
    /// Create company - refactored version
    pub async fn create_company_refactored(&self, company: CompanyCreate) -> Result<Company, Error> {
        execute_with_client(&self.client, |client| {
            Box::pin(async move {
                let result: Option<Company> = client
                    .create("company")
                    .content(company)
                    .await?;
                result.ok_or_else(|| create_db_error("create", "company", None))
            })
        }).await
    }
    
    /// Update company - refactored version  
    pub async fn update_company_refactored(&self, id: &str, company: CompanyUpdate) -> Result<Company, Error> {
        validate_entity_id(id, "company")?;
        
        execute_with_client(&self.client, |client| {
            let table_id = format!("company:{}", id);
            Box::pin(async move {
                let result: Option<Company> = client
                    .update(&table_id)
                    .merge(company)
                    .await?;
                result.ok_or_else(|| create_db_error("update", "company", Some("not found")))
            })
        }).await
    }
    
    /// Delete company - refactored version
    pub async fn delete_company_refactored(&self, id: &str) -> Result<Company, Error> {
        validate_entity_id(id, "company")?;
        
        execute_with_client(&self.client, |client| {
            let table_id = format!("company:{}", id);
            Box::pin(async move {
                let result: Option<Company> = client.delete(&table_id).await?;
                result.ok_or_else(|| create_db_error("delete", "company", Some("not found")))
            })
        }).await
    }
    
    // =============================================================================
    // EXAMPLE: More Complex Operations Also Benefit
    // =============================================================================
    
    /// Get company by abbreviation - refactored version
    pub async fn get_company_by_abbreviation_refactored(&self, abbreviation: &str) -> Result<Option<Company>, Error> {
        if abbreviation.is_empty() {
            return Err(create_db_error("get", "company", Some("abbreviation cannot be empty")));
        }
        
        execute_with_client(&self.client, |client| {
            Box::pin(async move {
                let query = "SELECT * FROM company WHERE abbreviation = $abbreviation LIMIT 1";
                let mut result = client
                    .query(query)
                    .bind(("abbreviation", abbreviation))
                    .await?;
                
                let companies: Vec<Company> = result.take(0)?;
                Ok(companies.into_iter().next())
            })
        }).await
    }
    
    /// Create project with auto-generated number - refactored version
    pub async fn create_project_with_number_refactored(&self, project: ProjectCreate) -> Result<Project, Error> {
        execute_with_client(&self.client, |client| {
            Box::pin(async move {
                // Get country code
                let country_query = "SELECT dial_code FROM country WHERE name = $country LIMIT 1";
                let mut country_result = client
                    .query(country_query)
                    .bind(("country", &project.country))
                    .await?;
                
                let countries: Vec<Country> = country_result.take(0)?;
                let country = countries.into_iter().next()
                    .ok_or_else(|| create_db_error("find", "country", Some(&project.country)))?;
                
                // Generate project number
                let year = chrono::Utc::now().format("%y").to_string();
                let next_seq_query = format!(
                    "SELECT VALUE count() FROM project WHERE number LIKE '{}{}%'",
                    year, country.dial_code
                );
                
                let mut seq_result = client.query(&next_seq_query).await?;
                let count: Option<i64> = seq_result.take(0)?;
                let sequence = count.unwrap_or(0) + 1;
                
                let project_number = format!("{}-{}{:02}", year, country.dial_code, sequence);
                
                // Create project with generated number
                let mut project_with_number = project;
                project_with_number.number = project_number;
                
                let result: Option<Project> = client
                    .create("project")
                    .content(project_with_number)
                    .await?;
                
                result.ok_or_else(|| create_db_error("create", "project", None))
            })
        }).await
    }
}

// =============================================================================
// ALTERNATIVE: Using Macros for Even More Reduction
// =============================================================================

/*
If we wanted to go even further, we could create macros for database operations too:

```rust
// This macro could generate the standard CRUD operations
db_crud_operations!(Company, CompanyCreate, CompanyUpdate, "company");

// Which would expand to all the basic CRUD methods:
// - get_companies() -> Result<Vec<Company>, Error>
// - get_company(id: &str) -> Result<Company, Error>  
// - create_company(data: CompanyCreate) -> Result<Company, Error>
// - update_company(id: &str, data: CompanyUpdate) -> Result<Company, Error>
// - delete_company(id: &str) -> Result<Company, Error>
```

This would reduce each entity's CRUD operations from ~75 lines to just 1 line!
*/

// =============================================================================
// MIGRATION STRATEGY
// =============================================================================

/*
## Step-by-Step Database Refactoring:

### Phase 1: Add Utility Functions ✅
- Create db/utils.rs with helper functions
- Add module declaration to db/mod.rs  
- Test utility functions work correctly

### Phase 2: Refactor One Entity at a Time
1. Start with Company operations (most used)
2. Replace old methods with refactored versions
3. Update command layer to use new methods
4. Test thoroughly before proceeding

### Phase 3: Remove Duplicate Code
1. Remove old implementations once refactored versions are tested
2. Clean up unused imports and helper functions
3. Update documentation and comments

### Phase 4: Apply to All Entities
1. Apply same patterns to Contact, Project, Fee entities
2. Standardize error messages and validation
3. Consider using macros for ultimate DRY compliance

## Expected Benefits:

- **60% reduction** in database layer code
- **Consistent error handling** across all operations  
- **Standardized validation** for all entity operations
- **Easier maintenance** - single point of change for common patterns
- **Better testing** - utilities can be unit tested independently
- **Type safety** maintained throughout the refactoring

## Files That Will Be Impacted:

1. `/src-tauri/src/db/mod.rs` - Main database implementation
2. `/src-tauri/src/commands/mod.rs` - Update to use new DB methods  
3. Tests - Update test cases to use new method names

The database refactoring should be done carefully with thorough testing
at each step since it affects core business logic.
*/