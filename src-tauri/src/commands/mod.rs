use crate::db::{DatabaseManager, ConnectionStatus, Project, Company, Contact, Proposal};
use std::sync::{Arc, Mutex};
use tauri::State;
use log::{error, info};

// Application state that holds the database manager
pub type AppState = Arc<Mutex<DatabaseManager>>;

// Check database connection status
#[tauri::command]
pub async fn check_db_connection(state: State<'_, AppState>) -> Result<bool, String> {
    info!("Checking database connection");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    let is_connected = manager_clone.check_connection().await;
    Ok(is_connected)
}

// Get connection status with details
#[tauri::command]
pub async fn get_connection_status(state: State<'_, AppState>) -> Result<ConnectionStatus, String> {
    info!("Getting detailed connection status");
    
    let manager = state.lock().map_err(|e| e.to_string())?;
    let status = manager.get_status();
    Ok(status)
}

// Get all projects from database
#[tauri::command]
pub async fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    info!("Fetching projects from database");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.get_projects().await {
        Ok(projects) => {
            info!("Successfully fetched {} projects", projects.len());
            Ok(projects)
        }
        Err(e) => {
            error!("Failed to fetch projects: {}", e);
            Err(format!("Failed to fetch projects: {}", e))
        }
    }
}

// Get all companies from database
#[tauri::command]
pub async fn get_companies(state: State<'_, AppState>) -> Result<Vec<Company>, String> {
    info!("Fetching companies from database");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.get_companies().await {
        Ok(companies) => {
            info!("Successfully fetched {} companies", companies.len());
            Ok(companies)
        }
        Err(e) => {
            error!("Failed to fetch companies: {}", e);
            Err(format!("Failed to fetch companies: {}", e))
        }
    }
}

// Get all contacts from database
#[tauri::command]
pub async fn get_contacts(state: State<'_, AppState>) -> Result<Vec<Contact>, String> {
    info!("Fetching contacts from database");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.get_contacts().await {
        Ok(contacts) => {
            info!("Successfully fetched {} contacts", contacts.len());
            Ok(contacts)
        }
        Err(e) => {
            error!("Failed to fetch contacts: {}", e);
            Err(format!("Failed to fetch contacts: {}", e))
        }
    }
}

// Get all proposals from database
#[tauri::command]
pub async fn get_proposals(state: State<'_, AppState>) -> Result<Vec<Proposal>, String> {
    info!("Fetching proposals from database");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.get_proposals().await {
        Ok(proposals) => {
            info!("Successfully fetched {} proposals", proposals.len());
            Ok(proposals)
        }
        Err(e) => {
            error!("Failed to fetch proposals: {}", e);
            Err(format!("Failed to fetch proposals: {}", e))
        }
    }
}

// Create a new project
#[tauri::command]
pub async fn create_project(project: Project, state: State<'_, AppState>) -> Result<Project, String> {
    info!("Creating new project: {}", project.name);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.create_project(project).await {
        Ok(created_project) => {
            info!("Successfully created project with id: {:?}", created_project.id);
            Ok(created_project)
        }
        Err(e) => {
            error!("Failed to create project: {}", e);
            Err(format!("Failed to create project: {}", e))
        }
    }
}

// Create a new company
#[tauri::command]
pub async fn create_company(company: Company, state: State<'_, AppState>) -> Result<Company, String> {
    info!("Creating new company: {}", company.name);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.create_company(company).await {
        Ok(created_company) => {
            info!("Successfully created company with id: {:?}", created_company.id);
            Ok(created_company)
        }
        Err(e) => {
            error!("Failed to create company: {}", e);
            Err(format!("Failed to create company: {}", e))
        }
    }
}

// Create a new contact
#[tauri::command]
pub async fn create_contact(contact: Contact, state: State<'_, AppState>) -> Result<Contact, String> {
    info!("Creating new contact: {}", contact.name);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.create_contact(contact).await {
        Ok(created_contact) => {
            info!("Successfully created contact with id: {:?}", created_contact.id);
            Ok(created_contact)
        }
        Err(e) => {
            error!("Failed to create contact: {}", e);
            Err(format!("Failed to create contact: {}", e))
        }
    }
}

// Create a new proposal
#[tauri::command]
pub async fn create_proposal(proposal: Proposal, state: State<'_, AppState>) -> Result<Proposal, String> {
    info!("Creating new proposal: {}", proposal.title);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    match manager_clone.create_proposal(proposal).await {
        Ok(created_proposal) => {
            info!("Successfully created proposal with id: {:?}", created_proposal.id);
            Ok(created_proposal)
        }
        Err(e) => {
            error!("Failed to create proposal: {}", e);
            Err(format!("Failed to create proposal: {}", e))
        }
    }
}

// Health check command
#[tauri::command]
pub async fn health_check() -> Result<String, String> {
    info!("Application health check");
    Ok("Application is running".to_string())
}

// Get detailed database connection info for debugging
#[tauri::command]
pub async fn get_db_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Getting database connection information");
    
    let (connection_status, config) = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        (manager.get_status(), manager.config.clone())
    };
    
    let info = serde_json::json!({
        "url": config.url,
        "namespace": config.namespace,
        "database": config.database,
        "username": config.username,
        "connection_status": connection_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "troubleshooting": {
            "dns_issue": format!("If you see 'No such host is known', the server {} cannot be resolved", config.url),
            "network_check": format!("Try pinging the server: ping {}", config.url.replace("ws://", "").replace(":8000", "")),
            "surrealdb_check": format!("Verify SurrealDB is running: telnet {} 8000", config.url.replace("ws://", "").replace(":8000", "")),
            "config_source": "Configuration loaded from .env file or environment variables"
        }
    });
    
    Ok(info)
}

// Get application statistics
#[tauri::command]
pub async fn get_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Fetching application statistics");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    let projects = manager_clone.get_projects().await.unwrap_or_default();
    let companies = manager_clone.get_companies().await.unwrap_or_default();
    let contacts = manager_clone.get_contacts().await.unwrap_or_default();
    let proposals = manager_clone.get_proposals().await.unwrap_or_default();
    
    let active_proposals = proposals.iter()
        .filter(|p| p.status != "rejected")
        .count();
    
    let stats = serde_json::json!({
        "totalProjects": projects.len(),
        "activeProposals": active_proposals,
        "totalCompanies": companies.len(),
        "totalContacts": contacts.len(),
        "totalProposals": proposals.len()
    });
    
    info!("Successfully calculated statistics");
    Ok(stats)
}