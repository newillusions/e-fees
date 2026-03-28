//! # Updater Logger Module
//!
//! Provides enhanced logging for the Tauri auto-updater functionality.
//! This module helps diagnose silent updater failures by logging detailed
//! information about the update process.
//!
//! ## Features
//!
//! - Conditional logging based on dev_mode setting
//! - File-based logging to /tmp/e-fees-updater.log
//! - Detailed progress tracking for update checks and downloads
//! - Error logging that's always enabled regardless of dev_mode

use chrono::Local;
use log::{debug, error, info, warn};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Log file path for updater diagnostics
const UPDATER_LOG_PATH: &str = "/tmp/e-fees-updater.log";

/// UpdaterLogger provides verbose logging for the auto-updater.
///
/// When dev_mode is enabled, all log messages are written to both
/// the standard log system and the dedicated updater log file.
/// Errors are always logged regardless of dev_mode setting.
pub struct UpdaterLogger {
    dev_mode: bool,
    log_file: Option<File>,
}

impl UpdaterLogger {
    /// Create a new UpdaterLogger instance.
    ///
    /// # Arguments
    /// * `dev_mode` - When true, enables verbose logging
    pub fn new(dev_mode: bool) -> Self {
        let log_file = if dev_mode {
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(UPDATER_LOG_PATH)
            {
                Ok(file) => Some(file),
                Err(e) => {
                    error!("[UPDATER] Failed to open log file: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let mut logger = Self { dev_mode, log_file };

        if dev_mode {
            logger.write_log("========================================");
            logger.write_log(&format!(
                "Updater logger initialized at {}",
                Local::now().format("%Y-%m-%d %H:%M:%S")
            ));
            logger.write_log(&format!("Dev mode: {}", dev_mode));
            logger.write_log("========================================");
        }

        logger
    }

    /// Write a message to the log file
    fn write_log(&mut self, message: &str) {
        if let Some(ref mut file) = self.log_file {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let log_line = format!("[{}] {}\n", timestamp, message);
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush();
        }
    }

    /// Log the start of an update check
    pub fn log_check_start(&mut self) {
        let msg = "[UPDATER] Starting update check...";
        if self.dev_mode {
            info!("{}", msg);
            self.write_log(msg);
        }
    }

    /// Log the update manifest URL being fetched
    pub fn log_manifest_fetch(&mut self, url: &str) {
        let msg = format!("[UPDATER] Fetching update manifest from: {}", url);
        if self.dev_mode {
            info!("{}", msg);
            self.write_log(&msg);
        }
    }

    /// Log version comparison results
    pub fn log_version_comparison(&mut self, current: &str, available: &str, has_update: bool) {
        let msg = format!(
            "[UPDATER] Version comparison - Current: {}, Available: {}, Update available: {}",
            current, available, has_update
        );
        if self.dev_mode {
            info!("{}", msg);
            self.write_log(&msg);
        }
    }

    /// Log the start of a download
    pub fn log_download_start(&mut self, url: &str) {
        let msg = format!("[UPDATER] Starting download from: {}", url);
        if self.dev_mode {
            info!("{}", msg);
            self.write_log(&msg);
        }
    }

    /// Log download progress
    pub fn log_download_progress(&mut self, bytes_downloaded: u64, total_bytes: Option<u64>) {
        let msg = if let Some(total) = total_bytes {
            let percent = (bytes_downloaded as f64 / total as f64 * 100.0) as u32;
            format!(
                "[UPDATER] Download progress: {} / {} bytes ({}%)",
                bytes_downloaded, total, percent
            )
        } else {
            format!("[UPDATER] Download progress: {} bytes", bytes_downloaded)
        };

        if self.dev_mode {
            debug!("{}", msg);
            self.write_log(&msg);
        }
    }

    /// Log signature verification start
    pub fn log_signature_verification_start(&mut self) {
        let msg = "[UPDATER] Starting signature verification...";
        if self.dev_mode {
            info!("{}", msg);
            self.write_log(msg);
        }
    }

    /// Log signature verification result
    pub fn log_signature_verification_result(&mut self, success: bool) {
        let msg = format!(
            "[UPDATER] Signature verification {}",
            if success { "PASSED" } else { "FAILED" }
        );
        if self.dev_mode {
            if success {
                info!("{}", msg);
            } else {
                warn!("{}", msg);
            }
            self.write_log(&msg);
        }
    }

    /// Log installation start
    pub fn log_install_start(&mut self) {
        let msg = "[UPDATER] Starting installation...";
        if self.dev_mode {
            info!("{}", msg);
            self.write_log(msg);
        }
    }

    /// Log an error - ALWAYS logs regardless of dev_mode
    pub fn log_error(&mut self, context: &str, error: &str) {
        let msg = format!("[UPDATER] ERROR in {}: {}", context, error);
        error!("{}", msg);

        // Always write errors to log file if possible
        if self.log_file.is_none() && self.dev_mode {
            // Try to open the log file for error logging
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(UPDATER_LOG_PATH)
            {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let log_line = format!("[{}] {}\n", timestamp, msg);
                let _ = file.write_all(log_line.as_bytes());
            }
        } else {
            self.write_log(&msg);
        }
    }

    /// Log a success message
    pub fn log_success(&mut self, message: &str) {
        let msg = format!("[UPDATER] SUCCESS: {}", message);
        if self.dev_mode {
            info!("{}", msg);
            self.write_log(&msg);
        }
    }

    /// Log a general debug message
    pub fn log_debug(&mut self, message: &str) {
        let msg = format!("[UPDATER] DEBUG: {}", message);
        if self.dev_mode {
            debug!("{}", msg);
            self.write_log(&msg);
        }
    }

    /// Log app startup and current version
    pub fn log_app_startup(&mut self, version: &str) {
        let msg = format!("[UPDATER] App started - Version: {}", version);
        if self.dev_mode {
            info!("{}", msg);
            self.write_log(&msg);
        }
    }

    /// Log updater configuration
    pub fn log_config(&mut self, endpoint: &str, pubkey_present: bool) {
        let msg = format!(
            "[UPDATER] Configuration - Endpoint: {}, Public key configured: {}",
            endpoint, pubkey_present
        );
        if self.dev_mode {
            info!("{}", msg);
            self.write_log(&msg);
        }
    }

    /// Get the path to the log file
    pub fn get_log_path() -> PathBuf {
        PathBuf::from(UPDATER_LOG_PATH)
    }
}

impl Drop for UpdaterLogger {
    fn drop(&mut self) {
        if self.dev_mode {
            self.write_log("[UPDATER] Logger shutting down");
            self.write_log("========================================\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = UpdaterLogger::new(false);
        assert!(!logger.dev_mode);
        assert!(logger.log_file.is_none());
    }

    #[test]
    fn test_log_path() {
        let path = UpdaterLogger::get_log_path();
        assert_eq!(path.to_str().unwrap(), "/tmp/e-fees-updater.log");
    }
}
