/**
 * Generic CRUD Utilities
 * 
 * This module provides standardized CRUD operations and state management
 * patterns that can be reused across all entity types. It eliminates
 * duplicate API call patterns and provides consistent error handling.
 */

import { writable, type Writable } from 'svelte/store';

/**
 * Generic CRUD state interface.
 */
export interface CrudState<T> {
  /** Array of entities */
  items: T[];
  /** Loading state for read operations */
  loading: boolean;
  /** Error message if any operation fails */
  error: string | null;
  /** Loading state for create/update/delete operations */
  saving: boolean;
}

/**
 * Generic CRUD API interface that must be implemented for each entity type.
 */
export interface CrudApi<T> {
  /** Fetch all entities */
  getAll(): Promise<T[]>;
  /** Create a new entity */
  create(data: Omit<T, 'id'>): Promise<T>;
  /** Update an existing entity */
  update(id: string, data: Partial<T>): Promise<T>;
  /** Delete an entity */
  delete(id: string): Promise<T>;
}

/**
 * CRUD actions interface returned by useCrudStore.
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
}

/**
 * Creates a standardized CRUD store with actions for any entity type.
 * 
 * @param api - The CRUD API implementation for the entity
 * @param idExtractor - Function to extract ID from entity (handles SurrealDB Things)
 * @returns Object containing the store and actions
 * 
 * @example
 * const { store, actions } = useCrudStore(companyApi, extractSurrealId);
 * 
 * // Load data
 * await actions.load();
 * 
 * // Create new entity
 * const newCompany = await actions.create({ name: 'Test Co' });
 * 
 * // Update entity
 * await actions.update('123', { name: 'Updated Name' });
 */
export function useCrudStore<T extends { id: any }>(
  api: CrudApi<T>,
  idExtractor: (id: any) => string | null
) {
  const initialState: CrudState<T> = {
    items: [],
    loading: false,
    error: null,
    saving: false,
  };

  const store = writable(initialState);

  const actions: CrudActions<T> = {
    async load() {
      store.update(state => ({ ...state, loading: true, error: null }));
      
      try {
        const items = await api.getAll();
        store.update(state => ({ ...state, items, loading: false }));
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to load data';
        store.update(state => ({ 
          ...state, 
          loading: false, 
          error: errorMessage 
        }));
        throw error;
      }
    },

    async create(data) {
      store.update(state => ({ ...state, saving: true, error: null }));
      
      try {
        const newItem = await api.create(data);
        store.update(state => ({
          ...state,
          items: [...state.items, newItem],
          saving: false
        }));
        return newItem;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to create item';
        store.update(state => ({ 
          ...state, 
          saving: false, 
          error: errorMessage 
        }));
        throw error;
      }
    },

    async update(id, data) {
      store.update(state => ({ ...state, saving: true, error: null }));
      
      try {
        const updatedItem = await api.update(id, data);
        store.update(state => ({
          ...state,
          items: state.items.map(item => {
            const itemId = idExtractor(item.id);
            return itemId === id ? updatedItem : item;
          }),
          saving: false
        }));
        return updatedItem;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to update item';
        store.update(state => ({ 
          ...state, 
          saving: false, 
          error: errorMessage 
        }));
        throw error;
      }
    },

    async delete(id) {
      store.update(state => ({ ...state, saving: true, error: null }));
      
      try {
        const deletedItem = await api.delete(id);
        store.update(state => ({
          ...state,
          items: state.items.filter(item => {
            const itemId = idExtractor(item.id);
            return itemId !== id;
          }),
          saving: false
        }));
        return deletedItem;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to delete item';
        store.update(state => ({ 
          ...state, 
          saving: false, 
          error: errorMessage 
        }));
        throw error;
      }
    },

    async refresh() {
      return actions.load();
    },

    clear() {
      store.set(initialState);
    },

    setError(error) {
      store.update(state => ({ ...state, error }));
    }
  };

  return { store, actions };
}

/**
 * Modal state management utilities.
 */
export interface ModalState<T> {
  /** Whether the modal is open */
  isOpen: boolean;
  /** Modal mode: create new or edit existing */
  mode: 'create' | 'edit';
  /** Item being edited (null for create mode) */
  item: T | null;
}

/**
 * Creates standardized modal state management.
 * 
 * @returns Modal store and actions
 */
export function useModalState<T>() {
  const initialState: ModalState<T> = {
    isOpen: false,
    mode: 'create',
    item: null,
  };

  const store = writable(initialState);

  const actions = {
    openCreate() {
      store.set({ isOpen: true, mode: 'create', item: null });
    },

    openEdit(item: T) {
      store.set({ isOpen: true, mode: 'edit', item });
    },

    close() {
      store.set(initialState);
    },
  };

  return { store, actions };
}

/**
 * Common CRUD operation states.
 */
export interface OperationState {
  /** Whether an operation is in progress */
  loading: boolean;
  /** Whether a save operation is in progress */
  saving: boolean;
  /** Whether a delete operation is in progress */
  deleting: boolean;
  /** Success message to display */
  message: string;
  /** Error message to display */
  error: string | null;
}

/**
 * Creates standardized operation state management.
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

  const actions = {
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
 * Utility function to handle common async operations with loading states.
 * 
 * @param operation - The async operation to perform
 * @param operationState - The operation state actions
 * @param loadingType - Which loading state to set ('loading', 'saving', or 'deleting')
 * @returns Result of the operation
 */
export async function withLoadingState<T>(
  operation: () => Promise<T>,
  operationState: ReturnType<typeof useOperationState>['actions'],
  loadingType: 'loading' | 'saving' | 'deleting' = 'loading'
): Promise<T> {
  operationState.clearMessages();
  
  const setLoading = {
    loading: operationState.setLoading,
    saving: operationState.setSaving,
    deleting: operationState.setDeleting,
  }[loadingType];

  setLoading(true);

  try {
    const result = await operation();
    setLoading(false);
    return result;
  } catch (error) {
    setLoading(false);
    const errorMessage = error instanceof Error ? error.message : 'Operation failed';
    operationState.setError(errorMessage);
    throw error;
  }
}

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