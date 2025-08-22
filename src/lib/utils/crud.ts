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

import { writable, type Writable } from 'svelte/store';
import { extractSurrealId, compareSurrealIds } from './surrealdb';
import { logger, logApiError, type LogContext } from '../services/logger';
import type { UnknownSurrealThing } from '../../types';

// ============================================================================
// TYPE DEFINITIONS AND INTERFACES
// ============================================================================

/**
 * Enhanced CRUD state interface with additional features.
 */
export interface CrudState<T> {
  /** Array of entities */
  items: T[];
  /** Filtered items (subset of items) */
  filteredItems: T[];
  /** Loading state for read operations */
  loading: boolean;
  /** Error message if any operation fails */
  error: string | null;
  /** Loading state for create/update/delete operations */
  saving: boolean;
  /** Current search query */
  searchQuery: string;
  /** Current filter criteria */
  filters: Record<string, unknown>;
  /** Current sort configuration */
  sort: {
    field: string;
    direction: 'asc' | 'desc';
  } | null;
  /** Last successful update timestamp */
  lastUpdated: Date | null;
  /** Optimistic update tracking */
  optimisticUpdates: Map<string, T>;
}

/**
 * Enhanced CRUD API interface with SurrealDB support.
 */
export interface CrudApi<T> {
  /** Fetch all entities */
  getAll(): Promise<T[]>;
  /** Search entities with query */
  search?(query: string): Promise<T[]>;
  /** Filter entities with criteria */
  filter?(criteria: Record<string, unknown>): Promise<T[]>;
  /** Create a new entity */
  create(data: Omit<T, 'id'>): Promise<T>;
  /** Update an existing entity */
  update(id: string, data: Partial<T>): Promise<T>;
  /** Delete an entity */
  delete(id: string): Promise<T>;
}

/**
 * Options for configuring CRUD store behavior.
 */
export interface CrudStoreOptions {
  /** Enable optimistic updates */
  enableOptimistic?: boolean;
  /** Enable logging */
  enableLogging?: boolean;
  /** Component name for logging context */
  component?: string;
  /** Custom ID extractor function */
  idExtractor?: (item: any) => string | null;
  /** Auto-refresh interval in milliseconds */
  autoRefresh?: number;
}

/**
 * Enhanced CRUD actions interface with additional operations.
 */
export interface CrudActions<T> {
  /** Load all entities from API */
  load(): Promise<void>;
  /** Create a new entity */
  create(data: Omit<T, 'id'>): Promise<T>;
  /** Update an existing entity */
  update(id: string, data: Partial<T>): Promise<T>;
  /** Delete an entity */
  delete(id: string): Promise<T>;
  /** Refresh data (alias for load) */
  refresh(): Promise<void>;
  /** Clear all data and reset state */
  clear(): void;
  /** Set error message */
  setError(error: string | null): void;
  /** Search entities */
  search(query: string): Promise<void>;
  /** Apply filters */
  applyFilters(filters: Record<string, unknown>): Promise<void>;
  /** Sort entities */
  sort(field: string, direction?: 'asc' | 'desc'): void;
  /** Reset filters and search */
  resetFilters(): void;
  /** Rollback optimistic updates */
  rollback(): void;
  /** Get entity by ID */
  getById(id: string): T | null;
}

/**
 * Complete CRUD store interface returned by useCrudStore.
 */
export interface CrudStoreInterface<T> {
  store: Writable<CrudState<T>>;
  actions: CrudActions<T>;
  destroy: () => void;
}

/**
 * Generic interface for entities with SurrealDB ID support.
 */
export interface SurrealEntity {
  id?: UnknownSurrealThing;
}

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
    autoRefresh
  } = options;

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

  /**
   * Apply current filters and search to items
   */
  const applyFiltersAndSearch = (items: T[], searchQuery: string, filters: Record<string, unknown>): T[] => {
    let filtered = [...items];

    // Apply search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(item => {
        const searchableText = JSON.stringify(item).toLowerCase();
        return searchableText.includes(query);
      });
    }

    // Apply filters
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== null && value !== undefined && value !== '') {
        filtered = filtered.filter(item => {
          const itemValue = (item as any)[key];
          if (typeof value === 'string' && typeof itemValue === 'string') {
            return itemValue.toLowerCase().includes(value.toLowerCase());
          }
          return itemValue === value;
        });
      }
    });

    return filtered;
  };

  /**
   * Apply sorting to items
   */
  const applySorting = (items: T[], sort: { field: string; direction: 'asc' | 'desc' } | null): T[] => {
    if (!sort) return items;

    return [...items].sort((a, b) => {
      const aValue = (a as any)[sort.field];
      const bValue = (b as any)[sort.field];
      
      let comparison = 0;
      if (aValue < bValue) comparison = -1;
      else if (aValue > bValue) comparison = 1;
      
      return sort.direction === 'desc' ? -comparison : comparison;
    });
  };

  const actions: CrudActions<T> = {
    async load() {
      store.update(state => ({ ...state, loading: true, error: null }));
      componentLogger?.info('Loading entities');
      
      try {
        const items = await api.getAll();
        const currentState = store;
        let stateValue: CrudState<T>;
        const unsubscribe = currentState.subscribe(state => stateValue = state);
        unsubscribe();
        
        const filteredItems = applySorting(
          applyFiltersAndSearch(items, stateValue!.searchQuery, stateValue!.filters),
          stateValue!.sort
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
        const errorMessage = error instanceof Error ? error.message : 'Failed to load data';
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
            applyFiltersAndSearch(newItems, state.searchQuery, state.filters),
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
          let items = [...state.items];
          
          if (enableOptimistic && optimisticItem) {
            // Replace optimistic item with real item
            const tempIndex = items.findIndex(item => idExtractor(item.id) === tempId);
            if (tempIndex !== -1) {
              items[tempIndex] = newItem;
            }
          } else {
            items.push(newItem);
          }
          
          const filteredItems = applySorting(
            applyFiltersAndSearch(items, state.searchQuery, state.filters),
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
              applyFiltersAndSearch(items, state.searchQuery, state.filters),
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
        
        const errorMessage = error instanceof Error ? error.message : 'Failed to create item';
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
              applyFiltersAndSearch(newItems, state.searchQuery, state.filters),
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
            applyFiltersAndSearch(items, state.searchQuery, state.filters),
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
              applyFiltersAndSearch(items, state.searchQuery, state.filters),
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
        
        const errorMessage = error instanceof Error ? error.message : 'Failed to update item';
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
              applyFiltersAndSearch(newItems, state.searchQuery, state.filters),
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
            applyFiltersAndSearch(items, state.searchQuery, state.filters),
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
              applyFiltersAndSearch(newItems, state.searchQuery, state.filters),
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
        
        const errorMessage = error instanceof Error ? error.message : 'Failed to delete item';
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
              applyFiltersAndSearch(searchResults, query, state.filters),
              state.sort
            );
            return {
              ...state,
              items: searchResults,
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
              applyFiltersAndSearch(state.items, query, state.filters),
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
            applyFiltersAndSearch(state.items, query, state.filters),
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
              applyFiltersAndSearch(filterResults, state.searchQuery, filters),
              state.sort
            );
            return {
              ...state,
              items: filterResults,
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
              applyFiltersAndSearch(state.items, state.searchQuery, filters),
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
            applyFiltersAndSearch(state.items, state.searchQuery, filters),
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
          applyFiltersAndSearch(state.items, state.searchQuery, state.filters),
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
        let items = [...state.items];
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
          applyFiltersAndSearch(items, state.searchQuery, state.filters),
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
      let currentState: CrudState<T>;
      const unsubscribe = store.subscribe(state => currentState = state);
      unsubscribe();
      
      return currentState!.items.find(item => {
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
// MODAL STATE MANAGEMENT UTILITIES
// ============================================================================

/**
 * Modal state interface for entity operations.
 */
export interface ModalState<T> {
  isOpen: boolean;
  mode: 'create' | 'edit';
  item: T | null;
}

/**
 * Modal actions interface.
 */
export interface ModalActions<T> {
  openCreate(): void;
  openEdit(item: T): void;
  close(): void;
}

/**
 * Creates a modal state store for entity management.
 */
export function useModalState<T>() {
  const initialState: ModalState<T> = {
    isOpen: false,
    mode: 'create',
    item: null,
  };

  const store = writable(initialState);

  const actions: ModalActions<T> = {
    openCreate() {
      store.update(state => ({
        ...state,
        isOpen: true,
        mode: 'create',
        item: null,
      }));
    },

    openEdit(item: T) {
      store.update(state => ({
        ...state,
        isOpen: true,
        mode: 'edit',
        item,
      }));
    },

    close() {
      store.set(initialState);
    },
  };

  return { store, actions };
}

// ============================================================================
// OPERATION STATE MANAGEMENT
// ============================================================================

/**
 * Operation state interface for tracking async operations.
 */
export interface OperationState {
  loading: boolean;
  saving: boolean;
  deleting: boolean;
  message: string;
  error: string | null;
}

/**
 * Operation actions interface.
 */
export interface OperationActions {
  setLoading(loading: boolean): void;
  setSaving(saving: boolean): void;
  setDeleting(deleting: boolean): void;
  setMessage(message: string): void;
  setError(error: string | null): void;
  clearMessages(): void;
  reset(): void;
}

/**
 * Creates an operation state store for tracking async operations.
 */
export function useOperationState() {
  const initialState: OperationState = {
    loading: false,
    saving: false,
    deleting: false,
    message: '',
    error: null,
  };

  const store = writable(initialState);

  const actions: OperationActions = {
    setLoading(loading: boolean) {
      store.update(state => ({ ...state, loading }));
    },

    setSaving(saving: boolean) {
      store.update(state => ({ ...state, saving }));
    },

    setDeleting(deleting: boolean) {
      store.update(state => ({ ...state, deleting }));
    },

    setMessage(message: string) {
      store.update(state => ({ ...state, message, error: null }));
    },

    setError(error: string | null) {
      store.update(state => ({ ...state, error, message: '' }));
    },

    clearMessages() {
      store.update(state => ({ ...state, message: '', error: null }));
    },

    reset() {
      store.set(initialState);
    },
  };

  return { store, actions };
}

/**
 * Wraps an async operation with loading state management.
 */
export async function withLoadingState<T>(
  operation: () => Promise<T>,
  actions: OperationActions,
  loadingType: 'loading' | 'saving' | 'deleting' = 'loading'
): Promise<T> {
  try {
    actions.clearMessages();
    actions[loadingType === 'loading' ? 'setLoading' : loadingType === 'saving' ? 'setSaving' : 'setDeleting'](true);
    
    const result = await operation();
    
    actions[loadingType === 'loading' ? 'setLoading' : loadingType === 'saving' ? 'setSaving' : 'setDeleting'](false);
    return result;
  } catch (error) {
    actions[loadingType === 'loading' ? 'setLoading' : loadingType === 'saving' ? 'setSaving' : 'setDeleting'](false);
    actions.setError(error instanceof Error ? error.message : 'An error occurred');
    throw error;
  }
}

// ============================================================================
// SURREALDB-SPECIFIC UTILITIES
// ============================================================================

/**
 * Validates SurrealDB ID format.
 */
export function validateSurrealId(id: unknown): boolean {
  if (!id) return false;
  
  if (typeof id === 'string') {
    return id.length > 0;
  }
  
  if (typeof id === 'object' && id !== null) {
    const thing = id as any;
    return 'tb' in thing && 'id' in thing && thing.tb && thing.id;
  }
  
  return false;
}

/**
 * Compares two SurrealDB IDs for equality.
 */
export function compareSurrealIdsLocal(id1: UnknownSurrealThing, id2: UnknownSurrealThing): boolean {
  return compareSurrealIds(id1, id2);
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
// TYPE UTILITIES AND INTERFACES
// ============================================================================

/**
 * Type utility for extracting the ID type from an entity.
 */
export type EntityId<T> = T extends { id: infer U } ? U : never;

/**
 * Type utility for creating form data (entity without ID).
 */
export type FormData<T> = Omit<T, 'id'>;

/**
 * Type utility for creating update data (partial entity without ID).
 */
export type UpdateData<T> = Partial<Omit<T, 'id'>>;

/**
 * Interface for paginated results.
 */
export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

/**
 * Interface for search results with metadata.
 */
export interface SearchResult<T> {
  items: T[];
  query: string;
  totalMatches: number;
  searchTime: number;
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