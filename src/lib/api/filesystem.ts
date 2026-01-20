/**
 * Filesystem API methods
 *
 * Handles all file and folder operations including folder selection,
 * opening folders in explorer, and project folder checks.
 */

import { invoke } from '@tauri-apps/api/core';
import { logApiError } from '../services/logger';

/**
 * Opens a native folder selection dialog.
 * @returns Selected folder path or null if cancelled
 */
export async function selectFolder(): Promise<string | null> {
  try {
    const folder = await invoke<string | null>('select_folder');
    return folder;
  } catch (error) {
    logApiError('selectFolder', error as Error, { component: 'FilesystemApi' });
    return null;
  }
}

/**
 * Opens a folder in the system's default file explorer.
 * @param folderPath - Absolute path to the folder to open
 * @returns Success message or error description
 */
export async function openFolderInExplorer(folderPath: string): Promise<string> {
  try {
    const result = await invoke<string>('open_folder_in_explorer', { folderPath });
    return result;
  } catch (error) {
    logApiError('openFolderInExplorer', error as Error, { component: 'FilesystemApi' });
    return 'Failed to open folder';
  }
}

/**
 * Check if a project folder already exists.
 * @param projectNumber - The project number (e.g., "25-97107")
 * @param projectShortName - The short project name (e.g., "Cove Boulevard")
 * @returns true if folder exists, false if not
 */
export async function checkProjectFolderExists(
  projectNumber: string,
  projectShortName: string
): Promise<boolean> {
  try {
    const exists = await invoke<boolean>('check_project_folder_exists', {
      projectNumber,
      projectShortName
    });
    return exists;
  } catch (error) {
    logApiError('checkProjectFolderExists', error as Error, { component: 'FilesystemApi' });
    return false;
  }
}

/**
 * Check if a var.json file already exists in a project folder.
 * @param projectNumber - The project number (e.g., "25-97107")
 * @param projectShortName - The short project name (e.g., "Cove Boulevard")
 * @returns true if var.json exists, false if not
 */
export async function checkVarJsonExists(
  projectNumber: string,
  projectShortName: string
): Promise<boolean> {
  try {
    const exists = await invoke<boolean>('check_var_json_exists', {
      projectNumber,
      projectShortName
    });
    return exists;
  } catch (error) {
    logApiError('checkVarJsonExists', error as Error, { component: 'FilesystemApi' });
    return false;
  }
}

/**
 * Check if a var.json template file (with "Default Values") exists in a project folder.
 * @param projectNumber - The project number (e.g., "25-97107")
 * @param projectShortName - The short project name (e.g., "Cove Boulevard")
 * @returns true if template file exists, false if not
 */
export async function checkVarJsonTemplateExists(
  projectNumber: string,
  projectShortName: string
): Promise<boolean> {
  try {
    const exists = await invoke<boolean>('check_var_json_template_exists', {
      projectNumber,
      projectShortName
    });
    return exists;
  } catch (error) {
    logApiError('checkVarJsonTemplateExists', error as Error, { component: 'FilesystemApi' });
    return false;
  }
}

/**
 * Rename an existing folder with _old suffix.
 * @param projectNumber - The project number (e.g., "25-97107")
 * @param projectShortName - The short project name (e.g., "Cove Boulevard")
 * @returns Success message or error
 */
export async function renameFolderWithOldSuffix(
  projectNumber: string,
  projectShortName: string
): Promise<string> {
  try {
    const result = await invoke<string>('rename_folder_with_old_suffix', {
      projectNumber,
      projectShortName
    });
    return result;
  } catch (error) {
    logApiError('renameFolderWithOldSuffix', error as Error, { component: 'FilesystemApi' });
    throw error;
  }
}

/**
 * Rename an existing var.json file with _old suffix.
 * @param projectNumber - The project number (e.g., "25-97107")
 * @param projectShortName - The short project name (e.g., "Cove Boulevard")
 * @returns Success message or error
 */
export async function renameVarJsonWithOldSuffix(
  projectNumber: string,
  projectShortName: string
): Promise<string> {
  try {
    const result = await invoke<string>('rename_var_json_with_old_suffix', {
      projectNumber,
      projectShortName
    });
    return result;
  } catch (error) {
    logApiError('renameVarJsonWithOldSuffix', error as Error, { component: 'FilesystemApi' });
    throw error;
  }
}
