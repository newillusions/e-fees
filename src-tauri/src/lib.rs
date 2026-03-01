use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, error};
use tauri::Manager;

// v0.10.18 - lowercase branding, passive Windows installer (with dev mode and enhanced logging)
mod db;
mod commands;
mod updater_logger;
mod agent_server;
mod excel_export;

use db::{DatabaseManager, DatabaseConfig};
use commands::{
    check_db_connection,
    get_connection_status,
    reconnect_database,
    get_projects,
    search_projects,
    get_companies,
    get_contacts,
    get_fees,
    create_project,
    update_project,
    delete_project,
    create_company,
    update_company,
    delete_company,
    create_contact,
    update_contact,
    delete_contact,
    create_fee,
    update_fee,
    update_fee_pricing,
    delete_fee,
    clone_fee_revision,
    get_fees_for_project,
    // Pagination commands
    get_projects_page,
    get_companies_page,
    get_contacts_page,
    get_fees_page,
    // Single entity fetch (for on-demand loading)
    get_project_by_id,
    get_company_by_id,
    get_contact_by_id,
    health_check,
    get_stats,
    get_db_info,
    get_table_schema,
    position_window_4k,
    get_settings,
    save_settings,
    get_dev_mode,
    reload_database_config,
    select_folder,
    open_folder_in_explorer,
    investigate_record,
    search_countries,
    generate_next_project_number,
    validate_project_number,
    create_project_with_template,
    copy_project_template,
    get_area_suggestions,
    get_all_cities,
    get_city_suggestions,
    write_fee_to_json,
    write_fee_to_json_safe,
    // Activity log commands
    create_activity_log,
    get_activity_logs,
    check_project_folder_exists,
    check_var_json_exists,
    check_var_json_template_exists,
    rename_folder_with_old_suffix,
    rename_var_json_with_old_suffix,
    populate_project_data,
    get_project_folder_location,
    move_project_folder,
    move_project_from_rfp,
    move_project_to_archive,
    list_projects_in_folder,
    validate_project_base_path,
    // Folder sync commands
    scan_folder_sync,
    resolve_folder_inconsistency,
    log_message,
    // Log level control
    set_log_level,
    get_log_level,
    // Import wizard commands
    import_scan_directory,
    import_execute,
    // Excel export
    export_fee_excel,
    export_fee_template,
    // Batch operations
    batch_delete_entities,
    batch_update_status,
};

/// Load database configuration from the settings system.
/// 
/// This function attempts to load database configuration from the application settings
/// stored in the app data directory, which is essential for production builds where
/// environment variables are not available.
/// 
/// # Arguments
/// 
/// * `app_handle` - Tauri application handle for accessing settings
/// 
/// # Returns
/// 
/// - `Ok(DatabaseConfig)`: Successfully loaded and parsed configuration
/// - `Err(String)`: Configuration not found or invalid
async fn load_database_config_from_settings(app_handle: &tauri::AppHandle) -> Result<DatabaseConfig, String> {
    // Try to load settings from the app data directory
    // Use internal function to get full settings including password (not exposed to frontend)
    match commands::settings::get_settings_internal(app_handle).await {
        Ok(settings) => {
            // Convert settings to database configuration
            DatabaseConfig::from_settings(&settings)
        },
        Err(e) => {
            Err(format!("Failed to load settings: {}", e))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            info!("Single instance detected - bringing existing window to front");
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.set_focus() {
                    error!("Failed to set focus on window: {}", e);
                }
                if let Err(e) = window.unminimize() {
                    error!("Failed to unminimize window: {}", e);
                }
                if let Err(e) = window.show() {
                    error!("Failed to show window: {}", e);
                }
            }
            info!("Single instance enforcement - prevented duplicate launch");
        }))
        .setup(|app| {
            // Setup logging — init plugin at Debug so it captures everything,
            // then control visibility via the runtime max_level filter
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Debug)
                    .build(),
            )?;
            // Default runtime filter to Info (user can change via Settings)
            log::set_max_level(log::LevelFilter::Info);
            
            // Setup MCP plugin - don't crash app if it fails
            info!("Attempting to initialize MCP plugin with socket server");
            match app.handle().plugin(
                tauri_plugin_mcp::init_with_config(
                    tauri_plugin_mcp::PluginConfig::new("app".to_string())
                        .start_socket_server(true)
                        .socket_path("/tmp/tauri-mcp.sock".into())
                )
            ) {
                Ok(_) => info!("MCP plugin initialized successfully"),
                Err(e) => {
                    error!("Failed to initialize MCP plugin: {}", e);
                    error!("Continuing without MCP functionality");
                    // Don't crash the app, just log the error and continue
                }
            }

            info!("Initializing Fee Proposal Management Application");
            
            // Load environment variables as fallback for development
            if let Err(e) = dotenvy::dotenv() {
                info!("No .env file found or error loading it: {}", e);
                info!("Will try settings system for production");
            }

            // Initialize with unconfigured state - will be configured async
            let db_manager = DatabaseManager::new_unconfigured();
            let app_state = Arc::new(RwLock::new(db_manager));

            // Clone state for heartbeat monitoring
            let heartbeat_state = app_state.clone();
            // Get status reference for heartbeat - we need to do this synchronously during setup
            let status = {
                // Use try_read to avoid blocking, fall back to a new status if lock can't be acquired
                // This should always succeed since we just created the state
                let manager = app_state.try_read().expect("Failed to read newly created state");
                manager.status.clone()
            };
            
            // Set up the application state
            app.manage(app_state.clone());

            // Initialize database connection in async context using Tauri's runtime
            let init_state = app_state.clone();
            let app_handle_clone = app.handle().clone();
            
            tauri::async_runtime::spawn(async move {
                info!("Starting database initialization");

                // Restore saved log level from settings
                if let Ok(settings) = commands::settings::get_settings_internal(&app_handle_clone).await {
                    if let Some(ref level) = settings.log_level {
                        let filter = match level.as_str() {
                            "off" => log::LevelFilter::Off,
                            "error" => log::LevelFilter::Error,
                            "warn" => log::LevelFilter::Warn,
                            "debug" => log::LevelFilter::Debug,
                            "trace" => log::LevelFilter::Trace,
                            _ => log::LevelFilter::Info,
                        };
                        log::set_max_level(filter);
                        info!("Log level restored from settings: {}", level);
                    }
                }

                // Try to load configuration from settings first, then from environment
                let configured_manager = match load_database_config_from_settings(&app_handle_clone).await {
                    Ok(config) => {
                        info!("Database configuration loaded from settings");
                        Some(DatabaseManager::from_config(config))
                    },
                    Err(settings_err) => {
                        info!("Settings configuration not available ({}), trying environment variables", settings_err);
                        
                        match DatabaseManager::new() {
                            Ok(manager) => {
                                info!("Database configuration loaded from environment variables");
                                Some(manager)
                            },
                            Err(env_err) => {
                                info!("Environment configuration also not available ({})", env_err);
                                info!("User will need to configure database through FirstRunSetup");
                                None
                            }
                        }
                    }
                };
                
                let initialized = if let Some(mut manager) = configured_manager {
                    // We have a configured manager, try to initialize it
                    match manager.initialize().await {
                        Ok(_) => {
                            info!("Database initialized successfully");
                            // Update the original manager in the state using write lock
                            let mut state_manager = init_state.write().await;
                            *state_manager = manager;
                            true
                        }
                        Err(e) => {
                            error!("Failed to initialize database: {}", e);
                            false
                        }
                    }
                } else {
                    info!("Database manager remains unconfigured, skipping initialization");
                    false
                };
                
                if initialized {
                    // Start heartbeat monitoring
                    info!("Starting database heartbeat monitoring");
                    DatabaseManager::start_heartbeat(status, heartbeat_state).await;

                    // Start agent API server (non-blocking, runs in background)
                    let agent_state = init_state.clone();
                    let agent_port = std::env::var("EFEES_AGENT_PORT")
                        .ok()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or(3100);
                    tauri::async_runtime::spawn(async move {
                        agent_server::start_agent_server(agent_state, agent_port).await;
                    });
                }
            });
            
            info!("Application setup completed successfully");
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            check_db_connection,
            get_connection_status,
            reconnect_database,
            get_projects,
            search_projects,
            get_companies,
            get_contacts,
            get_fees,
            create_project,
            update_project,
            delete_project,
            create_company,
            update_company,
            delete_company,
            create_contact,
            update_contact,
            delete_contact,
            create_fee,
            update_fee,
            update_fee_pricing,
            delete_fee,
            clone_fee_revision,
            get_fees_for_project,
            // Pagination commands
            get_projects_page,
            get_companies_page,
            get_contacts_page,
            get_fees_page,
            // Single entity fetch (for on-demand loading)
            get_project_by_id,
            get_company_by_id,
            get_contact_by_id,
            health_check,
            get_stats,
            get_db_info,
            get_table_schema,
            position_window_4k,
            get_settings,
            save_settings,
            get_dev_mode,
            reload_database_config,
            select_folder,
            open_folder_in_explorer,
            investigate_record,
            search_countries,
            generate_next_project_number,
            validate_project_number,
            create_project_with_template,
            copy_project_template,
            get_area_suggestions,
            get_all_cities,
            get_city_suggestions,
            write_fee_to_json,
            write_fee_to_json_safe,
            check_project_folder_exists,
            check_var_json_exists,
            check_var_json_template_exists,
            rename_folder_with_old_suffix,
            rename_var_json_with_old_suffix,
            populate_project_data,
            get_project_folder_location,
            move_project_folder,
            move_project_from_rfp,
            move_project_to_archive,
            list_projects_in_folder,
            validate_project_base_path,
            // Folder sync commands
            scan_folder_sync,
            resolve_folder_inconsistency,
            log_message,
            // Log level control
            set_log_level,
            get_log_level,
            // Activity log commands
            create_activity_log,
            get_activity_logs,
            // Import wizard commands
            import_scan_directory,
            import_execute,
            // Excel export
            export_fee_excel,
            export_fee_template,
            // Batch operations
            batch_delete_entities,
            batch_update_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}