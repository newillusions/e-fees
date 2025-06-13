use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{Error, Surreal};
use tokio::time::interval;
use log::{error, info, warn};
use chrono;
use std::env;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

// Database configuration structure
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl DatabaseConfig {
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

// Connection status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub is_connected: bool,
    pub last_check: Option<String>,
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

// Database entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub status: String, // 'active', 'completed', 'on-hold'
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: Option<String>,
    pub name: String,
    pub industry: String,
    pub contact_count: i32,
    pub address: Option<String>,
    pub website: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Option<String>,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub position: Option<String>,
    pub company_id: String,
    pub company: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: Option<String>,
    pub title: String,
    pub client: String,
    pub amount: f64,
    pub status: String, // 'draft', 'sent', 'approved', 'rejected'
    pub project_id: Option<String>,
    pub created_at: String,
    pub due_date: Option<String>,
}

// Database manager
#[derive(Clone)]
pub struct DatabaseManager {
    pub client: Option<Surreal<Client>>,
    pub status: Arc<Mutex<ConnectionStatus>>,
    pub config: DatabaseConfig,
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
        
        let db = match Surreal::new::<Ws>(&self.config.url).await {
            Ok(connection) => {
                info!("Successfully established WebSocket connection to SurrealDB at {}", self.config.url);
                connection
            }
            Err(e) => {
                error!("Failed to establish WebSocket connection to {}: {}", self.config.url, e);
                return Err(e);
            }
        };
        
        // Sign in with provided credentials
        info!("Authenticating with username: {}", self.config.username);
        match db.signin(Root {
            username: &self.config.username,
            password: &self.config.password,
        }).await {
            Ok(_) => info!("Successfully authenticated with SurrealDB"),
            Err(e) => {
                error!("Failed to authenticate with SurrealDB: {}", e);
                return Err(e);
            }
        }
        
        // Select namespace and database
        info!("Selecting namespace '{}' and database '{}'", self.config.namespace, self.config.database);
        match db.use_ns(&self.config.namespace).use_db(&self.config.database).await {
            Ok(_) => info!("Successfully selected namespace and database"),
            Err(e) => {
                error!("Failed to select namespace/database: {}", e);
                return Err(e);
            }
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
            let projects: Vec<Project> = client.select("projects").await.unwrap_or_default();
            Ok(projects)
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Get all companies
    pub async fn get_companies(&self) -> Result<Vec<Company>, Error> {
        if let Some(client) = &self.client {
            let companies: Vec<Company> = client.select("companies").await.unwrap_or_default();
            Ok(companies)
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Get all contacts
    pub async fn get_contacts(&self) -> Result<Vec<Contact>, Error> {
        if let Some(client) = &self.client {
            let contacts: Vec<Contact> = client.select("contacts").await.unwrap_or_default();
            Ok(contacts)
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Get all proposals
    pub async fn get_proposals(&self) -> Result<Vec<Proposal>, Error> {
        if let Some(client) = &self.client {
            let proposals: Vec<Proposal> = client.select("proposals").await.unwrap_or_default();
            Ok(proposals)
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Create a new project
    pub async fn create_project(&self, project: Project) -> Result<Project, Error> {
        if let Some(client) = &self.client {
            let created: Option<Project> = client
                .create("projects")
                .content(project)
                .await?
                .into_iter()
                .next();
            
            created.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to create project".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Create a new company
    pub async fn create_company(&self, company: Company) -> Result<Company, Error> {
        if let Some(client) = &self.client {
            let created: Option<Company> = client
                .create("companies")
                .content(company)
                .await?
                .into_iter()
                .next();
            
            created.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to create company".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Create a new contact
    pub async fn create_contact(&self, contact: Contact) -> Result<Contact, Error> {
        if let Some(client) = &self.client {
            let created: Option<Contact> = client
                .create("contacts")
                .content(contact)
                .await?
                .into_iter()
                .next();
            
            created.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to create contact".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }

    // Create a new proposal
    pub async fn create_proposal(&self, proposal: Proposal) -> Result<Proposal, Error> {
        if let Some(client) = &self.client {
            let created: Option<Proposal> = client
                .create("proposals")
                .content(proposal)
                .await?
                .into_iter()
                .next();
            
            created.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("Failed to create proposal".to_string())))
        } else {
            Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidRequest("No database connection".to_string())))
        }
    }
}