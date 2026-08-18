/**
 * Folder Reconcile API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  previewFolderReconcile,
  executeFolderReconcile,
  previewTrashProjectFolder,
  executeTrashProjectFolder,
  previewBackupCleanup,
  executeBackupCleanup
} from './folderReconcile';
import type {
  FolderReconcilePreview,
  FolderReconcileOutcome,
  ReconcileResolution,
  TrashFolderPreview,
  TrashFolderOutcome,
  BackupCleanupPreview,
  BackupCleanupOutcome
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

  describe('previewBackupCleanup', () => {
    it('should call invoke with cutoffDays when provided', async () => {
      const mockPreview: BackupCleanupPreview = {
        base_path: '/base',
        cutoff_days: 30,
        eligible: [
          {
            timestamp: '20260601T000000Z',
            path: '/base/.reconcile-backups/20260601T000000Z',
            age_days: 78,
            file_count: 3,
            total_size_bytes: 4096
          }
        ],
        eligible_count: 1,
        total_size_bytes: 4096
      };
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await previewBackupCleanup(30);

      expect(mockInvoke).toHaveBeenCalledWith('preview_backup_cleanup', {
        cutoffDays: 30
      });
      expect(result).toEqual(mockPreview);
    });

    it('should call invoke with cutoffDays: null when omitted (backend default applies)', async () => {
      const mockPreview: BackupCleanupPreview = {
        base_path: '/base',
        cutoff_days: 30,
        eligible: [],
        eligible_count: 0,
        total_size_bytes: 0
      };
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await previewBackupCleanup();

      expect(mockInvoke).toHaveBeenCalledWith('preview_backup_cleanup', {
        cutoffDays: null
      });
      expect(result.eligible_count).toBe(0);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('PROJECT_FOLDER_PATH not configured'));

      await expect(previewBackupCleanup()).rejects.toThrow('PROJECT_FOLDER_PATH not configured');
    });
  });

  describe('executeBackupCleanup', () => {
    it('should call invoke with dryRun and cutoffDays', async () => {
      const mockOutcome: BackupCleanupOutcome = {
        dry_run: true,
        deleted_count: 2,
        freed_bytes: 8192,
        errors: []
      };
      mockInvoke.mockResolvedValueOnce(mockOutcome);

      const result = await executeBackupCleanup(true, 30);

      expect(mockInvoke).toHaveBeenCalledWith('execute_backup_cleanup', {
        dryRun: true,
        cutoffDays: 30
      });
      expect(result).toEqual(mockOutcome);
    });

    it('should pass dryRun=false through unchanged for the real run', async () => {
      const mockOutcome: BackupCleanupOutcome = {
        dry_run: false,
        deleted_count: 2,
        freed_bytes: 8192,
        errors: []
      };
      mockInvoke.mockResolvedValueOnce(mockOutcome);

      await executeBackupCleanup(false, 30);

      expect(mockInvoke).toHaveBeenCalledWith(
        'execute_backup_cleanup',
        expect.objectContaining({ dryRun: false })
      );
    });

    it('should default cutoffDays to null when omitted', async () => {
      const mockOutcome: BackupCleanupOutcome = {
        dry_run: true,
        deleted_count: 0,
        freed_bytes: 0,
        errors: []
      };
      mockInvoke.mockResolvedValueOnce(mockOutcome);

      await executeBackupCleanup(true);

      expect(mockInvoke).toHaveBeenCalledWith('execute_backup_cleanup', {
        dryRun: true,
        cutoffDays: null
      });
    });

    it('should surface per-directory errors without throwing', async () => {
      const mockOutcome: BackupCleanupOutcome = {
        dry_run: false,
        deleted_count: 1,
        freed_bytes: 100,
        errors: ["Failed to delete '/base/.reconcile-backups/20260101T000000Z': not found"]
      };
      mockInvoke.mockResolvedValueOnce(mockOutcome);

      const result = await executeBackupCleanup(false, 30);

      expect(result.errors).toHaveLength(1);
      expect(result.deleted_count).toBe(1);
    });

    it('should throw on invoke error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('PROJECT_FOLDER_PATH not configured'));

      await expect(executeBackupCleanup(false, 30)).rejects.toThrow(
        'PROJECT_FOLDER_PATH not configured'
      );
    });
  });
});
