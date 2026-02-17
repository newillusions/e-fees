import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { logger } from '../services/logger';

const settingsLogger = logger.child({ component: 'SettingsStore' });

/**
 * Settings interface returned by get_settings command (public/safe version).
 *
 * **Security:** Password is never exposed to the frontend. The `has_password`
 * boolean indicates whether a password is configured without revealing the value.
 */
export interface AppSettingsPublic {
  surrealdb_url?: string;
  surrealdb_ns?: string;
  surrealdb_db?: string;
  surrealdb_user?: string;
  /** Indicates whether a password is configured (password value is never exposed) */
  has_password: boolean;
  staff_name?: string;
  staff_email?: string;
  staff_phone?: string;
  staff_position?: string;
  project_folder_path?: string;
  /** Development mode - enables verbose logging for debugging */
  dev_mode?: boolean;
  /** Log level - "off", "info", "debug", "trace" */
  log_level?: string;
}

/**
 * Settings interface for save_settings command (includes password for saving).
 *
 * This is used when sending settings to the backend for persistence.
 * The password field is optional - if not provided, the existing password is preserved.
 */
export interface AppSettings {
  surrealdb_url?: string;
  surrealdb_ns?: string;
  surrealdb_db?: string;
  surrealdb_user?: string;
  /** Password value for saving - only sent when user explicitly enters a new password */
  surrealdb_pass?: string;
  staff_name?: string;
  staff_email?: string;
  staff_phone?: string;
  staff_position?: string;
  project_folder_path?: string;
  /** Development mode - enables verbose logging for debugging */
  dev_mode?: boolean;
  /** Log level - "off", "info", "debug", "trace" */
  log_level?: string;
}

// Settings store - holds the public settings (without password)
export const settingsStore = writable<AppSettingsPublic>({ has_password: false });
export const settingsLoading = writable<boolean>(false);
export const settingsError = writable<string | null>(null);

// Settings actions
export const settingsActions = {
  /**
   * Load settings from the backend.
   * Returns public settings (password is never exposed, only has_password flag).
   */
  async load() {
    settingsLoading.set(true);
    settingsError.set(null);

    try {
      const settings = await invoke<AppSettingsPublic>('get_settings');
      settingsStore.set(settings);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load settings';
      settingsError.set(errorMessage);
      settingsLogger.error('Failed to load settings', { error });
    } finally {
      settingsLoading.set(false);
    }
  },

  /**
   * Save settings to the backend.
   * Accepts full settings including password for persistence.
   * After save, refreshes the store with public settings (without password).
   */
  async save(settings: AppSettings) {
    settingsLoading.set(true);
    settingsError.set(null);

    try {
      await invoke<string>('save_settings', { settings });
      // Reload public settings after save to update store
      const publicSettings = await invoke<AppSettingsPublic>('get_settings');
      settingsStore.set(publicSettings);
      return true;
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to save settings';
      settingsError.set(errorMessage);
      settingsLogger.error('Failed to save settings', { error });
      throw error;
    } finally {
      settingsLoading.set(false);
    }
  }
};

// Export convenience functions
export const loadSettings = () => settingsActions.load();
export const saveSettings = (settings: AppSettings) => settingsActions.save(settings);