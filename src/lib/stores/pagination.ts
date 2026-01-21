/**
 * Pagination Store Module
 *
 * Provides pagination functionality for lazy loading and infinite scroll patterns.
 * Works alongside existing CRUD stores to enable paginated data loading.
 */

import { writable, get, type Writable } from 'svelte/store';
import { extractSurrealId } from '../utils/surrealdb';
import type { PaginatedResponse, PaginationState } from '../../types';
import type { UnknownSurrealThing } from '../../types';

// ============================================================================
// CONSTANTS
// ============================================================================

/** Default number of records to load per page */
export const DEFAULT_PAGE_SIZE = 50;

/** Delay before starting background loading (ms) */
export const BACKGROUND_LOAD_DELAY = 2000;

/** Delay between loading pages to avoid overwhelming the server (ms) */
export const PAGE_LOAD_DELAY_MS = 100;

// ============================================================================
// PAGINATION STATE INTERFACE
// ============================================================================

/**
 * Extended state interface that includes pagination metadata.
 */
export interface PaginatedStoreState<T> {
  /** All loaded items (accumulated across pages) */
  items: T[];
  /** Current pagination state */
  pagination: PaginationState;
  /** Whether initial load has completed */
  initialized: boolean;
  /** Error message if any */
  error: string | null;
}

/**
 * Actions available for paginated stores.
 */
export interface PaginatedStoreActions<T> {
  /** Load the first page of data */
  loadInitialPage(): Promise<void>;
  /** Load the next page (if available) */
  loadNextPage(): Promise<void>;
  /** Load all remaining pages in background */
  loadAllBackground(): Promise<void>;
  /** Reset pagination and reload from beginning */
  reset(): Promise<void>;
  /** Refresh data - reloads from beginning while preserving scroll position */
  refresh(): Promise<void>;
  /** Add a single item (from create operation) */
  addItem(item: T): void;
  /** Update an item in the store */
  updateItem(id: string, updatedItem: T): void;
  /** Remove an item from the store */
  removeItem(id: string): void;
  /** Check if an ID is already loaded */
  isLoaded(id: string): boolean;
  /** Get current state snapshot */
  getState(): PaginatedStoreState<T>;
}

// ============================================================================
// PAGINATION STORE FACTORY
// ============================================================================

/**
 * Creates a paginated store for an entity type.
 *
 * @param fetchPage - Function to fetch a page of data
 * @param pageSize - Number of items per page (default: 50)
 * @returns Store and actions for paginated data management
 *
 * @example
 * ```typescript
 * const { store, actions } = createPaginatedStore(
 *   (page, pageSize) => ApiClient.getProjectsPage(page, pageSize)
 * );
 *
 * // Load initial data
 * await actions.loadInitialPage();
 *
 * // Load more on scroll
 * await actions.loadNextPage();
 * ```
 */
export function createPaginatedStore<T extends { id?: UnknownSurrealThing }>(
  fetchPage: (page: number, pageSize: number) => Promise<PaginatedResponse<T>>,
  pageSize: number = DEFAULT_PAGE_SIZE
): {
  store: Writable<PaginatedStoreState<T>>;
  actions: PaginatedStoreActions<T>;
} {
  // Initialize state
  const initialState: PaginatedStoreState<T> = {
    items: [],
    pagination: {
      currentPage: 0,
      pageSize,
      totalRecords: 0,
      hasMore: true,
      loadedIds: new Set<string>(),
      isLoading: false,
    },
    initialized: false,
    error: null,
  };

  const store = writable<PaginatedStoreState<T>>(initialState);

  // Helper to extract ID from item
  const getItemId = (item: T): string | null => {
    return extractSurrealId(item.id);
  };

  // Helper to check for duplicates and append unique items
  const appendUniqueItems = (
    existingItems: T[],
    newItems: T[],
    loadedIds: Set<string>
  ): { items: T[]; addedIds: string[] } => {
    const addedIds: string[] = [];
    const uniqueNewItems = newItems.filter((item) => {
      const id = getItemId(item);
      if (id && !loadedIds.has(id)) {
        addedIds.push(id);
        return true;
      }
      return false;
    });

    return {
      items: [...existingItems, ...uniqueNewItems],
      addedIds,
    };
  };

  const actions: PaginatedStoreActions<T> = {
    async loadInitialPage(): Promise<void> {
      const currentState = get(store);

      // Prevent concurrent loads
      if (currentState.pagination.isLoading) {
        return;
      }

      store.update((state) => ({
        ...state,
        pagination: { ...state.pagination, isLoading: true },
        error: null,
      }));

      try {
        const response = await fetchPage(1, pageSize);

        // Process items and track IDs
        const loadedIds = new Set<string>();
        response.items.forEach((item) => {
          const id = getItemId(item);
          if (id) loadedIds.add(id);
        });

        store.update((state) => ({
          ...state,
          items: response.items,
          pagination: {
            currentPage: 1,
            pageSize,
            totalRecords: response.total,
            hasMore: response.has_more,
            loadedIds,
            isLoading: false,
          },
          initialized: true,
          error: null,
        }));
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to load data';
        store.update((state) => ({
          ...state,
          pagination: { ...state.pagination, isLoading: false },
          error: errorMessage,
        }));
        throw error;
      }
    },

    async loadNextPage(): Promise<void> {
      const currentState = get(store);

      // Prevent concurrent loads or loading when no more data
      if (
        currentState.pagination.isLoading ||
        !currentState.pagination.hasMore
      ) {
        return;
      }

      const nextPage = currentState.pagination.currentPage + 1;

      store.update((state) => ({
        ...state,
        pagination: { ...state.pagination, isLoading: true },
        error: null,
      }));

      try {
        const response = await fetchPage(nextPage, pageSize);

        store.update((state) => {
          const { items, addedIds } = appendUniqueItems(
            state.items,
            response.items,
            state.pagination.loadedIds
          );

          // Add new IDs to the set
          const newLoadedIds = new Set(state.pagination.loadedIds);
          addedIds.forEach((id) => newLoadedIds.add(id));

          return {
            ...state,
            items,
            pagination: {
              ...state.pagination,
              currentPage: nextPage,
              totalRecords: response.total,
              hasMore: response.has_more,
              loadedIds: newLoadedIds,
              isLoading: false,
            },
            error: null,
          };
        });
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to load more data';
        store.update((state) => ({
          ...state,
          pagination: { ...state.pagination, isLoading: false },
          error: errorMessage,
        }));
        throw error;
      }
    },

    async loadAllBackground(): Promise<void> {
      const currentState = get(store);

      // Don't start background loading if already loading or no more data
      if (
        currentState.pagination.isLoading ||
        !currentState.pagination.hasMore
      ) {
        return;
      }

      // Load remaining pages in sequence
      while (get(store).pagination.hasMore) {
        await actions.loadNextPage();

        // Small delay between pages to avoid overwhelming the server
        await new Promise((resolve) => setTimeout(resolve, PAGE_LOAD_DELAY_MS));
      }
    },

    async reset(): Promise<void> {
      store.set(initialState);
      await actions.loadInitialPage();
    },

    async refresh(): Promise<void> {
      // Get current page count to reload same amount of data
      const currentState = get(store);
      const pagesToReload = Math.max(1, currentState.pagination.currentPage);

      // Reset to initial state
      store.set(initialState);

      // Reload the first page
      await actions.loadInitialPage();

      // Reload additional pages if user had scrolled
      for (let i = 1; i < pagesToReload; i++) {
        if (get(store).pagination.hasMore) {
          await actions.loadNextPage();
        }
      }
    },

    addItem(item: T): void {
      const id = getItemId(item);
      if (!id) return;

      store.update((state) => {
        // Check for duplicates
        if (state.pagination.loadedIds.has(id)) {
          return state;
        }

        const newLoadedIds = new Set(state.pagination.loadedIds);
        newLoadedIds.add(id);

        return {
          ...state,
          items: [item, ...state.items], // Add to beginning (newest first)
          pagination: {
            ...state.pagination,
            totalRecords: state.pagination.totalRecords + 1,
            loadedIds: newLoadedIds,
          },
        };
      });
    },

    updateItem(id: string, updatedItem: T): void {
      store.update((state) => ({
        ...state,
        items: state.items.map((item) =>
          getItemId(item) === id ? updatedItem : item
        ),
      }));
    },

    removeItem(id: string): void {
      store.update((state) => {
        const newLoadedIds = new Set(state.pagination.loadedIds);
        newLoadedIds.delete(id);

        return {
          ...state,
          items: state.items.filter((item) => getItemId(item) !== id),
          pagination: {
            ...state.pagination,
            totalRecords: Math.max(0, state.pagination.totalRecords - 1),
            loadedIds: newLoadedIds,
          },
        };
      });
    },

    isLoaded(id: string): boolean {
      return get(store).pagination.loadedIds.has(id);
    },

    getState(): PaginatedStoreState<T> {
      return get(store);
    },
  };

  return { store, actions };
}

// ============================================================================
// ON-DEMAND LOADING UTILITIES
// ============================================================================

/**
 * Creates a cache for on-demand entity loading.
 * Used to fetch related records that aren't in the paginated store yet.
 *
 * @param fetchById - Function to fetch a single entity by ID
 * @returns Functions to fetch and cache entities
 */
export function createOnDemandLoader<T>(
  fetchById: (id: string) => Promise<T | null>
): {
  /** Get an entity from cache or fetch it */
  getOrFetch: (id: string) => Promise<T | null>;
  /** Check if entity is in cache */
  has: (id: string) => boolean;
  /** Add entity to cache manually */
  add: (id: string, entity: T) => void;
  /** Clear the cache */
  clear: () => void;
} {
  const cache = new Map<string, T>();
  const pending = new Map<string, Promise<T | null>>();

  return {
    async getOrFetch(id: string): Promise<T | null> {
      // Return from cache if available
      if (cache.has(id)) {
        return cache.get(id)!;
      }

      // Return pending promise if already fetching
      if (pending.has(id)) {
        return pending.get(id)!;
      }

      // Fetch and cache
      const fetchPromise = fetchById(id).then((entity) => {
        pending.delete(id);
        if (entity) {
          cache.set(id, entity);
        }
        return entity;
      });

      pending.set(id, fetchPromise);
      return fetchPromise;
    },

    has(id: string): boolean {
      return cache.has(id);
    },

    add(id: string, entity: T): void {
      cache.set(id, entity);
    },

    clear(): void {
      cache.clear();
      pending.clear();
    },
  };
}

// ============================================================================
// SCROLL TRIGGER UTILITY
// ============================================================================

/**
 * Creates a scroll trigger for infinite scroll patterns.
 * Calls the callback when scroll position reaches the threshold.
 *
 * @param callback - Function to call when threshold is reached
 * @param threshold - Percentage of scroll (0-1) to trigger loading (default: 0.8)
 * @returns Functions to attach and detach scroll listener
 */
export function createScrollTrigger(
  callback: () => void,
  threshold: number = 0.8
): {
  attach: (element: HTMLElement) => void;
  detach: () => void;
} {
  let element: HTMLElement | null = null;
  let attached = false;

  const handleScroll = () => {
    if (!element) return;

    const scrollTop = element.scrollTop;
    const scrollHeight = element.scrollHeight;
    const clientHeight = element.clientHeight;

    const scrollPercentage = (scrollTop + clientHeight) / scrollHeight;

    if (scrollPercentage >= threshold) {
      callback();
    }
  };

  return {
    attach(el: HTMLElement): void {
      if (attached) return;
      element = el;
      element.addEventListener('scroll', handleScroll);
      attached = true;
    },

    detach(): void {
      if (!attached || !element) return;
      element.removeEventListener('scroll', handleScroll);
      element = null;
      attached = false;
    },
  };
}

// ============================================================================
// SORT ORDER MAINTENANCE
// ============================================================================

/**
 * Sorts items maintaining the specified order.
 * Default: newest first (time.created_at DESC)
 */
export function sortByCreatedAt<T extends { time?: { created_at?: string } }>(
  items: T[],
  direction: 'asc' | 'desc' = 'desc'
): T[] {
  return [...items].sort((a, b) => {
    const aTime = a.time?.created_at || '';
    const bTime = b.time?.created_at || '';
    const comparison = aTime.localeCompare(bTime);
    return direction === 'desc' ? -comparison : comparison;
  });
}
