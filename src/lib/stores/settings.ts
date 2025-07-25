import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  surrealdb_url?: string;
  surrealdb_ns?: string;
  surrealdb_db?: string;
  surrealdb_user?: string;
  surrealdb_pass?: string;
  staff_name?: string;
  staff_email?: string;
  staff_phone?: string;
  staff_position?: string;
  project_folder_path?: string;
}

// Settings store
export const settingsStore = writable<AppSettings>({});
export const settingsLoading = writable<boolean>(false);
export const settingsError = writable<string | null>(null);

// Settings actions
export const settingsActions = {
  async load() {
    settingsLoading.set(true);
    settingsError.set(null);
    
    try {
      const settings = await invoke<AppSettings>('get_settings');
      settingsStore.set(settings);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load settings';
      settingsError.set(errorMessage);
      console.error('Failed to load settings:', error);
    } finally {
      settingsLoading.set(false);
    }
  },

  async save(settings: AppSettings) {
    settingsLoading.set(true);
    settingsError.set(null);
    
    try {
      await invoke<string>('save_settings', { settings });
      settingsStore.set(settings);
      return true;
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to save settings';
      settingsError.set(errorMessage);
      console.error('Failed to save settings:', error);
      throw error;
    } finally {
      settingsLoading.set(false);
    }
  }
};

// Export convenience functions
export const loadSettings = () => settingsActions.load();
export const saveSettings = (settings: AppSettings) => settingsActions.save(settings);