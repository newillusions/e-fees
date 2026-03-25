/**
 * Optimistic update helpers for CRUD operations.
 *
 * Each function applies an immediate state change and returns
 * commit/rollback handlers so the caller can finalize or revert
 * after the async API call resolves.
 */

import { type Writable } from 'svelte/store';
import type { CrudState } from './crudTypes';
import type { UnknownSurrealThing } from '../../types';

type IdExtractor = (id: UnknownSurrealThing | undefined) => string | null;
type FilterFn<T> = (items: T[], searchQuery: string, filters: Record<string, unknown>) => T[];
type SortFn<T> = (items: T[], sort: CrudState<T>['sort']) => T[];

/** Recompute filteredItems from current state. */
export function recomputeFiltered<T>(
  store: Writable<CrudState<T>>,
  filterAndSearch: FilterFn<T>,
  applySorting: SortFn<T>,
): void {
  store.update(state => ({
    ...state,
    filteredItems: applySorting(
      filterAndSearch(state.items, state.searchQuery, state.filters),
      state.sort,
    ),
  }));
}

function recompute<T>(
  items: T[],
  state: CrudState<T>,
  filterAndSearch: FilterFn<T>,
  applySorting: SortFn<T>,
): T[] {
  return applySorting(
    filterAndSearch(items, state.searchQuery, state.filters),
    state.sort,
  );
}

// ---------------------------------------------------------------------------
// optimisticCreate
// ---------------------------------------------------------------------------

export function optimisticCreate<T extends { id?: UnknownSurrealThing }>(
  store: Writable<CrudState<T>>,
  data: Omit<T, 'id'>,
  idExtractor: IdExtractor,
  filterAndSearch: FilterFn<T>,
  applySorting: SortFn<T>,
): { tempId: string; commit: (realItem: T) => void; rollback: () => void } {
  const tempId = `temp_${Date.now()}`;
  const optimisticItem = { ...data, id: tempId } as T;

  store.update(state => {
    const newItems = [...state.items, optimisticItem];
    return {
      ...state,
      items: newItems,
      filteredItems: recompute(newItems, state, filterAndSearch, applySorting),
      optimisticUpdates: new Map(state.optimisticUpdates).set(tempId, optimisticItem),
    };
  });

  return {
    tempId,
    commit(realItem: T) {
      store.update(state => {
        const items = [...state.items];
        const tempIndex = items.findIndex(item => idExtractor(item.id) === tempId);
        if (tempIndex !== -1) {
          items[tempIndex] = realItem;
        } else {
          items.push(realItem);
        }
        const updates = new Map(state.optimisticUpdates);
        updates.delete(tempId);
        return {
          ...state,
          items,
          filteredItems: recompute(items, state, filterAndSearch, applySorting),
          optimisticUpdates: updates,
        };
      });
    },
    rollback() {
      store.update(state => {
        const items = state.items.filter(item => idExtractor(item.id) !== tempId);
        const updates = new Map(state.optimisticUpdates);
        updates.delete(tempId);
        return {
          ...state,
          items,
          filteredItems: recompute(items, state, filterAndSearch, applySorting),
          optimisticUpdates: updates,
        };
      });
    },
  };
}

// ---------------------------------------------------------------------------
// optimisticUpdate
// ---------------------------------------------------------------------------

export function optimisticUpdate<T extends { id?: UnknownSurrealThing }>(
  store: Writable<CrudState<T>>,
  id: string,
  data: Partial<T>,
  idExtractor: IdExtractor,
  filterAndSearch: FilterFn<T>,
  applySorting: SortFn<T>,
): { originalItem: T | null; commit: (updatedItem: T) => void; rollback: () => void } {
  let originalItem: T | null = null;

  store.update(state => {
    const idx = state.items.findIndex(item => idExtractor(item.id) === id);
    if (idx === -1) return state;

    originalItem = state.items[idx];
    const merged = { ...originalItem, ...data };
    const newItems = [...state.items];
    newItems[idx] = merged;

    return {
      ...state,
      items: newItems,
      filteredItems: recompute(newItems, state, filterAndSearch, applySorting),
      optimisticUpdates: new Map(state.optimisticUpdates).set(id, originalItem),
    };
  });

  return {
    originalItem,
    commit(updatedItem: T) {
      store.update(state => {
        const items = state.items.map(item =>
          idExtractor(item.id) === id ? updatedItem : item,
        );
        const updates = new Map(state.optimisticUpdates);
        updates.delete(id);
        return {
          ...state,
          items,
          filteredItems: recompute(items, state, filterAndSearch, applySorting),
          optimisticUpdates: updates,
        };
      });
    },
    rollback() {
      if (!originalItem) return;
      store.update(state => {
        const items = state.items.map(item =>
          idExtractor(item.id) === id ? originalItem! : item,
        );
        const updates = new Map(state.optimisticUpdates);
        updates.delete(id);
        return {
          ...state,
          items,
          filteredItems: recompute(items, state, filterAndSearch, applySorting),
          optimisticUpdates: updates,
        };
      });
    },
  };
}

// ---------------------------------------------------------------------------
// optimisticDelete
// ---------------------------------------------------------------------------

export function optimisticDelete<T extends { id?: UnknownSurrealThing }>(
  store: Writable<CrudState<T>>,
  id: string,
  idExtractor: IdExtractor,
  filterAndSearch: FilterFn<T>,
  applySorting: SortFn<T>,
): { deletedItem: T | null; itemIndex: number; commit: () => void; rollback: () => void } {
  let deletedItem: T | null = null;
  let itemIndex = -1;

  store.update(state => {
    itemIndex = state.items.findIndex(item => idExtractor(item.id) === id);
    if (itemIndex === -1) return state;

    deletedItem = state.items[itemIndex];
    const newItems = state.items.filter(item => idExtractor(item.id) !== id);

    return {
      ...state,
      items: newItems,
      filteredItems: recompute(newItems, state, filterAndSearch, applySorting),
      optimisticUpdates: new Map(state.optimisticUpdates).set(id, deletedItem),
    };
  });

  return {
    deletedItem,
    itemIndex,
    commit() {
      store.update(state => {
        const items = state.items.filter(item => idExtractor(item.id) !== id);
        const updates = new Map(state.optimisticUpdates);
        updates.delete(id);
        return {
          ...state,
          items,
          filteredItems: recompute(items, state, filterAndSearch, applySorting),
          optimisticUpdates: updates,
        };
      });
    },
    rollback() {
      if (!deletedItem || itemIndex === -1) return;
      store.update(state => {
        const newItems = [...state.items];
        newItems.splice(itemIndex, 0, deletedItem!);
        const updates = new Map(state.optimisticUpdates);
        updates.delete(id);
        return {
          ...state,
          items: newItems,
          filteredItems: recompute(newItems, state, filterAndSearch, applySorting),
          optimisticUpdates: updates,
        };
      });
    },
  };
}
