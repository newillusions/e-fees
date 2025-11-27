# Production Database Connection Fix

## Issue Summary
The Fee Proposal Management app cannot connect to the database in production builds while development mode works perfectly.

## Root Cause
The application uses two different environment variable loading mechanisms:
1. **Development**: `dotenvy::dotenv()` loads from project root `.env` 
2. **Production**: App should use settings system that loads from `~/Library/Application Support/com.emittiv.fee-proposal-manager/.env`

However, during database initialization, both modes try to use `dotenvy::dotenv()` which only works in development.

## Technical Analysis

### Working in Development
- Current directory: `/Volumes/base/dev/claude-code-app-1/`
- `dotenvy::dotenv()` finds `.env` in project root
- Database connection established successfully

### Failing in Production
- Current directory: `/path/to/app.app/Contents/MacOS/`
- `dotenvy::dotenv()` cannot find `.env` (doesn't exist in bundle)
- `DatabaseManager::new()` fails with "environment variables not available"
- App falls back to unconfigured mode

### App Configuration System
- Settings properly stored in: `~/Library/Application Support/com.emittiv.fee-proposal-manager/.env`
- `get_settings()` command can read this file correctly
- Settings system works, but isn't used during initial database setup

## The Fix

### 1. Modify Database Initialization (Recommended)

Update `src-tauri/src/lib.rs` to use the settings system instead of environment variables for production:

```rust
// In the setup function, replace the database initialization section:

// Load database configuration using the settings system for production
let db_manager = if cfg!(debug_assertions) {
    // Development: use dotenvy for .env loading
    if let Err(e) = dotenvy::dotenv() {
        info!("No .env file found or error loading it: {}", e);
    }
    
    match DatabaseManager::new() {
        Ok(manager) => {
            info!("Database manager initialized with environment configuration");
            manager
        },
        Err(e) => {
            info!("Environment variables not available ({}), using unconfigured database manager", e);
            DatabaseManager::new_unconfigured()
        }
    }
} else {
    // Production: use settings system
    match load_database_config_from_settings(&app.handle()).await {
        Ok(config) => {
            info!("Database manager initialized with settings configuration");
            DatabaseManager::from_config(config)
        },
        Err(e) => {
            info!("Settings not available ({}), using unconfigured database manager", e);
            DatabaseManager::new_unconfigured()
        }
    }
};
```

### 2. Add Helper Function

Add to `src-tauri/src/db/mod.rs`:

```rust
impl DatabaseManager {
    /// Create a new DatabaseManager from a DatabaseConfig
    pub fn from_config(config: DatabaseConfig) -> Self {
        info!("Database config loaded from settings - URL: {}, NS: {}, DB: {}, User: {}", 
              config.url, config.namespace, config.database, config.username);
        
        Self {
            client: None,
            status: Arc::new(Mutex::new(ConnectionStatus::default())),
            config,
        }
    }
}

/// Load database configuration from the app settings system
pub async fn load_database_config_from_settings(app_handle: &AppHandle) -> Result<DatabaseConfig, String> {
    // Read settings using the existing get_settings command logic
    let env_path = if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        app_data_dir.join(".env")
    } else {
        return Err("Cannot determine app data directory".to_string());
    };
    
    if !env_path.exists() {
        return Err("Settings file not found".to_string());
    }
    
    let content = std::fs::read_to_string(&env_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    
    let mut url = None;
    let mut namespace = None;
    let mut database = None;
    let mut username = None;
    let mut password = None;
    
    // Parse .env file
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            
            match key {
                "SURREALDB_URL" => url = Some(value.to_string()),
                "SURREALDB_NS" => namespace = Some(value.to_string()),
                "SURREALDB_DB" => database = Some(value.to_string()),
                "SURREALDB_USER" => username = Some(value.to_string()),
                "SURREALDB_PASS" => password = Some(value.to_string()),
                _ => {}
            }
        }
    }
    
    Ok(DatabaseConfig {
        url: url.ok_or("SURREALDB_URL not found in settings")?,
        namespace: namespace.ok_or("SURREALDB_NS not found in settings")?,
        database: database.ok_or("SURREALDB_DB not found in settings")?,
        username: username.ok_or("SURREALDB_USER not found in settings")?,
        password: password.ok_or("SURREALDB_PASS not found in settings")?,
        verify_certificates: true,
        accept_invalid_hostnames: false,
    })
}
```

### 3. Quick Test Fix (Alternative)

For immediate testing, copy the settings file to where dotenvy expects it:

```bash
# Copy app data .env to project root before building
cp ~/Library/Application\ Support/com.emittiv.fee-proposal-manager/.env /Volumes/base/dev/claude-code-app-1/src-tauri/.env

# Rebuild
npm run tauri:build
```

## Verification

### Tauri Configuration ✅
- Network permissions: `com.apple.security.network.client: true`
- WebSocket support: CSP allows `ws:` and `wss:`
- No sandbox restrictions blocking connections

### Environment Variables ✅
- Settings file exists and contains valid database credentials
- App data directory: `~/Library/Application Support/com.emittiv.fee-proposal-manager/.env`
- Format is correct with proper SurrealDB configuration

### Database Connection Code ✅  
- WebSocket connection handling is robust
- Proper error handling and fallback mechanisms
- Multiple authentication level support (root/namespace/database)

## Recommended Implementation Order

1. **Immediate Fix**: Copy settings to bundle directory for testing
2. **Proper Fix**: Implement settings-based database initialization
3. **Long-term**: Consider bundling default config or FirstRunSetup integration

This fix will enable the production app to connect to the database using the same configuration that works in the settings system.