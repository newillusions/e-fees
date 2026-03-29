import { describe, it, expect } from 'vitest';
import { applyFiltersAndSearch, applySorting } from './crudPipeline';

interface TestItem {
  id: string;
  name: string;
  status: string;
  city?: string;
  count?: number;
}

const items: TestItem[] = [
  { id: '1', name: 'Alpha Project', status: 'active', city: 'Dubai', count: 10 },
  { id: '2', name: 'Beta Corp', status: 'inactive', city: 'Riyadh', count: 5 },
  { id: '3', name: 'Gamma Design', status: 'active', city: 'Dubai', count: 20 },
  { id: '4', name: 'Delta Build', status: 'completed', city: 'Beirut', count: 15 }
];

describe('applyFiltersAndSearch', () => {
  it('returns all items when no query or filters', () => {
    const result = applyFiltersAndSearch(items, '', {});
    expect(result.length).toBe(4);
  });

  it('filters by search query (case-insensitive)', () => {
    const result = applyFiltersAndSearch(items, 'alpha', {});
    expect(result.length).toBe(1);
    expect(result[0].name).toBe('Alpha Project');
  });

  it('filters by field value (substring match)', () => {
    const result = applyFiltersAndSearch(items, '', { status: 'active' });
    // "active" matches "active" and "inactive" (includes-based)
    expect(result.length).toBe(3);

    const exact = applyFiltersAndSearch(items, '', { status: 'completed' });
    expect(exact.length).toBe(1);
  });

  it('combines search and filter', () => {
    const result = applyFiltersAndSearch(items, 'design', { status: 'active' });
    expect(result.length).toBe(1);
    expect(result[0].name).toBe('Gamma Design');
  });

  it('ignores empty/null filter values', () => {
    const result = applyFiltersAndSearch(items, '', {
      status: '',
      city: null as unknown as string
    });
    expect(result.length).toBe(4);
  });

  it('uses searchFields when provided', () => {
    // Search only in 'status' field — "active" matches "active" and "inactive"
    const result = applyFiltersAndSearch(items, 'active', {}, ['status']);
    expect(result.length).toBe(3);

    const noMatch = applyFiltersAndSearch(items, 'Alpha', {}, ['status']);
    expect(noMatch.length).toBe(0);
  });

  it('falls back to JSON.stringify when no searchFields', () => {
    // Without searchFields, should find "Dubai" in JSON
    const result = applyFiltersAndSearch(items, 'dubai', {});
    expect(result.length).toBe(2);
  });
});

describe('applySorting', () => {
  it('returns items unchanged when sort is null', () => {
    const result = applySorting(items, null);
    expect(result).toEqual(items);
  });

  it('sorts by string field ascending', () => {
    const result = applySorting(items, { field: 'name', direction: 'asc' });
    expect(result[0].name).toBe('Alpha Project');
    expect(result[3].name).toBe('Gamma Design');
  });

  it('sorts by string field descending', () => {
    const result = applySorting(items, { field: 'name', direction: 'desc' });
    expect(result[0].name).toBe('Gamma Design');
    expect(result[3].name).toBe('Alpha Project');
  });

  it('sorts by number field ascending', () => {
    const result = applySorting(items, { field: 'count', direction: 'asc' });
    expect(result[0].count).toBe(5);
    expect(result[3].count).toBe(20);
  });

  it('sorts by number field descending', () => {
    const result = applySorting(items, { field: 'count', direction: 'desc' });
    expect(result[0].count).toBe(20);
    expect(result[3].count).toBe(5);
  });

  it('pushes null/undefined values to the end', () => {
    const itemsWithNull = [
      ...items,
      { id: '5', name: 'Epsilon', status: 'draft', count: undefined }
    ];
    const result = applySorting(itemsWithNull, { field: 'count', direction: 'asc' });
    expect(result[result.length - 1].id).toBe('5');
  });

  it('does not mutate the original array', () => {
    const original = [...items];
    applySorting(items, { field: 'name', direction: 'desc' });
    expect(items).toEqual(original);
  });
});
