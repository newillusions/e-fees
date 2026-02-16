import { invoke } from '@tauri-apps/api/core';
import type { Fee } from '../../types';
import { logApiError } from '../services/logger';

/**
 * Clone a fee proposal as a new revision
 *
 * @param feeId - ID of the fee to clone (extracted string ID, not Thing object)
 * @returns Promise resolving to newly created Fee with incremented revision number
 * @throws Error if cloning fails
 *
 * @example
 * const newFee = await cloneFeeRevision('abc123')
 */
export async function cloneFeeRevision(feeId: string): Promise<Fee> {
  try {
    return await invoke<Fee>('clone_fee_revision', { feeId });
  } catch (error) {
    logApiError('cloneFeeRevision', error as Error, { component: 'RevisionsApi' });
    throw error;
  }
}

/**
 * Get all fee proposals for a specific project
 *
 * @param projectId - ID of the project (extracted string ID)
 * @returns Promise resolving to array of all fees for the project
 * @throws Error if query fails
 *
 * @example
 * const fees = await getFeesForProject('proj123')
 */
export async function getFeesForProject(projectId: string): Promise<Fee[]> {
  try {
    return await invoke<Fee[]>('get_fees_for_project', { projectId });
  } catch (error) {
    logApiError('getFeesForProject', error as Error, { component: 'RevisionsApi' });
    throw error;
  }
}

/**
 * Export a fee proposal as a pricing template Excel file
 *
 * @param feeId - ID of the fee to export (extracted string ID)
 * @param outputPath - Optional file path from save dialog. Falls back to temp dir if omitted.
 * @returns Promise resolving to file path where template was exported
 * @throws Error if export fails
 *
 * @example
 * const path = await exportFeeTemplate('abc123', '/Users/me/Desktop/Pricing.xlsx')
 */
export async function exportFeeTemplate(feeId: string, outputPath?: string): Promise<string> {
  try {
    return await invoke<string>('export_fee_template', { feeId, outputPath });
  } catch (error) {
    logApiError('exportFeeTemplate', error as Error, { component: 'ExportApi' });
    throw error;
  }
}
