/**
 * System API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  getStats,
  healthCheck,
  getDbInfo,
  getTableSchema,
  positionWindow4K,
  investigateRecord
} from './system';
import type { DatabaseStats, DatabaseInfo, TableSchema } from '../../types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('System API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('getStats', () => {
    it('should call invoke and return stats', async () => {
      const mockStats: DatabaseStats = {
        totalProjects: 50,
        totalCompanies: 20,
        totalContacts: 100,
        totalFees: 30,
        activeFees: 10
      };
      mockInvoke.mockResolvedValueOnce(mockStats);

      const result = await getStats();

      expect(mockInvoke).toHaveBeenCalledWith('get_stats');
      expect(result).toEqual(mockStats);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Stats unavailable'));

      await expect(getStats()).rejects.toThrow('Stats unavailable');
    });
  });

  describe('healthCheck', () => {
    it('should call invoke and return health status', async () => {
      mockInvoke.mockResolvedValueOnce('All systems operational');

      const result = await healthCheck();

      expect(mockInvoke).toHaveBeenCalledWith('health_check');
      expect(result).toBe('All systems operational');
    });

    it('should return failure message on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Health check failed'));

      const result = await healthCheck();

      expect(result).toBe('Health check failed');
    });
  });

  describe('getDbInfo', () => {
    it('should call invoke and return db info', async () => {
      const mockInfo: DatabaseInfo = {
        status: 'connected',
        url: 'ws://localhost:8000',
        namespace: 'emittiv',
        database: 'projects'
      };
      mockInvoke.mockResolvedValueOnce(mockInfo);

      const result = await getDbInfo();

      expect(mockInvoke).toHaveBeenCalledWith('get_db_info');
      expect(result).toEqual(mockInfo);
    });

    it('should return error info on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Connection lost'));

      const result = await getDbInfo();

      expect(result.status).toBe('error');
      expect(result.error).toContain('Connection lost');
    });
  });

  describe('getTableSchema', () => {
    it('should call invoke with table name', async () => {
      const mockSchema: TableSchema = {
        table: 'projects',
        fields: [{ name: 'id', type: 'string', required: true }],
        relationships: [],
        retrieved_at: '2025-01-18T00:00:00Z'
      };
      mockInvoke.mockResolvedValueOnce(mockSchema);

      const result = await getTableSchema('projects');

      expect(mockInvoke).toHaveBeenCalledWith('get_table_schema', { tableName: 'projects' });
      expect(result).toEqual(mockSchema);
    });

    it('should return error schema on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Table not found'));

      const result = await getTableSchema('nonexistent');

      expect(result.table).toBe('nonexistent');
      expect(result.error).toContain('Table not found');
      expect(result.fields).toEqual([]);
    });
  });

  describe('positionWindow4K', () => {
    it('should call invoke and return success message', async () => {
      mockInvoke.mockResolvedValueOnce('Window positioned');

      const result = await positionWindow4K();

      expect(mockInvoke).toHaveBeenCalledWith('position_window_4k');
      expect(result).toBe('Window positioned');
    });

    it('should return failure message on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Window error'));

      const result = await positionWindow4K();

      expect(result).toBe('Failed to position window');
    });
  });

  describe('investigateRecord', () => {
    it('should call invoke with record id', async () => {
      const mockRecord = { id: 'projects:test', data: { name: 'Test' } };
      mockInvoke.mockResolvedValueOnce(mockRecord);

      const result = await investigateRecord('projects:test');

      expect(mockInvoke).toHaveBeenCalledWith('investigate_record', { recordId: 'projects:test' });
      expect(result).toEqual(mockRecord);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Record not found'));

      await expect(investigateRecord('invalid:id')).rejects.toThrow('Record not found');
    });
  });
});
