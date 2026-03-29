/**
 * Modal State Management
 *
 * Generic modal state store for entity create/edit modals.
 * Extracted from crud.ts — no CRUD store dependency.
 */

import { writable } from 'svelte/store';
import type { ModalState, ModalActions } from './crudTypes';

export type { ModalState, ModalActions };

export function useModalState<T>() {
  const initialState: ModalState<T> = {
    isOpen: false,
    mode: 'create',
    item: null
  };

  const store = writable(initialState);

  const actions: ModalActions<T> = {
    openCreate() {
      store.update(state => ({
        ...state,
        isOpen: true,
        mode: 'create',
        item: null
      }));
    },

    openEdit(item: T) {
      store.update(state => ({
        ...state,
        isOpen: true,
        mode: 'edit',
        item
      }));
    },

    close() {
      store.set(initialState);
    }
  };

  return { store, actions };
}
