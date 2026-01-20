/**
 * Settings API methods
 *
 * Handles application settings management and configuration operations.
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Get application settings from the backend.
 * @returns Current application settings
 */
export async function getSettings(): Promise<any> {
  try {
    return await invoke<any>('get_settings');
  } catch (error) {
    console.error('Failed to get settings:', error);
    throw error;
  }
}

/**
 * Save application settings to the backend.
 * @param settings - Settings object to save
 * @returns Success or error message
 */
export async function saveSettings(settings: Record<string, unknown>): Promise<string> {
  try {
    return await invoke<string>('save_settings', { settings });
  } catch (error) {
    console.error('Failed to save settings:', error);
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
    console.error('Failed to reload database configuration:', error);
    throw error;
  }
}
