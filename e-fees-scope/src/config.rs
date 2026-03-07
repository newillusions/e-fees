use std::env;

pub struct Config {
    pub surreal_url: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    pub api_keys: Vec<String>,
    pub port: u16,
    pub ollama_url: String,
    pub ollama_model: String,
    pub docling_url: String,
    pub corpus_path: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let raw_keys = env::var("API_KEY").expect("API_KEY required");
        let api_keys: Vec<String> = raw_keys
            .split(',')
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();
        assert!(
            !api_keys.is_empty(),
            "API_KEY must contain at least one non-empty key"
        );

        Self {
            surreal_url: env::var("SURREAL_URL").expect("SURREAL_URL required"),
            surreal_ns: env::var("SURREAL_NS").unwrap_or_else(|_| "emittiv".into()),
            surreal_db: env::var("SURREAL_DB").unwrap_or_else(|_| "projects".into()),
            surreal_user: env::var("SURREAL_USER").expect("SURREAL_USER required"),
            surreal_pass: env::var("SURREAL_PASS").expect("SURREAL_PASS required"),
            api_keys,
            port: env::var("API_PORT")
                .unwrap_or_else(|_| "3201".into())
                .parse()
                .expect("Invalid API_PORT"),
            ollama_url: env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://10.0.21.20:11434".into()),
            ollama_model: env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen3:4b".into()),
            docling_url: env::var("DOCLING_URL")
                .unwrap_or_else(|_| "http://10.0.21.42:5001".into()),
            corpus_path: env::var("CORPUS_PATH").ok(),
        }
    }
}
