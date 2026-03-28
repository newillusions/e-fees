//! Database configuration and connection status types.

use log::info;
use serde::{Deserialize, Serialize};
use std::env;

/// Interval for database connection health checks (30 seconds)
pub const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Database configuration structure loaded from environment variables or settings.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub verify_certificates: bool,
    pub accept_invalid_hostnames: bool,
}

impl DatabaseConfig {
    /// Creates a new DatabaseConfig from environment variables.
    pub fn from_env() -> Result<Self, String> {
        let url = env::var("SURREALDB_URL").map_err(|_| {
            "SURREALDB_URL environment variable is required but not set".to_string()
        })?;

        let verify_certificates = env::var("SURREALDB_VERIFY_CERTS")
            .map(|v| v.parse().unwrap_or(true))
            .unwrap_or(true);

        let accept_invalid_hostnames = env::var("SURREALDB_ACCEPT_INVALID_HOSTNAMES")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false);

        let namespace = env::var("SURREALDB_NS")
            .map_err(|_| "SURREALDB_NS environment variable is required but not set".to_string())?;
        let database = env::var("SURREALDB_DB")
            .map_err(|_| "SURREALDB_DB environment variable is required but not set".to_string())?;
        let username = env::var("SURREALDB_USER").map_err(|_| {
            "SURREALDB_USER environment variable is required but not set".to_string()
        })?;
        let password = env::var("SURREALDB_PASS").map_err(|_| {
            "SURREALDB_PASS environment variable is required but not set".to_string()
        })?;

        Ok(DatabaseConfig {
            url,
            namespace,
            database,
            username,
            password,
            verify_certificates,
            accept_invalid_hostnames,
        })
    }

    /// Creates a DatabaseConfig from AppSettings (for production builds).
    pub fn from_settings(settings: &crate::commands::AppSettings) -> Result<Self, String> {
        let url = settings
            .surrealdb_url
            .as_ref()
            .ok_or("SurrealDB URL not configured in settings".to_string())?
            .clone();

        let namespace = settings
            .surrealdb_ns
            .as_ref()
            .ok_or("SurrealDB namespace not configured in settings".to_string())?
            .clone();

        let database = settings
            .surrealdb_db
            .as_ref()
            .ok_or("SurrealDB database not configured in settings".to_string())?
            .clone();

        let username = settings
            .surrealdb_user
            .as_ref()
            .ok_or("SurrealDB username not configured in settings".to_string())?
            .clone();

        let password = settings
            .surrealdb_pass
            .as_ref()
            .ok_or("SurrealDB password not configured in settings".to_string())?
            .clone();

        Ok(DatabaseConfig {
            url,
            namespace,
            database,
            username,
            password,
            verify_certificates: true,
            accept_invalid_hostnames: false,
        })
    }

    /// Creates an empty/unconfigured DatabaseConfig.
    pub fn unconfigured() -> Self {
        Self {
            url: String::new(),
            namespace: String::new(),
            database: String::new(),
            username: String::new(),
            password: String::new(),
            verify_certificates: true,
            accept_invalid_hostnames: false,
        }
    }

    /// Logs the configuration (without password).
    pub fn log_info(&self) {
        info!(
            "Database config - URL: {}, NS: {}, DB: {}, User: {}",
            self.url, self.namespace, self.database, self.username
        );
    }

    /// Checks if all required fields are populated.
    pub fn is_configured(&self) -> bool {
        !self.url.is_empty()
            && !self.namespace.is_empty()
            && !self.database.is_empty()
            && !self.username.is_empty()
            && !self.password.is_empty()
    }
}

/// Connection status tracking structure for real-time monitoring.
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

impl ConnectionStatus {
    /// Creates a status indicating database is not configured.
    pub fn not_configured() -> Self {
        Self {
            is_connected: false,
            last_check: None,
            error_message: Some("Database not configured. Please use the settings to configure database connection.".to_string()),
        }
    }

    /// Updates the status with connection result.
    pub fn update(&mut self, is_connected: bool, error_message: Option<String>) {
        self.is_connected = is_connected;
        self.last_check = Some(chrono::Utc::now().to_rfc3339());
        self.error_message = error_message;
    }
}
