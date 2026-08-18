/**
 * Folder Reconcile API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  previewFolderReconcile,
  executeFolderReconcile,
  previewTrashProjectFolder,
  executeTrashProjectFolder
} from './folderReconcile';
import type {
  FolderReconcilePreview,
  FolderReconcileOutcome,
  ReconcileResolution,
  TrashFolderPreview,
  TrashFolderOutcome
} from '../../types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('Folder Reconcile API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('previewFolderReconcile', () => {
    it('should call invoke with sourcePath, targetPath, sourceNumber and targetNumber', async () => {
      const mockPreview: FolderReconcilePreview = {
        source_path: '/base/01 RFPs/26-97105 Old Duplicate',
        target_path: '/base/11 Current/26-97104 Real Project',
        entries: [
          {
            relative_path: 'new.pdf',
            dest_relative_path: 'new.pdf',
            status: 'new',
            source_size: 100,
            dest_size: null
          },
          {
            relative_path: '26-97105-var.json',
            dest_relative_path: '26-97104-var.json',
            status: 'conflict',
            source_size: 50,
            dest_size: 60
          }
        ],
        new_count: 1,
        conflict_count: 1,
        identical_count: 0
      };
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await previewFolderReconcile('/source', '/target', '26-97105', '26-97104');

      expect(mockInvoke).toHaveBeenCalledWith('preview_folder_reconcile', {
        sourcePath: '/source',
        targetPath: '/target',
        sourceNumber: '26-97105',
        targetNumber: '26-97104'
      });
      expect(result).toEqual(mockPreview);
      // Renamed base file pairs with a different dest_relative_path -
      // the whole point of passing the two project numbers through.
      expect(result.entries[1].dest_relative_path).not.toBe(result.entries[1].relative_path);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Directory does not exist: /source'));

      await expect(
        previewFolderReconcile('/source', '/target', '26-97105', '26-97104')
      ).rejects.toThrow('Directory does not exist');
    });
  });

  describe('executeFolderReconcile', () => {
    const resolutions: ReconcileResolution[] = [
      { relative_path: 'new.pdf', action: 'copy' },
      { relative_path: 'clash.pdf', action: 'overwrite' }
    ];

    it('should call invoke with sourcePath, targetPath, resolutions, dryRun, sourceNumber and targetNumber', async () => {
      const mockOutcome: FolderReconcileOutcome = {
        dry_run: true,
        copied: 1,
        overwritten: 1,
        kept_both: 0,
        skipped: 0,
        backups_created: ['/base/.reconcile-backups/20260818T000000Z/clash.pdf'],
        errors: []
      };
      mockInvoke.mockResolvedValueOnce(mockOutcome);

      const result = await executeFolderReconcile(
        '/source',
        '/target',
        resolutions,
        true,
        '26-97105',
        '26-97104'
      );

      expect(mockInvoke).toHaveBeenCalledWith('execute_folder_reconcile', {
        sourcePath: '/source',
        targetPath: '/target',
        resolutions,
        dryRun: true,
        sourceNumber: '26-97105',
        targetNumber: '26-97104'
      });
      expect(result).toEqual(mockOutcome);
    });

    it('should pass dryRun=false through unchanged for the real run', async () => {
      const mockOutcome: FolderReconcileOutcome = {
        dry_run: false,
        copied: 1,
        overwritten: 1,
        kept_both: 0,
        skipped: 0,
        backups_created: [],
        errors: []
      };
      mockInvoke.mockResolvedValueOnce(mockOutcome);

      await executeFolderReconcile('/source', '/target', resolutions, false, '26-97105', '26-97104');

      expect(mockInvoke).toHaveBeenCalledWith(
        'execute_folder_reconcile',
        expect.objectContaining({ dryRun: false })
      );
    });

    it('should pass empty-string project numbers through unchanged (pairing disabled)', async () => {
      const mockOutcome: FolderReconcileOutcome = {
        dry_run: true,
        copied: 0,
        overwritten: 0,
        kept_both: 0,
        skipped: 0,
        backups_created: [],
        errors: []
      };
      mockInvoke.mockResolvedValueOnce(mockOutcome);

      await executeFolderReconcile('/source', '/target', resolutions, true, '', '');

      expect(mockInvoke).toHaveBeenCalledWith(
        'execute_folder_reconcile',
        expect.objectContaining({ sourceNumber: '', targetNumber: '' })
      );
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error("relative_path must not contain '..'"));

      await expect(
        executeFolderReconcile('/source', '/target', resolutions, false, '26-97105', '26-97104')
      ).rejects.toThrow("relative_path must not contain '..'");
    });
  });

  describe('previewTrashProjectFolder', () => {
    it('should call invoke with projectNumber', async () => {
      const mockPreview: TrashFolderPreview = {
        project_number: '26-97105',
        folder_exists: true,
        source_path: '/base/01 RFPs/26-97105 Old Duplicate',
        file_count: 4,
        total_size_bytes: 2048
      };
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await previewTrashProjectFolder('26-97105');

      expect(mockInvoke).toHaveBeenCalledWith('preview_trash_project_folder', {
        projectNumber: '26-97105'
      });
      expect(result).toEqual(mockPreview);
    });

    it('should reflect a project with no on-disk folder', async () => {
      const mockPreview: TrashFolderPreview = {
        project_number: '26-97199',
        folder_exists: false,
        source_path: null,
        file_count: 0,
        total_size_bytes: 0
      };
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await previewTrashProjectFolder('26-97199');

      expect(result.folder_exists).toBe(false);
    });
  });

  describe('executeTrashProjectFolder', () => {
    it('should call invoke with projectNumber', async () => {
      const mockOutcome: TrashFolderOutcome = {
        project_number: '26-97105',
        moved: true,
        backup_path: '/base/.reconcile-backups/20260818T000000Z/26-97105 Old Duplicate',
        message: 'Folder moved to /base/.reconcile-backups/20260818T000000Z/26-97105 Old Duplicate'
      };
      mockInvoke.mockResolvedValueOnce(mockOutcome);

      const result = await executeTrashProjectFolder('26-97105');

      expect(mockInvoke).toHaveBeenCalledWith('execute_trash_project_folder', {
        projectNumber: '26-97105'
      });
      expect(result).toEqual(mockOutcome);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Failed to create backup directory'));

      await expect(executeTrashProjectFolder('26-97105')).rejects.toThrow(
        'Failed to create backup directory'
      );
    });
  });
});
