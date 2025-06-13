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
    get_companies,
    get_contacts,
    get_proposals,
    create_project,
    create_company,
    create_contact,
    create_proposal,
    health_check,
    get_stats,
    get_db_info,
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
        .invoke_handler(tauri::generate_handler![
            check_db_connection,
            get_connection_status,
            get_projects,
            get_companies,
            get_contacts,
            get_proposals,
            create_project,
            create_company,
            create_contact,
            create_proposal,
            health_check,
            get_stats,
            get_db_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}