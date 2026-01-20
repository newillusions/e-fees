/**
 * Filesystem API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  selectFolder,
  openFolderInExplorer,
  checkProjectFolderExists,
  checkVarJsonExists,
  checkVarJsonTemplateExists,
  renameFolderWithOldSuffix,
  renameVarJsonWithOldSuffix
} from './filesystem';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('Filesystem API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('selectFolder', () => {
    it('should call invoke and return selected path', async () => {
      mockInvoke.mockResolvedValueOnce('/Users/test/Projects');

      const result = await selectFolder();

      expect(mockInvoke).toHaveBeenCalledWith('select_folder');
      expect(result).toBe('/Users/test/Projects');
    });

    it('should return null when cancelled', async () => {
      mockInvoke.mockResolvedValueOnce(null);

      const result = await selectFolder();

      expect(result).toBeNull();
    });

    it('should return null on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Dialog failed'));

      const result = await selectFolder();

      expect(result).toBeNull();
    });
  });

  describe('openFolderInExplorer', () => {
    it('should call invoke with folder path', async () => {
      mockInvoke.mockResolvedValueOnce('Success');

      const result = await openFolderInExplorer('/path/to/folder');

      expect(mockInvoke).toHaveBeenCalledWith('open_folder_in_explorer', { folderPath: '/path/to/folder' });
      expect(result).toBe('Success');
    });

    it('should return error message on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Folder not found'));

      const result = await openFolderInExplorer('/invalid/path');

      expect(result).toBe('Failed to open folder');
    });
  });

  describe('checkProjectFolderExists', () => {
    it('should call invoke with project details', async () => {
      mockInvoke.mockResolvedValueOnce(true);

      const result = await checkProjectFolderExists('25-97101', 'Test Project');

      expect(mockInvoke).toHaveBeenCalledWith('check_project_folder_exists', {
        projectNumber: '25-97101',
        projectShortName: 'Test Project'
      });
      expect(result).toBe(true);
    });

    it('should return false on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Check failed'));

      const result = await checkProjectFolderExists('25-97101', 'Test');

      expect(result).toBe(false);
    });
  });

  describe('checkVarJsonExists', () => {
    it('should call invoke and return existence status', async () => {
      mockInvoke.mockResolvedValueOnce(true);

      const result = await checkVarJsonExists('25-97101', 'Test Project');

      expect(mockInvoke).toHaveBeenCalledWith('check_var_json_exists', {
        projectNumber: '25-97101',
        projectShortName: 'Test Project'
      });
      expect(result).toBe(true);
    });

    it('should return false on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Check failed'));

      const result = await checkVarJsonExists('25-97101', 'Test');

      expect(result).toBe(false);
    });
  });

  describe('checkVarJsonTemplateExists', () => {
    it('should call invoke and return template existence', async () => {
      mockInvoke.mockResolvedValueOnce(false);

      const result = await checkVarJsonTemplateExists('25-97101', 'Test');

      expect(mockInvoke).toHaveBeenCalledWith('check_var_json_template_exists', {
        projectNumber: '25-97101',
        projectShortName: 'Test'
      });
      expect(result).toBe(false);
    });
  });

  describe('renameFolderWithOldSuffix', () => {
    it('should call invoke and return success message', async () => {
      mockInvoke.mockResolvedValueOnce('Folder renamed to _old');

      const result = await renameFolderWithOldSuffix('25-97101', 'Test');

      expect(mockInvoke).toHaveBeenCalledWith('rename_folder_with_old_suffix', {
        projectNumber: '25-97101',
        projectShortName: 'Test'
      });
      expect(result).toBe('Folder renamed to _old');
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Rename failed'));

      await expect(renameFolderWithOldSuffix('25-97101', 'Test')).rejects.toThrow('Rename failed');
    });
  });

  describe('renameVarJsonWithOldSuffix', () => {
    it('should call invoke and return success message', async () => {
      mockInvoke.mockResolvedValueOnce('var.json renamed to _old');

      const result = await renameVarJsonWithOldSuffix('25-97101', 'Test');

      expect(mockInvoke).toHaveBeenCalledWith('rename_var_json_with_old_suffix', {
        projectNumber: '25-97101',
        projectShortName: 'Test'
      });
      expect(result).toBe('var.json renamed to _old');
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Rename failed'));

      await expect(renameVarJsonWithOldSuffix('25-97101', 'Test')).rejects.toThrow('Rename failed');
    });
  });
});
