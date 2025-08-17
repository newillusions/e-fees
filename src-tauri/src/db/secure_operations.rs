//! # Secure Database Operations
//! 
//! This module provides secure implementations using input validation
//! to prevent SQL injection attacks.

use crate::db::{DatabaseClient, NewProject, Project, Company, CompanyCreate, Contact, ContactCreate};
use crate::db::security::{InputValidator, ValidationError};
use crate::commands::ContactUpdate;
use surrealdb::Error;
use log::info;

impl DatabaseClient {
    /// Securely create a new project with input validation
    pub async fn secure_create_project(&self, project: NewProject) -> Result<Option<Project>, Error> {
        info!("Creating project with secure validation");
        
        // Validate all input fields
        if let Err(e) = InputValidator::validate_project_name(&project.name) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid project name: {}", e))));
        }

        if let Err(e) = InputValidator::validate_text_field("name_short", &project.name_short, 1, 50) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid project short name: {}", e))));
        }

        if let Err(e) = InputValidator::validate_status(&project.status, &["Draft", "Active", "On Hold", "Completed", "Cancelled"]) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid project status: {}", e))));
        }

        if let Err(e) = InputValidator::validate_project_number(&project.number.id) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid project number: {}", e))));
        }

        // Use the existing create_new_project with validated input
        self.create_new_project(project).await
    }

    /// Securely create a new company with validation
    pub async fn secure_create_company(&self, company: CompanyCreate) -> Result<Option<Company>, Error> {
        info!("Creating company with secure validation");

        // Validate input fields
        if let Err(e) = InputValidator::validate_text_field("name", &company.name, 2, 200) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid company name: {}", e))));
        }

        if let Err(e) = InputValidator::validate_text_field("abbreviation", &company.abbreviation, 1, 10) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid abbreviation: {}", e))));
        }

        if let Err(e) = InputValidator::validate_id(&company.abbreviation) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid abbreviation format: {}", e))));
        }

        // Use the existing create_company with validated input
        self.create_company(company).await
    }

    /// Securely create a new contact with validation
    pub async fn secure_create_contact(&self, contact: ContactCreate) -> Result<Option<Contact>, Error> {
        info!("Creating contact with secure validation");

        // Validate input fields
        if let Err(e) = InputValidator::validate_text_field("first_name", &contact.first_name, 1, 50) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid first name: {}", e))));
        }

        if let Err(e) = InputValidator::validate_text_field("last_name", &contact.last_name, 1, 50) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid last name: {}", e))));
        }

        if let Err(e) = InputValidator::validate_email(&contact.email) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid email: {}", e))));
        }

        if let Err(e) = InputValidator::validate_phone(&contact.phone) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid phone: {}", e))));
        }

        if let Err(e) = InputValidator::validate_id(&contact.company) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid company ID: {}", e))));
        }

        // Use the existing create_contact with validated input
        self.create_contact(contact).await
    }

    /// Securely update a contact with validation
    pub async fn secure_update_contact(&self, id: &str, contact_update: ContactUpdate) -> Result<Option<Contact>, Error> {
        info!("Updating contact with secure validation: {}", id);

        // Validate ID
        if let Err(e) = InputValidator::validate_id(id) {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid contact ID: {}", e))));
        }

        // Validate optional fields if provided
        if let Some(ref first_name) = contact_update.first_name {
            if let Err(e) = InputValidator::validate_text_field("first_name", first_name, 1, 50) {
                return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid first name: {}", e))));
            }
        }

        if let Some(ref last_name) = contact_update.last_name {
            if let Err(e) = InputValidator::validate_text_field("last_name", last_name, 1, 50) {
                return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid last name: {}", e))));
            }
        }

        if let Some(ref email) = contact_update.email {
            if let Err(e) = InputValidator::validate_email(email) {
                return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid email: {}", e))));
            }
        }

        if let Some(ref phone) = contact_update.phone {
            if let Err(e) = InputValidator::validate_phone(phone) {
                return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid phone: {}", e))));
            }
        }

        if let Some(ref position) = contact_update.position {
            if let Err(e) = InputValidator::validate_text_field("position", position, 1, 100) {
                return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid position: {}", e))));
            }
        }

        if let Some(ref company) = contact_update.company {
            if let Err(e) = InputValidator::validate_id(company) {
                return Err(Error::Api(surrealdb::error::Api::InvalidRequest(format!("Invalid company ID: {}", e))));
            }
        }

        // Use the existing update_contact_partial with validated input
        self.update_contact_partial(id, contact_update).await
    }

    /// Securely search projects with input validation
    pub async fn secure_search_projects(&self, query: &str) -> Result<Vec<Project>, Error> {
        info!("Searching projects with secure query: {}", query);

        // Validate search query
        if query.len() > 100 {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest("Search query too long".to_string())));
        }

        // Sanitize search query - remove any SQL injection attempts
        let safe_query = InputValidator::sanitize_for_display(query);
        if safe_query != query {
            info!("Search query was sanitized from '{}' to '{}'", query, safe_query);
        }

        // Use the existing search_projects with sanitized input
        self.search_projects(&safe_query).await
    }

    /// Securely search countries with validation
    pub async fn secure_search_countries(&self, query: &str) -> Result<Vec<serde_json::Value>, Error> {
        info!("Searching countries with secure query: {}", query);

        // Validate and sanitize search query
        if query.len() > 50 {
            return Err(Error::Api(surrealdb::error::Api::InvalidRequest("Country search query too long".to_string())));
        }

        let safe_query = InputValidator::sanitize_for_display(query);
        
        // Use the existing search_countries with sanitized input
        self.search_countries(&safe_query).await
    }
}

/// Convert ValidationError to SurrealDB Error
impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        Error::Api(surrealdb::error::Api::InvalidRequest(err.to_string()))
    }
}