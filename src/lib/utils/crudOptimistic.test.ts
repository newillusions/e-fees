import { describe, it, expect } from 'vitest';
import { writable, get } from 'svelte/store';
import type { CrudState } from './crudTypes';
import {
  optimisticCreate,
  optimisticUpdate,
  optimisticDelete,
  recomputeFiltered
} from './crudOptimistic';

// Minimal test entity
interface TestEntity {
  id?: string;
  name: string;
}

// Test helpers
function createTestStore(
  items: TestEntity[] = []
): ReturnType<typeof writable<CrudState<TestEntity>>> {
  return writable<CrudState<TestEntity>>({
    items,
    filteredItems: [...items],
    loading: false,
    error: null,
    saving: false,
    searchQuery: '',
    filters: {},
    sort: null,
    lastUpdated: null,
    optimisticUpdates: new Map()
  });
}

const testIdExtractor = (id: unknown) => String(id ?? '');
const noopFilter = (items: TestEntity[]) => items;
const noopSort = (items: TestEntity[]) => items;

describe('crudOptimistic', () => {
  describe('recomputeFiltered', () => {
    it('applies filter and sort to items using current state', () => {
      const store = createTestStore([
        { id: 'a', name: 'Alpha' },
        { id: 'b', name: 'Beta' }
      ]);
      const reverseSort = (items: TestEntity[]) => [...items].reverse();

      recomputeFiltered(store, noopFilter, reverseSort);

      const state = get(store);
      expect(state.filteredItems[0].name).toBe('Beta');
      expect(state.filteredItems[1].name).toBe('Alpha');
    });
  });

  describe('optimisticCreate', () => {
    it('adds temporary item to store immediately', () => {
      const store = createTestStore([{ id: '1', name: 'Existing' }]);

      const result = optimisticCreate(
        store,
        { name: 'New Item' },
        testIdExtractor,
        noopFilter,
        noopSort
      );

      const state = get(store);
      expect(state.items).toHaveLength(2);
      expect(state.items[1].name).toBe('New Item');
      expect(result.tempId).toMatch(/^temp_/);
    });

    it('commit replaces temp item with real item', () => {
      const store = createTestStore([]);

      const { commit } = optimisticCreate(
        store,
        { name: 'Temp' },
        testIdExtractor,
        noopFilter,
        noopSort
      );
      commit({ id: 'real-1', name: 'Real Item' });

      const state = get(store);
      expect(state.items).toHaveLength(1);
      expect(state.items[0].id).toBe('real-1');
      expect(state.items[0].name).toBe('Real Item');
      expect(state.optimisticUpdates.size).toBe(0);
    });

    it('rollback removes temp item on failure', () => {
      const store = createTestStore([{ id: '1', name: 'Existing' }]);

      const { rollback } = optimisticCreate(
        store,
        { name: 'Will Fail' },
        testIdExtractor,
        noopFilter,
        noopSort
      );
      rollback();

      const state = get(store);
      expect(state.items).toHaveLength(1);
      expect(state.items[0].name).toBe('Existing');
      expect(state.optimisticUpdates.size).toBe(0);
    });
  });

  describe('optimisticUpdate', () => {
    it('replaces item with merged version immediately', () => {
      const store = createTestStore([{ id: '1', name: 'Original' }]);

      optimisticUpdate(store, '1', { name: 'Updated' }, testIdExtractor, noopFilter, noopSort);

      const state = get(store);
      expect(state.items[0].name).toBe('Updated');
    });

    it('commit finalizes with server response', () => {
      const store = createTestStore([{ id: '1', name: 'Original' }]);

      const { commit } = optimisticUpdate(
        store,
        '1',
        { name: 'Optimistic' },
        testIdExtractor,
        noopFilter,
        noopSort
      );
      commit({ id: '1', name: 'Server Response' });

      const state = get(store);
      expect(state.items[0].name).toBe('Server Response');
      expect(state.optimisticUpdates.size).toBe(0);
    });

    it('rollback restores original item on failure', () => {
      const store = createTestStore([{ id: '1', name: 'Original' }]);

      const { rollback } = optimisticUpdate(
        store,
        '1',
        { name: 'Will Fail' },
        testIdExtractor,
        noopFilter,
        noopSort
      );
      rollback();

      const state = get(store);
      expect(state.items[0].name).toBe('Original');
      expect(state.optimisticUpdates.size).toBe(0);
    });

    it('returns null originalItem when id not found', () => {
      const store = createTestStore([{ id: '1', name: 'Only' }]);

      const { originalItem } = optimisticUpdate(
        store,
        'nonexistent',
        { name: 'X' },
        testIdExtractor,
        noopFilter,
        noopSort
      );

      expect(originalItem).toBeNull();
    });
  });

  describe('optimisticDelete', () => {
    it('removes item from store immediately', () => {
      const store = createTestStore([
        { id: '1', name: 'First' },
        { id: '2', name: 'Second' }
      ]);

      optimisticDelete(store, '1', testIdExtractor, noopFilter, noopSort);

      const state = get(store);
      expect(state.items).toHaveLength(1);
      expect(state.items[0].name).toBe('Second');
    });

    it('commit keeps item removed', () => {
      const store = createTestStore([{ id: '1', name: 'Gone' }]);

      const { commit } = optimisticDelete(store, '1', testIdExtractor, noopFilter, noopSort);
      commit();

      const state = get(store);
      expect(state.items).toHaveLength(0);
      expect(state.optimisticUpdates.size).toBe(0);
    });

    it('rollback restores deleted item at original position', () => {
      const store = createTestStore([
        { id: '1', name: 'First' },
        { id: '2', name: 'Second' },
        { id: '3', name: 'Third' }
      ]);

      const { rollback } = optimisticDelete(store, '2', testIdExtractor, noopFilter, noopSort);
      rollback();

      const state = get(store);
      expect(state.items).toHaveLength(3);
      expect(state.items[1].name).toBe('Second');
    });

    it('returns null deletedItem when id not found', () => {
      const store = createTestStore([{ id: '1', name: 'Only' }]);

      const { deletedItem } = optimisticDelete(
        store,
        'nonexistent',
        testIdExtractor,
        noopFilter,
        noopSort
      );

      expect(deletedItem).toBeNull();
    });
  });
});
