/**
 * Stage Dictionary API methods
 *
 * Autocomplete support for stage creation. The stage dictionary tracks all
 * pricing stages that have been used in the system, enabling suggestions
 * when creating new fees.
 */

import { invoke } from '@tauri-apps/api/core';
import { logApiError } from '../services/logger';

/**
 * Represents an entry in the stage dictionary.
 *
 * The dictionary is built from historical stage usage and supports
 * fuzzy search to help users create stages consistently.
 */
export interface StageDictionaryEntry {
  /** Canonical name of the stage (e.g., "Concept Design") */
  canonical_name: string;
  /** Default display label (often same as canonical_name) */
  default_label: string;
  /** Optional short code (e.g., "CD") */
  code?: string;
  /** True if this is a post-contract phase */
  is_post_contract: boolean;
  /** Number of times this stage has been used */
  usage_count: number;
}

/**
 * Searches the stage dictionary for entries matching the query.
 *
 * Search is case-insensitive substring match on both canonical_name and
 * default_label. Returns up to 20 results ordered by usage frequency
 * (most-used first).
 *
 * @param query - Search query (e.g., "concept", "design", "admin")
 * @returns Array of matching stage dictionary entries, ordered by usage
 * @throws Error if database query fails
 *
 * @example
 * const results = await searchStageDictionary('design')
 * // Returns entries like "Concept Design", "Scheme Design", "Detailed Design", etc.
 * // Sorted with most frequently used first
 *
 * @example
 * const results = await searchStageDictionary('admin')
 * // Returns "Construction Admin", "Project Admin", etc.
 */
export async function searchStageDictionary(query: string): Promise<StageDictionaryEntry[]> {
  try {
    const entries = await invoke<StageDictionaryEntry[]>('search_stage_dictionary', { query });
    return entries;
  } catch (error) {
    logApiError('searchStageDictionary', error as Error, { query, component: 'StageDictionaryApi' });
    throw error;
  }
}
