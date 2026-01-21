/**
 * Connection API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { checkDbConnection, getConnectionStatus } from './connection';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('Connection API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('checkDbConnection', () => {
    it('should call invoke with correct command', async () => {
      mockInvoke.mockResolvedValueOnce(true);

      const result = await checkDbConnection();

      expect(mockInvoke).toHaveBeenCalledWith('check_db_connection');
      expect(result).toBe(true);
    });

    it('should return false on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Connection failed'));

      const result = await checkDbConnection();

      expect(result).toBe(false);
    });
  });

  describe('getConnectionStatus', () => {
    it('should call invoke with correct command', async () => {
      const mockStatus = {
        is_connected: true,
        last_check: '2025-01-18T12:00:00Z',
        error_message: undefined
      };
      mockInvoke.mockResolvedValueOnce(mockStatus);

      const result = await getConnectionStatus();

      expect(mockInvoke).toHaveBeenCalledWith('get_connection_status');
      expect(result).toEqual(mockStatus);
    });

    it('should return error status on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'));

      const result = await getConnectionStatus();

      expect(result.is_connected).toBe(false);
      expect(result.error_message).toContain('Network error');
    });
  });
});
