/**
 * Folder Sync Types
 *
 * Types for detecting and resolving inconsistencies between database
 * project statuses and actual file system folder locations.
 */

/**
 * Type of inconsistency detected between DB and file system
 */
export type InconsistencyType =
  | 'wrong_location' // Folder exists but in different status folder
  | 'missing' // No folder found anywhere
  | 'orphan' // Folder exists but no matching DB record
  | 'duplicate'; // Same project folder in multiple locations

/**
 * Represents a single folder/DB inconsistency
 */
export interface FolderInconsistency {
  /** Database project ID (empty string for orphans) */
  projectId: string;
  /** Project number, e.g., "25-97108" */
  projectNumber: string;
  /** Project name from DB or parsed from folder */
  projectName: string;
  /** Full folder name, e.g., "25-97108 RAK Beach District" */
  folderName: string;
  /** Type of inconsistency detected */
  type: InconsistencyType;
  /** Expected status from database (null for orphans) */
  dbStatus: string | null;
  /** Expected folder path based on DB status (null for orphans) */
  expectedPath: string | null;
  /** Actual path where folder was found (null if missing) */
  actualPath: string | null;
  /** Status inferred from actual folder location (null if missing) */
  actualStatus: string | null;
  /** All paths where folder was found (only for duplicate type) */
  duplicatePaths?: string[];
}

/**
 * Result of a folder sync scan operation
 */
export interface FolderSyncResult {
  /** ISO timestamp when scan was performed */
  scannedAt: string;
  /** Base path that was scanned */
  basePath: string;
  /** Total number of projects in database */
  totalProjects: number;
  /** Total number of project folders found on disk */
  totalFolders: number;
  /** List of detected inconsistencies */
  inconsistencies: FolderInconsistency[];
  /** Any errors encountered during scan */
  errors: string[];
}

/**
 * Resolution action to fix an inconsistency
 */
export type ResolutionAction =
  | { type: 'update_db'; projectId: string; newStatus: string }
  | { type: 'move_folder'; fromPath: string; toPath: string }
  | { type: 'create_folder'; projectId: string }
  | { type: 'ignore' };

/**
 * Request payload for resolving an inconsistency (sent to Rust backend)
 */
export interface ResolutionRequest {
  /** The inconsistency being resolved */
  inconsistency: FolderInconsistency;
  /** Action type: 'update_db', 'move_folder', 'create_folder', 'ignore' */
  action: string;
  /** Project ID for update_db and create_folder actions */
  projectId?: string;
  /** New status for update_db action */
  newStatus?: string;
  /** Source path for move_folder action */
  fromPath?: string;
  /** Destination path for move_folder action */
  toPath?: string;
}

/**
 * Response from resolution operation
 */
export interface ResolutionResponse {
  success: boolean;
  message: string;
}

/**
 * Status folder mapping (mirrors Rust backend)
 */
export const STATUS_TO_FOLDER: Record<string, string> = {
  rfp: '01 RFPs',
  active: '11 Current',
  completed: '99 Completed',
  cancelled: '00 Inactive'
};

/**
 * Reverse mapping from folder to status
 */
export const FOLDER_TO_STATUS: Record<string, string> = {
  '01 RFPs': 'rfp',
  '11 Current': 'active',
  '99 Completed': 'completed',
  '00 Inactive': 'cancelled'
};
