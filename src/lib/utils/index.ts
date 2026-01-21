import type { SurrealThing, UnknownSurrealThing, SurrealId } from '../../types';
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
 * Type guard to check if a value is a SurrealThing object
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
 * Extracts the actual ID from a SurrealDB Thing object or returns the string as-is
 * @param id - The ID to extract from (can be string or SurrealThing object)
 * @returns The extracted ID as a string
 */
export function extractId(id: SurrealId | UnknownSurrealThing): string {
  if (typeof id === 'string') {
    return id;
  }

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
 * Extracts the full ID string including table prefix (e.g., "projects:abc123")
 * @param id - The ID to extract from
 * @returns The full ID string with table prefix, or empty string if invalid
 */
export function extractFullId(id: SurrealId | UnknownSurrealThing): string {
  if (typeof id === 'string') {
    return id;
  }

  if (isSurrealThing(id)) {
    const idPart = typeof id.id === 'string'
      ? id.id
      : (id.id && typeof id.id === 'object' && 'String' in id.id ? id.id.String : '');
    return idPart ? `${id.tb}:${idPart}` : '';
  }

  return '';
}

/**
 * Compares two IDs, handling both string and SurrealThing formats
 * @param id1 - First ID to compare
 * @param id2 - Second ID to compare
 * @returns true if the IDs match
 */
export function compareIds(id1: SurrealId | UnknownSurrealThing, id2: SurrealId | UnknownSurrealThing): boolean {
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
export function getEntityDisplayName(entity: { name?: string; name_short?: string } | undefined): string {
  if (!entity) return 'Unknown';
  return entity.name_short || entity.name || 'Unknown';
}