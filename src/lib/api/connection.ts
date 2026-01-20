/**
 * Database connection methods
 */

import { invoke } from '@tauri-apps/api/core';
import type { ConnectionStatus } from '../../types';

/**
 * Performs a simple database connection test.
 * @returns True if connection successful, false otherwise
 */
export async function checkDbConnection(): Promise<boolean> {
  try {
    return await invoke<boolean>('check_db_connection');
  } catch (error) {
    console.error('Failed to check database connection:', error);
    return false;
  }
}

/**
 * Retrieves comprehensive database connection status information.
 * @returns Detailed connection information including status, timestamp, and errors
 */
export async function getConnectionStatus(): Promise<ConnectionStatus> {
  try {
    return await invoke<ConnectionStatus>('get_connection_status');
  } catch (error) {
    console.error('Failed to get connection status:', error);
    return {
      is_connected: false,
      last_check: undefined,
      error_message: error?.toString() || 'Unknown error'
    };
  }
}
