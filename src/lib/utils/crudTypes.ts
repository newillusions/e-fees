/**
 * CRUD Type Definitions
 *
 * All interfaces and type aliases used by the CRUD store system.
 * Extracted from crud.ts for cleaner module boundaries.
 */

import type { Writable } from 'svelte/store';
import type { UnknownSurrealThing } from '../../types';

// ============================================================================
// CORE CRUD TYPES
// ============================================================================

export interface CrudState<T> {
  items: T[];
  filteredItems: T[];
  loading: boolean;
  error: string | null;
  saving: boolean;
  searchQuery: string;
  filters: Record<string, unknown>;
  sort: {
    field: string;
    direction: 'asc' | 'desc';
  } | null;
  lastUpdated: Date | null;
  optimisticUpdates: Map<string, T>;
}

export interface CrudApi<T> {
  getAll(): Promise<T[]>;
  search?(query: string): Promise<T[]>;
  filter?(criteria: Record<string, unknown>): Promise<T[]>;
  create(data: Omit<T, 'id'>): Promise<T>;
  update(id: string, data: Partial<T>): Promise<T>;
  delete(id: string): Promise<T>;
}

export interface CrudStoreOptions<T = unknown> {
  enableOptimistic?: boolean;
  enableLogging?: boolean;
  component?: string;
  idExtractor?: (id: UnknownSurrealThing) => string | null;
  autoRefresh?: number;
  searchFields?: (keyof T)[];
}

export interface CrudActions<T> {
  load(): Promise<void>;
  create(data: Omit<T, 'id'>): Promise<T>;
  update(id: string, data: Partial<T>): Promise<T>;
  delete(id: string): Promise<T>;
  refresh(): Promise<void>;
  clear(): void;
  setError(error: string | null): void;
  search(query: string): Promise<void>;
  applyFilters(filters: Record<string, unknown>): Promise<void>;
  sort(field: string, direction?: 'asc' | 'desc'): void;
  resetFilters(): void;
  rollback(): void;
  getById(id: string): T | null;
}

export interface CrudStoreInterface<T> {
  store: Writable<CrudState<T>>;
  actions: CrudActions<T>;
  destroy: () => void;
}

export interface SurrealEntity {
  id?: UnknownSurrealThing;
}

// ============================================================================
// MODAL STATE TYPES
// ============================================================================

export interface ModalState<T> {
  isOpen: boolean;
  mode: 'create' | 'edit';
  item: T | null;
}

export interface ModalActions<T> {
  openCreate(): void;
  openEdit(item: T): void;
  close(): void;
}

// ============================================================================
// OPERATION STATE TYPES
// ============================================================================

export interface OperationState {
  loading: boolean;
  saving: boolean;
  deleting: boolean;
  message: string;
  error: string | null;
}

export interface OperationActions {
  setLoading(loading: boolean): void;
  setSaving(saving: boolean): void;
  setDeleting(deleting: boolean): void;
  setMessage(message: string): void;
  setError(error: string | null): void;
  clearMessages(): void;
  reset(): void;
}

// ============================================================================
// TYPE UTILITIES
// ============================================================================

export type EntityId<T> = T extends { id: infer U } ? U : never;

export type FormData<T> = Omit<T, 'id'>;

export type UpdateData<T> = Partial<Omit<T, 'id'>>;

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

export interface SearchResultSet<T> {
  items: T[];
  query: string;
  totalMatches: number;
  searchTime: number;
}
