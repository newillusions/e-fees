/**
 * Import Wizard API Module
 *
 * Handles scanning RFPs JSON export directories and executing imports
 * into E-Fees SurrealDB.
 */

import { invoke } from '@tauri-apps/api/core';
import { logApiError } from '../services/logger';

// ============================================================================
// TYPES
// ============================================================================

/** Preview of staff rates to be imported */
export interface StaffRatesPreview {
  count: number;
  currency: string;
  roles: string[];
}

/** Preview of a single proposal to be imported */
export interface ProposalPreview {
  source_file: string;
  project_name: string;
  project_number: string;
  client: string;
  total_fee: number;
  currency: string;
  stage_type: 'custom_milestones' | 'phased_stages';
  stage_count: number;
  phase_count: number;
  add_on_count: number;
  has_conversion_rate: boolean;
}

/** Preview of methodology to be imported */
export interface MethodologyPreview {
  wht_rate: number;
  aed_to_sar: number;
  overhead_rate: number;
  margin_rate: number;
  default_stage_count: number;
}

/** Preview of a client record to be imported */
export interface ClientPreview {
  name: string;
  client_type: string;
  project_count: number;
}

/** Complete import preview returned by scan */
export interface ImportPreview {
  staff_rates: StaffRatesPreview | null;
  proposals: ProposalPreview[];
  methodology: MethodologyPreview | null;
  clients: ClientPreview[];
  warnings: string[];
}

/** Result of executing the import */
export interface ImportResult {
  success: boolean;
  projects_created: number;
  fees_created: number;
  companies_created: number;
  companies_skipped: number;
  errors: string[];
  warnings: string[];
}

// ============================================================================
// API FUNCTIONS
// ============================================================================

/**
 * Scan a directory for RFPs JSON export files and return a preview
 * of what can be imported.
 *
 * @param directory - Absolute path to the directory containing JSON exports
 * @returns Preview of importable data
 */
export async function importScanDirectory(directory: string): Promise<ImportPreview> {
  try {
    const preview = await invoke<ImportPreview>('import_scan_directory', { directory });
    return preview;
  } catch (error) {
    logApiError('importScanDirectory', error as Error, { component: 'ImportApi' });
    throw error;
  }
}

/**
 * Execute the import of RFPs JSON data into E-Fees SurrealDB.
 *
 * @param directory - Absolute path to the JSON export directory
 * @param importProposals - Whether to import proposal/fee records
 * @param importCompanies - Whether to import company records
 * @returns Import result with counts and any errors
 */
export async function importExecute(
  directory: string,
  importProposals: boolean,
  importCompanies: boolean
): Promise<ImportResult> {
  try {
    const result = await invoke<ImportResult>('import_execute', {
      directory,
      importProposals,
      importCompanies,
    });
    return result;
  } catch (error) {
    logApiError('importExecute', error as Error, { component: 'ImportApi' });
    throw error;
  }
}
