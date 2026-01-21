/**
 * Settings API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { getSettings, saveSettings, reloadDatabaseConfig } from './settings';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('Settings API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('getSettings', () => {
    it('should call invoke and return settings', async () => {
      const mockSettings = {
        surrealdb_url: 'ws://localhost:8000',
        surrealdb_ns: 'emittiv',
        surrealdb_db: 'projects',
        dev_mode: false
      };
      mockInvoke.mockResolvedValueOnce(mockSettings);

      const result = await getSettings();

      expect(mockInvoke).toHaveBeenCalledWith('get_settings');
      expect(result).toEqual(mockSettings);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Settings not found'));

      await expect(getSettings()).rejects.toThrow('Settings not found');
    });
  });

  describe('saveSettings', () => {
    it('should call invoke with settings object', async () => {
      mockInvoke.mockResolvedValueOnce('Settings saved');

      const settings = {
        surrealdb_url: 'ws://10.0.1.17:8000',
        dev_mode: true
      };

      const result = await saveSettings(settings);

      expect(mockInvoke).toHaveBeenCalledWith('save_settings', { settings });
      expect(result).toBe('Settings saved');
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Save failed'));

      await expect(saveSettings({})).rejects.toThrow('Save failed');
    });
  });

  describe('reloadDatabaseConfig', () => {
    it('should call invoke and return success message', async () => {
      mockInvoke.mockResolvedValueOnce('Database config reloaded');

      const result = await reloadDatabaseConfig();

      expect(mockInvoke).toHaveBeenCalledWith('reload_database_config');
      expect(result).toBe('Database config reloaded');
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Reload failed'));

      await expect(reloadDatabaseConfig()).rejects.toThrow('Reload failed');
    });
  });
});
