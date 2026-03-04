use std::env;

/// API server configuration loaded from environment variables.
pub struct Config {
    pub surreal_url: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    /// Comma-separated list of valid API keys. A single key with no commas works unchanged.
    pub api_keys: Vec<String>,
    pub port: u16,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Required: SURREAL_URL, SURREAL_USER, SURREAL_PASS, API_KEY
    /// Optional: SURREAL_NS (default: emittiv), SURREAL_DB (default: projects), API_PORT (default: 3200)
    ///
    /// API_KEY supports comma-separated values for multiple keys.
    pub fn from_env() -> Self {
        let raw_keys = env::var("API_KEY").expect("API_KEY required");
        let api_keys: Vec<String> = raw_keys
            .split(',')
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();
        assert!(!api_keys.is_empty(), "API_KEY must contain at least one non-empty key");

        Self {
            surreal_url: env::var("SURREAL_URL").expect("SURREAL_URL required"),
            surreal_ns: env::var("SURREAL_NS").unwrap_or_else(|_| "emittiv".into()),
            surreal_db: env::var("SURREAL_DB").unwrap_or_else(|_| "projects".into()),
            surreal_user: env::var("SURREAL_USER").expect("SURREAL_USER required"),
            surreal_pass: env::var("SURREAL_PASS").expect("SURREAL_PASS required"),
            api_keys,
            port: env::var("API_PORT")
                .unwrap_or_else(|_| "3200".into())
                .parse()
                .expect("Invalid API_PORT"),
        }
    }
}
