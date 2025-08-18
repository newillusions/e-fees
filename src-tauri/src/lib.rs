use std::sync::{Arc, Mutex};
use log::{info, error};
use tauri::Manager;

mod db;
mod commands;

use db::{DatabaseManager, DatabaseConfig};
use commands::{
    check_db_connection,
    get_connection_status,
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
    delete_fee,
    health_check,
    get_stats,
    get_db_info,
    get_table_schema,
    position_window_4k,
    get_settings,
    save_settings,
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
    match commands::get_settings(app_handle.clone()).await {
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
        .setup(|app| {
            // Setup logging
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
            )?;
            
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
            let app_state = Arc::new(Mutex::new(db_manager));
            
            // Clone state for heartbeat monitoring
            let heartbeat_state = app_state.clone();
            let status = {
                if let Ok(manager) = app_state.lock() {
                    manager.status.clone()
                } else {
                    error!("Failed to get status from database manager");
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to initialize database manager"
                    )));
                }
            };
            
            // Set up the application state
            app.manage(app_state.clone());
            
            // Position window on right half of screen after a slight delay - DISABLED for new environment
            // let window_handle = app.handle().clone();
            // tauri::async_runtime::spawn(async move {
            //     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            //     if let Some(window) = window_handle.get_webview_window("main") {
            //         // Use logical position for proper scaling support
            //         // At 150% scaling: 2560x1440 logical pixels
            //         let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x: 1280.0, y: 50.0 }));
            //         // Also ensure the size is correct using logical size
            //         let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width: 1280.0, height: 1200.0 }));
            //         info!("Window positioned on right half of screen with scaling support");
            //     }
            // });
            
            // Initialize database connection in async context using Tauri's runtime
            let init_state = app_state.clone();
            let app_handle_clone = app.handle().clone();
            
            tauri::async_runtime::spawn(async move {
                info!("Starting database initialization");
                
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
                            // Update the original manager in the state
                            if let Ok(mut state_manager) = init_state.lock() {
                                *state_manager = manager;
                            }
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
                }
            });
            
            info!("Application setup completed successfully");
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            check_db_connection,
            get_connection_status,
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
            delete_fee,
            health_check,
            get_stats,
            get_db_info,
            get_table_schema,
            position_window_4k,
            get_settings,
            save_settings,
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
            validate_project_base_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}