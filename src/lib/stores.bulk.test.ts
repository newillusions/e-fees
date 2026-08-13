/**
 * Bulk Project Actions Tests (multiselect write path)
 *
 * projectsActions.bulkUpdateStatus() / bulkDelete() are the orchestration layer
 * the Projects.svelte multiselect UI calls. Before this fix, the UI called
 * batchUpdateStatus/batchDeleteEntities directly and skipped this layer
 * entirely, which meant:
 *   1. No activity_log entries were ever written for bulk status changes or
 *      bulk deletes (verified live in prod: zero activity_log rows during the
 *      exact window 11 projects were bulk-moved to "Lost").
 *   2. The caller had no way to detect a partial application (fewer records
 *      updated/deleted than requested).
 *
 * These tests exercise the real activity-logging call chain (real
 * projectLogger -> real api.ts -> mocked Tauri invoke) so a regression that
 * silently drops the activity_log call would fail here, not just in a mock
 * assertion on a stubbed logger.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { projectsActions, paginatedProjectsStore } from './stores';
import type { Project } from '../types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

function makeProject(overrides: Partial<Project>): Project {
  return {
    id: 'projects:26_97101',
    name: 'Test Project',
    name_short: 'TP',
    status: 'RFP',
    city: 'Dubai',
    country: 'United Arab Emirates',
    time: { created_at: '2026-01-01T00:00:00Z', updated_at: '2026-01-01T00:00:00Z' },
    ...overrides
  };
}

/** Reset the paginated store to empty, then seed it with the given projects. */
function seedPaginatedProjects(projects: Project[]) {
  paginatedProjectsStore.store.set({
    items: [],
    pagination: {
      currentPage: 1,
      pageSize: 50,
      totalRecords: 0,
      hasMore: false,
      loadedIds: new Set(),
      isLoading: false
    },
    initialized: true,
    error: null
  });
  for (const project of projects) {
    paginatedProjectsStore.actions.addItem(project);
  }
}

describe('projectsActions bulk write path (multiselect)', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    seedPaginatedProjects([]);
  });

  describe('bulkUpdateStatus', () => {
    it('calls the batch_update_status Tauri command with the mapped table', async () => {
      const project = makeProject({ id: 'projects:26_97101', status: 'RFP' });
      seedPaginatedProjects([project]);
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === 'batch_update_status') return 1;
        if (cmd === 'create_activity_log') return { id: 'activity_log:1' };
        throw new Error(`unexpected invoke call: ${cmd}`);
      });

      await projectsActions.bulkUpdateStatus(['26_97101'], 'Lost');

      expect(mockInvoke).toHaveBeenCalledWith('batch_update_status', {
        table: 'projects',
        ids: ['26_97101'],
        status: 'Lost'
      });
    });

    it('patches the paginated store item in place (no full refetch)', async () => {
      const project = makeProject({ id: 'projects:26_97101', status: 'RFP' });
      seedPaginatedProjects([project]);
      mockInvoke.mockResolvedValue(1);

      await projectsActions.bulkUpdateStatus(['26_97101'], 'Lost');

      const updated = paginatedProjectsStore.actions
        .getState()
        .items.find(p => p.id === 'projects:26_97101');
      expect(updated?.status).toBe('Lost');
      // Only the batch call + the activity log call - never get_projects_page
      // (which is what a refresh()/refetch would trigger).
      expect(mockInvoke).not.toHaveBeenCalledWith('get_projects_page', expect.anything());
    });

    it('writes an activity_log status_change entry for each affected project (THE FIX)', async () => {
      const project = makeProject({
        id: 'projects:26_97101',
        name: 'RIXOS Branded Villas',
        status: 'RFP'
      });
      seedPaginatedProjects([project]);
      mockInvoke.mockResolvedValue(1);

      await projectsActions.bulkUpdateStatus(['26_97101'], 'Lost');

      expect(mockInvoke).toHaveBeenCalledWith(
        'create_activity_log',
        expect.objectContaining({
          log: expect.objectContaining({
            action: 'status_change',
            entity_type: 'project',
            old_value: 'RFP',
            new_value: 'Lost'
          })
        })
      );
    });

    it('does not log a status_change when the new status equals the old status', async () => {
      const project = makeProject({ id: 'projects:26_97101', status: 'Lost' });
      seedPaginatedProjects([project]);
      mockInvoke.mockResolvedValue(1);

      await projectsActions.bulkUpdateStatus(['26_97101'], 'Lost');

      expect(mockInvoke).not.toHaveBeenCalledWith(
        'create_activity_log',
        expect.anything()
      );
    });

    it('reports a partial application when the backend updates fewer rows than requested', async () => {
      seedPaginatedProjects([
        makeProject({ id: 'projects:exists', status: 'RFP' })
      ]);
      // Backend updated only 1 of the 2 requested ids (one no longer exists).
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === 'batch_update_status') return 1;
        return { id: 'activity_log:1' };
      });

      const result = await projectsActions.bulkUpdateStatus(['exists', 'gone'], 'Lost');

      expect(result).toEqual({ requested: 2, applied: 1 });
    });

    it('is a no-op for an empty id list (never calls invoke)', async () => {
      const result = await projectsActions.bulkUpdateStatus([], 'Lost');
      expect(result).toEqual({ requested: 0, applied: 0 });
      expect(mockInvoke).not.toHaveBeenCalled();
    });
  });

  describe('bulkDelete', () => {
    it('calls the batch_delete_entities Tauri command with the mapped table', async () => {
      seedPaginatedProjects([makeProject({ id: 'projects:26_97101' })]);
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === 'batch_delete_entities') return [{ id: 'projects:26_97101' }];
        return { id: 'activity_log:1' };
      });

      await projectsActions.bulkDelete(['26_97101']);

      expect(mockInvoke).toHaveBeenCalledWith('batch_delete_entities', {
        table: 'projects',
        ids: ['26_97101']
      });
    });

    it('removes deleted items from the paginated store', async () => {
      seedPaginatedProjects([
        makeProject({ id: 'projects:26_97101' }),
        makeProject({ id: 'projects:26_97102' })
      ]);
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === 'batch_delete_entities') return [{ id: 'projects:26_97101' }];
        return { id: 'activity_log:1' };
      });

      await projectsActions.bulkDelete(['26_97101']);

      const remaining = paginatedProjectsStore.actions.getState().items.map(p => p.id);
      expect(remaining).toEqual(['projects:26_97102']);
    });

    it('writes an activity_log delete entry for each affected project (THE FIX)', async () => {
      seedPaginatedProjects([makeProject({ id: 'projects:26_97101', name: 'Bermuda Beach' })]);
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === 'batch_delete_entities') return [{ id: 'projects:26_97101' }];
        return { id: 'activity_log:1' };
      });

      await projectsActions.bulkDelete(['26_97101']);

      expect(mockInvoke).toHaveBeenCalledWith(
        'create_activity_log',
        expect.objectContaining({
          log: expect.objectContaining({
            action: 'delete',
            entity_type: 'project',
            entity_id: '26_97101',
            entity_name: 'Bermuda Beach'
          })
        })
      );
    });

    it('reports a partial application when fewer records are deleted than requested', async () => {
      seedPaginatedProjects([makeProject({ id: 'projects:exists' })]);
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === 'batch_delete_entities') return [{ id: 'projects:exists' }];
        return { id: 'activity_log:1' };
      });

      const result = await projectsActions.bulkDelete(['exists', 'already-gone']);

      expect(result).toEqual({ requested: 2, applied: 1 });
    });

    it('is a no-op for an empty id list (never calls invoke)', async () => {
      const result = await projectsActions.bulkDelete([]);
      expect(result).toEqual({ requested: 0, applied: 0 });
      expect(mockInvoke).not.toHaveBeenCalled();
    });
  });
});
