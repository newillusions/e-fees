/**
 * Shared filtering utilities for consistent filtering across components.
 * 
 * This module provides reusable filtering functions that eliminate duplicate
 * code patterns across route components and ensure consistent filtering behavior.
 */

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
  /** Custom sort function (defaults to updated_at descending) */
  sortFunction?: (a: T, b: T) => number;
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
  config: FilterConfig<T>
): T[] {
  return items.filter(item => {
    // Apply search filter if query is provided
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      const matchesSearch = config.searchFields.some(field => {
        const value = item[field];
        return typeof value === 'string' && value.toLowerCase().includes(query);
      });
      if (!matchesSearch) return false;
    }
    
    // Apply dynamic filters
    for (const [filterKey, filterValue] of Object.entries(filters)) {
      if (filterValue && config.filterFields?.[filterKey]) {
        const itemValue = config.filterFields[filterKey](item);
        if (itemValue !== filterValue) return false;
      }
    }
    
    return true;
  }).sort(config.sortFunction || defaultSortFunction);
}

/**
 * Default sort function that sorts by updated_at timestamp in descending order.
 * 
 * @param a - First item to compare
 * @param b - Second item to compare
 * @returns Sort comparison result
 */
function defaultSortFunction<T>(a: any, b: any): number {
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
export function getUniqueFieldValues<T>(
  items: T[],
  fieldExtractor: (item: T) => string
): string[] {
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
  searchQuery: string = ''
): boolean {
  return searchQuery.trim() !== '' || 
         Object.values(filters).some(value => value && value.trim() !== '');
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
  config: FilterConfig<T>
): { total: number; filtered: number } {
  const filteredItems = createFilterFunction(items, searchQuery, filters, config);
  
  return {
    total: items.length,
    filtered: filteredItems.length
  };
}