/**
 * Settings API methods
 *
 * Handles application settings management and configuration operations.
 */

import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, AppSettingsPublic } from '$lib/stores/settings';
import { logger } from '../services/logger';

const settingsLogger = logger.child({ component: 'SettingsAPI' });

/**
 * Get application settings from the backend (public/safe version).
 *
 * **Security:** Password is never returned from the backend.
 * The `has_password` boolean indicates whether a password is configured.
 *
 * @returns Current application settings with sensitive data redacted
 */
export async function getSettings(): Promise<AppSettingsPublic> {
  try {
    return await invoke<AppSettingsPublic>('get_settings');
  } catch (error) {
    settingsLogger.error('Failed to get settings', { error });
    throw error;
  }
}

/**
 * Save application settings to the backend.
 *
 * @param settings - Settings object to save (can include password)
 * @returns Success or error message
 */
export async function saveSettings(settings: AppSettings): Promise<string> {
  try {
    return await invoke<string>('save_settings', { settings });
  } catch (error) {
    settingsLogger.error('Failed to save settings', { error });
    throw error;
  }
}

/**
 * Reload database configuration without requiring app restart.
 *
 * Triggers the backend to reload database settings and reinitialize
 * the database connection with the new parameters.
 *
 * @returns Success or failure message from the backend
 */
export async function reloadDatabaseConfig(): Promise<string> {
  try {
    return await invoke<string>('reload_database_config');
  } catch (error) {
    settingsLogger.error('Failed to reload database configuration', { error });
    throw error;
  }
}

/**
 * Force a database reconnection.
 *
 * Use this when the database server was temporarily unavailable or
 * network connectivity was restored. This closes any existing connection
 * and establishes a new one using the current saved configuration.
 *
 * @returns Success or failure message from the backend
 */
export async function reconnectDatabase(): Promise<string> {
  try {
    return await invoke<string>('reconnect_database');
  } catch (error) {
    settingsLogger.error('Failed to reconnect to database', { error });
    throw error;
  }
}
