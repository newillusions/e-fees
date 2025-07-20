use std::sync::{Arc, Mutex};
use log::{info, error};
use tauri::Manager;

mod db;
mod commands;

use db::DatabaseManager;
use commands::{
    check_db_connection,
    get_connection_status,
    get_projects,
    search_projects,
    get_companies,
    get_contacts,
    get_rfps,
    create_project,
    update_project,
    delete_project,
    create_company,
    update_company,
    delete_company,
    create_contact,
    update_contact,
    create_rfp,
    update_rfp,
    delete_rfp,
    health_check,
    get_stats,
    get_db_info,
    get_table_schema,
    position_window_4k,
    get_settings,
    save_settings,
    select_folder,
    open_folder_in_explorer,
    investigate_record,
    search_countries,
    generate_next_project_number,
    validate_project_number,
    create_project_with_template,
    get_area_suggestions,
    get_all_cities,
    get_city_suggestions,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Setup logging
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            info!("Initializing Fee Proposal Management Application");
            
            // Load environment variables
            if let Err(e) = dotenvy::dotenv() {
                info!("No .env file found or error loading it: {}", e);
                info!("Using default configuration or environment variables");
            }

            // Initialize database manager
            let db_manager = match DatabaseManager::new() {
                Ok(manager) => manager,
                Err(e) => {
                    error!("Failed to initialize database manager: {}", e);
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Database configuration error: {}", e)
                    )));
                }
            };
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
            
            tauri::async_runtime::spawn(async move {
                info!("Starting database initialization");
                
                // Clone the manager and initialize it
                let mut manager_clone = {
                    if let Ok(manager) = init_state.lock() {
                        manager.clone()
                    } else {
                        error!("Failed to acquire database manager lock during initialization");
                        return;
                    }
                };
                
                // Initialize database with the cloned manager
                let initialized = match manager_clone.initialize().await {
                    Ok(_) => {
                        info!("Database initialized successfully");
                        // Update the original manager in the state
                        if let Ok(mut manager) = init_state.lock() {
                            *manager = manager_clone;
                        }
                        true
                    }
                    Err(e) => {
                        error!("Failed to initialize database: {}", e);
                        false
                    }
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
            get_rfps,
            create_project,
            update_project,
            delete_project,
            create_company,
            update_company,
            delete_company,
            create_contact,
            update_contact,
            create_rfp,
            update_rfp,
            delete_rfp,
            health_check,
            get_stats,
            get_db_info,
            get_table_schema,
            position_window_4k,
            get_settings,
            save_settings,
            select_folder,
            open_folder_in_explorer,
            investigate_record,
            search_countries,
            generate_next_project_number,
            validate_project_number,
            create_project_with_template,
            get_area_suggestions,
            get_all_cities,
            get_city_suggestions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}