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
 * Base interface for entities that can be logged.
 * All entity types (Project, Company, Contact, Fee) satisfy this interface.
 * Uses unknown types for fields that have different structures across entities.
 */
export interface LoggableEntity {
  id?: unknown;
  name?: string;
  project_number?: string;
  number?: unknown; // Can be string (Fee) or object (Project)
  full_name?: string;
  first_name?: string;
  last_name?: string;
}

/**
 * Get a display name for an entity based on its type and data
 */
export function getEntityDisplayName(entityType: EntityType, entity: LoggableEntity): string {
  switch (entityType) {
    case 'project':
      return entity.name || entity.project_number || 'Unknown Project';
    case 'fee':
      // Fee.number is a string
      return entity.name || (typeof entity.number === 'string' ? entity.number : undefined) || 'Unknown Fee';
    case 'company':
      return entity.name || 'Unknown Company';
    case 'contact':
      return entity.full_name ||
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
  entity: LoggableEntity,
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
  entity: LoggableEntity,
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
// ENTITY LOGGER FACTORY
// ============================================================================

/** Logger interface for entities without status tracking */
export interface EntityLogger {
  onCreate(entity: LoggableEntity): Promise<void>;
  onUpdate(entity: LoggableEntity, changedFields?: string[]): Promise<void>;
  onDelete(entityId: string, entityName: string): Promise<void>;
}

/** Logger interface for entities with status tracking */
export interface EntityLoggerWithStatus extends EntityLogger {
  onStatusChange(entity: LoggableEntity, oldStatus: string, newStatus: string): Promise<void>;
}

/**
 * Factory function to create entity loggers.
 * Eliminates duplication across project, fee, company, and contact loggers.
 */
function createEntityLogger(entityType: EntityType): EntityLogger {
  return {
    async onCreate(entity: LoggableEntity): Promise<void> {
      const id = extractSurrealId(entity.id as UnknownSurrealThing) || 'unknown';
      await logCreate(entityType, id, entity);
    },
    async onUpdate(entity: LoggableEntity, changedFields?: string[]): Promise<void> {
      const id = extractSurrealId(entity.id as UnknownSurrealThing) || 'unknown';
      await logUpdate(entityType, id, entity, changedFields);
    },
    async onDelete(entityId: string, entityName: string): Promise<void> {
      await logDelete(entityType, entityId, entityName);
    }
  };
}

/**
 * Factory function to create entity loggers with status change support.
 */
function createEntityLoggerWithStatus(entityType: EntityType): EntityLoggerWithStatus {
  const baseLogger = createEntityLogger(entityType);
  return {
    ...baseLogger,
    async onStatusChange(entity: LoggableEntity, oldStatus: string, newStatus: string): Promise<void> {
      const id = extractSurrealId(entity.id as UnknownSurrealThing) || 'unknown';
      const name = getEntityDisplayName(entityType, entity);
      await logStatusChange(entityType, id, name, oldStatus, newStatus);
    }
  };
}

// ============================================================================
// ENTITY-SPECIFIC LOGGERS (created via factory)
// ============================================================================

/** Project activity logger (with status change support) */
export const projectLogger = createEntityLoggerWithStatus('project');

/** Fee/Proposal activity logger (with status change support) */
export const feeLogger = createEntityLoggerWithStatus('fee');

/** Company activity logger */
export const companyLogger = createEntityLogger('company');

/** Contact activity logger */
export const contactLogger = createEntityLogger('contact');
