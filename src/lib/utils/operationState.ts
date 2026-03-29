/**
 * Operation State Management
 *
 * Tracks async operation lifecycle (loading/saving/deleting) with
 * message and error state. Extracted from crud.ts.
 */

import { writable } from 'svelte/store';
import type { OperationState, OperationActions } from './crudTypes';

export type { OperationState, OperationActions };

/**
 * Extract a human-readable message from an unknown error.
 * Tauri IPC errors arrive as plain strings, not Error instances.
 */
export function getErrorMessage(error: unknown, fallback: string): string {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  return fallback;
}

export function useOperationState() {
  const initialState: OperationState = {
    loading: false,
    saving: false,
    deleting: false,
    message: '',
    error: null
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
    }
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
  const loadingActions = {
    loading: actions.setLoading,
    saving: actions.setSaving,
    deleting: actions.setDeleting
  } as const;
  const setLoadingState = loadingActions[loadingType];

  try {
    actions.clearMessages();
    setLoadingState(true);

    const result = await operation();

    setLoadingState(false);
    return result;
  } catch (error) {
    setLoadingState(false);
    actions.setError(getErrorMessage(error, 'An error occurred'));
    throw error;
  }
}
