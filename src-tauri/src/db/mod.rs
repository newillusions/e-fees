//! # Database Operations Module
//! 
//! This module provides comprehensive database connectivity and operations for the
//! Fee Proposal Management System using SurrealDB as the backend database.
//! 
//! ## Overview
//! 
//! The database module implements a robust connection management system with support
//! for both WebSocket and HTTP connections to SurrealDB. It provides CRUD operations
//! for all business entities including projects, companies, contacts, and fees.
//! 
//! ## Key Features
//! 
//! - **Dual Connection Support**: Automatic fallback from WebSocket to HTTP
//! - **Connection Monitoring**: Real-time health checks and status tracking
//! - **Flexible Authentication**: Support for Root, Namespace, and Database level auth
//! - **Comprehensive Error Handling**: User-friendly error messages and logging
//! - **Business Logic**: Project numbering, country lookup, and data validation
//! - **Performance Optimization**: Connection pooling and query optimization

pub mod utils;

//  
// ## Architecture
//  
// ```text
// ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
// │   Tauri Commands │◄──►│ DatabaseManager │◄──►│   SurrealDB     │
// │                 │    │                 │    │                 │
// │ - create_project│    │ - Connection    │    │ - Projects      │
// │ - get_companies │    │ - Health Check  │    │ - Companies     │
// │ - search_fees   │    │ - CRUD Ops      │    │ - Contacts      │
// └─────────────────┘    └─────────────────┘    └─────────────────┘
// ```
//  
// ## Database Schema
//  
// The module works with the following primary entities:
// - **Projects**: Core business projects with auto-generated numbering
// - **Companies**: Client organizations with contact information
// - **Contacts**: Individual contacts linked to companies
// - **Fees**: Fee proposals with revision tracking
// - **Countries**: Reference data for project numbering and location
//  
// ## Connection Management
//  
// The DatabaseManager handles connection lifecycle:
// 1. **Initialization**: Establish connection using environment configuration
// 2. **Authentication**: Multi-level auth with graceful fallback
// 3. **Monitoring**: Continuous health checks with automatic reconnection
// 4. **Cleanup**: Proper resource management and connection cleanup
//  
// ## Error Handling
//  
// All database operations implement comprehensive error handling:
// - Network connectivity issues
// - Authentication failures  
// - Query syntax errors
// - Data validation failures
// - Business rule violations
//  
// @fileoverview Database connectivity and CRUD operations for SurrealDB
// @author Fee Proposal Management System
// @version 2.0.0

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::engine::remote::http::{Client as HttpClient, Http};
use surrealdb::opt::auth::{Root, Namespace, Database};
use surrealdb::{Error, Surreal, Value};
use surrealdb::sql::Thing;
use tokio::time::interval;
use log::{error, info, warn};
use chrono::{self, Datelike};
use std::env;
use crate::commands::CompanyUpdate;

/// Interval for database connection health checks (30 seconds)
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

// ============================================================================
// CONFIGURATION STRUCTURES
// ============================================================================

/// Database configuration structure loaded from environment variables.
/// 
/// This structure contains all necessary connection parameters for SurrealDB
/// and implements automatic loading from environment variables with sensible
/// defaults for development environments.
/// 
/// # Environment Variables
/// 
/// - `SURREALDB_URL`: Database connection URL (default: ws://10.0.1.17:8000)
/// - `SURREALDB_NS`: Namespace name (default: emittiv)
/// - `SURREALDB_DB`: Database name (default: projects)
/// - `SURREALDB_USER`: Username (default: martin)
/// - `SURREALDB_PASS`: Password (required, no default)
/// 
/// # Examples
/// 
/// ```rust
/// let config = DatabaseConfig::from_env()?;
/// println!("Connecting to: {}", config.url);
/// ```
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database connection URL (WebSocket or HTTP)
    pub url: String,
    /// SurrealDB namespace name
    pub namespace: String,
    /// SurrealDB database name within the namespace
    pub database: String,
    /// Authentication username
    pub username: String,
    /// Authentication password
    pub password: String,
}

impl DatabaseConfig {
    /// Creates a new DatabaseConfig from environment variables.
    /// 
    /// This method reads database configuration from environment variables,
    /// providing sensible defaults for development while requiring the password
    /// to be explicitly set for security.
    /// 
    /// # Returns
    /// 
    /// - `Ok(DatabaseConfig)`: Successfully loaded configuration
    /// - `Err(String)`: Missing required SURREALDB_PASS environment variable
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// // Set environment variables first
    /// std::env::set_var("SURREALDB_PASS", "your_password");
    /// 
    /// match DatabaseConfig::from_env() {
    ///     Ok(config) => println!("Config loaded: {}", config.url),
    ///     Err(e) => eprintln!("Config error: {}", e),
    /// }
    /// ```
    /// 
    /// # Security Note
    /// 
    /// The password is the only required environment variable to prevent
    /// accidental connections with default credentials in production.
    pub fn from_env() -> Result<Self, String> {
        Ok(DatabaseConfig {
            url: env::var("SURREALDB_URL")
                .unwrap_or_else(|_| "ws://10.0.1.17:8000".to_string()),
            namespace: env::var("SURREALDB_NS")
                .unwrap_or_else(|_| "emittiv".to_string()),
            database: env::var("SURREALDB_DB")
                .unwrap_or_else(|_| "projects".to_string()),
            username: env::var("SURREALDB_USER")
                .unwrap_or_else(|_| "martin".to_string()),
            password: env::var("SURREALDB_PASS")
                .map_err(|_| "SURREALDB_PASS environment variable is required".to_string())?,
        })
    }
}

/// Connection status tracking structure for real-time monitoring.
/// 
/// This structure provides detailed information about the current database
/// connection state and is used by the frontend ConnectionStatus component
/// to display real-time connectivity information to users.
/// 
/// # Fields
/// 
/// - `is_connected`: Current connection state (true/false)
/// - `last_check`: ISO 8601 timestamp of last connectivity check
/// - `error_message`: Human-readable error description if disconnected
/// 
/// # Serialization
/// 
/// This structure is automatically serialized to JSON for frontend consumption
/// through the Tauri command system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    /// Whether the database connection is currently active
    pub is_connected: bool,
    /// ISO 8601 timestamp of the last connection check
    pub last_check: Option<String>,
    /// Human-readable error message if connection failed
    pub error_message: Option<String>,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self {
            is_connected: false,
            last_check: None,
            error_message: None,
        }
    }
}

// ============================================================================
// DATABASE ENTITY STRUCTURES
// ============================================================================

/// Project entity representing core business projects.
/// 
/// Projects are the central entity in the Fee Proposal Management System,
/// representing architectural or engineering projects for clients. Each project
/// has a unique auto-generated number and tracks location, status, and metadata.
/// 
/// # Project Numbering
/// 
/// Projects use the format YY-CCCNN where:
/// - YY: 2-digit year (25 = 2025)
/// - CCC: Country dial code (971 = UAE)
/// - NN: Sequential number (01-99 per year/country)
/// 
/// # Status Values
/// 
/// - `Draft`: Initial state, project being defined
/// - `Active`: Active project stage
/// - `Active`: Project is ongoing
/// - `On Hold`: Temporarily suspended
/// - `Completed`: Project finished successfully
/// - `Cancelled`: Project terminated
/// 
/// # File System Integration
/// 
/// The `folder` field contains the absolute path to the project's file
/// directory, enabling direct integration with template systems and
/// file management operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// SurrealDB record identifier (auto-generated)
    pub id: Option<Thing>,
    /// Full project name for display and documentation
    pub name: String,
    /// Abbreviated name for folder structures and internal use
    pub name_short: String,
    /// Current project status (see status values above)
    pub status: String, // 'Draft', 'Active', 'On Hold', 'Completed', 'Cancelled'
    /// Project area/district within the city
    pub area: String,
    /// City where the project is located
    pub city: String,
    /// Country where the project is located
    pub country: String,
    /// Absolute file system path to project folder
    pub folder: String,
    /// Structured project number (see ProjectNumber)
    pub number: ProjectNumber,
    /// Creation and modification timestamps
    pub time: TimeStamps,
}

/// CompanyCreate represents a new company being created (without auto-managed fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyCreate {
    /// Full legal company name
    pub name: String,
    /// Display name for UI and reports
    pub name_short: String,
    /// Short code for quick reference (e.g., "CHE", "DMCC")
    pub abbreviation: String,
    /// City where company headquarters is located
    pub city: String,
    /// Country where company is incorporated
    pub country: String,
    /// Company registration number (optional)
    pub reg_no: Option<String>,
    /// Tax identification number (optional)
    pub tax_no: Option<String>,
}

// Project creation struct without auto-managed fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProject {
    pub name: String,
    pub name_short: String,
    pub status: String,
    pub area: String,
    pub city: String,
    pub country: String,
    pub folder: String,
    pub number: ProjectNumber,
}

/// Project number structure implementing the YY-CCCNN numbering system.
/// 
/// This structure stores both the individual components and the formatted
/// string representation of project numbers. The numbering system ensures
/// unique identification while encoding temporal and geographical information.
/// 
/// # Format Specification
/// 
/// - **Year**: 2-digit year (2025 → 25)
/// - **Country**: 3-digit dial code (UAE → 971, Saudi → 966)
/// - **Sequence**: 2-digit sequential number (01-99)
/// - **ID**: Formatted string "YY-CCCNN" (e.g., "25-97105")
/// 
/// # Business Rules
/// 
/// - Maximum 99 projects per country per year
/// - Year range: 20-50 (2020-2050)
/// - Country codes must exist in countries table
/// - Sequential numbering starts at 01
/// 
/// # Examples
/// 
/// - "25-97105": 2025, UAE (971), 5th project
/// - "24-96601": 2024, Saudi Arabia (966), 1st project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectNumber {
    /// 2-digit year component (20-50)
    pub year: i32,
    /// 3-digit country dial code
    pub country: i32,
    /// 2-digit sequential number (1-99)
    pub seq: i32,
    /// Formatted project number string (YY-CCCNN)
    pub id: String, // The formatted number like "24-97101"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStamps {
    pub created_at: String,
    pub updated_at: String,
}

/// Company entity representing client organizations.
/// 
/// Companies are organizations that engage the firm for projects. Each company
/// record contains identification information, location details, and optional
/// registration numbers for legal compliance and documentation.
/// 
/// # Naming Convention
/// 
/// - `name`: Full legal company name
/// - `name_short`: Display name for UI and reports
/// - `abbreviation`: Short code for quick reference (2-4 characters)
/// 
/// # Registration Information
/// 
/// Optional fields for legal compliance:
/// - `reg_no`: Company registration number
/// - `tax_no`: Tax identification number
/// 
/// # Relationships
/// 
/// Companies are referenced by:
/// - Contacts (one-to-many)
/// - Fees (project proposals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    /// SurrealDB record identifier (auto-generated)
    pub id: Option<Thing>,
    /// Full legal company name
    pub name: String,
    /// Display name for UI and reports
    pub name_short: String,
    /// Short code for quick reference (e.g., "CHE", "DMCC")
    pub abbreviation: String,
    /// City where company headquarters is located
    pub city: String,
    /// Country where company is incorporated
    pub country: String,
    /// Company registration number (optional)
    pub reg_no: Option<String>,
    /// Tax identification number (optional)
    pub tax_no: Option<String>,
    /// Creation and modification timestamps
    pub time: TimeStamps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Option<Thing>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>, // Auto-computed
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub position: Option<String>,
    #[serde(default)]
    pub company: Option<Thing>, // Reference to company record
    #[serde(default)]
    pub time: Option<TimeStamps>,
}

// Contact creation struct without auto-managed fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactCreate {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub position: String,
    pub company: String, // Company ID as string (e.g., "CHE")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeCreate {
    pub name: String,
    pub number: String,
    pub rev: i32,
    pub status: String, // 'Draft', 'Prepared', 'Active', 'Sent', 'Under Review', 'Clarification', 'Negotiation', 'Awarded', 'Lost', 'Cancelled'
    pub issue_date: String,
    pub activity: String,
    pub package: String,
    pub project_id: String, // Project ID as string (e.g., "25_97107")
    pub company_id: String, // Company ID as string (e.g., "EMITTIV")
    pub contact_id: String, // Contact ID as string
    pub staff_name: String,
    pub staff_email: String,
    pub staff_phone: String,
    pub staff_position: String,
    pub strap_line: String,
    pub revisions: Vec<Revision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeUpdate {
    pub name: String,
    pub number: String,
    pub rev: i32,
    pub status: String, // 'Draft', 'Prepared', 'Active', 'Sent', 'Under Review', 'Clarification', 'Negotiation', 'Awarded', 'Lost', 'Cancelled'
    pub issue_date: String,
    pub activity: String,
    pub package: String,
    pub project_id: String, // Project ID as string (e.g., "25_97107")
    pub company_id: String, // Company ID as string (e.g., "EMITTIV")
    pub contact_id: String, // Contact ID as string
    pub staff_name: String,
    pub staff_email: String,
    pub staff_phone: String,
    pub staff_position: String,
    pub strap_line: String,
    pub revisions: Vec<Revision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    pub id: Option<Thing>,
    pub name: String,
    pub number: String,
    pub rev: i32, // Auto-computed from revisions
    pub status: String, // 'Draft', 'Prepared', 'Active', 'Sent', 'Under Review', 'Clarification', 'Negotiation', 'Awarded', 'Lost', 'Cancelled'
    pub issue_date: String, // YYMMDD format
    pub activity: String,
    pub package: String,
    pub project_id: Thing,
    pub company_id: Thing,
    pub contact_id: Thing,
    pub staff_name: String,
    pub staff_email: String,
    pub staff_phone: String,
    pub staff_position: String,
    pub strap_line: String,
    pub revisions: Vec<Revision>,
    pub time: TimeStamps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub revision_number: i32,
    pub revision_date: String,
    pub author_email: String,
    pub author_name: String,
    pub notes: String,
}

// Database manager - using HTTP client as primary, WS as fallback
#[derive(Clone)]
pub struct DatabaseManager {
    pub client: Option<DatabaseClient>,
    pub status: Arc<Mutex<ConnectionStatus>>,
    pub config: DatabaseConfig,
}

// Enum to handle different connection types
#[derive(Clone)]
pub enum DatabaseClient {
    Http(Surreal<HttpClient>),
    WebSocket(Surreal<Client>),
}

impl DatabaseClient {
    pub async fn health(&self) -> Result<(), Error> {
        match self {
            DatabaseClient::Http(client) => client.health().await,
            DatabaseClient::WebSocket(client) => client.health().await,
        }
    }
    
    
    pub async fn signin_root(&self, username: &str, password: &str) -> Result<(), Error> {
        match self {
            DatabaseClient::Http(client) => {
                client.signin(Root { username, password }).await?;
                Ok(())
            },
            DatabaseClient::WebSocket(client) => {
                client.signin(Root { username, password }).await?;
                Ok(())
            },
        }
    }
    
    pub async fn signin_namespace(&self, namespace: &str, username: &str, password: &str) -> Result<(), Error> {
        match self {
            DatabaseClient::Http(client) => {
                client.signin(Namespace { namespace, username, password }).await?;
                Ok(())
            },
            DatabaseClient::WebSocket(client) => {
                client.signin(Namespace { namespace, username, password }).await?;
                Ok(())
            },
        }
    }
    
    pub async fn signin_database(&self, namespace: &str, database: &str, username: &str, password: &str) -> Result<(), Error> {
        match self {
            DatabaseClient::Http(client) => {
                client.signin(Database { namespace, database, username, password }).await?;
                Ok(())
            },
            DatabaseClient::WebSocket(client) => {
                client.signin(Database { namespace, database, username, password }).await?;
                Ok(())
            },
        }
    }
    
    pub async fn use_ns_db(&self, namespace: &str, database: &str) -> Result<(), Error> {
        match self {
            DatabaseClient::Http(client) => client.use_ns(namespace).use_db(database).await,
            DatabaseClient::WebSocket(client) => client.use_ns(namespace).use_db(database).await,
        }
    }
    
    pub async fn select<T>(&self, table: &str) -> Result<Vec<T>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        match self {
            DatabaseClient::Http(client) => client.select(table).await,
            DatabaseClient::WebSocket(client) => client.select(table).await,
        }
    }

    
    pub async fn create_project(&self, project: Project) -> Result<Option<Project>, Error> {
        match self {
            DatabaseClient::Http(client) => client.create("projects").content(project).await,
            DatabaseClient::WebSocket(client) => client.create("projects").content(project).await,
        }
    }
    
    pub async fn create_new_project(&self, project: NewProject) -> Result<Option<Project>, Error> {
        // Use a custom query to let database auto-manage the time field
        // Convert project number to database-safe format (replace - with _)
        let project_id = project.number.id.replace("-", "_");
        
        // Build SET clause with project fields
        let set_clauses = vec![
            format!("name = '{}'", project.name.replace("'", "''")),
            format!("name_short = '{}'", project.name_short.replace("'", "''")),
            format!("status = '{}'", project.status.replace("'", "''")),
            format!("area = '{}'", project.area.replace("'", "''")),
            format!("city = '{}'", project.city.replace("'", "''")),
            format!("country = '{}'", project.country.replace("'", "''")),
            format!("folder = '{}'", project.folder.replace("'", "''")),
            format!("number = {{ year: {}, country: {}, seq: {}, id: '{}' }}", 
                project.number.year, project.number.country, project.number.seq, project.number.id.replace("'", "''"))
        ];
        
        let query = format!("CREATE projects:{} SET {}", project_id, set_clauses.join(", "));
        
        info!("Executing project creation query: {}", query);
        
        let mut response = match self {
            DatabaseClient::Http(client) => client.query(&query).await?,
            DatabaseClient::WebSocket(client) => client.query(&query).await?,
        };
        
        let result: Result<Vec<Project>, _> = response.take(0);
        match result {
            Ok(mut projects) => Ok(projects.pop()),
            Err(e) => Err(e),
        }
    }
    
    pub async fn create_company(&self, company: CompanyCreate) -> Result<Option<Company>, Error> {
        // Use the abbreviation as the company ID and escape single quotes
        let query = format!(
            "CREATE company:{} SET name = '{}', name_short = '{}', abbreviation = '{}', city = '{}', country = '{}', reg_no = {}, tax_no = {}, time = {{ created_at: time::now(), updated_at: time::now() }}",
            company.abbreviation,
            company.name.replace("'", "''"),
            company.name_short.replace("'", "''"),
            company.abbreviation.replace("'", "''"),
            company.city.replace("'", "''"),
            company.country.replace("'", "''"),
            company.reg_no.map_or("NONE".to_string(), |v| format!("'{}'", v.replace("'", "''"))),
            company.tax_no.map_or("NONE".to_string(), |v| format!("'{}'", v.replace("'", "''")))
        );
        
        let mut response = match self {
            DatabaseClient::Http(client) => client.query(&query).await?,
            DatabaseClient::WebSocket(client) => client.query(&query).await?,
        };
        
        let result: Result<Vec<Company>, _> = response.take(0);
        match result {
            Ok(mut companies) => Ok(companies.pop()),
            Err(e) => Err(e),
        }
    }
    
    
    pub async fn update_company_partial(&self, id: &str, company_update: CompanyUpdate) -> Result<Option<Company>, Error> {
        match self {
            DatabaseClient::Http(client) => client.update(("company", id)).merge(company_update).await,
            DatabaseClient::WebSocket(client) => client.update(("company", id)).merge(company_update).await,
        }
    }
    
    pub async fn delete_company(&self, id: &str) -> Result<Option<Company>, Error> {
        match self {
            DatabaseClient::Http(client) => client.delete(("company", id)).await,
            DatabaseClient::WebSocket(client) => client.delete(("company", id)).await,
        }
    }
    
    pub async fn create_contact(&self, contact: ContactCreate) -> Result<Option<Contact>, Error> {
        info!("Creating contact with company ID: {}", contact.company);
        
        // Create the contact with ALL required fields explicitly, let database auto-generate ID
        let full_name = format!("{} {}", contact.first_name, contact.last_name);
        let query = format!(
            "CREATE contacts SET first_name = '{}', last_name = '{}', full_name = '{}', email = '{}', phone = '{}', position = '{}', company = company:{}, time = {{ created_at: time::now(), updated_at: time::now() }}",
            contact.first_name.replace("'", "''"),
            contact.last_name.replace("'", "''"),
            full_name.replace("'", "''"),
            contact.email.replace("'", "''"),
            contact.phone.replace("'", "''"),
            contact.position.replace("'", "''"),
            contact.company
        );
        
        info!("Executing contact creation query: {}", query);
        
        let mut response = match self {
            DatabaseClient::Http(client) => client.query(&query).await?,
            DatabaseClient::WebSocket(client) => client.query(&query).await?,
        };
        
        let result: Result<Vec<Contact>, _> = response.take(0);
        match result {
            Ok(mut contacts) => Ok(contacts.pop()),
            Err(e) => Err(e),
        }
    }
    
    pub async fn update_contact_partial(&self, id: &str, contact_update: crate::commands::ContactUpdate) -> Result<Option<Contact>, Error> {
        match self {
            DatabaseClient::Http(client) => client.update(("contacts", id)).merge(contact_update).await,
            DatabaseClient::WebSocket(client) => client.update(("contacts", id)).merge(contact_update).await,
        }
    }
    
    
    pub async fn create_fee(&self, fee: FeeCreate) -> Result<Option<Fee>, Error> {
        // Use raw SQL query like contacts do to avoid Thing format issues
        // Generate Fee ID in format: project_number_rev (e.g., "25_97107_1")
        let fee_id = format!("{}_{}", fee.project_id.replace("-", "_"), fee.rev);
        
        let query = format!(
            "CREATE fee:{} SET name = '{}', number = '{}', rev = {}, project_id = {}, company_id = {}, contact_id = {}, status = '{}', issue_date = '{}', activity = '{}', package = '{}', strap_line = '{}', staff_name = '{}', staff_email = '{}', staff_phone = '{}', staff_position = '{}', revisions = [], time = {{ created_at: time::now(), updated_at: time::now() }}",
            fee_id,
            fee.name.replace("'", "''"),
            fee.number.replace("'", "''"),
            fee.rev,
            fee.project_id.replace("'", "''"),
            fee.company_id.replace("'", "''"), 
            fee.contact_id.replace("'", "''"),
            fee.status.replace("'", "''"),
            fee.issue_date.replace("'", "''"),
            fee.activity.replace("'", "''"),
            fee.package.replace("'", "''"),
            fee.strap_line.replace("'", "''"),
            fee.staff_name.replace("'", "''"),
            fee.staff_email.replace("'", "''"),
            fee.staff_phone.replace("'", "''"),
            fee.staff_position.replace("'", "''")
        );
        
        info!("Executing Fee creation query: {}", query);
        
        let mut response = match self {
            DatabaseClient::Http(client) => client.query(&query).await?,
            DatabaseClient::WebSocket(client) => client.query(&query).await?,
        };
        
        let result: Result<Vec<Fee>, _> = response.take(0);
        match result {
            Ok(mut fees) => Ok(fees.pop()),
            Err(e) => Err(e),
        }
    }
    
    pub async fn update_fee(&self, id: &str, fee: FeeUpdate) -> Result<Option<Fee>, Error> {
        info!("DatabaseClient::update_fee called with id: '{}' and fee: {:?}", id, fee);
        
        let query = format!(
            "UPDATE fee:{} SET name = '{}', number = '{}', rev = {}, project_id = {}, company_id = {}, contact_id = {}, status = '{}', issue_date = '{}', activity = '{}', package = '{}', strap_line = '{}', staff_name = '{}', staff_email = '{}', staff_phone = '{}', staff_position = '{}', time = {{ created_at: time.created_at OR time::now(), updated_at: time::now() }} RETURN AFTER",
            id,
            fee.name.replace("'", "''"),
            fee.number.replace("'", "''"),
            fee.rev,
            fee.project_id.replace("'", "''"),
            fee.company_id.replace("'", "''"),
            fee.contact_id.replace("'", "''"),
            fee.status.replace("'", "''"),
            fee.issue_date.replace("'", "''"),
            fee.activity.replace("'", "''"),
            fee.package.replace("'", "''"),
            fee.strap_line.replace("'", "''"),
            fee.staff_name.replace("'", "''"),
            fee.staff_email.replace("'", "''"),
            fee.staff_phone.replace("'", "''"),
            fee.staff_position.replace("'", "''")
        );
        
        info!("Executing update query: {}", query);
        
        let mut response = match self {
            DatabaseClient::Http(client) => client.query(&query).await?,
            DatabaseClient::WebSocket(client) => client.query(&query).await?,
        };
        
        let result: Result<Vec<Fee>, _> = response.take(0);
        match result {
            Ok(mut fees) => {
                info!("Update query returned {} records", fees.len());
                if fees.is_empty() {
                    warn!("Update query returned empty result set");
                }
                Ok(fees.pop())
            },
            Err(e) => {
                error!("Failed to parse update response: {}", e);
                Err(e)
            },
        }
    }
    
    pub async fn delete_fee(&self, id: &str) -> Result<Option<Fee>, Error> {
        match self {
            DatabaseClient::Http(client) => client.delete(("fee", id)).await,
            DatabaseClient::WebSocket(client) => client.delete(("fee", id)).await,
        }
    }

    pub async fn update_project(&self, id: &str, project_data: crate::commands::ProjectUpdate) -> Result<Option<Project>, Error> {
        match self {
            DatabaseClient::Http(client) => client.update(("projects", id)).merge(project_data).await,
            DatabaseClient::WebSocket(client) => client.update(("projects", id)).merge(project_data).await,
        }
    }

    pub async fn delete_project(&self, id: &str) -> Result<Option<Project>, Error> {
        match self {
            DatabaseClient::Http(client) => client.delete(("projects", id)).await,
            DatabaseClient::WebSocket(client) => client.delete(("projects", id)).await,
        }
    }
}

impl DatabaseManager {
    pub fn new() -> Result<Self, String> {
        let config = DatabaseConfig::from_env()?;
        info!("Database config loaded - URL: {}, NS: {}, DB: {}, User: {}", 
              config.url, config.namespace, config.database, config.username);
        
        Ok(Self {
            client: None,
            status: Arc::new(Mutex::new(ConnectionStatus::default())),
            config,
        })
    }

    // Initialize database connection
    pub async fn initialize(&mut self) -> Result<(), Error> {
        info!("Initializing database connection to {}", self.config.url);
        info!("Connection details - Namespace: {}, Database: {}", self.config.namespace, self.config.database);
        
        match self.connect().await {
            Ok(_) => {
                info!("Database connection established successfully");
                
                // Test the connection with a simple query
                if let Some(client) = &self.client {
                    match client.health().await {
                        Ok(_) => {
                            info!("Database health check passed");
                            self.update_status(true, None);
                        }
                        Err(e) => {
                            warn!("Database health check failed: {}", e);
                            self.update_status(false, Some(format!("Health check failed: {}", e)));
                        }
                    }
                }
                
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to establish database connection: {}", e);
                error!("{}", error_msg);
                
                // Provide more specific error messages
                let user_friendly_error = match e.to_string().as_str() {
                    s if s.contains("No such host is known") => {
                        format!("Cannot resolve hostname '10.0.1.17'. Please check if the SurrealDB server is accessible and the IP address is correct.")
                    },
                    s if s.contains("Connection refused") => {
                        format!("Connection refused by SurrealDB server at {}. Please check if SurrealDB is running.", self.config.url)
                    },
                    s if s.contains("Authentication failed") => {
                        format!("Authentication failed. Please check username '{}' and password are correct.", self.config.username)
                    },
                    s if s.contains("Namespace") || s.contains("Database") => {
                        format!("Failed to select namespace '{}' or database '{}'. Please check if they exist.", self.config.namespace, self.config.database)
                    },
                    _ => error_msg
                };
                
                self.update_status(false, Some(user_friendly_error.clone()));
                Err(e)
            }
        }
    }

    // Connect to SurrealDB
    async fn connect(&mut self) -> Result<(), Error> {
        info!("Attempting to connect to SurrealDB at {}", self.config.url);
        
        // Use WebSocket connection (preferred for SurrealDB)
        let db = if self.config.url.starts_with("ws://") || self.config.url.starts_with("wss://") {
            info!("Connecting to SurrealDB using WebSocket at {}", self.config.url);
            
            // Parse URL to remove protocol for Ws connection
            let connection_address = self.config.url
                .strip_prefix("ws://")
                .or_else(|| self.config.url.strip_prefix("wss://"))
                .unwrap_or(&self.config.url);
            
            match Surreal::new::<Ws>(connection_address).await {
                Ok(connection) => {
                    info!("Successfully established WebSocket connection to SurrealDB at {}", connection_address);
                    DatabaseClient::WebSocket(connection)
                }
                Err(err) => {
                    error!("Failed to establish WebSocket connection to {}: {}", connection_address, err);
                    return Err(err);
                }
            }
        } else {
            // Fallback to HTTP if not a WebSocket URL
            info!("Connecting to SurrealDB using HTTP at {}", self.config.url);
            
            match Surreal::new::<Http>(&self.config.url).await {
                Ok(connection) => {
                    info!("Successfully established HTTP connection to SurrealDB at {}", self.config.url);
                    DatabaseClient::Http(connection)
                }
                Err(err) => {
                    error!("Failed to establish HTTP connection to {}: {}", self.config.url, err);
                    return Err(err);
                }
            }
        };
        
        // Try different authentication methods
        info!("Authenticating with username: {}", self.config.username);
        
        // First try database authentication (which should work for the 'martin' user)
        let db_auth_result = db.signin_database(&self.config.namespace, &self.config.database, &self.config.username, &self.config.password).await;
        
        if let Err(db_err) = db_auth_result {
            warn!("Database authentication failed: {}, trying namespace authentication", db_err);
            
            // Try namespace authentication
            let ns_auth_result = db.signin_namespace(&self.config.namespace, &self.config.username, &self.config.password).await;
            
            if let Err(ns_err) = ns_auth_result {
                warn!("Namespace authentication failed: {}, trying root authentication", ns_err);
                
                // Try root authentication as last resort
                match db.signin_root(&self.config.username, &self.config.password).await {
                    Ok(_) => info!("Successfully authenticated with root-level credentials"),
                    Err(root_err) => {
                        error!("All authentication methods failed. Database: {}, Namespace: {}, Root: {}", 
                               db_err, ns_err, root_err);
                        return Err(db_err);
                    }
                }
            } else {
                info!("Successfully authenticated with namespace-level credentials");
            }
        } else {
            info!("Successfully authenticated with database-level credentials");
        }
        
        // Select namespace and database
        info!("Selecting namespace '{}' and database '{}'", self.config.namespace, self.config.database);
        match db.use_ns_db(&self.config.namespace, &self.config.database).await {
            Ok(_) => info!("Successfully selected namespace and database"),
            Err(e) => {
                error!("Failed to select namespace/database: {}", e);
                return Err(e);
            }
        }
        
        // Debug: Check what user/permissions we have
        info!("Testing authentication and permissions...");
        let info_query_result = match &db {
            DatabaseClient::Http(client) => {
                client.query("INFO FOR DB").await
            },
            DatabaseClient::WebSocket(client) => {
                client.query("INFO FOR DB").await
            }
        };
        
        match info_query_result {
            Ok(mut result) => {
                let info: Result<Vec<serde_json::Value>, _> = result.take(0);
                match info {
                    Ok(db_info) => {
                        info!("Database info query successful");
                        if let Some(first_result) = db_info.first() {
                            if let Some(tables) = first_result.get("tables") {
                                info!("Available tables: {}", tables);
                            }
                        }
                    },
                    Err(e) => error!("Failed to parse database info: {}", e),
                }
            },
            Err(e) => error!("INFO FOR DB query failed: {}", e),
        }
        
        self.client = Some(db);
        info!("SurrealDB connection fully established and ready");
        Ok(())
    }

    // Check if database is connected and responsive
    pub async fn check_connection(&self) -> bool {
        if let Some(client) = &self.client {
            match client.health().await {
                Ok(_) => {
                    info!("Database heartbeat successful");
                    true
                }
                Err(e) => {
                    warn!("Database heartbeat failed: {}", e);
                    false
                }
            }
        } else {
            warn!("No database client available");
            false
        }
    }

    // Update connection status
    fn update_status(&self, is_connected: bool, error_message: Option<String>) {
        if let Ok(mut status) = self.status.lock() {
            status.is_connected = is_connected;
            status.last_check = Some(chrono::Utc::now().to_rfc3339());
            status.error_message = error_message;
        }
    }

    // Get current connection status
    pub fn get_status(&self) -> ConnectionStatus {
        self.status.lock().unwrap().clone()
    }

    // Start heartbeat monitoring
    pub async fn start_heartbeat(status: Arc<Mutex<ConnectionStatus>>, manager: Arc<Mutex<DatabaseManager>>) {
        let mut interval = interval(HEARTBEAT_INTERVAL);
        
        tauri::async_runtime::spawn(async move {
            loop {
                interval.tick().await;
                
                let manager_clone = {
                    if let Ok(mgr) = manager.lock() {
                        mgr.clone()
                    } else {
                        warn!("Failed to acquire database manager lock for heartbeat");
                        continue;
                    }
                };
                
                let is_connected = manager_clone.check_connection().await;
                
                // Update status
                if let Ok(mut status_guard) = status.lock() {
                    status_guard.is_connected = is_connected;
                    status_guard.last_check = Some(chrono::Utc::now().to_rfc3339());
                    if !is_connected && status_guard.error_message.is_none() {
                        status_guard.error_message = Some("Heartbeat check failed".to_string());
                    } else if is_connected {
                        status_guard.error_message = None;
                    }
                }
                
                if is_connected {
                    info!("Database heartbeat: Connected");
                } else {
                    warn!("Database heartbeat: Disconnected");
                }
            }
        });
    }

    // Get all projects
    pub async fn get_projects(&self) -> Result<Vec<Project>, Error> {
        if let Some(client) = &self.client {
            info!("Attempting to query projects table");
            
            // Try both select() and raw query to debug
            let select_result: Result<Vec<Project>, Error> = client.select("projects").await;
            match &select_result {
                Ok(projects) => info!("select('projects') returned {} records", projects.len()),
                Err(e) => error!("select('projects') failed: {}", e),
            }
            
            
            select_result.or_else(|_| Ok(vec![]))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Search projects with fuzzy-like matching
    pub async fn search_projects(&self, query: &str) -> Result<Vec<Project>, Error> {
        if let Some(client) = &self.client {
            info!("Searching projects with query: {}", query);
            
            // Escape the query to prevent SQL injection
            let escaped_query = query.replace("'", "\\'");
            
            // Build a SurrealQL query that searches across multiple fields
            let search_query = format!(
                r#"SELECT * FROM projects WHERE 
                   string::lowercase(name) CONTAINS string::lowercase('{}') OR
                   string::lowercase(name_short) CONTAINS string::lowercase('{}') OR
                   string::lowercase(number.id) CONTAINS string::lowercase('{}') OR
                   string::lowercase(city) CONTAINS string::lowercase('{}') OR
                   string::lowercase(area) CONTAINS string::lowercase('{}') OR
                   string::lowercase(country) CONTAINS string::lowercase('{}') OR
                   string::lowercase(folder) CONTAINS string::lowercase('{}')
                   ORDER BY time.created_at DESC"#,
                escaped_query, escaped_query, escaped_query, escaped_query, 
                escaped_query, escaped_query, escaped_query
            );
            
            let result: Result<Vec<Project>, surrealdb::Error> = match client {
                DatabaseClient::Http(http_client) => {
                    let mut response = http_client.query(&search_query).await?;
                    response.take(0)
                },
                DatabaseClient::WebSocket(ws_client) => {
                    let mut response = ws_client.query(&search_query).await?;
                    response.take(0)
                }
            };
            
            match result {
                Ok(projects) => {
                    info!("Search query returned {} projects", projects.len());
                    Ok(projects)
                },
                Err(e) => {
                    error!("Search query failed: {}", e);
                    Err(e)
                }
            }
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Get all companies
    pub async fn get_companies(&self) -> Result<Vec<Company>, Error> {
        if let Some(client) = &self.client {
            info!("Attempting to query company table");
            
            // Try both select() and raw query to debug
            let select_result: Result<Vec<Company>, Error> = client.select("company").await;
            match &select_result {
                Ok(companies) => info!("select('company') returned {} records", companies.len()),
                Err(e) => error!("select('company') failed: {}", e),
            }
            
            select_result.or_else(|_| Ok(vec![]))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Get all contacts
    pub async fn get_contacts(&self) -> Result<Vec<Contact>, Error> {
        if let Some(client) = &self.client {
            info!("Attempting to query contacts table");
            
            let all_contacts: Vec<Contact> = client.select("contacts").await.unwrap_or_default();
            info!("Raw fetched {} contacts", all_contacts.len());
            
            // Filter out incomplete contacts (those missing required fields)
            let valid_contacts: Vec<Contact> = all_contacts.into_iter()
                .filter(|contact| {
                    // Check that all required fields are present and non-empty
                    let has_first_name = contact.first_name.as_ref().map_or(false, |s| !s.is_empty());
                    let has_last_name = contact.last_name.as_ref().map_or(false, |s| !s.is_empty());
                    let has_email = contact.email.as_ref().map_or(false, |s| !s.is_empty());
                    let has_phone = contact.phone.as_ref().map_or(false, |s| !s.is_empty());
                    let has_position = contact.position.as_ref().map_or(false, |s| !s.is_empty());
                    let has_company = contact.company.is_some();
                    
                    if !has_first_name || !has_last_name || !has_email {
                        info!("Filtering out incomplete contact with ID: {:?}", contact.id);
                    }
                    
                    has_first_name && has_last_name && has_email && has_phone && has_position && has_company
                })
                .collect();
            
            info!("Successfully fetched {} valid contacts (filtered from raw)", valid_contacts.len());
            
            Ok(valid_contacts)
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Get all fees
    pub async fn get_fees(&self) -> Result<Vec<Fee>, Error> {
        if let Some(client) = &self.client {
            info!("Attempting to query fee table");
            
            // Try the query and handle potential deserialization errors
            let result: Result<Vec<Fee>, Error> = client.select("fee").await;
            match result {
                Ok(fees) => {
                    info!("Successfully fetched {} fee records", fees.len());
                    Ok(fees)
                }
                Err(e) => {
                    error!("Failed to select from fee table: {}", e);
                    info!("This likely means deserialization failed - the Fee struct doesn't match the database schema");
                    // Return empty vec for now so the app doesn't crash
                    Ok(Vec::new())
                }
            }
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Create a new project
    pub async fn create_project(&self, project: Project) -> Result<Project, Error> {
        if let Some(client) = &self.client {
            let created: Option<Project> = client.create_project(project).await?;
            
            created.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to create project".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Create a new project from NewProject struct (time auto-managed by database)
    pub async fn create_new_project(&self, project: NewProject) -> Result<Project, Error> {
        if let Some(client) = &self.client {
            let created: Option<Project> = client.create_new_project(project).await?;
            
            created.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to create project".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Create a new company
    pub async fn create_company(&self, company: CompanyCreate) -> Result<Company, Error> {
        if let Some(client) = &self.client {
            let created: Option<Company> = client.create_company(company).await?;
            
            created.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to create company".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }


    // Update an existing company with partial data
    pub async fn update_company_partial(&self, id: &str, company_update: CompanyUpdate) -> Result<Company, Error> {
        if let Some(client) = &self.client {
            let updated: Option<Company> = client.update_company_partial(id, company_update).await?;
            
            updated.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to update company".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Delete a company
    pub async fn delete_company(&self, id: &str) -> Result<Company, Error> {
        if let Some(client) = &self.client {
            let deleted: Option<Company> = client.delete_company(id).await?;
            
            deleted.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to delete company".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Create a new contact
    pub async fn create_contact(&self, contact: ContactCreate) -> Result<Contact, Error> {
        if let Some(client) = &self.client {
            let created: Option<Contact> = client.create_contact(contact).await?;
            
            created.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to create contact".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Update an existing contact with partial data
    pub async fn update_contact_partial(&self, id: &str, contact_update: crate::commands::ContactUpdate) -> Result<Contact, Error> {
        if let Some(client) = &self.client {
            let updated: Option<Contact> = client.update_contact_partial(id, contact_update).await?;
            
            updated.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to update contact".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }



    // Create a new fee
    pub async fn create_fee(&self, fee: FeeCreate) -> Result<Fee, Error> {
        if let Some(client) = &self.client {
            let created: Option<Fee> = client.create_fee(fee).await?;
            
            created.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to create fee".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Update an existing fee
    pub async fn update_fee(&self, id: &str, fee: FeeUpdate) -> Result<Fee, Error> {
        if let Some(client) = &self.client {
            let updated: Option<Fee> = client.update_fee(id, fee).await?;
            
            updated.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to update fee".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Delete a fee
    pub async fn delete_fee(&self, id: &str) -> Result<Fee, Error> {
        if let Some(client) = &self.client {
            let deleted: Option<Fee> = client.delete_fee(id).await?;
            
            deleted.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to delete fee".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Update an existing project
    pub async fn update_project(&self, id: &str, project_update: crate::commands::ProjectUpdate) -> Result<Project, Error> {
        if let Some(client) = &self.client {
            let updated: Option<Project> = client.update_project(id, project_update).await?;
            
            updated.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to update project".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Delete a project
    pub async fn delete_project(&self, id: &str) -> Result<Project, Error> {
        if let Some(client) = &self.client {
            let deleted: Option<Project> = client.delete_project(id).await?;
            
            deleted.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to delete project".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Get table schema information
    pub async fn get_table_schema(&self, table_name: &str) -> Result<serde_json::Value, Error> {
        if let Some(client) = &self.client {
            let query = format!("INFO FOR TABLE {};", table_name);
            
            let mut result = match client {
                DatabaseClient::Http(client) => client.query(&query).await?,
                DatabaseClient::WebSocket(client) => client.query(&query).await?,
            };
            
            let schema: Option<serde_json::Value> = result.take(0)?;
            Ok(schema.unwrap_or(serde_json::json!({})))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Investigate a specific database record
    pub async fn investigate_record(&self, record_id: &str) -> Result<serde_json::Value, Error> {
        if let Some(client) = &self.client {
            info!("Investigating record: {}", record_id);
            
            let mut queries = Vec::new();
            let mut results = serde_json::json!({
                "record_id": record_id,
                "investigation": {}
            });
            
            // Try different approaches to query the record
            
            // 1. Try direct selection if it looks like a record ID
            if record_id.contains(":") {
                queries.push(format!("SELECT * FROM {};", record_id));
            }
            
            // 2. Try selecting from table if it looks like a table name
            if !record_id.contains(":") {
                queries.push(format!("SELECT * FROM {} LIMIT 5;", record_id));
            }
            
            // 3. Try to get info about the table/record
            let table_part = if record_id.contains(":") {
                record_id.split(":").next().unwrap_or("")
            } else {
                record_id
            };
            
            if !table_part.is_empty() {
                queries.push(format!("INFO FOR TABLE {};", table_part));
                queries.push(format!("SELECT count() FROM {} GROUP ALL;", table_part));
            }
            
            // 4. If it's a Fee record, also search for similar patterns
            if record_id.starts_with("fee:") {
                queries.push("SELECT * FROM fee WHERE string::contains(string(id), '23_966') LIMIT 10;".to_string());
                queries.push("SELECT * FROM fee WHERE string::contains(string(id), '⟨') LIMIT 10;".to_string());
            }
            
            // Execute each query and collect results
            for (i, query) in queries.iter().enumerate() {
                info!("Executing query {}: {}", i + 1, query);
                
                let query_result = match client {
                    DatabaseClient::Http(client) => {
                        match client.query(query).await {
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
                        }
                    },
                    DatabaseClient::WebSocket(client) => {
                        match client.query(query).await {
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
                        }
                    }
                };
                
                results["investigation"][format!("query_{}", i + 1)] = serde_json::json!({
                    "query": query,
                    "result": query_result
                });
            }
            
            Ok(results)
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }
    
    // Generate next project number for given country name and year
    pub async fn generate_next_project_number(&self, country_name: &str, year: Option<u8>) -> Result<String, Error> {
        info!("Generating next project number for country: {}, year: {:?}", country_name, year);
        
        if let Some(client) = &self.client {
            // First, look up the dial code from the country name  
            let country_lookup_query = format!(
                "SELECT dial_code FROM country WHERE name = '{}' LIMIT 1",
                country_name
            );
            
            info!("Looking up country code for: {}", country_name);
            
            let mut country_response = match client {
                DatabaseClient::Http(client) => client.query(&country_lookup_query).await?,
                DatabaseClient::WebSocket(client) => client.query(&country_lookup_query).await?,
            };
            
            let country_result: Result<Vec<serde_json::Value>, _> = country_response.take(0);
            let country_code = match country_result {
                Ok(records) => {
                    if let Some(first) = records.first() {
                        if let Some(dial_code_value) = first.get("dial_code") {
                            if let Some(dial_code) = dial_code_value.as_u64() {
                                dial_code as u16
                            } else {
                                return Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest(
                                    format!("Dial code is not a number for country: {}", country_name)
                                )));
                            }
                        } else {
                            return Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest(
                                format!("No dial_code field found for country: {}", country_name)
                            )));
                        }
                    } else {
                        return Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest(
                            format!("Country not found: {}", country_name)
                        )));
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            };
            
            info!("Found country code {} for country {}", country_code, country_name);
            
            // Get current year if not provided
            let year = year.unwrap_or_else(|| {
                let current_year = (chrono::Utc::now().year() % 100) as u8;
                current_year
            });
            
            // Query to find the max sequence number for the given year and country
            let query = format!(
                "SELECT number.seq FROM projects WHERE number.year = {} AND number.country = {} AND number.seq >= 1 AND number.seq <= 99 ORDER BY number.seq DESC LIMIT 1",
                year, country_code
            );
            
            info!("Executing query: {}", query);
            
            let mut response = match client {
                DatabaseClient::Http(client) => client.query(&query).await?,
                DatabaseClient::WebSocket(client) => client.query(&query).await?,
            };
            
            let result: Result<Vec<serde_json::Value>, _> = response.take(0);
            let next_seq = match result {
                Ok(records) => {
                    info!("Query result records: {:?}", records);
                    if let Some(first) = records.first() {
                        info!("First record: {:?}", first);
                        // The query returns the full object structure, so we need to navigate to number.seq
                        if let Some(number_obj) = first.get("number") {
                            if let Some(seq_value) = number_obj.get("seq") {
                                info!("Seq value found: {:?}", seq_value);
                                if let Some(seq) = seq_value.as_u64() {
                                    info!("Current max seq: {}, next will be: {}", seq, seq + 1);
                                    (seq + 1) as u8
                                } else {
                                    info!("Seq value is not a number, defaulting to 1");
                                    1
                                }
                            } else {
                                info!("No 'seq' field found in number object, defaulting to 1");
                                1
                            }
                        } else {
                            info!("No 'number' field found in record, defaulting to 1");
                            1
                        }
                    } else {
                        info!("No records found, starting with sequence 1");
                        1
                    }
                }
                Err(e) => {
                    error!("Query failed: {}", e);
                    1
                },
            };
            
            // Check if sequence would exceed 99 (business rule limit)
            if next_seq > 99 {
                error!("Sequence number {} exceeds limit of 99 for year {} country {}", next_seq, year, country_code);
                return Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest(
                    format!("Maximum of 99 projects per year per country reached for year {} country {}", year, country_code)
                )));
            }
            
            // Format the project number: YY-CCCNN (sequence always 2 digits)
            let project_number = format!("{:02}-{}{:02}", year, country_code, next_seq);
            info!("Generated project number: {}", project_number);
            
            Ok(project_number)
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }
    
    // Validate project number doesn't already exist
    pub async fn validate_project_number(&self, project_number: &str) -> Result<bool, Error> {
        info!("Validating project number: {}", project_number);
        
        if let Some(client) = &self.client {
            // Parse the project number format YY-CCCNN
            let parts: Vec<&str> = project_number.split('-').collect();
            if parts.len() != 2 || parts[0].len() != 2 || parts[1].len() != 5 {
                return Ok(false); // Invalid format
            }
            
            let year = parts[0].parse::<u8>().map_err(|_| {
                surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Invalid year format".to_string()))
            })?;
            
            let country = parts[1][..3].parse::<u16>().map_err(|_| {
                surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Invalid country code".to_string()))
            })?;
            
            let seq = parts[1][3..].parse::<u8>().map_err(|_| {
                surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Invalid sequence number".to_string()))
            })?;
            
            // Check if a project with this number already exists
            let query = format!(
                "SELECT count() FROM projects WHERE number.year = {} AND number.country = {} AND number.seq = {}",
                year, country, seq
            );
            
            let mut response = match client {
                DatabaseClient::Http(client) => client.query(&query).await?,
                DatabaseClient::WebSocket(client) => client.query(&query).await?,
            };
            
            let result: Result<Value, _> = response.take(0);
            match result {
                Ok(value) => {
                    // Convert Value to JSON to extract count
                    let json_value = serde_json::to_value(&value).unwrap_or_else(|_| serde_json::json!(null));
                    let count = if let Some(count_value) = json_value.as_u64() {
                        count_value
                    } else if let Some(obj) = json_value.as_object() {
                        obj.get("count").and_then(|v| v.as_u64()).unwrap_or(0)
                    } else {
                        0
                    };
                    
                    let is_valid = count == 0;
                    info!("Project number {} validation - count: {}, is_valid: {}", project_number, count, is_valid);
                    Ok(is_valid)
                }
                Err(e) => {
                    error!("Failed to validate project number: {}", e);
                    Err(e)
                }
            }
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }
    
    // Search countries with fuzzy matching
    pub async fn search_countries(&self, query: &str) -> Result<Vec<serde_json::Value>, Error> {
        info!("Searching countries with query: {}", query);
        
        if let Some(client) = &self.client {
            let search_query = format!(
                "SELECT name, name_formal, name_official, code, code_alt, dial_code FROM country WHERE (name IS NOT NONE AND string::lowercase(name) CONTAINS string::lowercase('{}')) OR (name_formal IS NOT NONE AND string::lowercase(name_formal) CONTAINS string::lowercase('{}')) OR (name_official IS NOT NONE AND string::lowercase(name_official) CONTAINS string::lowercase('{}')) OR (code IS NOT NONE AND string::lowercase(code) CONTAINS string::lowercase('{}')) OR (code_alt IS NOT NONE AND string::lowercase(code_alt) CONTAINS string::lowercase('{}')) OR (dial_code IS NOT NONE AND string::contains(<string>dial_code, '{}')) ORDER BY name ASC LIMIT 15",
                query, query, query, query, query, query
            );
            
            info!("Executing country search query: {}", search_query);
            
            let mut response = match client {
                DatabaseClient::Http(client) => client.query(&search_query).await?,
                DatabaseClient::WebSocket(client) => client.query(&search_query).await?,
            };
            
            let result: Result<Vec<serde_json::Value>, _> = response.take(0);
            match result {
                Ok(countries) => {
                    info!("Found {} countries matching '{}'", countries.len(), query);
                    Ok(countries)
                }
                Err(e) => {
                    error!("Failed to search countries: {}", e);
                    Err(e)
                }
            }
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Get area suggestions for a country
    pub async fn get_area_suggestions(&self, country: &str) -> Result<Vec<String>, Error> {
        info!("Getting area suggestions for country: {}", country);
        
        if let Some(client) = &self.client {
            let query = format!(
                "SELECT area FROM projects WHERE country = '{}' AND area IS NOT NONE GROUP BY area ORDER BY area ASC LIMIT 20",
                country.replace("'", "''") // Escape single quotes
            );
            
            info!("Executing area suggestions query: {}", query);
            
            let mut response = match client {
                DatabaseClient::Http(client) => client.query(&query).await?,
                DatabaseClient::WebSocket(client) => client.query(&query).await?,
            };
            
            let result: Result<Vec<serde_json::Value>, _> = response.take(0);
            match result {
                Ok(areas) => {
                    let area_strings: Vec<String> = areas
                        .into_iter()
                        .filter_map(|area| area.get("area").and_then(|a| a.as_str()).map(|s| s.to_string()))
                        .collect();
                    info!("Found {} area suggestions for '{}'", area_strings.len(), country);
                    Ok(area_strings)
                }
                Err(e) => {
                    error!("Failed to get area suggestions: {}", e);
                    Err(e)
                }
            }
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Get city suggestions for a country
    pub async fn get_city_suggestions(&self, country: &str) -> Result<Vec<String>, Error> {
        info!("Getting city suggestions for country: {}", country);
        
        if let Some(client) = &self.client {
            let query = format!(
                "SELECT city FROM projects WHERE country = '{}' AND city IS NOT NONE GROUP BY city ORDER BY city ASC LIMIT 20",
                country.replace("'", "''") // Escape single quotes
            );
            
            info!("Executing city suggestions query: {}", query);
            
            let mut response = match client {
                DatabaseClient::Http(client) => client.query(&query).await?,
                DatabaseClient::WebSocket(client) => client.query(&query).await?,
            };
            
            let result: Result<Vec<serde_json::Value>, _> = response.take(0);
            match result {
                Ok(cities) => {
                    let city_strings: Vec<String> = cities
                        .into_iter()
                        .filter_map(|city| city.get("city").and_then(|c| c.as_str()).map(|s| s.to_string()))
                        .collect();
                    info!("Found {} city suggestions for '{}'", city_strings.len(), country);
                    Ok(city_strings)
                }
                Err(e) => {
                    error!("Failed to get city suggestions: {}", e);
                    Err(e)
                }
            }
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    pub async fn get_all_cities(&self) -> Result<Vec<String>, Error> {
        info!("Getting all city suggestions from projects and companies");
        
        if let Some(client) = &self.client {
            // SurrealDB doesn't support UNION, so we'll get cities from both tables separately
            // and combine them in Rust
            
            // Get cities from projects
            let projects_query = "SELECT city FROM projects WHERE city IS NOT NONE GROUP BY city ORDER BY city ASC";
            info!("Executing projects cities query");
            
            let mut projects_response = match client {
                DatabaseClient::Http(client) => client.query(projects_query).await?,
                DatabaseClient::WebSocket(client) => client.query(projects_query).await?,
            };
            
            let projects_result: Result<Vec<serde_json::Value>, _> = projects_response.take(0);
            let mut all_cities = Vec::new();
            
            match projects_result {
                Ok(cities) => {
                    let project_cities: Vec<String> = cities
                        .into_iter()
                        .filter_map(|city| city.get("city").and_then(|c| c.as_str()).map(|s| s.to_string()))
                        .collect();
                    all_cities.extend(project_cities);
                }
                Err(e) => {
                    error!("Failed to get cities from projects: {}", e);
                }
            }
            
            // Get cities from companies
            let companies_query = "SELECT city FROM company WHERE city IS NOT NONE GROUP BY city ORDER BY city ASC";
            info!("Executing companies cities query");
            
            let mut companies_response = match client {
                DatabaseClient::Http(client) => client.query(companies_query).await?,
                DatabaseClient::WebSocket(client) => client.query(companies_query).await?,
            };
            
            let companies_result: Result<Vec<serde_json::Value>, _> = companies_response.take(0);
            
            match companies_result {
                Ok(cities) => {
                    let company_cities: Vec<String> = cities
                        .into_iter()
                        .filter_map(|city| city.get("city").and_then(|c| c.as_str()).map(|s| s.to_string()))
                        .collect();
                    all_cities.extend(company_cities);
                }
                Err(e) => {
                    error!("Failed to get cities from companies: {}", e);
                }
            }
            
            // Remove duplicates and sort
            all_cities.sort();
            all_cities.dedup();
            
            // Limit to 50 cities
            all_cities.truncate(50);
            
            info!("Found {} total unique city suggestions", all_cities.len());
            Ok(all_cities)
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }
}