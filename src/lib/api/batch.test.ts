/**
 * Batch Operation API Module Tests
 *
 * Covers the multiselect bulk-action write path: entity-type -> table mapping,
 * the Tauri invoke contract, and error propagation. batch_ops.rs / db/client.rs
 * had zero test coverage before this file.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { batchDeleteEntities, batchUpdateStatus } from './batch';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('Batch Operation API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('batchDeleteEntities', () => {
    it('maps the "projects" entity type to the "projects" table', async () => {
      mockInvoke.mockResolvedValueOnce([]);

      await batchDeleteEntities('projects', ['26_97101', '26_97102']);

      expect(mockInvoke).toHaveBeenCalledWith('batch_delete_entities', {
        table: 'projects',
        ids: ['26_97101', '26_97102']
      });
    });

    it('maps the "companies" entity type to the singular "company" table', async () => {
      mockInvoke.mockResolvedValueOnce([]);

      await batchDeleteEntities('companies', ['abc']);

      expect(mockInvoke).toHaveBeenCalledWith('batch_delete_entities', {
        table: 'company',
        ids: ['abc']
      });
    });

    it('maps the "fees" entity type to the singular "fee" table', async () => {
      mockInvoke.mockResolvedValueOnce([]);

      await batchDeleteEntities('fees', ['26_97101_1']);

      expect(mockInvoke).toHaveBeenCalledWith('batch_delete_entities', {
        table: 'fee',
        ids: ['26_97101_1']
      });
    });

    it('returns the deleted records from the backend', async () => {
      const deleted = [{ id: 'projects:26_97101' }, { id: 'projects:26_97102' }];
      mockInvoke.mockResolvedValueOnce(deleted);

      const result = await batchDeleteEntities('projects', ['26_97101', '26_97102']);

      expect(result).toEqual(deleted);
    });

    it('throws for an unknown entity type without calling invoke', async () => {
      await expect(batchDeleteEntities('widgets', ['1'])).rejects.toThrow(
        'Unknown entity type: widgets'
      );
      expect(mockInvoke).not.toHaveBeenCalled();
    });

    it('propagates a backend error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('DB unreachable'));

      await expect(batchDeleteEntities('projects', ['26_97101'])).rejects.toThrow(
        'DB unreachable'
      );
    });
  });

  describe('batchUpdateStatus', () => {
    it('calls invoke with the mapped table, ids, and status', async () => {
      mockInvoke.mockResolvedValueOnce(2);

      await batchUpdateStatus('projects', ['26_97101', '26_97102'], 'Lost');

      expect(mockInvoke).toHaveBeenCalledWith('batch_update_status', {
        table: 'projects',
        ids: ['26_97101', '26_97102'],
        status: 'Lost'
      });
    });

    it('returns the updated-record count from the backend', async () => {
      mockInvoke.mockResolvedValueOnce(3);

      const result = await batchUpdateStatus('projects', ['a', 'b', 'c'], 'Awarded');

      expect(result).toBe(3);
    });

    it('surfaces a partial application - count can be lower than the requested id count', async () => {
      // The backend does not error when some ids no longer exist; it simply
      // updates whatever matches. Callers MUST compare the returned count
      // against ids.length to detect a partial application themselves.
      mockInvoke.mockResolvedValueOnce(1);

      const result = await batchUpdateStatus('projects', ['exists', 'gone'], 'Lost');

      expect(result).toBe(1);
    });

    it('throws for an unknown entity type without calling invoke', async () => {
      await expect(batchUpdateStatus('widgets', ['1'], 'X')).rejects.toThrow(
        'Unknown entity type: widgets'
      );
      expect(mockInvoke).not.toHaveBeenCalled();
    });

    it('propagates a backend error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid table name: widgets'));

      await expect(batchUpdateStatus('projects', ['26_97101'], 'Lost')).rejects.toThrow(
        'Invalid table name: widgets'
      );
    });
  });
});
