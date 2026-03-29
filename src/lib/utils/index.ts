import type { SurrealThing, SurrealV3RecordId, UnknownSurrealThing, SurrealId } from '../../types';
import { getVersion } from '@tauri-apps/api/app';

/**
 * Gets the application version from Tauri config
 * @returns Promise resolving to version string (e.g., "0.10.4")
 */
export async function getAppVersion(): Promise<string> {
  try {
    return await getVersion();
  } catch (error) {
    // Fallback for dev/browser environments
    console.warn('Could not get Tauri version, using fallback');
    return '0.0.0';
  }
}

/**
 * Type guard: v2 SurrealDB Thing object ({tb, id})
 */
function isSurrealThing(value: unknown): value is SurrealThing {
  return (
    typeof value === 'object' &&
    value !== null &&
    'tb' in value &&
    'id' in value &&
    typeof (value as SurrealThing).tb === 'string'
  );
}

/**
 * Type guard: v3 SurrealDB RecordId object ({table, key})
 */
function isSurrealV3RecordId(value: unknown): value is SurrealV3RecordId {
  return (
    typeof value === 'object' &&
    value !== null &&
    'table' in value &&
    'key' in value &&
    typeof (value as SurrealV3RecordId).table === 'string'
  );
}

/**
 * Extracts the key string from a v3 RecordIdKey enum variant.
 * Handles: plain string, {String: "..."}, {Number: N}
 */
function extractRecordIdKey(key: unknown): string {
  if (typeof key === 'string') return key;
  if (typeof key === 'number') return String(key);
  if (key && typeof key === 'object') {
    if ('String' in key) return (key as { String: string }).String;
    if ('Number' in key) return String((key as { Number: number }).Number);
  }
  return '';
}

/**
 * Extracts the actual ID from a SurrealDB Thing/RecordId object or returns the string as-is.
 * Handles v2 ({tb, id}) and v3 ({table, key}) formats.
 */
export function extractId(id: SurrealId | UnknownSurrealThing): string {
  if (typeof id === 'string') {
    return id;
  }

  // v3 format: {table: "projects", key: {String: "24_97109"}}
  if (isSurrealV3RecordId(id)) {
    return extractRecordIdKey(id.key);
  }

  // v2 format: {tb: "projects", id: "24_97109"}
  if (isSurrealThing(id)) {
    if (typeof id.id === 'string') {
      return id.id;
    } else if (id.id && typeof id.id === 'object' && 'String' in id.id) {
      return id.id.String;
    }
  }

  return '';
}

/**
 * Extracts the full ID string including table prefix (e.g., "projects:abc123").
 * Handles v2 ({tb, id}) and v3 ({table, key}) formats.
 */
export function extractFullId(id: SurrealId | UnknownSurrealThing): string {
  if (typeof id === 'string') {
    return id;
  }

  // v3 format
  if (isSurrealV3RecordId(id)) {
    const keyPart = extractRecordIdKey(id.key);
    return keyPart ? `${id.table}:${keyPart}` : '';
  }

  // v2 format
  if (isSurrealThing(id)) {
    const idPart =
      typeof id.id === 'string'
        ? id.id
        : id.id && typeof id.id === 'object' && 'String' in id.id
          ? id.id.String
          : '';
    return idPart ? `${id.tb}:${idPart}` : '';
  }

  return '';
}

/**
 * Compares two IDs, handling string, v2 Thing, and v3 RecordId formats
 */
export function compareIds(
  id1: SurrealId | UnknownSurrealThing,
  id2: SurrealId | UnknownSurrealThing
): boolean {
  const extractedId1 = extractId(id1);
  const extractedId2 = extractId(id2);

  if (!extractedId1 || !extractedId2) {
    return false;
  }

  return extractedId1 === extractedId2;
}

/**
 * Finds an entity in an array by its ID, handling SurrealThing objects
 * @param entities - Array of entities to search through
 * @param targetId - The ID to find
 * @returns The found entity or undefined
 */
export function findEntityById<T extends { id?: SurrealId | UnknownSurrealThing }>(
  entities: T[],
  targetId: SurrealId | UnknownSurrealThing
): T | undefined {
  return entities.find(entity => compareIds(entity.id, targetId));
}

/**
 * Gets a display name for an entity, preferring short name over full name
 * @param entity - The entity to get name from
 * @returns The display name
 */
export function getEntityDisplayName(
  entity: { name?: string; name_short?: string } | undefined
): string {
  if (!entity) return 'Unknown';
  return entity.name_short || entity.name || 'Unknown';
}
