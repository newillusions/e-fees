# Tauri v2 Development Guide for E-Fees Project

## Tech Stack
- **Tauri**: v2.x (latest stable)
- **Frontend**: Svelte 5
- **Backend**: Rust
- **Database**: SurrealDB

## Tauri v2 Key Differences from v1

### 1. Plugin System
**v1 (deprecated):**
```rust
use tauri::plugin::Plugin;
```

**v2 (current):**
```rust
use tauri::plugin::{Builder, TauriPlugin};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("plugin-name")
        .invoke_handler(tauri::generate_handler![command])
        .build()
}
```

### 2. Window Management
**v2 API:**
```rust
use tauri::{WebviewWindow, WebviewWindowBuilder};

// Create window
let window = WebviewWindowBuilder::new(
    app_handle,
    "main",
    WebviewUrl::App("index.html".into())
)
.title("E-Fees Manager")
.inner_size(1200.0, 800.0)
.build()?;

// Access window
let window = app.get_webview_window("main").unwrap();
```

### 3. IPC Communication

**Frontend (Svelte 5):**
```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Invoke command
const result = await invoke<string>('greet', { name: 'World' });

// Listen to events
const unlisten = await listen<string>('event-name', (event) => {
  console.log('Received:', event.payload);
});

// Cleanup
unlisten();
```

**Backend (Rust):**
```rust
use tauri::{Emitter, Manager};

#[tauri::command]
async fn greet(name: String) -> Result<String, String> {
    Ok(format!("Hello, {}!", name))
}

// Emit event
app_handle.emit("event-name", "payload")?;
```

### 4. State Management

**Global State:**
```rust
use tauri::{State, Manager};
use std::sync::Mutex;

struct AppState {
    db: Mutex<Option<Database>>,
}

#[tauri::command]
fn get_data(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    // Use db...
    Ok("data".to_string())
}

// In main
fn main() {
    tauri::Builder::default()
        .manage(AppState {
            db: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![get_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 5. File System Access

**v2 Scoped FS:**
```rust
use tauri::Manager;

#[tauri::command]
async fn read_file(
    app: tauri::AppHandle,
    path: String,
) -> Result<String, String> {
    let resource_path = app.path().resolve(path, BaseDirectory::Resource)?;
    std::fs::read_to_string(resource_path)
        .map_err(|e| e.to_string())
}
```

**Frontend:**
```typescript
import { BaseDirectory, readTextFile } from '@tauri-apps/plugin-fs';

const content = await readTextFile('data.json', {
  baseDir: BaseDirectory::Resource,
});
```

## Common Patterns for E-Fees

### Database Operations with SurrealDB

```rust
use surrealdb::{Surreal, engine::local::RocksDb};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Fee {
    id: String,
    amount: f64,
    description: String,
}

#[tauri::command]
async fn get_fees(
    state: State<'_, AppState>,
) -> Result<Vec<Fee>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("Database not initialized")?;
    
    let fees: Vec<Fee> = db
        .query("SELECT * FROM fees")
        .await
        .map_err(|e| e.to_string())?
        .take(0)
        .map_err(|e| e.to_string())?;
    
    Ok(fees)
}

#[tauri::command]
async fn create_fee(
    state: State<'_, AppState>,
    fee: Fee,
) -> Result<Fee, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("Database not initialized")?;
    
    let created: Option<Fee> = db
        .create("fees")
        .content(fee)
        .await
        .map_err(|e| e.to_string())?;
    
    created.ok_or("Failed to create fee".to_string())
}
```

### Error Handling Best Practices

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] surrealdb::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

#[tauri::command]
async fn safe_operation() -> Result<String, AppError> {
    // Operations that can fail
    Ok("success".to_string())
}
```

### Window Communication

```rust
// Backend
#[tauri::command]
async fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    let window = WebviewWindowBuilder::new(
        &app,
        "settings",
        WebviewUrl::App("settings.html".into())
    )
    .title("Settings")
    .inner_size(600.0, 400.0)
    .build()
    .map_err(|e| e.to_string())?;
    
    Ok(())
}

// Send data between windows
#[tauri::command]
async fn send_to_window(
    app: tauri::AppHandle,
    window_label: String,
    message: String,
) -> Result<(), String> {
    app.emit_to(&window_label, "message", message)
        .map_err(|e| e.to_string())
}
```

### Menu System (v2)

```rust
use tauri::menu::{Menu, MenuItem, Submenu};

fn create_menu(app: &tauri::App) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, Some("Cmd+,"))?;
    
    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[&settings, &quit],
    )?;
    
    Menu::with_items(app, &[&file_menu])
}

// Handle menu events
fn handle_menu_event(app: &tauri::AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "quit" => {
            app.exit(0);
        }
        "settings" => {
            // Open settings window
        }
        _ => {}
    }
}
```

## Development Workflow

### 1. Hot Reload Setup
```bash
# Start development server with hot reload
npm run tauri dev

# Frontend only (faster iteration)
npm run dev
```

### 2. Build Process
```bash
# Development build
cargo build

# Production build
npm run tauri build

# Build for specific target
npm run tauri build -- --target x86_64-apple-darwin
```

### 3. Debugging

**Frontend (DevTools):**
- Right-click → Inspect Element
- Or use `Cmd+Option+I` (macOS)

**Backend (Rust):**
```rust
// Add logging
use log::{debug, info, warn, error};

info!("Application started");
debug!("Debug info: {:?}", data);
error!("Error occurred: {}", err);
```

```bash
# Run with logs
RUST_LOG=debug npm run tauri dev
```

### 4. Testing

**Unit Tests (Rust):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_calculation() {
        let fee = calculate_fee(100.0);
        assert_eq!(fee, 5.0);
    }
}
```

**Integration Tests:**
```rust
#[tauri::test]
async fn test_command() {
    let app = tauri::test::mock_app();
    let result = greet(app.state(), "Test".to_string());
    assert!(result.is_ok());
}
```

## Security Best Practices

### 1. Command Validation
```rust
#[tauri::command]
async fn process_input(input: String) -> Result<String, String> {
    // Validate input
    if input.is_empty() || input.len() > 1000 {
        return Err("Invalid input length".to_string());
    }
    
    // Sanitize
    let sanitized = input.trim();
    
    Ok(format!("Processed: {}", sanitized))
}
```

### 2. CSP Configuration
```json
// tauri.conf.json
{
  "tauri": {
    "security": {
      "csp": "default-src 'self'; img-src 'self' data: https:; script-src 'self' 'unsafe-inline'"
    }
  }
}
```

### 3. Capability System
```json
// capabilities/default.json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "fs:read",
    "fs:write",
    "db:query"
  ]
}
```

## Performance Optimization

### 1. Async Operations
```rust
// Use tokio for async operations
#[tauri::command]
async fn heavy_operation() -> Result<String, String> {
    tokio::task::spawn_blocking(|| {
        // CPU-intensive work
        "result".to_string()
    })
    .await
    .map_err(|e| e.to_string())
}
```

### 2. Event Batching
```rust
// Batch events to reduce IPC overhead
use tokio::time::{interval, Duration};

async fn batch_updates(app: tauri::AppHandle) {
    let mut interval = interval(Duration::from_millis(100));
    let mut batch = Vec::new();
    
    loop {
        interval.tick().await;
        if !batch.is_empty() {
            app.emit("batch-update", &batch).ok();
            batch.clear();
        }
    }
}
```

## Common Pitfalls

1. **❌ Blocking the main thread**
   - ✅ Use `tokio::task::spawn_blocking` for CPU-intensive work

2. **❌ Large IPC payloads**
   - ✅ Use streaming or pagination

3. **❌ Not handling window close events**
   - ✅ Implement proper cleanup in window event handlers

4. **❌ Forgetting to unlock Mutex**
   - ✅ Use scoped locks and RAII patterns

5. **❌ Not validating user input**
   - ✅ Always validate and sanitize input from frontend

## Useful Resources

- Tauri v2 Docs: https://v2.tauri.app/
- Tauri Discord: https://discord.gg/tauri
- Svelte 5 Docs: https://svelte-5-preview.vercel.app/
- SurrealDB Docs: https://surrealdb.com/docs
