import { invoke } from '@tauri-apps/api/core';
import { logApiError } from '../services/logger';

export interface StageDictEntry {
  canonical_name: string;
  default_label: string;
  aliases: string[];
  sort_order: number;
}

let cachedDictionary: StageDictEntry[] | null = null;

/**
 * Fetch the stage dictionary from the scope service (via Tauri).
 * Results are cached for the session — stages rarely change.
 */
export async function getStageDictionary(): Promise<StageDictEntry[]> {
  if (cachedDictionary) return cachedDictionary;

  try {
    const entries = await invoke<StageDictEntry[]>('get_stage_dictionary');
    cachedDictionary = entries;
    return entries;
  } catch (error) {
    logApiError('getStageDictionary', error as Error, { component: 'StagesApi' });
    return [];
  }
}

/**
 * Filter dictionary entries by query string.
 * Matches against default_label and aliases (case-insensitive, substring).
 */
export function filterStageDictionary(
  entries: StageDictEntry[],
  query: string
): StageDictEntry[] {
  if (!query.trim()) return entries;
  const q = query.toLowerCase();
  return entries.filter(entry =>
    entry.default_label.toLowerCase().includes(q) ||
    entry.aliases.some(alias => alias.toLowerCase().includes(q))
  );
}

/**
 * Generate a short code from a stage name.
 * Takes the first letter of each word, uppercase, max 4 chars.
 */
export function generateStageCode(name: string): string {
  return name.split(/\s+/).map(w => w.charAt(0).toUpperCase()).join('').slice(0, 4) || 'NS';
}
