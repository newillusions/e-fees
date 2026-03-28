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
pub use types::{AppSettings, AppSettingsPublic, CompanyUpdate, ContactUpdate, ProjectUpdate};

// ============================================================================
// DOMAIN-SPECIFIC COMMAND MODULES
// ============================================================================

// Project management
pub mod projects;
pub use projects::{
    create_project, delete_project, generate_next_project_number, get_project_by_id, get_projects,
    get_projects_page, search_projects, update_project, validate_project_number,
};

// Fee proposal management
pub mod fees;
pub use fees::{
    clone_fee_revision, create_fee, delete_fee, get_fees, get_fees_for_project, get_fees_page,
    update_fee, update_fee_pricing, write_fee_to_json, write_fee_to_json_safe,
};

// Company management
pub mod companies;
pub use companies::{
    create_company, delete_company, get_companies, get_companies_page, get_company_by_id,
    update_company,
};

// Contact management
pub mod contacts;
pub use contacts::{
    create_contact, delete_contact, get_contact_by_id, get_contacts, get_contacts_page,
    update_contact,
};

// Template and folder operations
pub mod template_ops;
pub use template_ops::{
    check_project_folder_exists, check_var_json_exists, check_var_json_template_exists,
    copy_project_template, create_project_with_template, populate_project_data,
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
    get_project_folder_location, list_projects_in_folder, move_project_folder,
    move_project_from_rfp, move_project_to_archive, validate_project_base_path,
};

// Folder sync
pub mod folder_sync;
pub use folder_sync::{resolve_folder_inconsistency, scan_folder_sync};

// Import wizard
pub mod import_wizard;
pub use import_wizard::{import_execute, import_scan_directory};

// Reference data
pub mod reference_data;
pub use reference_data::{
    get_all_cities, get_area_suggestions, get_city_suggestions, search_countries,
};

// Settings management
pub mod settings;
pub use settings::{
    get_dev_mode, get_settings, open_folder_in_explorer, reload_database_config, save_settings,
    select_folder,
};

// System commands
pub mod system;
pub use system::{
    get_db_info, get_log_level, get_stats, get_table_schema, health_check, investigate_record,
    log_message, position_window_4k, set_log_level,
};

// Excel export
pub mod export;
pub use export::{export_fee_excel, export_fee_template};

// Batch operations (multi-select)
pub mod batch_ops;
pub use batch_ops::{batch_delete_entities, batch_update_status};

// Fee stage access (scope–pricing linkage)
pub mod fee_stages;
pub use fee_stages::{add_stage_to_fee, get_fee_stages, get_stage_dictionary};

// Stage dictionary (autocomplete for stage names)
pub mod stage_dictionary;
pub use stage_dictionary::{add_stage_to_dictionary, search_stage_dictionary};

// Scope markdown export
pub mod scope_export;
pub use scope_export::export_scope_markdown;
