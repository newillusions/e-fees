use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use emittiv_container_utils::config_manager::ConfigManager;

/// Nextcloud folder creation config (optional — only if NC_SSH_HOST is set).
pub struct FolderConfig {
    pub ssh_host: String,
    pub ssh_user: String,
    pub ssh_key: String,
    pub script_path: String,
    pub nc_base_path: String,
    /// UID:GID for Nextcloud file ownership (default: "99:100" = nobody:users on Unraid).
    pub nc_owner: String,
}

/// API server configuration. Non-secret values come from config.yaml via ConfigManager;
/// secrets (credentials, API keys) come from environment variables.
pub struct Config {
    pub surreal_url: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    /// Comma-separated list of valid API keys. A single key with no commas works unchanged.
    pub api_keys: Vec<String>,
    pub port: u16,
    pub host: String,
    /// Optional folder creation config (None if NC_SSH_HOST not set).
    pub folder_config: Option<FolderConfig>,
    /// ConfigManager for hot-reloadable non-secret config.
    pub config_manager: Arc<ConfigManager>,
}

impl Config {
    /// Load configuration from config.yaml (non-secrets) and environment variables (secrets).
    ///
    /// config.yaml keys: server.port, server.host, database.url, database.namespace,
    ///                   database.database, log_level
    ///
    /// Required env vars: SURREAL_USER, SURREAL_PASS, API_KEY
    /// Optional env vars: NC_SSH_HOST (enables folder creation), API_PORT (overrides config.yaml port)
    ///
    /// API_KEY supports comma-separated values for multiple keys.
    pub fn load() -> Self {
        // Build defaults matching config.yaml structure
        let mut defaults: HashMap<String, serde_yaml::Value> = HashMap::new();
        defaults.insert("server.port".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(3200u64)));
        defaults.insert("server.host".to_string(), serde_yaml::Value::String("0.0.0.0".to_string()));
        defaults.insert("database.url".to_string(), serde_yaml::Value::String("ws://10.0.23.11:8000".to_string()));
        defaults.insert("database.namespace".to_string(), serde_yaml::Value::String("emittiv".to_string()));
        defaults.insert("database.database".to_string(), serde_yaml::Value::String("projects".to_string()));
        defaults.insert("log_level".to_string(), serde_yaml::Value::String("info".to_string()));

        // Look for config.yaml in /data/config.yaml (Docker) or ./config.yaml (local dev)
        let config_path = if std::path::Path::new("/data/config.yaml").exists() {
            "/data/config.yaml".to_string()
        } else {
            "config.yaml".to_string()
        };

        let cm = Arc::new(ConfigManager::new(
            config_path,
            defaults,
            HashMap::new(), // no schema validation for now
            60,             // poll every 60s
        ));
        cm.start_watching();

        // Non-secret config from ConfigManager
        let port: u16 = env::var("API_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                cm.get_i64("server.port")
                    .map(|p| p as u16)
                    .unwrap_or(3200)
            });

        let host = cm.get_str("server.host").unwrap_or_else(|| "0.0.0.0".to_string());

        let surreal_url = env::var("SURREAL_URL")
            .unwrap_or_else(|_| cm.get_str("database.url").unwrap_or_else(|| "ws://10.0.23.11:8000".to_string()));

        let surreal_ns = env::var("SURREAL_NS")
            .unwrap_or_else(|_| cm.get_str("database.namespace").unwrap_or_else(|| "emittiv".to_string()));

        let surreal_db = env::var("SURREAL_DB")
            .unwrap_or_else(|_| cm.get_str("database.database").unwrap_or_else(|| "projects".to_string()));

        // Secrets from env vars only
        let surreal_user = env::var("SURREAL_USER").expect("SURREAL_USER required");
        let surreal_pass = env::var("SURREAL_PASS").expect("SURREAL_PASS required");

        let raw_keys = env::var("API_KEY").expect("API_KEY required");
        let api_keys: Vec<String> = raw_keys
            .split(',')
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();
        assert!(!api_keys.is_empty(), "API_KEY must contain at least one non-empty key");

        // Folder config from env vars (infrastructure-specific, not in config.yaml)
        let folder_config = env::var("NC_SSH_HOST").ok().map(|ssh_host| FolderConfig {
            ssh_host,
            ssh_user: env::var("NC_SSH_USER").unwrap_or_else(|_| "root".into()),
            ssh_key: env::var("NC_SSH_KEY")
                .unwrap_or_else(|_| "/root/.ssh/id_ed25519".into()),
            script_path: env::var("NC_SCRIPT_PATH")
                .unwrap_or_else(|_| "/mnt/user/appdata/scripts/nc-project-create.sh".into()),
            nc_base_path: env::var("NC_BASE_PATH")
                .unwrap_or_else(|_| "/mnt/user/emittiv/nc/__groupfolders/1/01 Projects".into()),
            nc_owner: env::var("NC_OWNER").unwrap_or_else(|_| "99:100".into()),
        });

        Self {
            surreal_url,
            surreal_ns,
            surreal_db,
            surreal_user,
            surreal_pass,
            api_keys,
            port,
            host,
            folder_config,
            config_manager: cm,
        }
    }
}
