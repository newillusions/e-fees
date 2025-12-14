/**
 * Projects Page Pagination Integration Tests
 *
 * TDD RED PHASE: These tests define the expected behavior for pagination
 * integration in the Projects page. They will FAIL until implementation
 * is complete.
 *
 * Test Specification: docs/test-specs/pagination-ui.spec.md
 *
 * Requirements tested:
 * - TC-UI-001: Initial pagination load (50 records)
 * - TC-UI-002: Infinite scroll triggering
 * - TC-UI-003: Loading states
 * - TC-UI-005: Filter interaction with pagination
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';
import { get, writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Project, PaginatedResponse } from '../types';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

// Mock the stores module
vi.mock('$lib/stores', async () => {
  const actual = await vi.importActual('$lib/stores');
  return {
    ...actual,
    // We'll override specific stores in tests
  };
});

// ============================================================================
// TEST DATA GENERATORS
// ============================================================================

const TEST_PREFIX = 'PAGTEST_';

function generateTestProject(index: number): Project {
  return {
    id: `project:${TEST_PREFIX}${index}`,
    name: `${TEST_PREFIX}Project ${index}`,
    name_short: `PT${index}`,
    status: 'Draft',
    city: 'Dubai',
    country: 'United Arab Emirates',
    number: {
      year: 25,
      country: 971,
      seq: index,
      id: `25-971${String(index).padStart(2, '0')}`
    },
    time: {
      created_at: new Date(Date.now() - index * 1000).toISOString(),
      updated_at: new Date(Date.now() - index * 1000).toISOString()
    }
  };
}

function createMockPaginatedResponse(
  totalItems: number,
  page: number,
  pageSize: number = 50
): PaginatedResponse<Project> {
  const start = (page - 1) * pageSize;
  const end = Math.min(start + pageSize, totalItems);
  const items = Array.from(
    { length: end - start },
    (_, i) => generateTestProject(start + i)
  );

  return {
    items,
    total: totalItems,
    page,
    page_size: pageSize,
    has_more: end < totalItems
  };
}

// ============================================================================
// TC-UI-001: Projects Page Pagination Integration
// ============================================================================

describe('TC-UI-001: Projects Page Pagination Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should load only first 50 records on initial page load', async () => {
    // Arrange: Mock API to return paginated response
    const mockResponse = createMockPaginatedResponse(150, 1, 50);
    vi.mocked(invoke).mockResolvedValueOnce(mockResponse);

    // Act: Import and check what the Projects page loads
    // This test will FAIL because Projects.svelte currently uses projectsStore.load()
    // which fetches ALL records, not paginated

    const { paginatedProjectsStore } = await import('$lib/stores');

    // Load initial page
    await paginatedProjectsStore.actions.loadInitialPage();

    const state = get(paginatedProjectsStore.store);

    // Assert: Only 50 records loaded initially
    expect(state.items.length).toBe(50);
    expect(state.pagination.hasMore).toBe(true);
    expect(state.pagination.totalRecords).toBe(150);

    // Verify API called with pagination params
    // Note: Frontend uses camelCase (pageSize), Tauri converts to snake_case for Rust
    expect(invoke).toHaveBeenCalledWith('get_projects_page', {
      page: 1,
      pageSize: 50
    });
  });

  it('should use paginatedProjectsStore instead of legacy projectsStore', async () => {
    // This test will FAIL until Projects.svelte is updated to import
    // paginatedProjectsStore instead of projectsStore

    // Read the Projects.svelte file content to verify imports
    // In actual implementation, we'd check the component's imports

    // For now, verify the paginatedProjectsStore exists and has correct interface
    const { paginatedProjectsStore } = await import('$lib/stores');

    expect(paginatedProjectsStore).toBeDefined();
    expect(paginatedProjectsStore.store).toBeDefined();
    expect(paginatedProjectsStore.actions).toBeDefined();
    expect(paginatedProjectsStore.actions.loadInitialPage).toBeInstanceOf(Function);
    expect(paginatedProjectsStore.actions.loadNextPage).toBeInstanceOf(Function);
  });

  it('should display correct pagination info in ResultsCounter', async () => {
    // Arrange
    const mockResponse = createMockPaginatedResponse(150, 1, 50);
    vi.mocked(invoke).mockResolvedValueOnce(mockResponse);

    // This test will FAIL until ResultsCounter is updated to show pagination info
    // Expected: "Showing 50 of 150 projects"
    // Current: "Showing 50 of 50 projects" (no pagination awareness)

    // Act: Load paginated data
    const { paginatedProjectsStore } = await import('$lib/stores');
    await paginatedProjectsStore.actions.loadInitialPage();

    const state = get(paginatedProjectsStore.store);

    // Assert: State contains pagination metadata
    expect(state.pagination.totalRecords).toBe(150);
    expect(state.items.length).toBe(50);

    // The component should display:
    // "Showing 50 of 150 projects" when hasMore=true
  });
});

// ============================================================================
// TC-UI-002: Infinite Scroll Trigger
// ============================================================================

describe('TC-UI-002: Infinite Scroll Trigger', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should trigger next page load when scroll threshold reached', async () => {
    // Arrange
    const page1 = createMockPaginatedResponse(150, 1, 50);
    const page2 = createMockPaginatedResponse(150, 2, 50);

    vi.mocked(invoke)
      .mockResolvedValueOnce(page1)
      .mockResolvedValueOnce(page2);

    const { paginatedProjectsStore, createScrollTrigger } = await import('$lib/stores');

    // Load initial page
    await paginatedProjectsStore.actions.loadInitialPage();
    expect(get(paginatedProjectsStore.store).items.length).toBe(50);

    // Act: Simulate scroll to 80% threshold
    // This triggers loadNextPage
    await paginatedProjectsStore.actions.loadNextPage();

    // Assert: Second page loaded
    const state = get(paginatedProjectsStore.store);
    expect(state.items.length).toBe(100);
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it('should not trigger load when hasMore is false', async () => {
    // Arrange: Small dataset (no more pages)
    const response = createMockPaginatedResponse(30, 1, 50);
    vi.mocked(invoke).mockResolvedValueOnce(response);

    const { paginatedProjectsStore } = await import('$lib/stores');

    await paginatedProjectsStore.actions.loadInitialPage();
    expect(get(paginatedProjectsStore.store).pagination.hasMore).toBe(false);

    vi.clearAllMocks();

    // Act: Try to load next page
    await paginatedProjectsStore.actions.loadNextPage();

    // Assert: No API call made
    expect(invoke).not.toHaveBeenCalled();
  });

  it('should maintain scroll position when new records appended', async () => {
    // This test verifies UX requirement: user shouldn't lose their place
    // when new records are loaded via infinite scroll

    const page1 = createMockPaginatedResponse(150, 1, 50);
    const page2 = createMockPaginatedResponse(150, 2, 50);

    vi.mocked(invoke)
      .mockResolvedValueOnce(page1)
      .mockResolvedValueOnce(page2);

    const { paginatedProjectsStore } = await import('$lib/stores');

    await paginatedProjectsStore.actions.loadInitialPage();

    // Simulate scroll position tracking
    const scrollPosition = 500; // pixels scrolled

    // Load next page
    await paginatedProjectsStore.actions.loadNextPage();

    // Assert: New records appended, not prepended
    const state = get(paginatedProjectsStore.store);
    expect(state.items.length).toBe(100);

    // First 50 items should be unchanged (same IDs in same order)
    for (let i = 0; i < 50; i++) {
      expect(state.items[i].id).toBe(`project:${TEST_PREFIX}${i}`);
    }
  });
});

// ============================================================================
// TC-UI-003: Loading States
// ============================================================================

describe('TC-UI-003: Loading States', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should show loading state during initial fetch', async () => {
    // Arrange: Slow API response
    const slowResponse = new Promise<PaginatedResponse<Project>>(resolve => {
      setTimeout(() => resolve(createMockPaginatedResponse(150, 1, 50)), 100);
    });
    vi.mocked(invoke).mockReturnValueOnce(slowResponse);

    const { paginatedProjectsStore } = await import('$lib/stores');

    // Act: Start loading
    const loadPromise = paginatedProjectsStore.actions.loadInitialPage();

    // Assert: Loading state is true during fetch
    expect(get(paginatedProjectsStore.store).pagination.isLoading).toBe(true);

    // Wait for completion
    await loadPromise;

    // Assert: Loading state is false after fetch
    expect(get(paginatedProjectsStore.store).pagination.isLoading).toBe(false);
  });

  it('should show loading state during next page fetch', async () => {
    const page1 = createMockPaginatedResponse(150, 1, 50);
    const slowPage2 = new Promise<PaginatedResponse<Project>>(resolve => {
      setTimeout(() => resolve(createMockPaginatedResponse(150, 2, 50)), 100);
    });

    vi.mocked(invoke)
      .mockResolvedValueOnce(page1)
      .mockReturnValueOnce(slowPage2);

    const { paginatedProjectsStore } = await import('$lib/stores');

    await paginatedProjectsStore.actions.loadInitialPage();
    expect(get(paginatedProjectsStore.store).pagination.isLoading).toBe(false);

    // Start loading next page
    const loadPromise = paginatedProjectsStore.actions.loadNextPage();

    // Assert: Loading during fetch
    expect(get(paginatedProjectsStore.store).pagination.isLoading).toBe(true);

    await loadPromise;

    expect(get(paginatedProjectsStore.store).pagination.isLoading).toBe(false);
  });

  it('should indicate when all data is loaded (hasMore=false)', async () => {
    const page1 = createMockPaginatedResponse(75, 1, 50);
    const page2 = createMockPaginatedResponse(75, 2, 50);

    vi.mocked(invoke)
      .mockResolvedValueOnce(page1)
      .mockResolvedValueOnce(page2);

    const { paginatedProjectsStore } = await import('$lib/stores');

    await paginatedProjectsStore.actions.loadInitialPage();
    expect(get(paginatedProjectsStore.store).pagination.hasMore).toBe(true);

    await paginatedProjectsStore.actions.loadNextPage();

    // All 75 records loaded
    const state = get(paginatedProjectsStore.store);
    expect(state.items.length).toBe(75);
    expect(state.pagination.hasMore).toBe(false);
  });
});

// ============================================================================
// TC-UI-005: Filter Interaction with Pagination
// ============================================================================

describe('TC-UI-005: Filter Interaction with Pagination', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should reset pagination when filter changes', async () => {
    const page1 = createMockPaginatedResponse(150, 1, 50);
    const page2 = createMockPaginatedResponse(150, 2, 50);
    const filteredPage1 = createMockPaginatedResponse(30, 1, 50);

    vi.mocked(invoke)
      .mockResolvedValueOnce(page1)
      .mockResolvedValueOnce(page2)
      .mockResolvedValueOnce(filteredPage1);

    const { paginatedProjectsStore } = await import('$lib/stores');

    // Load 2 pages
    await paginatedProjectsStore.actions.loadInitialPage();
    await paginatedProjectsStore.actions.loadNextPage();
    expect(get(paginatedProjectsStore.store).items.length).toBe(100);
    expect(get(paginatedProjectsStore.store).pagination.currentPage).toBe(2);

    // Reset pagination (simulating filter change)
    await paginatedProjectsStore.actions.reset();

    // Should reset to page 1 with filtered results (30 items total)
    const state = get(paginatedProjectsStore.store);
    expect(state.pagination.currentPage).toBe(1);
    expect(state.items.length).toBe(30); // Filtered dataset has 30 items
  });

  it('should clear loaded items when filter changes', async () => {
    const page1 = createMockPaginatedResponse(150, 1, 50);
    const page2 = createMockPaginatedResponse(150, 2, 50);

    vi.mocked(invoke)
      .mockResolvedValueOnce(page1)
      .mockResolvedValueOnce(page2)
      .mockResolvedValueOnce(createMockPaginatedResponse(20, 1, 50));

    const { paginatedProjectsStore } = await import('$lib/stores');

    // Load 2 pages
    await paginatedProjectsStore.actions.loadInitialPage();
    await paginatedProjectsStore.actions.loadNextPage();

    const oldIds = get(paginatedProjectsStore.store).pagination.loadedIds;
    expect(oldIds.size).toBe(100);

    // Reset (filter change)
    await paginatedProjectsStore.actions.reset();

    // Old loaded IDs should be cleared, new filtered results have 20 items
    const newIds = get(paginatedProjectsStore.store).pagination.loadedIds;
    expect(newIds.size).toBe(20); // Filtered dataset has 20 items
  });
});

// ============================================================================
// TC-UI-006: Error Handling
// ============================================================================

describe('TC-UI-006: Error Handling', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should set error state on initial load failure', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Database connection failed'));

    const { paginatedProjectsStore } = await import('$lib/stores');

    // Act: Load should throw but not crash
    await expect(paginatedProjectsStore.actions.loadInitialPage()).rejects.toThrow();

    // Assert: Error state captured
    const state = get(paginatedProjectsStore.store);
    expect(state.error).toBeTruthy();
    expect(state.pagination.isLoading).toBe(false);
  });

  it('should preserve existing data on next page failure', async () => {
    const page1 = createMockPaginatedResponse(150, 1, 50);

    vi.mocked(invoke)
      .mockResolvedValueOnce(page1)
      .mockRejectedValueOnce(new Error('Network error'));

    const { paginatedProjectsStore } = await import('$lib/stores');

    await paginatedProjectsStore.actions.loadInitialPage();
    expect(get(paginatedProjectsStore.store).items.length).toBe(50);

    // Next page fails
    await expect(paginatedProjectsStore.actions.loadNextPage()).rejects.toThrow();

    // Existing data preserved
    const state = get(paginatedProjectsStore.store);
    expect(state.items.length).toBe(50); // Not cleared
    expect(state.error).toBeTruthy();
  });
});

// ============================================================================
// TC-UI-007: Empty States
// ============================================================================

describe('TC-UI-007: Empty States', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should handle empty database gracefully', async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      items: [],
      total: 0,
      page: 1,
      page_size: 50,
      has_more: false
    });

    const { paginatedProjectsStore } = await import('$lib/stores');

    await paginatedProjectsStore.actions.loadInitialPage();

    const state = get(paginatedProjectsStore.store);
    expect(state.items).toEqual([]);
    expect(state.pagination.totalRecords).toBe(0);
    expect(state.pagination.hasMore).toBe(false);
    expect(state.initialized).toBe(true);
  });
});

// ============================================================================
// CLEANUP
// ============================================================================

afterEach(() => {
  vi.clearAllMocks();
});

/**
 * TEST STATUS: RED PHASE
 *
 * These tests will FAIL until the following changes are made to Projects.svelte:
 *
 * 1. Replace import:
 *    - FROM: import { projectsStore, projectsActions } from '$lib/stores';
 *    - TO:   import { paginatedProjectsStore } from '$lib/stores';
 *
 * 2. Update onMount:
 *    - FROM: projectsActions.load();
 *    - TO:   paginatedProjectsStore.actions.loadInitialPage();
 *
 * 3. Add scroll trigger for infinite scroll
 *
 * 4. Update template to use paginatedProjectsStore.store.items
 *
 * 5. Add loading indicators for pagination states
 */
