import { invoke } from '@tauri-apps/api/core';

export interface FolderOperationResult {
  success: boolean;
  message: string;
  old_path?: string;
  new_path?: string;
}

export interface ProjectFolderInfo {
  project_number: string;
  current_location: string;
  full_path: string;
  exists: boolean;
}

/**
 * Get current location of a project folder
 */
export async function getProjectFolderLocation(projectNumber: string): Promise<ProjectFolderInfo> {
  return await invoke('get_project_folder_location', { 
    projectNumber 
  });
}

/**
 * Move a project folder to a new status location
 */
export async function moveProjectFolder(
  projectNumber: string, 
  newStatus: string
): Promise<FolderOperationResult> {
  return await invoke('move_project_folder', { 
    projectNumber, 
    newStatus 
  });
}

/**
 * Move project from RFP folder to current, archive, or inactive
 */
export async function moveProjectFromRfp(
  projectNumber: string, 
  destination: 'current' | 'archive' | 'inactive'
): Promise<FolderOperationResult> {
  return await invoke('move_project_from_rfp', { 
    projectNumber, 
    destination 
  });
}

/**
 * Move project from current to archive (completed projects)
 */
export async function moveProjectToArchive(projectNumber: string): Promise<FolderOperationResult> {
  return await invoke('move_project_to_archive', { 
    projectNumber 
  });
}

/**
 * List all project folders in a specific status directory
 */
export async function listProjectsInFolder(folderPath: string): Promise<string[]> {
  return await invoke('list_projects_in_folder', { 
    folderPath 
  });
}

/**
 * Validate that the project base path is accessible
 */
export async function validateProjectBasePath(): Promise<string> {
  return await invoke('validate_project_base_path');
}

// Status to folder mapping for UI displays
export const statusToFolder: Record<string, string> = {
  'rfp': '01 RFPs',
  'proposal': '01 RFPs', 
  'submitted': '01 RFPs',
  'active': '11 Current',
  'current': '11 Current',
  'awarded': '11 Current',
  'ongoing': '11 Current',
  'completed': '99 Completed',
  'finished': '99 Completed',
  'delivered': '99 Completed',
  'cancelled': '00 Inactive',
  'inactive': '00 Inactive',
  'lost': '00 Inactive'
};

export const folderToStatus: Record<string, string> = {
  '01 RFPs': 'rfp',
  '11 Current': 'active', 
  '99 Completed': 'completed',
  '00 Inactive': 'cancelled'
};

/**
 * Get the expected folder for a project status
 */
export function getFolderForStatus(status: string): string {
  return statusToFolder[status.toLowerCase()] || '01 RFPs';
}

/**
 * Get the status for a folder location
 */
export function getStatusForFolder(folder: string): string {
  return folderToStatus[folder] || 'rfp';
}