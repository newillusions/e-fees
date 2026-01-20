/**
 * Activity Log API methods
 *
 * Handles activity log creation and retrieval for tracking user actions.
 */

import { invoke } from '@tauri-apps/api/core';
import type { ActivityLog, ActivityLogCreate } from '../../types';

/**
 * Create a new activity log entry.
 *
 * Records user actions for the activity feed. Logs are automatically
 * timestamped by the database and synced across all machines.
 *
 * @param log - Activity log entry data
 * @returns Created log with database ID
 */
export async function createActivityLog(log: ActivityLogCreate): Promise<ActivityLog> {
  try {
    return await invoke<ActivityLog>('create_activity_log', { log });
  } catch (error) {
    console.error('Failed to create activity log:', error);
    throw error;
  }
}

/**
 * Get recent activity logs with optional filtering.
 *
 * Retrieves activity logs for display in the dashboard's Recent Activity
 * panel. Logs are returned in reverse chronological order (newest first).
 *
 * @param limit - Maximum number of logs to return (default: 50)
 * @param entityType - Optional filter by entity type
 * @param offset - Optional offset for pagination
 * @returns Array of activity logs
 */
export async function getActivityLogs(
  limit?: number,
  entityType?: 'project' | 'fee' | 'company' | 'contact',
  offset?: number
): Promise<ActivityLog[]> {
  try {
    return await invoke<ActivityLog[]>('get_activity_logs', {
      limit,
      entityType,
      offset
    });
  } catch (error) {
    console.error('Failed to get activity logs:', error);
    return [];
  }
}
