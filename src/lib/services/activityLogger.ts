/**
 * Activity Logger Service
 *
 * Provides utility functions for logging CRUD operations to the activity log.
 * Activities are stored in the database and synced across all machines.
 *
 * Usage:
 * - Call logActivity() after successful CRUD operations
 * - Use the entity-specific helpers for common operations
 * - Activity logging is fire-and-forget (errors don't block the main operation)
 */

import { ApiClient } from '../api';
import type { ActivityLogCreate, UnknownSurrealThing } from '../../types';
import { extractSurrealId } from '../utils/surrealdb';

// Entity type definitions
export type EntityType = 'project' | 'fee' | 'company' | 'contact';
export type ActionType = 'create' | 'update' | 'delete' | 'status_change';

/**
 * Get a display name for an entity based on its type and data
 */
export function getEntityDisplayName(entityType: EntityType, entity: Record<string, unknown>): string {
  switch (entityType) {
    case 'project':
      return (entity.name as string) || (entity.project_number as string) || 'Unknown Project';
    case 'fee':
      return (entity.name as string) || (entity.number as string) || 'Unknown Fee';
    case 'company':
      return (entity.name as string) || 'Unknown Company';
    case 'contact':
      return (entity.full_name as string) ||
             `${entity.first_name || ''} ${entity.last_name || ''}`.trim() ||
             'Unknown Contact';
    default:
      return 'Unknown Entity';
  }
}

/**
 * Generate a description for an activity based on action and entity type
 */
export function generateDescription(
  action: ActionType,
  entityType: EntityType,
  entityName: string,
  oldValue?: string,
  newValue?: string
): string {
  const entityLabel = entityType.charAt(0).toUpperCase() + entityType.slice(1);

  switch (action) {
    case 'create':
      return `Created new ${entityLabel.toLowerCase()}: ${entityName}`;
    case 'update':
      return `Updated ${entityLabel.toLowerCase()}: ${entityName}`;
    case 'delete':
      return `Deleted ${entityLabel.toLowerCase()}: ${entityName}`;
    case 'status_change':
      if (oldValue && newValue) {
        return `Changed ${entityLabel.toLowerCase()} status from "${oldValue}" to "${newValue}"`;
      }
      return `Changed ${entityLabel.toLowerCase()} status`;
    default:
      return `${action} ${entityLabel.toLowerCase()}: ${entityName}`;
  }
}

/**
 * Log an activity to the database.
 * This is fire-and-forget - errors are logged but don't block the caller.
 */
export async function logActivity(log: ActivityLogCreate): Promise<void> {
  try {
    await ApiClient.createActivityLog(log);
  } catch (error) {
    // Log the error but don't throw - activity logging should never block operations
    console.error('[ActivityLogger] Failed to log activity:', error);
  }
}

/**
 * Log a create operation
 */
export async function logCreate(
  entityType: EntityType,
  entityId: string,
  entity: Record<string, unknown>,
  metadata?: Record<string, unknown>
): Promise<void> {
  const entityName = getEntityDisplayName(entityType, entity);
  await logActivity({
    action: 'create',
    entity_type: entityType,
    entity_id: entityId,
    entity_name: entityName,
    description: generateDescription('create', entityType, entityName),
    metadata
  });
}

/**
 * Log an update operation
 */
export async function logUpdate(
  entityType: EntityType,
  entityId: string,
  entity: Record<string, unknown>,
  changedFields?: string[],
  metadata?: Record<string, unknown>
): Promise<void> {
  const entityName = getEntityDisplayName(entityType, entity);
  const description = changedFields && changedFields.length > 0
    ? `Updated ${entityType}: ${entityName} (${changedFields.join(', ')})`
    : generateDescription('update', entityType, entityName);

  await logActivity({
    action: 'update',
    entity_type: entityType,
    entity_id: entityId,
    entity_name: entityName,
    description,
    metadata: { ...metadata, changedFields }
  });
}

/**
 * Log a delete operation
 */
export async function logDelete(
  entityType: EntityType,
  entityId: string,
  entityName: string,
  metadata?: Record<string, unknown>
): Promise<void> {
  await logActivity({
    action: 'delete',
    entity_type: entityType,
    entity_id: entityId,
    entity_name: entityName,
    description: generateDescription('delete', entityType, entityName),
    metadata
  });
}

/**
 * Log a status change operation
 */
export async function logStatusChange(
  entityType: EntityType,
  entityId: string,
  entityName: string,
  oldStatus: string,
  newStatus: string,
  metadata?: Record<string, unknown>
): Promise<void> {
  await logActivity({
    action: 'status_change',
    entity_type: entityType,
    entity_id: entityId,
    entity_name: entityName,
    description: generateDescription('status_change', entityType, entityName, oldStatus, newStatus),
    old_value: oldStatus,
    new_value: newStatus,
    metadata
  });
}

// ============================================================================
// ENTITY-SPECIFIC LOGGING HELPERS
// ============================================================================

/**
 * Project activity logging helpers
 */
export const projectLogger = {
  async onCreate(project: Record<string, unknown>): Promise<void> {
    const id = extractSurrealId(project.id as UnknownSurrealThing) || 'unknown';
    await logCreate('project', id, project);
  },

  async onUpdate(project: Record<string, unknown>, changedFields?: string[]): Promise<void> {
    const id = extractSurrealId(project.id as UnknownSurrealThing) || 'unknown';
    await logUpdate('project', id, project, changedFields);
  },

  async onDelete(projectId: string, projectName: string): Promise<void> {
    await logDelete('project', projectId, projectName);
  },

  async onStatusChange(project: Record<string, unknown>, oldStatus: string, newStatus: string): Promise<void> {
    const id = extractSurrealId(project.id as UnknownSurrealThing) || 'unknown';
    const name = getEntityDisplayName('project', project);
    await logStatusChange('project', id, name, oldStatus, newStatus);
  }
};

/**
 * Fee/Proposal activity logging helpers
 */
export const feeLogger = {
  async onCreate(fee: Record<string, unknown>): Promise<void> {
    const id = extractSurrealId(fee.id as UnknownSurrealThing) || 'unknown';
    await logCreate('fee', id, fee);
  },

  async onUpdate(fee: Record<string, unknown>, changedFields?: string[]): Promise<void> {
    const id = extractSurrealId(fee.id as UnknownSurrealThing) || 'unknown';
    await logUpdate('fee', id, fee, changedFields);
  },

  async onDelete(feeId: string, feeName: string): Promise<void> {
    await logDelete('fee', feeId, feeName);
  },

  async onStatusChange(fee: Record<string, unknown>, oldStatus: string, newStatus: string): Promise<void> {
    const id = extractSurrealId(fee.id as UnknownSurrealThing) || 'unknown';
    const name = getEntityDisplayName('fee', fee);
    await logStatusChange('fee', id, name, oldStatus, newStatus);
  }
};

/**
 * Company activity logging helpers
 */
export const companyLogger = {
  async onCreate(company: Record<string, unknown>): Promise<void> {
    const id = extractSurrealId(company.id as UnknownSurrealThing) || 'unknown';
    await logCreate('company', id, company);
  },

  async onUpdate(company: Record<string, unknown>, changedFields?: string[]): Promise<void> {
    const id = extractSurrealId(company.id as UnknownSurrealThing) || 'unknown';
    await logUpdate('company', id, company, changedFields);
  },

  async onDelete(companyId: string, companyName: string): Promise<void> {
    await logDelete('company', companyId, companyName);
  }
};

/**
 * Contact activity logging helpers
 */
export const contactLogger = {
  async onCreate(contact: Record<string, unknown>): Promise<void> {
    const id = extractSurrealId(contact.id as UnknownSurrealThing) || 'unknown';
    await logCreate('contact', id, contact);
  },

  async onUpdate(contact: Record<string, unknown>, changedFields?: string[]): Promise<void> {
    const id = extractSurrealId(contact.id as UnknownSurrealThing) || 'unknown';
    await logUpdate('contact', id, contact, changedFields);
  },

  async onDelete(contactId: string, contactName: string): Promise<void> {
    await logDelete('contact', contactId, contactName);
  }
};
