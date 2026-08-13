/**
 * Proposals Page - Load Error Visibility Regression Test
 *
 * Bug (2026-08-13, Martin hit live): paginatedFeesStore's loadInitialPage()
 * correctly captures a failed load into state.error (already covered by
 * Projects.test.ts's TC-UI-006 at the store layer), but Proposals.svelte's
 * store subscription never read state.error - a failed load left fees=[]
 * with no visible indication anything went wrong, rendering the ordinary
 * "No Proposals Yet" empty state exactly as if the account genuinely had
 * zero proposals. This is the same failure class Dashboard.svelte's stats
 * tile had before it was fixed with a statsError banner - the fix was never
 * applied here. Root cause chain (not fixed by this test/PR, tracked
 * separately): App.svelte's startup gate declares the app ready after a
 * flat timer without waiting for the DB connection, so a query firing
 * before the backend has connected fails immediately and silently.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import { invoke } from '@tauri-apps/api/core';
import Proposals from './Proposals.svelte';
import { paginatedFeesStore, projectsStore, companiesStore, contactsStore } from '$lib/stores';
import { get } from 'svelte/store';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

function resetPaginatedStore() {
  paginatedFeesStore.store.set({
    items: [],
    pagination: {
      currentPage: 0,
      pageSize: 50,
      totalRecords: 0,
      hasMore: true,
      loadedIds: new Set<string>(),
      isLoading: false
    },
    initialized: false,
    error: null
  });
}

describe('Proposals page - load error visibility', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetPaginatedStore();

    // Related-data loaders (projects/companies/contacts) - only called by
    // onMount when their plain store is empty; keep them harmless so the
    // test isolates the fees-loading path.
    if (!get(projectsStore).length) {
      // no-op: default invoke mock below resolves []
    }

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      switch (cmd) {
        case 'get_fees_page':
          throw new Error('No database client available');
        case 'get_projects':
        case 'get_companies':
        case 'get_contacts':
          return [];
        default:
          return [];
      }
    });
  });

  it('shows the real load error instead of the generic empty state', async () => {
    render(Proposals);

    await waitFor(() => {
      expect(screen.getByText(/No database client available/)).toBeInTheDocument();
    });

    expect(screen.getByText('Retry')).toBeInTheDocument();
    expect(screen.queryByText('No Proposals Yet')).not.toBeInTheDocument();

    const state = get(paginatedFeesStore.store);
    expect(state.error).toBeTruthy();
    expect(state.items).toEqual([]);
  });

  it('retries the load when the Retry button is clicked, clearing the error on success', async () => {
    render(Proposals);

    await waitFor(() => {
      expect(screen.getByText('Retry')).toBeInTheDocument();
    });

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_fees_page') {
        return { items: [], total: 0, page: 1, page_size: 50, has_more: false };
      }
      return [];
    });

    const { fireEvent } = await import('@testing-library/svelte');
    await fireEvent.click(screen.getByText('Retry'));

    await waitFor(() => {
      expect(screen.queryByText(/No database client available/)).not.toBeInTheDocument();
    });

    expect(screen.getByText('No Proposals Yet')).toBeInTheDocument();
  });

  it('shows the ordinary empty state (not an error) when the load genuinely succeeds with zero rows', async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_fees_page') {
        return { items: [], total: 0, page: 1, page_size: 50, has_more: false };
      }
      return [];
    });

    render(Proposals);

    await waitFor(() => {
      expect(screen.getByText('No Proposals Yet')).toBeInTheDocument();
    });

    expect(screen.queryByText('Retry')).not.toBeInTheDocument();
  });
});
