/**
 * Batch operation API methods
 *
 * Handles multi-select operations: batch delete and batch status update.
 */

import { invoke } from '@tauri-apps/api/core';
import { logApiError } from '../services/logger';

/**
 * Table name mapping from frontend entity names to SurrealDB table names.
 * SurrealDB uses inconsistent naming: projects/contacts (plural), company/fee (singular).
 */
const TABLE_MAP: Record<string, string> = {
  projects: 'projects',
  companies: 'company',
  contacts: 'contacts',
  fees: 'fee'
};

/**
 * Delete multiple entities by their IDs.
 * @param entityType - Frontend entity type (projects, companies, contacts, fees)
 * @param ids - Array of record keys (without table prefix)
 * @returns Array of deleted records (as JSON)
 */
export async function batchDeleteEntities(entityType: string, ids: string[]): Promise<unknown[]> {
  const table = TABLE_MAP[entityType];
  if (!table) throw new Error(`Unknown entity type: ${entityType}`);

  try {
    return await invoke<unknown[]>('batch_delete_entities', { table, ids });
  } catch (error) {
    logApiError('batchDeleteEntities', error as Error, { entityType, count: ids.length });
    throw error;
  }
}

/**
 * Update the status of multiple entities.
 * @param entityType - Frontend entity type (projects, companies, contacts, fees)
 * @param ids - Array of record keys (without table prefix)
 * @param status - New status value
 * @returns Count of updated records
 */
export async function batchUpdateStatus(
  entityType: string,
  ids: string[],
  status: string
): Promise<number> {
  const table = TABLE_MAP[entityType];
  if (!table) throw new Error(`Unknown entity type: ${entityType}`);

  try {
    return await invoke<number>('batch_update_status', { table, ids, status });
  } catch (error) {
    logApiError('batchUpdateStatus', error as Error, { entityType, count: ids.length, status });
    throw error;
  }
}
