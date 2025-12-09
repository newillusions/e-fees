/**
 * Search Normalization Utilities
 *
 * Provides text normalization for search matching, allowing queries like "uae"
 * to match "U.A.E." and "pt" to match "P&T".
 */

/**
 * Normalizes text for search comparison.
 * - Converts to lowercase
 * - Removes punctuation (dots, dashes, apostrophes, etc.)
 * - Collapses whitespace
 *
 * This allows "uae" to match "U.A.E." and "p&t" to match "P&T Group"
 *
 * @param text - Text to normalize
 * @returns Normalized text for search comparison
 */
export function normalizeForSearch(text: string): string {
  if (!text) return '';
  return text
    .toLowerCase()
    .replace(/[.\-']/g, '') // Remove dots, dashes, apostrophes
    .replace(/\s+/g, ' ')   // Collapse whitespace
    .trim();
}

/**
 * Checks if a search query matches a text value using normalized comparison.
 * Handles cases like "uae" matching "U.A.E." and "ptg" matching "P.T.G."
 *
 * @param text - The text to search within
 * @param query - The search query
 * @returns true if query matches text
 */
export function normalizedMatch(text: string, query: string): boolean {
  if (!text || !query) return false;
  const normalizedText = normalizeForSearch(text);
  const normalizedQuery = normalizeForSearch(query);
  return normalizedText.includes(normalizedQuery);
}
