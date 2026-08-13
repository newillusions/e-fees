/**
 * Companies Page - Load Error Visibility Regression Test
 *
 * Same class of bug as Proposals.error.test.ts (2026-08-13): the paginated
 * store correctly captures a failed load into state.error, but the route
 * component never read it - a failed load left companies=[] with no visible
 * indication anything went wrong, rendering "No Companies Yet" as if the
 * account genuinely had none.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';
import { invoke } from '@tauri-apps/api/core';
import Companies from './Companies.svelte';
import { paginatedCompaniesStore } from '$lib/stores';
import { get } from 'svelte/store';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

function resetPaginatedStore() {
  paginatedCompaniesStore.store.set({
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

describe('Companies page - load error visibility', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetPaginatedStore();

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_companies_page') {
        throw new Error('No database client available');
      }
      return [];
    });
  });

  it('shows the real load error instead of the generic empty state', async () => {
    render(Companies);

    await waitFor(() => {
      expect(screen.getByText(/No database client available/)).toBeInTheDocument();
    });

    expect(screen.getByText('Retry')).toBeInTheDocument();
    expect(screen.queryByText('No Companies Yet')).not.toBeInTheDocument();

    const state = get(paginatedCompaniesStore.store);
    expect(state.error).toBeTruthy();
    expect(state.items).toEqual([]);
  });

  it('retries the load when the Retry button is clicked, clearing the error on success', async () => {
    render(Companies);

    await waitFor(() => {
      expect(screen.getByText('Retry')).toBeInTheDocument();
    });

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_companies_page') {
        return { items: [], total: 0, page: 1, page_size: 50, has_more: false };
      }
      return [];
    });

    await fireEvent.click(screen.getByText('Retry'));

    await waitFor(() => {
      expect(screen.queryByText(/No database client available/)).not.toBeInTheDocument();
    });

    expect(screen.getByText('No Companies Yet')).toBeInTheDocument();
  });

  it('shows the ordinary empty state (not an error) when the load genuinely succeeds with zero rows', async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_companies_page') {
        return { items: [], total: 0, page: 1, page_size: 50, has_more: false };
      }
      return [];
    });

    render(Companies);

    await waitFor(() => {
      expect(screen.getByText('No Companies Yet')).toBeInTheDocument();
    });

    expect(screen.queryByText('Retry')).not.toBeInTheDocument();
  });
});
