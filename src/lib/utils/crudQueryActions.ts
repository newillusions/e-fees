/**
 * Query action factories for CRUD stores.
 *
 * Extracted from crud.ts — these actions mutate filteredItems
 * via search, filter, sort, and reset operations.
 */

import { get, type Writable } from 'svelte/store';
import { logApiError } from '../services/logger';
import { applySorting } from './crudPipeline';
import type { CrudState, CrudApi } from './crudTypes';
import type { UnknownSurrealThing } from '../../types';

type IdExtractor = (id: UnknownSurrealThing | undefined) => string | null;
type FilterFn<T> = (items: T[], searchQuery: string, filters: Record<string, unknown>) => T[];
type Logger = { info: (msg: string, ctx?: Record<string, unknown>) => void } | null;

interface QueryDeps<T> {
  store: Writable<CrudState<T>>;
  api: CrudApi<T>;
  filterAndSearch: FilterFn<T>;
  idExtractor: IdExtractor;
  componentLogger: Logger;
  enableLogging: boolean;
  component: string;
}

export interface QueryActions<T> {
  search: (query: string) => Promise<void>;
  applyFilters: (filters: Record<string, unknown>) => Promise<void>;
  sort: (field: string, direction?: 'asc' | 'desc') => void;
  resetFilters: () => void;
  rollback: () => void;
  getById: (id: string) => T | null;
}

export function createQueryActions<T extends { id?: UnknownSurrealThing }>(
  deps: QueryDeps<T>,
): QueryActions<T> {
  const { store, api, filterAndSearch, idExtractor, componentLogger, enableLogging, component } = deps;

  return {
    async search(query: string) {
      componentLogger?.info('Searching entities', { query });

      if (api.search && query.trim()) {
        try {
          store.update(state => ({ ...state, searchQuery: query, loading: true }));
          const searchResults = await api.search(query);
          store.update(state => ({
            ...state,
            filteredItems: applySorting(filterAndSearch(searchResults, query, state.filters), state.sort),
            loading: false,
          }));
        } catch (error) {
          if (enableLogging) await logApiError('search', error as Error, { component, query });
          store.update(state => ({
            ...state,
            error: 'Search unavailable — showing cached results',
            filteredItems: applySorting(filterAndSearch(state.items, query, state.filters), state.sort),
            loading: false,
          }));
        }
      } else {
        store.update(state => ({
          ...state,
          searchQuery: query,
          filteredItems: applySorting(filterAndSearch(state.items, query, state.filters), state.sort),
        }));
      }
    },

    async applyFilters(filters: Record<string, unknown>) {
      componentLogger?.info('Applying filters', { filters });

      if (api.filter && Object.keys(filters).length > 0) {
        try {
          store.update(state => ({ ...state, loading: true, filters }));
          const filterResults = await api.filter(filters);
          store.update(state => ({
            ...state,
            filteredItems: applySorting(filterAndSearch(filterResults, state.searchQuery, filters), state.sort),
            loading: false,
          }));
        } catch (error) {
          if (enableLogging) await logApiError('filter', error as Error, { component, filters });
          store.update(state => ({
            ...state,
            error: 'Filter unavailable — showing cached results',
            filters,
            filteredItems: applySorting(filterAndSearch(state.items, state.searchQuery, filters), state.sort),
            loading: false,
          }));
        }
      } else {
        store.update(state => ({
          ...state,
          filters,
          filteredItems: applySorting(filterAndSearch(state.items, state.searchQuery, filters), state.sort),
        }));
      }
    },

    sort(field: string, direction: 'asc' | 'desc' = 'asc') {
      componentLogger?.info('Sorting entities', { field, direction });
      store.update(state => {
        const sort = { field, direction };
        return {
          ...state,
          sort,
          filteredItems: applySorting(filterAndSearch(state.items, state.searchQuery, state.filters), sort),
        };
      });
    },

    resetFilters() {
      componentLogger?.info('Resetting filters and search');
      store.update(state => ({
        ...state,
        searchQuery: '',
        filters: {},
        sort: null,
        filteredItems: [...state.items],
      }));
    },

    rollback() {
      componentLogger?.info('Rolling back optimistic updates');
      store.update(state => {
        const items = [...state.items];
        state.optimisticUpdates.forEach((originalItem, id) => {
          const index = items.findIndex(item => idExtractor(item.id) === id);
          if (index !== -1) {
            if (id.startsWith('temp_')) {
              items.splice(index, 1);
            } else {
              items[index] = originalItem;
            }
          }
        });
        return {
          ...state,
          items,
          filteredItems: applySorting(filterAndSearch(items, state.searchQuery, state.filters), state.sort),
          optimisticUpdates: new Map(),
        };
      });
    },

    getById(id: string): T | null {
      const currentState = get(store);
      return currentState.items.find(item => idExtractor(item.id) === id) || null;
    },
  };
}
