/**
 * System API methods
 *
 * Handles system health checks, debugging, statistics, and window management.
 */

import { invoke } from '@tauri-apps/api/core';
import type { DatabaseStats, DatabaseInfo, TableSchema } from '../../types';

/**
 * Retrieves comprehensive application statistics for the dashboard.
 * @returns Object containing all statistics
 */
export async function getStats(): Promise<DatabaseStats> {
  try {
    const stats = await invoke<DatabaseStats>('get_stats');
    return stats;
  } catch (error) {
    console.error('Failed to fetch stats from database:', error);
    throw error;
  }
}

/**
 * Performs a comprehensive system health check.
 * @returns Health status message
 */
export async function healthCheck(): Promise<string> {
  try {
    return await invoke<string>('health_check');
  } catch (error) {
    console.error('Health check failed:', error);
    return 'Health check failed';
  }
}

/**
 * Retrieves detailed database connection information for debugging.
 * @returns Database information object
 */
export async function getDbInfo(): Promise<DatabaseInfo> {
  try {
    return await invoke<DatabaseInfo>('get_db_info');
  } catch (error) {
    console.error('Failed to get database info:', error);
    return {
      error: error?.toString() || 'Unknown error',
      status: 'error'
    };
  }
}

/**
 * Retrieves schema information for a specific database table.
 * @param tableName - Name of the table to get schema for
 * @returns Table schema information
 */
export async function getTableSchema(tableName: string): Promise<TableSchema> {
  try {
    return await invoke<TableSchema>('get_table_schema', { tableName });
  } catch (error) {
    console.error(`Failed to get schema for table ${tableName}:`, error);
    return {
      table: tableName,
      fields: [],
      relationships: [],
      error: error?.toString() || 'Unknown error',
      retrieved_at: new Date().toISOString()
    };
  }
}

/**
 * Positions the application window optimally for 4K displays.
 * @returns Success or error message
 */
export async function positionWindow4K(): Promise<string> {
  try {
    return await invoke<string>('position_window_4k');
  } catch (error) {
    console.error('Failed to position window:', error);
    return 'Failed to position window';
  }
}

/**
 * Investigates a database record for debugging and analysis.
 * @param recordId - SurrealDB record ID to investigate
 * @returns Detailed record information
 */
export async function investigateRecord(recordId: string): Promise<any> {
  try {
    const result = await invoke('investigate_record', { recordId });
    return result;
  } catch (error) {
    console.error('Failed to investigate record:', error);
    throw error;
  }
}
