use std::env;

/// API server configuration loaded from environment variables.
pub struct Config {
    pub surreal_url: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    pub api_key: String,
    pub port: u16,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Required: SURREAL_URL, SURREAL_USER, SURREAL_PASS, API_KEY
    /// Optional: SURREAL_NS (default: emittiv), SURREAL_DB (default: projects), API_PORT (default: 3200)
    pub fn from_env() -> Self {
        Self {
            surreal_url: env::var("SURREAL_URL").expect("SURREAL_URL required"),
            surreal_ns: env::var("SURREAL_NS").unwrap_or_else(|_| "emittiv".into()),
            surreal_db: env::var("SURREAL_DB").unwrap_or_else(|_| "projects".into()),
            surreal_user: env::var("SURREAL_USER").expect("SURREAL_USER required"),
            surreal_pass: env::var("SURREAL_PASS").expect("SURREAL_PASS required"),
            api_key: env::var("API_KEY").expect("API_KEY required"),
            port: env::var("API_PORT")
                .unwrap_or_else(|_| "3200".into())
                .parse()
                .expect("Invalid API_PORT"),
        }
    }
}
