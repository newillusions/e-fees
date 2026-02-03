//! # Database Operations Module
//!
//! This module provides comprehensive database connectivity and operations for the
//! Fee Proposal Management System using SurrealDB as the backend database.

pub mod config;
pub mod client;
pub mod types;
pub mod utils;
pub mod security;
pub mod secure_operations;
pub mod operations;
#[cfg(test)]
mod tests;

// Re-export public types
pub use config::{DatabaseConfig, ConnectionStatus, HEARTBEAT_INTERVAL_SECS};
pub use client::DatabaseClient;
pub use types::{
    Project, ProjectNumber, TimeStamps, NewProject,
    Company, CompanyCreate,
    Contact, ContactCreate,
    Fee, FeeCreate, FeeUpdate, Revision, PricingUpdate,
    PaginatedResponse, EntityCounts,
    ActivityLog, ActivityLogCreate,
};

use std::sync::Arc;
use std::time::Duration;
use surrealdb::Error;
use tokio::sync::RwLock;
use tokio::time::interval;
use log::{error, info, warn};

/// Database manager handling connection lifecycle and operations.
#[derive(Clone)]
pub struct DatabaseManager {
    pub client: Option<DatabaseClient>,
    pub status: Arc<RwLock<ConnectionStatus>>,
    pub config: DatabaseConfig,
}

impl DatabaseManager {
    pub fn new() -> Result<Self, String> {
        let config = DatabaseConfig::from_env()?;
        config.log_info();

        Ok(Self {
            client: None,
            status: Arc::new(RwLock::new(ConnectionStatus::default())),
            config,
        })
    }

    pub fn new_unconfigured() -> Self {
        Self {
            client: None,
            status: Arc::new(RwLock::new(ConnectionStatus::not_configured())),
            config: DatabaseConfig::unconfigured(),
        }
    }

    pub fn from_config(config: DatabaseConfig) -> Self {
        config.log_info();
        Self {
            client: None,
            status: Arc::new(RwLock::new(ConnectionStatus::default())),
            config,
        }
    }

    pub async fn reconfigure(&mut self, url: String, namespace: String, database: String, username: String, password: String) -> Result<(), String> {
        if url.is_empty() || namespace.is_empty() || database.is_empty() || username.is_empty() || password.is_empty() {
            return Err("All database configuration fields are required".to_string());
        }

        self.config = DatabaseConfig {
            url,
            namespace,
            database,
            username,
            password,
            verify_certificates: true,
            accept_invalid_hostnames: false,
        };

        self.client = None;
        self.update_status(false, Some("Database reconfigured. Connection will be attempted automatically.".to_string())).await;
        self.config.log_info();

        Ok(())
    }

    /// Force a reconnection to the database.
    /// This closes any existing connection and establishes a new one.
    pub async fn reconnect(&mut self) -> Result<(), String> {
        info!("Forcing database reconnection");

        // Close existing connection
        self.client = None;
        self.update_status(false, Some("Reconnecting...".to_string())).await;

        // Attempt to reconnect
        match self.initialize().await {
            Ok(_) => {
                info!("Database reconnection successful");
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Reconnection failed: {}", e);
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    pub async fn initialize(&mut self) -> Result<(), Error> {
        info!("Initializing database connection to {}", self.config.url);

        match self.connect().await {
            Ok(_) => {
                info!("Database connection established successfully");
                if let Some(client) = &self.client {
                    match client.health().await {
                        Ok(_) => {
                            info!("Database health check passed");
                            self.update_status(true, None).await;
                        }
                        Err(e) => {
                            warn!("Database health check failed: {}", e);
                            self.update_status(false, Some(format!("Health check failed: {}", e))).await;
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to establish database connection: {}", e);
                error!("{}", error_msg);

                let user_friendly_error = self.get_user_friendly_error(&e);
                self.update_status(false, Some(user_friendly_error.clone())).await;
                Err(e)
            }
        }
    }

    fn get_user_friendly_error(&self, e: &Error) -> String {
        let error_str = e.to_string();
        if error_str.contains("No such host is known") {
            format!("Cannot resolve hostname. Please check if the SurrealDB server is accessible.")
        } else if error_str.contains("Connection refused") {
            format!("Connection refused by SurrealDB server at {}. Please check if SurrealDB is running.", self.config.url)
        } else if error_str.contains("Authentication failed") {
            format!("Authentication failed. Please check username '{}' and password are correct.", self.config.username)
        } else if error_str.contains("Namespace") || error_str.contains("Database") {
            format!("Failed to select namespace '{}' or database '{}'. Please check if they exist.", self.config.namespace, self.config.database)
        } else {
            format!("Failed to establish database connection: {}", e)
        }
    }

    async fn connect(&mut self) -> Result<(), Error> {
        info!("Attempting to connect to SurrealDB at {}", self.config.url);

        let db = DatabaseClient::connect(&self.config.url).await?;

        // Try different authentication methods
        info!("Authenticating with username: {}", self.config.username);

        let db_auth_result = db.signin_database(
            &self.config.namespace,
            &self.config.database,
            &self.config.username,
            &self.config.password
        ).await;

        if let Err(db_err) = db_auth_result {
            warn!("Database authentication failed: {}, trying namespace authentication", db_err);

            let ns_auth_result = db.signin_namespace(
                &self.config.namespace,
                &self.config.username,
                &self.config.password
            ).await;

            if let Err(ns_err) = ns_auth_result {
                warn!("Namespace authentication failed: {}, trying root authentication", ns_err);

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
        db.use_ns_db(&self.config.namespace, &self.config.database).await?;

        // Debug: Check permissions
        self.check_database_info(&db).await;

        self.client = Some(db);
        info!("SurrealDB connection fully established and ready");
        Ok(())
    }

    async fn check_database_info(&self, db: &DatabaseClient) {
        info!("Testing authentication and permissions...");
        match db.query("INFO FOR DB").await {
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
    }

    pub async fn check_connection(&self) -> bool {
        if let Some(client) = &self.client {
            match client.health().await {
                Ok(_) => true,
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

    async fn update_status(&self, is_connected: bool, error_message: Option<String>) {
        let mut status = self.status.write().await;
        status.update(is_connected, error_message);
    }

    pub async fn get_status(&self) -> ConnectionStatus {
        self.status.read().await.clone()
    }

    pub async fn start_heartbeat(status: Arc<RwLock<ConnectionStatus>>, manager: Arc<RwLock<DatabaseManager>>) {
        let mut interval = interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

        tauri::async_runtime::spawn(async move {
            loop {
                interval.tick().await;

                let manager_clone = {
                    manager.read().await.clone()
                };

                let is_connected = manager_clone.check_connection().await;

                {
                    let mut status_guard = status.write().await;
                    status_guard.is_connected = is_connected;
                    status_guard.last_check = Some(chrono::Utc::now().to_rfc3339());
                    if !is_connected && status_guard.error_message.is_none() {
                        status_guard.error_message = Some("Heartbeat check failed".to_string());
                    } else if is_connected {
                        status_guard.error_message = None;
                    }
                }
            }
        });
    }

    // ==================== Helper Methods ====================
    // Note: Entity operations (projects, companies, contacts, fees, reference data,
    // activity logs, statistics, utilities) are in db/operations.rs

    /// Get a reference to the database client, returning an error if not connected.
    /// This is the safe way to access the client without using .unwrap().
    fn get_client(&self) -> Result<&DatabaseClient, Error> {
        self.client.as_ref().ok_or_else(|| self.invalid_request_error("No database connection"))
    }

    fn invalid_request_error(&self, message: &str) -> Error {
        Error::Api(surrealdb::error::Api::InvalidRequest(message.to_string()))
    }

    fn not_found_error(&self, operation: &str) -> Error {
        Error::Api(surrealdb::error::Api::InvalidRequest(format!("Failed to {}", operation)))
    }

    async fn paginate<T>(&self, table: &str, page: usize, page_size: usize) -> Result<PaginatedResponse<T>, Error>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        let client = self.get_client()?;

        let offset = (page - 1) * page_size;
        info!("Fetching {} page {} (offset: {}, limit: {})", table, page, offset, page_size);

        // Execute count and data fetch in a single query for better performance
        // Statement 0: count, Statement 1: paginated data
        let combined_query = format!(
            "SELECT count() FROM {} GROUP ALL; SELECT * FROM {} ORDER BY time.created_at DESC LIMIT {} START {}",
            table, table, page_size, offset
        );
        let mut response = client.query(&combined_query).await?;

        // Extract count from statement 0
        let count_result: Option<serde_json::Value> = response.take(0)?;
        let total = count_result.and_then(|v| v.get("count").and_then(|c| c.as_u64())).unwrap_or(0) as usize;

        // Extract items from statement 1
        let items: Vec<T> = response.take(1)?;

        info!("Fetched {} {} for page {} (total: {})", items.len(), table, page, total);
        Ok(PaginatedResponse::new(items, total, page, page_size))
    }

    async fn get_by_id<T>(&self, table: &str, id: &str) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let client = self.get_client()?;

        info!("Fetching {} by ID: {}", table, id);
        let query = format!("SELECT * FROM {}:{}", table, id);
        let mut response = client.query(&query).await?;
        let items: Vec<T> = response.take(0)?;
        Ok(items.into_iter().next())
    }
}
