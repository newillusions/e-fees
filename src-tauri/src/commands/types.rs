//! Shared type definitions for Tauri commands.
//!
//! This module contains all update and configuration structs used across
//! multiple command modules, eliminating duplicate definitions.

use serde::{Deserialize, Serialize};

/// Partial company update structure for modifying existing companies.
///
/// This struct allows updating specific fields of a company without affecting
/// other fields. All fields are optional to support partial updates via the
/// database merge operation.
///
/// # Fields
/// - `name`: Full company name (e.g., "Conrad Hilton Hotels")
/// - `name_short`: Abbreviated name (e.g., "Conrad Etihad")
/// - `abbreviation`: Short code (e.g., "CHE")
/// - `city`: Company headquarters city
/// - `country`: Company headquarters country
/// - `reg_no`: Company registration number (optional)
/// - `tax_no`: Tax identification number (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyUpdate {
    pub name: Option<String>,
    pub name_short: Option<String>,
    pub abbreviation: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub reg_no: Option<String>,
    pub tax_no: Option<String>,
}

/// Partial contact update structure for modifying existing contacts.
///
/// Similar to CompanyUpdate, this struct allows updating specific fields
/// of a contact without affecting other fields. All fields are optional
/// to support partial updates via the database merge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactUpdate {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub position: Option<String>,
    pub company: Option<String>, // Company ID as string
}

/// Partial project update structure for modifying existing projects.
///
/// This struct allows updating specific fields of a project without affecting
/// other fields. All fields are optional to support partial updates via the
/// database merge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub name_short: Option<String>,
    pub status: Option<String>, // ProjectStatus as string
    pub area: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub folder: Option<String>,
}

/// Application settings structure for environment configuration.
///
/// This struct represents all configurable settings that can be modified
/// through the application's settings interface. Settings are persisted
/// to the `.env` file in the project root.
///
/// # Database Configuration
/// - `surrealdb_url`: WebSocket URL for SurrealDB connection
/// - `surrealdb_ns`: Database namespace (typically "emittiv")
/// - `surrealdb_db`: Database name (typically "projects")
/// - `surrealdb_user`: Authentication username
/// - `surrealdb_pass`: Authentication password
///
/// # Staff Information
/// - `staff_name`: Default staff member name for proposals
/// - `staff_email`: Default email address
/// - `staff_phone`: Default phone number
/// - `staff_position`: Default position/title
///
/// # File System
/// - `project_folder_path`: Base path for project template folders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub surrealdb_url: Option<String>,
    pub surrealdb_ns: Option<String>,
    pub surrealdb_db: Option<String>,
    pub surrealdb_user: Option<String>,
    pub surrealdb_pass: Option<String>,
    pub staff_name: Option<String>,
    pub staff_email: Option<String>,
    pub staff_phone: Option<String>,
    pub staff_position: Option<String>,
    pub project_folder_path: Option<String>,
    /// Development mode flag - enables verbose logging when true
    #[serde(default)]
    pub dev_mode: Option<bool>,
}
