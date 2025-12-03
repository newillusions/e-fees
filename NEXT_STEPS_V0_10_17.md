# Next Steps: v0.10.17 Development

**Created**: 2025-12-03
**Goal**: Implement dev mode toggle and enhanced updater logging to diagnose silent download failure

## Quick Summary

v0.10.16 is published and working except for the updater download which fails silently with NO logs. We need v0.10.17 to add verbose logging to diagnose the actual error.

## Implementation Plan

### Phase 1: Dev Mode Infrastructure

#### 1.1 Update .env Structure
**File**: `src-tauri/src/db/mod.rs` (DatabaseConfig struct)

Add dev_mode field:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub dev_mode: bool,  // ADD THIS
}
```

Default to `false` in new configs.

#### 1.2 Update Settings UI
**File**: `src/lib/components/SettingsModal.svelte`

Add checkbox for dev mode:
```svelte
<label class="flex items-center gap-2">
  <input
    type="checkbox"
    bind:checked={editedSettings.dev_mode}
    class="h-4 w-4"
  />
  <span>Development Mode (verbose logging)</span>
</label>
```

#### 1.3 Pass Dev Mode to Frontend
**File**: `src-tauri/src/lib.rs`

Add to initialization payload or create new command:
```rust
#[tauri::command]
fn get_dev_mode(state: tauri::State<AppState>) -> Result<bool, String> {
    Ok(state.db_config.dev_mode)
}
```

### Phase 2: Enhanced Updater Logging

#### 2.1 Create Logging Module
**File**: `src-tauri/src/updater_logger.rs` (NEW)

```rust
use log::{info, warn, error, debug};

pub struct UpdaterLogger {
    dev_mode: bool,
}

impl UpdaterLogger {
    pub fn new(dev_mode: bool) -> Self {
        Self { dev_mode }
    }

    pub fn log_check_start(&self) {
        if self.dev_mode {
            info!("[UPDATER] Starting update check...");
        }
    }

    pub fn log_manifest_fetch(&self, url: &str) {
        if self.dev_mode {
            info!("[UPDATER] Fetching update manifest from: {}", url);
        }
    }

    pub fn log_version_comparison(&self, current: &str, available: &str, has_update: bool) {
        if self.dev_mode {
            info!("[UPDATER] Current: {}, Available: {}, Has update: {}",
                  current, available, has_update);
        }
    }

    pub fn log_download_start(&self, url: &str) {
        if self.dev_mode {
            info!("[UPDATER] Starting download from: {}", url);
        }
    }

    pub fn log_download_progress(&self, bytes: u64, total: u64) {
        if self.dev_mode {
            debug!("[UPDATER] Downloaded: {} / {} bytes", bytes, total);
        }
    }

    pub fn log_signature_verification(&self) {
        if self.dev_mode {
            info!("[UPDATER] Verifying signature...");
        }
    }

    pub fn log_error(&self, error: &str) {
        // Always log errors, even in production
        error!("[UPDATER] ERROR: {}", error);
    }

    pub fn log_success(&self, message: &str) {
        if self.dev_mode {
            info!("[UPDATER] SUCCESS: {}", message);
        }
    }
}
```

#### 2.2 Integrate Logging with Updater
**File**: `src-tauri/src/lib.rs` (setup function)

Wrap Tauri updater calls with logging:
```rust
use crate::updater_logger::UpdaterLogger;

// In setup() or wherever updater is initialized
let dev_mode = app_state.db_config.dev_mode;
let logger = UpdaterLogger::new(dev_mode);

logger.log_check_start();

// Wrap updater check
match app.updater().check().await {
    Ok(update) => {
        logger.log_version_comparison(
            env!("CARGO_PKG_VERSION"),
            &update.version,
            update.is_some()
        );

        if let Some(update_info) = update {
            logger.log_download_start(&update_info.download_url);

            // Try to install with progress logging
            match update_info.download_and_install().await {
                Ok(_) => logger.log_success("Update installed successfully"),
                Err(e) => logger.log_error(&format!("Download/install failed: {:?}", e)),
            }
        }
    }
    Err(e) => {
        logger.log_error(&format!("Update check failed: {:?}", e));
    }
}
```

**Note**: The exact Tauri v2 updater API might differ slightly - consult official docs at https://v2.tauri.app/plugin/updater/

#### 2.3 Add Logging to File
**File**: `src-tauri/src/lib.rs`

Configure file logging for production:
```rust
use log::LevelFilter;
use env_logger::Builder;
use std::fs::OpenOptions;

// In main() or setup()
let log_file = OpenOptions::create(true)
    .append(true)
    .open("/tmp/e-fees-updater.log")
    .unwrap();

Builder::new()
    .filter_level(if dev_mode { LevelFilter::Debug } else { LevelFilter::Info })
    .target(env_logger::Target::Pipe(Box::new(log_file)))
    .init();
```

### Phase 3: Optional Debug UI

**File**: `src/lib/components/DebugPanel.svelte` (NEW - OPTIONAL)

Create collapsible debug panel shown only when dev_mode is true:
```svelte
<script lang="ts">
  import { onMount } from 'svelte';

  let devMode = false;
  let logs: string[] = [];

  onMount(async () => {
    devMode = await invoke('get_dev_mode');
  });

  async function manualUpdateCheck() {
    logs.push('Manual update check triggered...');
    // Trigger updater manually
  }
</script>

{#if devMode}
  <div class="fixed bottom-0 right-0 p-4 bg-darker border-l border-dark">
    <h3 class="text-sm font-bold mb-2">Debug Panel</h3>
    <button on:click={manualUpdateCheck}>Check for Updates</button>
    <div class="text-xs max-h-48 overflow-y-auto">
      {#each logs as log}
        <div>{log}</div>
      {/each}
    </div>
  </div>
{/if}
```

## Release Workflow for v0.10.17

```bash
# 1. Update version comment in src-tauri/src/lib.rs
# Change line 5 to:
# // Auto-update test - v0.10.17 (with dev mode and enhanced logging)

# 2. Run version sync
npm run version:set 0.10.17

# 3. Verify changes
git status

# 4. Commit
git add -A
git commit -m "feat: Add dev mode toggle and enhanced updater logging

Implemented development mode switch in settings:
- Added dev_mode field to DatabaseConfig
- Created SettingsModal checkbox for dev mode
- Configurable via .env file

Enhanced updater logging:
- Created UpdaterLogger module for verbose logging
- Logs manifest fetch, version comparison, download progress
- Logs signature verification steps
- Always logs errors, even in production mode
- Writes to /tmp/e-fees-updater.log

This will help diagnose the silent updater download failure.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# 5. Tag
git tag -a v0.10.17 -m "Release v0.10.17"

# 6. Push to both remotes
git push origin main && git push origin v0.10.17
git push github main && git push github v0.10.17

# 7. Wait for GitHub Actions build (5-10 minutes)
gh run list --repo newillusions/e-fees --limit 5

# 8. Publish to web server
./scripts/publish-release.sh 0.10.17

# 9. Commit update.json
cp /Volumes/user/www/e-fees-releases/update.json ./update.json
git add update.json
git commit -m "chore: Update manifest for v0.10.17"
git push origin main && git push github main

# 10. Test with v0.10.16
# - Ensure v0.10.16 is installed
# - Enable dev mode in settings
# - Restart app
# - Check /tmp/e-fees-updater.log for verbose logs
# - Try update and capture actual error
```

## Expected Outcome

With v0.10.17 installed and dev mode enabled, we should see detailed logs like:
```
[UPDATER] Starting update check...
[UPDATER] Fetching update manifest from: https://raw.githubusercontent.com/...
[UPDATER] Current: 0.10.17, Available: 0.10.18, Has update: true
[UPDATER] Starting download from: https://apache.mms.name/e-fees-releases/0.10.18/...
[UPDATER] Downloaded: 1048576 / 11534336 bytes
[UPDATER] ERROR: Download failed: <ACTUAL ERROR MESSAGE HERE>
```

This will reveal the root cause of the silent failure.

## Testing Strategy

1. **Install v0.10.17** on the test machine
2. **Open settings** and enable "Development Mode"
3. **Restart the app** to apply dev mode
4. **Trigger update check** (should happen automatically on startup)
5. **Check logs**: `cat /tmp/e-fees-updater.log`
6. **Attempt manual update** if automatic fails
7. **Capture and analyze** the actual error message

## Files to Modify Summary

1. ✏️ `src-tauri/src/db/mod.rs` - Add dev_mode to DatabaseConfig
2. ✏️ `src/lib/components/SettingsModal.svelte` - Add dev mode checkbox
3. ✏️ `src-tauri/src/lib.rs` - Add get_dev_mode command, integrate logger
4. ✨ `src-tauri/src/updater_logger.rs` - NEW logging module
5. ✨ `src/lib/components/DebugPanel.svelte` - NEW (optional)

## Reference Documentation

- Current Status: AUTO_UPDATER_PROGRESS.md
- Release Process: RELEASE_PROCESS.md
- Known Issues: KNOWN_ISSUES.md
- Tauri Updater: https://v2.tauri.app/plugin/updater/
