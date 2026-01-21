/**
 * Activity Log API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { createActivityLog, getActivityLogs } from './activity';
import type { ActivityLog, ActivityLogCreate } from '../../types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

const mockActivityLog: ActivityLog = {
  id: 'activity:test',
  action: 'status_change',
  entity_type: 'project',
  entity_id: 'projects:test',
  entity_name: 'Test Project',
  description: 'Status changed from Draft to Active',
  old_value: 'Draft',
  new_value: 'Active',
  user: 'system',
  timestamp: '2025-01-18T12:00:00Z'
};

describe('Activity Log API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('createActivityLog', () => {
    it('should call invoke with log data', async () => {
      mockInvoke.mockResolvedValueOnce(mockActivityLog);

      const logData: ActivityLogCreate = {
        action: 'status_change',
        entity_type: 'project',
        entity_id: 'projects:test',
        entity_name: 'Test Project',
        description: 'Status changed',
        old_value: 'Draft',
        new_value: 'Active'
      };

      const result = await createActivityLog(logData);

      expect(mockInvoke).toHaveBeenCalledWith('create_activity_log', { log: logData });
      expect(result).toEqual(mockActivityLog);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Create failed'));

      await expect(createActivityLog({} as ActivityLogCreate)).rejects.toThrow('Create failed');
    });
  });

  describe('getActivityLogs', () => {
    it('should call invoke with default params', async () => {
      mockInvoke.mockResolvedValueOnce([mockActivityLog]);

      const result = await getActivityLogs();

      expect(mockInvoke).toHaveBeenCalledWith('get_activity_logs', {
        limit: undefined,
        entityType: undefined,
        offset: undefined
      });
      expect(result).toEqual([mockActivityLog]);
    });

    it('should call invoke with limit parameter', async () => {
      mockInvoke.mockResolvedValueOnce([mockActivityLog]);

      await getActivityLogs(20);

      expect(mockInvoke).toHaveBeenCalledWith('get_activity_logs', {
        limit: 20,
        entityType: undefined,
        offset: undefined
      });
    });

    it('should call invoke with entity type filter', async () => {
      mockInvoke.mockResolvedValueOnce([mockActivityLog]);

      await getActivityLogs(50, 'project');

      expect(mockInvoke).toHaveBeenCalledWith('get_activity_logs', {
        limit: 50,
        entityType: 'project',
        offset: undefined
      });
    });

    it('should call invoke with offset for pagination', async () => {
      mockInvoke.mockResolvedValueOnce([mockActivityLog]);

      await getActivityLogs(20, undefined, 40);

      expect(mockInvoke).toHaveBeenCalledWith('get_activity_logs', {
        limit: 20,
        entityType: undefined,
        offset: 40
      });
    });

    it('should return empty array on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Fetch failed'));

      const result = await getActivityLogs();

      expect(result).toEqual([]);
    });
  });
});
