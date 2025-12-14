/**
 * Folder Sync API
 *
 * Functions for scanning the file system and comparing with database
 * to detect and resolve inconsistencies between expected and actual
 * project folder locations.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  FolderSyncResult,
  FolderInconsistency,
  ResolutionResponse
} from '../../types/folderSync';

/**
 * Scan the file system and compare with database to find inconsistencies
 *
 * This function:
 * 1. Gets all projects from the database
 * 2. Scans all status folders (01 RFPs, 11 Current, etc.) for project folders
 * 3. Compares expected vs actual locations based on project status
 * 4. Returns a list of detected inconsistencies
 *
 * @param basePath - The root project folder path to scan
 * @returns FolderSyncResult containing inconsistencies and statistics
 */
export async function scanFolderSync(basePath: string): Promise<FolderSyncResult> {
  return await invoke('scan_folder_sync', { basePath });
}

/**
 * Resolve a folder inconsistency by updating database or moving folder
 *
 * Supported actions:
 * - update_db: Update project status in database to match folder location
 * - move_folder: Move folder to expected location based on DB status
 * - create_folder: Create missing folder using project template
 * - ignore: Do nothing (mark as intentionally ignored)
 *
 * @param inconsistency - The inconsistency to resolve
 * @param action - The resolution action type
 * @param options - Additional options based on action type
 * @returns ResolutionResponse with success status and message
 */
export async function resolveFolderInconsistency(
  inconsistency: FolderInconsistency,
  action: 'update_db' | 'move_folder' | 'create_folder' | 'ignore',
  options?: {
    /** New status for update_db action */
    newStatus?: string;
    /** Source path for move_folder action */
    fromPath?: string;
    /** Destination path for move_folder action */
    toPath?: string;
  }
): Promise<ResolutionResponse> {
  return await invoke('resolve_folder_inconsistency', {
    resolution: {
      inconsistency,
      action,
      projectId: inconsistency.projectId || undefined,
      newStatus: options?.newStatus,
      fromPath: options?.fromPath || inconsistency.actualPath,
      toPath: options?.toPath || inconsistency.expectedPath
    }
  });
}

/**
 * Helper to resolve by updating database status
 *
 * Updates the project's status in the database to match its current folder location.
 * Use this when the folder is in the correct physical location but the database
 * status is wrong.
 *
 * @param inconsistency - The inconsistency to resolve
 * @returns ResolutionResponse
 */
export async function resolveByUpdatingDatabase(
  inconsistency: FolderInconsistency
): Promise<ResolutionResponse> {
  if (!inconsistency.actualStatus) {
    throw new Error('Cannot update database: actual status is unknown');
  }
  return resolveFolderInconsistency(inconsistency, 'update_db', {
    newStatus: inconsistency.actualStatus
  });
}

/**
 * Helper to resolve by moving folder
 *
 * Moves the folder from its current location to the expected location
 * based on the database status. Use this when the database status is correct
 * but the folder was manually moved to the wrong location.
 *
 * @param inconsistency - The inconsistency to resolve
 * @returns ResolutionResponse
 */
export async function resolveByMovingFolder(
  inconsistency: FolderInconsistency
): Promise<ResolutionResponse> {
  if (!inconsistency.actualPath) {
    throw new Error('Cannot move folder: actual path is unknown');
  }
  if (!inconsistency.expectedPath) {
    throw new Error('Cannot move folder: expected path is unknown');
  }
  return resolveFolderInconsistency(inconsistency, 'move_folder', {
    fromPath: inconsistency.actualPath,
    toPath: inconsistency.expectedPath
  });
}

/**
 * Helper to resolve by creating missing folder
 *
 * Creates a new project folder at the expected location using the project template.
 * Use this when a project exists in the database but has no folder on disk.
 *
 * @param inconsistency - The inconsistency to resolve
 * @returns ResolutionResponse
 */
export async function resolveByCreatingFolder(
  inconsistency: FolderInconsistency
): Promise<ResolutionResponse> {
  if (!inconsistency.projectId) {
    throw new Error('Cannot create folder: project ID is unknown');
  }
  return resolveFolderInconsistency(inconsistency, 'create_folder');
}

/**
 * Helper to ignore an inconsistency
 *
 * Marks an inconsistency as intentionally ignored. Use this for orphan folders
 * that are intentional or for special cases where no action should be taken.
 *
 * @param inconsistency - The inconsistency to ignore
 * @returns ResolutionResponse
 */
export async function resolveByIgnoring(
  inconsistency: FolderInconsistency
): Promise<ResolutionResponse> {
  return resolveFolderInconsistency(inconsistency, 'ignore');
}

/**
 * Get human-readable description of an inconsistency type
 */
export function getInconsistencyDescription(type: string): string {
  switch (type) {
    case 'wrong_location':
      return 'Folder is in the wrong status directory';
    case 'missing':
      return 'Project folder not found on disk';
    case 'orphan':
      return 'Folder exists but no matching database record';
    case 'duplicate':
      return 'Project folder found in multiple locations';
    default:
      return 'Unknown inconsistency type';
  }
}

/**
 * Get suggested resolution for an inconsistency type
 */
export function getSuggestedResolution(inconsistency: FolderInconsistency): {
  action: 'update_db' | 'move_folder' | 'create_folder' | 'ignore';
  description: string;
} {
  switch (inconsistency.type) {
    case 'wrong_location':
      // If folder was moved intentionally, update DB; otherwise move folder back
      // Default suggestion: update DB (less destructive)
      return {
        action: 'update_db',
        description: `Update project status to "${inconsistency.actualStatus}" (or move folder back to ${inconsistency.expectedPath})`
      };
    case 'missing':
      return {
        action: 'create_folder',
        description: 'Create project folder from template'
      };
    case 'orphan':
      return {
        action: 'ignore',
        description: 'Review manually - this folder has no matching database record'
      };
    case 'duplicate':
      return {
        action: 'ignore',
        description: 'Review manually - remove duplicate folders'
      };
    default:
      return {
        action: 'ignore',
        description: 'Unknown issue - review manually'
      };
  }
}
