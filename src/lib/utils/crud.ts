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
      
      // Create temporary optimistic item for immediate UI feedback
      let optimisticItem: T | null = null;
      const tempId = `temp_${Date.now()}`;
      
      if (enableOptimistic) {
        optimisticItem = { ...data, id: tempId } as T;
        store.update(state => {
          const newItems = [...state.items, optimisticItem!];
          const newFilteredItems = applySorting(
            filterAndSearch(newItems, state.searchQuery, state.filters),
            state.sort
          );
          return {
            ...state,
            items: newItems,
            filteredItems: newFilteredItems,
            optimisticUpdates: new Map(state.optimisticUpdates).set(tempId, optimisticItem!)
          };
        });
      }
      
      try {
        const newItem = await api.create(data);
        
        store.update(state => {
          const items = [...state.items];
          
          if (enableOptimistic && optimisticItem) {
            // Replace optimistic item with real item
            const tempIndex = items.findIndex(item => idExtractor(item.id) === tempId);
            if (tempIndex !== -1) {
              items[tempIndex] = newItem;
            } else {
              // Optimistic item was rolled back, add real item
              items.push(newItem);
            }
          } else {
            items.push(newItem);
          }
          
          const filteredItems = applySorting(
            filterAndSearch(items, state.searchQuery, state.filters),
            state.sort
          );
          
          const newOptimisticUpdates = new Map(state.optimisticUpdates);
          newOptimisticUpdates.delete(tempId);
          
          return {
            ...state,
            items,
            filteredItems,
            saving: false,
            lastUpdated: new Date(),
            optimisticUpdates: newOptimisticUpdates
          };
        });
        
        componentLogger?.info('Successfully created entity', { id: idExtractor(newItem.id) });
        return newItem;
      } catch (error) {
        // Rollback optimistic update on error
        if (enableOptimistic && optimisticItem) {
          store.update(state => {
            const items = state.items.filter(item => idExtractor(item.id) !== tempId);
            const filteredItems = applySorting(
              filterAndSearch(items, state.searchQuery, state.filters),
              state.sort
            );
            const newOptimisticUpdates = new Map(state.optimisticUpdates);
            newOptimisticUpdates.delete(tempId);
            
            return {
              ...state,
              items,
              filteredItems,
              optimisticUpdates: newOptimisticUpdates
            };
          });
        }
        
        const errorMessage = getErrorMessage(error, 'Failed to create item');
        store.update(state => ({ 
          ...state, 
          saving: false, 
          error: errorMessage 
        }));
        
        if (enableLogging) {
          await logApiError('create', error as Error, { component, data });
        }
        throw error;
      }
    },

    async update(id, data) {
      store.update(state => ({ ...state, saving: true, error: null }));
      componentLogger?.info('Updating entity', { id, data });
      
      // Store original item for rollback
      let originalItem: T | null = null;
      let optimisticItem: T | null = null;
      
      if (enableOptimistic) {
        store.update(state => {
          const itemIndex = state.items.findIndex(item => {
            const itemId = idExtractor(item.id);
            return itemId === id;
          });
          
          if (itemIndex !== -1) {
            originalItem = state.items[itemIndex];
            optimisticItem = { ...originalItem, ...data };
            
            const newItems = [...state.items];
            newItems[itemIndex] = optimisticItem;
            
            const filteredItems = applySorting(
              filterAndSearch(newItems, state.searchQuery, state.filters),
              state.sort
            );
            
            return {
              ...state,
              items: newItems,
              filteredItems,
              optimisticUpdates: new Map(state.optimisticUpdates).set(id, optimisticItem)
            };
          }
          return state;
        });
      }
      
      try {
        const updatedItem = await api.update(id, data);
        
        store.update(state => {
          const items = state.items.map(item => {
            const itemId = idExtractor(item.id);
            return itemId === id ? updatedItem : item;
          });
          
          const filteredItems = applySorting(
            filterAndSearch(items, state.searchQuery, state.filters),
            state.sort
          );
          
          const newOptimisticUpdates = new Map(state.optimisticUpdates);
          newOptimisticUpdates.delete(id);
          
          return {
            ...state,
            items,
            filteredItems,
            saving: false,
            lastUpdated: new Date(),
            optimisticUpdates: newOptimisticUpdates
          };
        });
        
        componentLogger?.info('Successfully updated entity', { id });
        return updatedItem;
      } catch (error) {
        // Rollback optimistic update on error
        if (enableOptimistic && originalItem) {
          store.update(state => {
            const items = state.items.map(item => {
              const itemId = idExtractor(item.id);
              return itemId === id ? originalItem! : item;
            });
            
            const filteredItems = applySorting(
              filterAndSearch(items, state.searchQuery, state.filters),
              state.sort
            );
            
            const newOptimisticUpdates = new Map(state.optimisticUpdates);
            newOptimisticUpdates.delete(id);
            
            return {
              ...state,
              items,
              filteredItems,
              optimisticUpdates: newOptimisticUpdates
            };
          });
        }
        
        const errorMessage = getErrorMessage(error, 'Failed to update item');
        store.update(state => ({ 
          ...state, 
          saving: false, 
          error: errorMessage 
        }));
        
        if (enableLogging) {
          await logApiError('update', error as Error, { component, id, data });
        }
        throw error;
      }
    },

    async delete(id) {
      store.update(state => ({ ...state, saving: true, error: null }));
      componentLogger?.info('Deleting entity', { id });
      
      // Store item for rollback
      let deletedItem: T | null = null;
      let itemIndex = -1;
      
      if (enableOptimistic) {
        store.update(state => {
          itemIndex = state.items.findIndex(item => {
            const itemId = idExtractor(item.id);
            return itemId === id;
          });
          
          if (itemIndex !== -1) {
            deletedItem = state.items[itemIndex];
            const newItems = state.items.filter(item => {
              const itemId = idExtractor(item.id);
              return itemId !== id;
            });
            
            const filteredItems = applySorting(
              filterAndSearch(newItems, state.searchQuery, state.filters),
              state.sort
            );
            
            return {
              ...state,
              items: newItems,
              filteredItems,
              optimisticUpdates: new Map(state.optimisticUpdates).set(id, deletedItem!)
            };
          }
          return state;
        });
      }
      
      try {
        const result = await api.delete(id);
        
        store.update(state => {
          const items = state.items.filter(item => {
            const itemId = idExtractor(item.id);
            return itemId !== id;
          });
          
          const filteredItems = applySorting(
            filterAndSearch(items, state.searchQuery, state.filters),
            state.sort
          );
          
          const newOptimisticUpdates = new Map(state.optimisticUpdates);
          newOptimisticUpdates.delete(id);
          
          return {
            ...state,
            items,
            filteredItems,
            saving: false,
            lastUpdated: new Date(),
            optimisticUpdates: newOptimisticUpdates
          };
        });
        
        componentLogger?.info('Successfully deleted entity', { id });
        return result;
      } catch (error) {
        // Rollback optimistic delete on error
        if (enableOptimistic && deletedItem && itemIndex !== -1) {
          store.update(state => {
            const newItems = [...state.items];
            newItems.splice(itemIndex, 0, deletedItem!);
            
            const filteredItems = applySorting(
              filterAndSearch(newItems, state.searchQuery, state.filters),
              state.sort
            );
            
            const newOptimisticUpdates = new Map(state.optimisticUpdates);
            newOptimisticUpdates.delete(id);
            
            return {
              ...state,
              items: newItems,
              filteredItems,
              optimisticUpdates: newOptimisticUpdates
            };
          });
        }
        
        const errorMessage = getErrorMessage(error, 'Failed to delete item');
        store.update(state => ({ 
          ...state, 
          saving: false, 
          error: errorMessage 
        }));
        
        if (enableLogging) {
          await logApiError('delete', error as Error, { component, id });
        }
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

    async search(query) {
      componentLogger?.info('Searching entities', { query });
      store.update(state => ({ ...state, searchQuery: query }));
      
      if (api.search && query.trim()) {
        try {
          store.update(state => ({ ...state, loading: true }));
          const searchResults = await api.search(query);
          store.update(state => {
            const filteredItems = applySorting(
              filterAndSearch(searchResults, query, state.filters),
              state.sort
            );
            return {
              ...state,
              filteredItems,
              loading: false
            };
          });
        } catch (error) {
          if (enableLogging) {
            await logApiError('search', error as Error, { component, query });
          }
          // Fall back to client-side search
          store.update(state => {
            const filteredItems = applySorting(
              filterAndSearch(state.items, query, state.filters),
              state.sort
            );
            return {
              ...state,
              filteredItems,
              loading: false
            };
          });
        }
      } else {
        // Client-side search
        store.update(state => {
          const filteredItems = applySorting(
            filterAndSearch(state.items, query, state.filters),
            state.sort
          );
          return {
            ...state,
            filteredItems
          };
        });
      }
    },

    async applyFilters(filters) {
      componentLogger?.info('Applying filters', { filters });
      
      if (api.filter && Object.keys(filters).length > 0) {
        try {
          store.update(state => ({ ...state, loading: true, filters }));
          const filterResults = await api.filter(filters);
          store.update(state => {
            const filteredItems = applySorting(
              filterAndSearch(filterResults, state.searchQuery, filters),
              state.sort
            );
            return {
              ...state,
              filteredItems,
              loading: false
            };
          });
        } catch (error) {
          if (enableLogging) {
            await logApiError('filter', error as Error, { component, filters });
          }
          // Fall back to client-side filtering
          store.update(state => {
            const filteredItems = applySorting(
              filterAndSearch(state.items, state.searchQuery, filters),
              state.sort
            );
            return {
              ...state,
              filters,
              filteredItems,
              loading: false
            };
          });
        }
      } else {
        // Client-side filtering
        store.update(state => {
          const filteredItems = applySorting(
            filterAndSearch(state.items, state.searchQuery, filters),
            state.sort
          );
          return {
            ...state,
            filters,
            filteredItems
          };
        });
      }
    },

    sort(field, direction = 'asc') {
      componentLogger?.info('Sorting entities', { field, direction });
      store.update(state => {
        const sort = { field, direction };
        const filteredItems = applySorting(
          filterAndSearch(state.items, state.searchQuery, state.filters),
          sort
        );
        return {
          ...state,
          sort,
          filteredItems
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
        filteredItems: [...state.items]
      }));
    },

    rollback() {
      componentLogger?.info('Rolling back optimistic updates');
      store.update(state => {
        // Revert all optimistic updates
        const items = [...state.items];
        state.optimisticUpdates.forEach((originalItem, id) => {
          const index = items.findIndex(item => idExtractor(item.id) === id);
          if (index !== -1) {
            if (id.startsWith('temp_')) {
              // Remove temporary items
              items.splice(index, 1);
            } else {
              // Restore original items
              items[index] = originalItem;
            }
          }
        });
        
        const filteredItems = applySorting(
          filterAndSearch(items, state.searchQuery, state.filters),
          state.sort
        );
        
        return {
          ...state,
          items,
          filteredItems,
          optimisticUpdates: new Map()
        };
      });
    },

    getById(id) {
      const currentState = get(store);

      return currentState.items.find(item => {
        const itemId = idExtractor(item.id);
        return itemId === id;
      }) || null;
    }
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