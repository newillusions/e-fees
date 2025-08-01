/**
 * SurrealDB Utility Functions
 * 
 * This module provides standardized utilities for working with SurrealDB Thing objects
 * and other SurrealDB-specific data formats. These utilities eliminate code duplication
 * across components and provide a single source of truth for SurrealDB data handling.
 */

/**
 * Extracts the string ID from a SurrealDB Thing object.
 * 
 * SurrealDB returns IDs in various formats:
 * - Simple string: "123"
 * - Thing object: { tb: "table", id: { String: "123" } }
 * - Thing object: { tb: "table", id: "123" }
 * 
 * @param thing - The SurrealDB Thing object or string ID
 * @returns The extracted string ID, or null if extraction fails
 * 
 * @example
 * extractSurrealId("simple_id") // Returns: "simple_id"
 * extractSurrealId({ tb: "projects", id: { String: "25_97107" } }) // Returns: "25_97107"
 * extractSurrealId({ tb: "company", id: "EMITTIV" }) // Returns: "EMITTIV"
 */
export function extractSurrealId(thing: any): string | null {
  if (!thing) return null;
  
  // Handle simple string IDs
  if (typeof thing === 'string') {
    return thing;
  }
  
  // Handle SurrealDB Thing objects
  if (thing && typeof thing === 'object') {
    // Format: { tb: "table", id: { String: "value" } }
    if (thing.tb && thing.id) {
      if (typeof thing.id === 'string') {
        return thing.id;
      } else if (thing.id && typeof thing.id === 'object' && thing.id.String) {
        return thing.id.String;
      }
    }
    
    // Handle direct id objects: { String: "value" }
    if (thing.String) {
      return thing.String;
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
export function compareSurrealIds(id1: any, id2: any): boolean {
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
 * Type guard to check if an object is a SurrealDB Thing object.
 * 
 * @param obj - Object to check
 * @returns True if the object is a SurrealDB Thing
 */
export function isSurrealThing(obj: any): obj is { tb: string; id: any } {
  return obj && typeof obj === 'object' && typeof obj.tb === 'string' && obj.id !== undefined;
}

/**
 * Converts a SurrealDB Thing object to a human-readable string.
 * 
 * @param thing - The SurrealDB Thing object
 * @returns Human-readable string representation
 * 
 * @example
 * thingToString({ tb: "projects", id: "25_97107" }) // Returns: "projects:25_97107"
 */
export function thingToString(thing: any): string {
  if (typeof thing === 'string') return thing;
  
  if (isSurrealThing(thing)) {
    const id = extractSurrealId(thing);
    return id ? `${thing.tb}:${id}` : thing.tb;
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