/**
 * CRUD Pipeline — Pure filter, search, and sort functions.
 *
 * Extracted from useCrudStore() so they can be unit-tested independently.
 * No store access — operates on plain arrays.
 */

/**
 * Type-safe helper to access a property by key on an object.
 */
function getPropertyValue<T extends object>(obj: T, key: string): unknown {
  return (obj as Record<string, unknown>)[key];
}

/**
 * Apply search query and field filters to an item array.
 *
 * @param items - Source array
 * @param searchQuery - Free-text search string
 * @param filters - Key-value filter criteria
 * @param searchFields - If provided, only these fields are searched (avoids JSON.stringify)
 */
export function applyFiltersAndSearch<T extends object>(
  items: T[],
  searchQuery: string,
  filters: Record<string, unknown>,
  searchFields?: (keyof T)[]
): T[] {
  let filtered = [...items];

  if (searchQuery.trim()) {
    const query = searchQuery.toLowerCase();
    filtered = filtered.filter((item) => {
      if (searchFields && searchFields.length > 0) {
        for (const field of searchFields) {
          const value = item[field];
          if (value !== null && value !== undefined) {
            const stringValue = typeof value === 'string' ? value : String(value);
            if (stringValue.toLowerCase().includes(query)) {
              return true;
            }
          }
        }
        return false;
      }
      const searchableText = JSON.stringify(item).toLowerCase();
      return searchableText.includes(query);
    });
  }

  Object.entries(filters).forEach(([key, value]) => {
    if (value !== null && value !== undefined && value !== '') {
      filtered = filtered.filter((item) => {
        const itemValue = getPropertyValue(item, key);
        if (typeof value === 'string' && typeof itemValue === 'string') {
          return itemValue.toLowerCase().includes(value.toLowerCase());
        }
        return itemValue === value;
      });
    }
  });

  return filtered;
}

/**
 * Apply sorting to an item array. Returns a new sorted array.
 *
 * @param items - Source array (not mutated)
 * @param sort - Sort config or null (returns items unchanged)
 */
export function applySorting<T extends object>(
  items: T[],
  sort: { field: string; direction: 'asc' | 'desc' } | null
): T[] {
  if (!sort) return items;

  return [...items].sort((a, b) => {
    const aValue = getPropertyValue(a, sort.field);
    const bValue = getPropertyValue(b, sort.field);

    let comparison = 0;
    const aIsEmpty = aValue === undefined || aValue === null;
    const bIsEmpty = bValue === undefined || bValue === null;

    if (aIsEmpty && !bIsEmpty) comparison = 1;
    else if (!aIsEmpty && bIsEmpty) comparison = -1;
    else if (!aIsEmpty && !bIsEmpty) {
      if (aValue < bValue) comparison = -1;
      else if (aValue > bValue) comparison = 1;
    }

    return sort.direction === 'desc' ? -comparison : comparison;
  });
}
