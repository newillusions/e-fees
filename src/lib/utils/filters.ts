/**
 * Shared filtering utilities for consistent filtering across components.
 *
 * This module provides reusable filtering functions that eliminate duplicate
 * code patterns across route components and ensure consistent filtering behavior.
 */

import { normalizedMatch } from './searchNormalize';

/**
 * Date range filter with YYYY-MM month boundaries.
 */
export interface DateRange {
  from: string; // 'YYYY-MM' or ''
  to: string; // 'YYYY-MM' or ''
}

/**
 * Advanced filter options beyond basic text search and dropdown filters.
 */
export interface AdvancedFilters {
  /** Multi-select status filter — empty set means no filtering */
  statusSet?: Set<string>;
  /** Date range filter using month boundaries */
  dateRange?: DateRange;
}

/**
 * Configuration interface for the filter function.
 *
 * @template T - The type of items being filtered
 */
export interface FilterConfig<T> {
  /** Fields to search across when performing text search */
  searchFields: (keyof T)[];
  /** Custom filter fields with extraction functions */
  filterFields?: {
    [key: string]: (item: T) => string;
  };
  /** Additional custom search functions that return strings to search against */
  customSearchFunctions?: ((item: T) => string)[];
  /** Custom sort function (defaults to updated_at descending) */
  sortFunction?: (a: T, b: T) => number;
  /** Extracts a date string from the item for date range filtering */
  dateFieldExtractor?: (item: T) => string;
  /** Format of the date field: 'iso' (default) or 'yymmdd' */
  dateFieldFormat?: 'iso' | 'yymmdd';
}

/**
 * Creates a filtered and sorted array based on search query and filters.
 *
 * This function provides consistent filtering logic that can be used across
 * all route components, eliminating code duplication and ensuring uniform
 * filtering behavior throughout the application.
 *
 * @template T - The type of items being filtered
 * @param items - Array of items to filter
 * @param searchQuery - Text search query (case-insensitive)
 * @param filters - Object containing filter key-value pairs
 * @param config - Configuration object defining search and filter behavior
 * @returns Filtered and sorted array of items
 *
 * @example
 * ```typescript
 * const filteredProjects = createFilterFunction(
 *   projects,
 *   'museum',
 *   { status: 'Active', country: 'UAE' },
 *   {
 *     searchFields: ['name', 'area', 'city'],
 *     filterFields: {
 *       status: (project) => project.status,
 *       country: (project) => project.country
 *     }
 *   }
 * );
 * ```
 */
export function createFilterFunction<T>(
  items: T[],
  searchQuery: string,
  filters: Record<string, string>,
  config: FilterConfig<T>,
  advanced?: AdvancedFilters
): T[] {
  return items
    .filter(item => {
      // Apply search filter if query is provided
      if (searchQuery.trim()) {
        const query = searchQuery.trim();

        // Check direct field matches using normalized matching
        // This allows "uae" to match "U.A.E." and "p&t" to match "P&T Group"
        const matchesDirectFields = config.searchFields.some(field => {
          const value = item[field];
          return typeof value === 'string' && normalizedMatch(value, query);
        });

        // Check custom search functions (e.g., company name/code lookup)
        const matchesCustomSearch =
          config.customSearchFunctions?.some(fn => {
            const value = fn(item);
            return typeof value === 'string' && normalizedMatch(value, query);
          }) || false;

        if (!matchesDirectFields && !matchesCustomSearch) return false;
      }

      // Apply dynamic filters
      for (const [filterKey, filterValue] of Object.entries(filters)) {
        if (filterValue && config.filterFields?.[filterKey]) {
          const itemValue = config.filterFields[filterKey](item);
          if (itemValue !== filterValue) return false;
        }
      }

      // Apply advanced multi-select status filter
      if (advanced?.statusSet && advanced.statusSet.size > 0 && config.filterFields?.status) {
        const itemStatus = config.filterFields.status(item);
        if (!advanced.statusSet.has(itemStatus)) return false;
      }

      // Apply advanced date range filter
      if (advanced?.dateRange && config.dateFieldExtractor) {
        const { from, to } = advanced.dateRange;
        if (from || to) {
          const rawDate = config.dateFieldExtractor(item);
          if (!rawDate) return false;
          const yearMonth = extractYearMonth(rawDate, config.dateFieldFormat || 'iso');
          if (!yearMonth) return false;
          if (from && yearMonth < from) return false;
          if (to && yearMonth > to) return false;
        }
      }

      return true;
    })
    .sort(config.sortFunction || (defaultSortFunction as (a: T, b: T) => number));
}

/**
 * Extracts YYYY-MM from a date string.
 *
 * @param value - Raw date string
 * @param format - 'iso' for ISO 8601 dates, 'yymmdd' for 6-digit YYMMDD strings
 * @returns 'YYYY-MM' string or '' if unparseable
 */
export function extractYearMonth(value: string, format: 'iso' | 'yymmdd'): string {
  if (!value) return '';

  if (format === 'yymmdd') {
    // YYMMDD format: e.g. '260301' → '2026-03'
    if (value.length < 6) return '';
    const yy = value.slice(0, 2);
    const mm = value.slice(2, 4);
    const year = 2000 + parseInt(yy, 10);
    const month = parseInt(mm, 10);
    if (isNaN(year) || isNaN(month) || month < 1 || month > 12) return '';
    return `${year}-${String(month).padStart(2, '0')}`;
  }

  // ISO format: '2026-03-15T...' → '2026-03'
  if (value.length >= 7) {
    const candidate = value.slice(0, 7);
    // Validate it looks like YYYY-MM
    if (/^\d{4}-\d{2}$/.test(candidate)) return candidate;
  }
  return '';
}

/**
 * Default sort function that sorts by updated_at timestamp in descending order.
 *
 * @param a - First item to compare
 * @param b - Second item to compare
 * @returns Sort comparison result
 */
function defaultSortFunction<T extends { time?: { updated_at?: string; created_at?: string } }>(
  a: T,
  b: T
): number {
  const timeA = a.time?.updated_at || a.time?.created_at || '';
  const timeB = b.time?.updated_at || b.time?.created_at || '';
  return new Date(timeB).getTime() - new Date(timeA).getTime();
}

/**
 * Creates unique sorted values from an array of items for use in filter dropdowns.
 *
 * This function extracts unique values from a specific field across all items,
 * providing a clean list for dropdown options.
 *
 * @template T - The type of items being processed
 * @param items - Array of items to extract values from
 * @param fieldExtractor - Function to extract the value from each item
 * @returns Sorted array of unique values
 *
 * @example
 * ```typescript
 * const uniqueCountries = getUniqueFieldValues(
 *   projects,
 *   (project) => project.country
 * );
 * ```
 */
export function getUniqueFieldValues<T>(items: T[], fieldExtractor: (item: T) => string): string[] {
  const uniqueValues = new Set<string>();

  items.forEach(item => {
    const value = fieldExtractor(item);
    if (value && value.trim()) {
      uniqueValues.add(value);
    }
  });

  return Array.from(uniqueValues).sort();
}

/**
 * Checks if any filters are currently active (have non-empty values).
 *
 * @param filters - Object containing filter key-value pairs
 * @param searchQuery - Current search query
 * @returns True if any filters or search query are active
 */
export function hasActiveFilters(
  filters: Record<string, string>,
  searchQuery: string = '',
  advanced?: AdvancedFilters
): boolean {
  if (searchQuery.trim() !== '') return true;
  if (Object.values(filters).some(value => value && value.trim() !== '')) return true;
  if (advanced?.statusSet && advanced.statusSet.size > 0) return true;
  if (advanced?.dateRange && (advanced.dateRange.from || advanced.dateRange.to)) return true;
  return false;
}

/**
 * Clears all active filters and search query.
 *
 * @param filters - Object containing filter key-value pairs (will be mutated)
 * @returns Empty search query string
 */
export function clearAllFilters(filters: Record<string, string>): string {
  Object.keys(filters).forEach(key => {
    filters[key] = '';
  });
  return '';
}

/**
 * Returns a fresh AdvancedFilters object with all values cleared.
 */
export function clearAdvancedFilters(): AdvancedFilters {
  return {
    statusSet: new Set<string>(),
    dateRange: { from: '', to: '' }
  };
}

/**
 * Counts the total number of items that match current filters.
 *
 * @template T - The type of items being counted
 * @param items - Array of items to count
 * @param searchQuery - Text search query
 * @param filters - Object containing filter key-value pairs
 * @param config - Configuration object defining search and filter behavior
 * @returns Object containing total count and filtered count
 */
export function getFilterCounts<T>(
  items: T[],
  searchQuery: string,
  filters: Record<string, string>,
  config: FilterConfig<T>,
  advanced?: AdvancedFilters
): { total: number; filtered: number } {
  const filteredItems = createFilterFunction(items, searchQuery, filters, config, advanced);

  return {
    total: items.length,
    filtered: filteredItems.length
  };
}
