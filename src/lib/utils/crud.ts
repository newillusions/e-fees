/**
 * Enhanced Generic CRUD Utilities
 * 
 * This module provides comprehensive CRUD operations and state management
 * patterns with SurrealDB support, optimistic updates, and professional logging.
 * It eliminates duplicate API call patterns and provides consistent error handling.
 * 
 * Key Features:
 * - SurrealDB Thing object support
 * - Optimistic updates with rollback
 * - Professional logging integration
 * - Filtering, searching, and sorting
 * - Type-safe generic interfaces
 * - Comprehensive error handling
 */

import { writable, get } from 'svelte/store';
import { extractSurrealId, compareSurrealIds } from './surrealdb';
import { logger, logApiError, type LogContext } from '../services/logger';
import { applyFiltersAndSearch, applySorting } from './crudPipeline';
import { getErrorMessage } from './operationState';
import { optimisticCreate, optimisticUpdate, optimisticDelete, recomputeFiltered } from './crudOptimistic';
import { createQueryActions } from './crudQueryActions';
import type { UnknownSurrealThing } from '../../types';

// Re-export all types from crudTypes for backward compatibility
export type {
  CrudState,
  CrudApi,
  CrudStoreOptions,
  CrudActions,
  CrudStoreInterface,
  SurrealEntity,
  ModalState,
  ModalActions,
  OperationState,
  OperationActions,
  EntityId,
  FormData,
  UpdateData,
  PaginatedResult,
  SearchResultSet,
} from './crudTypes';

// Re-export extracted utilities for backward compatibility
export { useModalState } from './modalState';
export { useOperationState, withLoadingState, getErrorMessage } from './operationState';

import type {
  CrudState,
  CrudApi,
  CrudStoreOptions,
  CrudActions,
  CrudStoreInterface,
} from './crudTypes';

// ============================================================================
// ENHANCED CRUD STORE IMPLEMENTATION
// ============================================================================

/**
 * Enhanced CRUD store with SurrealDB support and optimistic updates.
 */
export function useCrudStore<T extends { id?: UnknownSurrealThing }>(
  api: CrudApi<T>,
  options: CrudStoreOptions = {}
): CrudStoreInterface<T> {
  const {
    enableOptimistic = true,
    enableLogging = true,
    component = 'CrudStore',
    idExtractor = extractSurrealId,
    autoRefresh,
    searchFields
  } = options as CrudStoreOptions<T>;

  const componentLogger = enableLogging ? logger.child({ component }) : null;

  const initialState: CrudState<T> = {
    items: [],
    filteredItems: [],
    loading: false,
    error: null,
    saving: false,
    searchQuery: '',
    filters: {},
    sort: null,
    lastUpdated: null,
    optimisticUpdates: new Map()
  };

  const store = writable(initialState);

  let autoRefreshInterval: NodeJS.Timeout | null = null;

  // Setup auto-refresh if enabled
  if (autoRefresh && autoRefresh > 0) {
    autoRefreshInterval = setInterval(() => {
      actions.load().catch(error => {
        componentLogger?.error('Auto-refresh failed', { error });
      });
    }, autoRefresh);
  }

  // Wrap imported pipeline functions to pass searchFields from closure
  const filterAndSearch = (items: T[], searchQuery: string, filters: Record<string, unknown>): T[] =>
    applyFiltersAndSearch(items, searchQuery, filters, searchFields as (keyof T)[] | undefined);

  const actions: CrudActions<T> = {
    async load() {
      store.update(state => ({ ...state, loading: true, error: null }));
      componentLogger?.info('Loading entities');
      
      try {
        const items = await api.getAll();
        const stateValue = get(store);

        const filteredItems = applySorting(
          filterAndSearch(items, stateValue.searchQuery, stateValue.filters),
          stateValue.sort
        );
        
        store.update(state => ({ 
          ...state, 
          items, 
          filteredItems,
          loading: false,
          lastUpdated: new Date(),
          optimisticUpdates: new Map() // Clear optimistic updates on fresh load
        }));
        
        componentLogger?.info('Successfully loaded entities', { count: items.length });
      } catch (error) {
        const errorMessage = getErrorMessage(error, 'Failed to load data');
        store.update(state => ({ 
          ...state, 
          loading: false, 
          error: errorMessage 
        }));
        
        if (enableLogging) {
          await logApiError('load', error as Error, { component });
        }
        throw error;
      }
    },

    async create(data) {
      store.update(state => ({ ...state, saving: true, error: null }));
      componentLogger?.info('Creating new entity', { data });

      const opt = enableOptimistic
        ? optimisticCreate(store, data, idExtractor, filterAndSearch, applySorting)
        : null;

      try {
        const newItem = await api.create(data);

        if (opt) {
          opt.commit(newItem);
        } else {
          store.update(state => {
            const items = [...state.items, newItem];
            return {
              ...state,
              items,
              filteredItems: applySorting(filterAndSearch(items, state.searchQuery, state.filters), state.sort),
            };
          });
        }

        store.update(state => ({ ...state, saving: false, lastUpdated: new Date() }));
        componentLogger?.info('Successfully created entity', { id: idExtractor(newItem.id) });
        return newItem;
      } catch (error) {
        opt?.rollback();
        const errorMessage = getErrorMessage(error, 'Failed to create item');
        store.update(state => ({ ...state, saving: false, error: errorMessage }));
        if (enableLogging) await logApiError('create', error as Error, { component, data });
        throw error;
      }
    },

    async update(id, data) {
      store.update(state => ({ ...state, saving: true, error: null }));
      componentLogger?.info('Updating entity', { id, data });

      const opt = enableOptimistic
        ? optimisticUpdate(store, id, data, idExtractor, filterAndSearch, applySorting)
        : null;

      try {
        const updatedItem = await api.update(id, data);

        if (opt) {
          opt.commit(updatedItem);
        } else {
          store.update(state => {
            const items = state.items.map(item =>
              idExtractor(item.id) === id ? updatedItem : item,
            );
            return {
              ...state,
              items,
              filteredItems: applySorting(filterAndSearch(items, state.searchQuery, state.filters), state.sort),
            };
          });
        }

        store.update(state => ({ ...state, saving: false, lastUpdated: new Date() }));
        componentLogger?.info('Successfully updated entity', { id });
        return updatedItem;
      } catch (error) {
        opt?.rollback();
        const errorMessage = getErrorMessage(error, 'Failed to update item');
        store.update(state => ({ ...state, saving: false, error: errorMessage }));
        if (enableLogging) await logApiError('update', error as Error, { component, id, data });
        throw error;
      }
    },

    async delete(id) {
      store.update(state => ({ ...state, saving: true, error: null }));
      componentLogger?.info('Deleting entity', { id });

      const opt = enableOptimistic
        ? optimisticDelete(store, id, idExtractor, filterAndSearch, applySorting)
        : null;

      try {
        const result = await api.delete(id);

        if (opt) {
          opt.commit();
        } else {
          store.update(state => {
            const items = state.items.filter(item => idExtractor(item.id) !== id);
            return {
              ...state,
              items,
              filteredItems: applySorting(filterAndSearch(items, state.searchQuery, state.filters), state.sort),
            };
          });
        }

        store.update(state => ({ ...state, saving: false, lastUpdated: new Date() }));
        componentLogger?.info('Successfully deleted entity', { id });
        return result;
      } catch (error) {
        opt?.rollback();
        const errorMessage = getErrorMessage(error, 'Failed to delete item');
        store.update(state => ({ ...state, saving: false, error: errorMessage }));
        if (enableLogging) await logApiError('delete', error as Error, { component, id });
        throw error;
      }
    },

    async refresh() {
      componentLogger?.info('Refreshing entities');
      return actions.load();
    },

    clear() {
      componentLogger?.info('Clearing all data');
      if (autoRefreshInterval) {
        clearInterval(autoRefreshInterval);
        autoRefreshInterval = null;
      }
      store.set(initialState);
    },

    setError(error) {
      store.update(state => ({ ...state, error }));
      if (error) {
        componentLogger?.error('Error set', { error });
      }
    },

    ...createQueryActions({
      store, api, filterAndSearch, idExtractor,
      componentLogger, enableLogging, component,
    })
  };

  // Cleanup function
  const destroy = () => {
    if (autoRefreshInterval) {
      clearInterval(autoRefreshInterval);
    }
  };

  return { store, actions, destroy };
}


// ============================================================================
// SURREALDB-SPECIFIC UTILITIES
// ============================================================================

/**
 * Type guard for SurrealDB Thing objects (v2: {tb, id} or v3: {table, key}).
 */
function isSurrealThingLike(id: unknown): id is { tb?: unknown; id?: unknown; table?: unknown; key?: unknown } {
  if (typeof id !== 'object' || id === null) return false;
  return ('tb' in id && 'id' in id) || ('table' in id && 'key' in id);
}

/**
 * Validates SurrealDB ID format (v2 and v3).
 */
export function validateSurrealId(id: unknown): boolean {
  if (!id) return false;

  if (typeof id === 'string') {
    return id.length > 0;
  }

  if (isSurrealThingLike(id)) {
    // v3 format
    if ('table' in id && 'key' in id) return !!id.table && !!id.key;
    // v2 format
    return !!id.tb && !!id.id;
  }

  return false;
}

// ============================================================================
// GENERIC API CLIENT BASE CLASS
// ============================================================================

/**
 * Base class for CRUD API implementations.
 */
export abstract class BaseCrudApi<T> implements CrudApi<T> {
  protected entityName: string;
  protected logger: ReturnType<typeof logger.child>;

  constructor(entityName: string) {
    this.entityName = entityName;
    this.logger = logger.child({ component: `${entityName}Api` });
  }

  abstract getAll(): Promise<T[]>;
  abstract create(data: Omit<T, 'id'>): Promise<T>;
  abstract update(id: string, data: Partial<T>): Promise<T>;
  abstract delete(id: string): Promise<T>;

  async search?(query: string): Promise<T[]> {
    this.logger.warn('Search not implemented for this entity', { entityName: this.entityName });
    throw new Error(`Search not implemented for ${this.entityName}`);
  }

  async filter?(criteria: Record<string, unknown>): Promise<T[]> {
    this.logger.warn('Filter not implemented for this entity', { entityName: this.entityName });
    throw new Error(`Filter not implemented for ${this.entityName}`);
  }

  /**
   * Helper method for handling API errors.
   */
  protected handleError(operation: string, error: unknown, context?: LogContext): never {
    const err = error instanceof Error ? error : new Error(String(error));
    this.logger.error(`${operation} failed`, { ...context, error: err.message });
    throw err;
  }

  /**
   * Helper method for logging successful operations.
   */
  protected logSuccess(operation: string, result: unknown, context?: LogContext): void {
    this.logger.info(`${operation} successful`, { ...context, result: !!result });
  }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Creates a debounced function for search operations.
 */
export function createDebounced<T extends unknown[]>(
  func: (...args: T) => void,
  delay: number
): (...args: T) => void {
  let timeoutId: NodeJS.Timeout;
  return (...args: T) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => func(...args), delay);
  };
}

/**
 * Creates a throttled function for frequent operations.
 */
export function createThrottled<T extends unknown[]>(
  func: (...args: T) => void,
  delay: number
): (...args: T) => void {
  let lastCall = 0;
  return (...args: T) => {
    const now = Date.now();
    if (now - lastCall >= delay) {
      lastCall = now;
      func(...args);
    }
  };
}

/**
 * Helper function to create a pre-configured CRUD store for common entity types.
 */
export function createEntityStore<T extends { id?: UnknownSurrealThing }>(
  api: CrudApi<T>,
  entityName: string,
  options: Partial<CrudStoreOptions> = {}
): CrudStoreInterface<T> {
  return useCrudStore(api, {
    component: `${entityName}Store`,
    enableOptimistic: true,
    enableLogging: true,
    ...options
  });
}