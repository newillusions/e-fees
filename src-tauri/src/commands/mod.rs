//! # Tauri Command Module
//!
//! This module contains all Tauri commands that provide the bridge between the
//! frontend Svelte application and the backend Rust functionality. Commands are
//! exposed as asynchronous functions that can be called from the frontend using
//! the Tauri API.
//!
//! ## Architecture Overview
//!
//! The command layer follows a clean architecture pattern:
//! 1. **Frontend** calls Tauri commands via `invoke()`
//! 2. **Commands** validate inputs and handle errors
//! 3. **Database Manager** performs actual business logic
//! 4. **Results** are serialized back to frontend
//!
//! ## Module Organization
//!
//! Commands are organized by domain:
//! - `projects` - Project CRUD, search, and number generation
//! - `fees` - Fee proposal CRUD and JSON export
//! - `companies` - Company CRUD operations
//! - `contacts` - Contact CRUD operations
//! - `template_ops` - Project template copying and folder operations
//! - `settings` - Application settings management
//! - `system` - Health checks and diagnostics
//!
//! ## State Management
//!
//! All commands receive an `AppState` which is a thread-safe wrapper around
//! the `DatabaseManager`. This allows multiple commands to safely access the
//! database concurrently while maintaining consistency.

use crate::db::DatabaseManager;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// TYPE DEFINITIONS AND APPLICATION STATE
// ============================================================================

/// Application state that holds the database manager instance.
///
/// This type provides thread-safe access to the database manager across
/// all Tauri commands. The Arc<RwLock<T>> pattern ensures that:
/// - Multiple commands can read the database concurrently (RwLock read)
/// - Write operations are exclusive and thread-safe (RwLock write)
/// - Async-friendly: doesn't block the tokio executor
/// - Memory is managed efficiently across the application lifetime
pub type AppState = Arc<RwLock<DatabaseManager>>;

// ============================================================================
// CORE UTILITIES AND TYPES
// ============================================================================

pub mod types;
pub mod utils;

// Re-export shared types for use in other modules
pub use types::{CompanyUpdate, ContactUpdate, ProjectUpdate, AppSettings, AppSettingsPublic};

// ============================================================================
// DOMAIN-SPECIFIC COMMAND MODULES
// ============================================================================

// Project management
pub mod projects;
pub use projects::{
    get_projects, get_projects_page, get_project_by_id,
    create_project, update_project, delete_project,
    search_projects, generate_next_project_number, validate_project_number,
};

// Fee proposal management
pub mod fees;
pub use fees::{
    get_fees, get_fees_page, create_fee, update_fee, update_fee_pricing, delete_fee,
    write_fee_to_json, write_fee_to_json_safe,
    clone_fee_revision, get_fees_for_project,
};

// Company management
pub mod companies;
pub use companies::{
    get_companies, get_companies_page, get_company_by_id,
    create_company, update_company, delete_company,
};

// Contact management
pub mod contacts;
pub use contacts::{
    get_contacts, get_contacts_page, get_contact_by_id,
    create_contact, update_contact, delete_contact,
};

// Template and folder operations
pub mod template_ops;
pub use template_ops::{
    create_project_with_template, copy_project_template, populate_project_data,
    check_project_folder_exists, check_var_json_exists, check_var_json_template_exists,
    rename_folder_with_old_suffix, rename_var_json_with_old_suffix,
};

// Activity logging
pub mod activity_logs;
pub use activity_logs::{create_activity_log, get_activity_logs};

// Database connection
pub mod db_connection;
pub use db_connection::{check_db_connection, get_connection_status, reconnect_database};

// Fee JSON helpers (internal use)
pub mod fee_json;

// Folder management
pub mod folder_management;
pub use folder_management::{
    get_project_folder_location, move_project_folder, move_project_from_rfp,
    move_project_to_archive, list_projects_in_folder, validate_project_base_path,
};

// Folder sync
pub mod folder_sync;
pub use folder_sync::{
    scan_folder_sync, resolve_folder_inconsistency,
};

// Import wizard
pub mod import_wizard;
pub use import_wizard::{
    import_scan_directory, import_execute,
};

// Reference data
pub mod reference_data;
pub use reference_data::{
    search_countries, get_area_suggestions, get_all_cities, get_city_suggestions,
};

// Settings management
pub mod settings;
pub use settings::{
    get_settings, save_settings, get_dev_mode, reload_database_config,
    select_folder, open_folder_in_explorer,
};

// System commands
pub mod system;
pub use system::{
    health_check, get_db_info, get_stats, get_table_schema,
    position_window_4k, investigate_record, log_message,
};

// Excel export
pub mod export;
pub use export::export_fee_excel;
