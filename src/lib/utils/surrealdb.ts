/**
 * SurrealDB Utility Functions
 *
 * This module provides standardized utilities for working with SurrealDB Thing objects
 * and other SurrealDB-specific data formats. These utilities eliminate code duplication
 * across components and provide a single source of truth for SurrealDB data handling.
 */

import type { SurrealThing, UnknownSurrealThing } from '../../types';

/**
 * Extracts the key string from a v3 RecordIdKey enum variant or v2 id field.
 * Handles: plain string, number, {String: "..."}, {Number: N}
 */
function extractKeyValue(key: unknown): string | null {
  if (typeof key === 'string') return key;
  if (typeof key === 'number') return String(key);
  if (key && typeof key === 'object') {
    if ('String' in key) return (key as { String: string }).String;
    if ('Number' in key) return String((key as { Number: number }).Number);
  }
  return null;
}

/**
 * Extracts the string ID from a SurrealDB Thing/RecordId object.
 *
 * Handles all SurrealDB formats:
 * - Simple string: "123"
 * - v2 Thing: { tb: "table", id: "123" } or { tb: "table", id: { String: "123" } }
 * - v3 RecordId: { table: "table", key: "123" } or { table: "table", key: { String: "123" } }
 */
export function extractSurrealId(thing: UnknownSurrealThing): string | null {
  if (!thing) return null;

  if (typeof thing === 'string') {
    return thing;
  }

  if (thing && typeof thing === 'object') {
    // v3 format: { table: "table", key: ... }
    if ('table' in thing && 'key' in thing && thing.table) {
      return extractKeyValue(thing.key);
    }

    // v2 format: { tb: "table", id: ... }
    if ('tb' in thing && 'id' in thing && thing.tb && thing.id) {
      return extractKeyValue(thing.id);
    }

    // Direct key object: { String: "value" }
    if ('String' in thing) {
      return (thing as { String: string }).String;
    }
  }

  return null;
}

/**
 * Compares two SurrealDB Thing objects or IDs for equality.
 *
 * This function handles the complex comparison logic needed when comparing
 * SurrealDB Thing objects that may be in different formats.
 *
 * @param id1 - First ID to compare
 * @param id2 - Second ID to compare
 * @returns True if the IDs represent the same entity
 *
 * @example
 * compareSurrealIds("123", { tb: "table", id: "123" }) // Returns: true
 * compareSurrealIds(thingObj1, thingObj2) // Returns: true if same ID
 */
export function compareSurrealIds(id1: UnknownSurrealThing, id2: UnknownSurrealThing): boolean {
  const extractedId1 = extractSurrealId(id1);
  const extractedId2 = extractSurrealId(id2);

  return extractedId1 !== null && extractedId2 !== null && extractedId1 === extractedId2;
}

/**
 * Formats an ID for use in SurrealDB relations.
 *
 * @param table - The table name
 * @param id - The record ID
 * @returns Formatted relation string
 *
 * @example
 * formatSurrealRelation("projects", "25_97107") // Returns: "projects:25_97107"
 */
export function formatSurrealRelation(table: string, id: string): string {
  return `${table}:${id}`;
}

/**
 * Extracts just the ID portion from a SurrealDB relation string.
 *
 * @param relation - The relation string (e.g., "projects:25_97107")
 * @returns The ID portion, or the original string if no colon found
 *
 * @example
 * extractIdFromRelation("projects:25_97107") // Returns: "25_97107"
 * extractIdFromRelation("simple_id") // Returns: "simple_id"
 */
export function extractIdFromRelation(relation: string): string {
  if (typeof relation !== 'string') return '';

  const colonIndex = relation.indexOf(':');
  return colonIndex !== -1 ? relation.substring(colonIndex + 1) : relation;
}

/**
 * Extracts the table name from a SurrealDB relation string.
 *
 * @param relation - The relation string (e.g., "projects:25_97107")
 * @returns The table name portion, or null if no colon found
 *
 * @example
 * extractTableFromRelation("projects:25_97107") // Returns: "projects"
 * extractTableFromRelation("simple_id") // Returns: null
 */
export function extractTableFromRelation(relation: string): string | null {
  if (typeof relation !== 'string') return null;

  const colonIndex = relation.indexOf(':');
  return colonIndex !== -1 ? relation.substring(0, colonIndex) : null;
}

/**
 * Type guard: v2 SurrealDB Thing object ({tb, id})
 */
export function isSurrealThing(obj: UnknownSurrealThing): obj is SurrealThing {
  return Boolean(
    obj &&
    typeof obj === 'object' &&
    'tb' in obj &&
    typeof obj.tb === 'string' &&
    'id' in obj &&
    obj.id !== undefined
  );
}

/**
 * Type guard: v3 SurrealDB RecordId object ({table, key})
 */
export function isSurrealV3RecordId(obj: UnknownSurrealThing): boolean {
  return Boolean(
    obj &&
    typeof obj === 'object' &&
    'table' in obj &&
    typeof (obj as Record<string, unknown>).table === 'string' &&
    'key' in obj
  );
}

/**
 * Converts a SurrealDB Thing/RecordId to a human-readable string.
 * Handles v2 ({tb, id}) and v3 ({table, key}) formats.
 */
export function thingToString(thing: UnknownSurrealThing): string {
  if (typeof thing === 'string') return thing;

  if (thing && typeof thing === 'object') {
    const id = extractSurrealId(thing);
    // v3 format
    if ('table' in thing && thing.table) {
      return id ? `${thing.table}:${id}` : String(thing.table);
    }
    // v2 format
    if (isSurrealThing(thing)) {
      return id ? `${thing.tb}:${id}` : thing.tb;
    }
  }

  return String(thing);
}

/**
 * Creates a SurrealDB Thing object from table and ID.
 *
 * @param table - Table name
 * @param id - Record ID
 * @returns SurrealDB Thing object
 */
export function createSurrealThing(table: string, id: string): { tb: string; id: string } {
  return { tb: table, id };
}

/**
 * Extracts the ID from an entity, handling all SurrealDB formats.
 *
 * PERF-L3: Consolidates the repeated pattern:
 *   extractSurrealId(entity.id) || extractSurrealId(entity) || entity.id || ''
 * into a single function call.
 *
 * @param entity - Entity object that may contain an id property, or a SurrealDB Thing
 * @returns The extracted string ID, or empty string if not found
 *
 * @example
 * getEntityId({ id: "123" }) // Returns: "123"
 * getEntityId({ id: { tb: "table", id: "123" } }) // Returns: "123"
 * getEntityId({ tb: "table", id: "123" }) // Returns: "123"
 */
export function getEntityId(
  entity: UnknownSurrealThing | { id?: UnknownSurrealThing } | null | undefined
): string {
  if (!entity) return '';

  // Try extracting from the entity directly (handles Thing objects and strings)
  const directId = extractSurrealId(entity as UnknownSurrealThing);
  if (directId) return directId;

  // Try extracting from the entity.id property
  if (typeof entity === 'object' && 'id' in entity) {
    const idFromProperty = extractSurrealId(entity.id as UnknownSurrealThing);
    if (idFromProperty) return idFromProperty;

    // Fallback to raw id if it's a string
    if (typeof entity.id === 'string') return entity.id;
  }

  return '';
}
