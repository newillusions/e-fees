/**
 * Integration Tests: Pagination / Lazy Loading
 *
 * Tests the pagination behavior across stores and API layer based on
 * pagination.spec.md test specification.
 *
 * These tests verify:
 * - Background loading of data in batches
 * - Scroll-triggered loading
 * - On-demand loading of related records
 * - Duplicate prevention
 * - Sort order maintenance
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Project, Company, Contact, Fee, PaginatedResponse } from '../../types';
import { extractSurrealId } from '../utils/surrealdb';
import { createPaginatedStore, createOnDemandLoader, DEFAULT_PAGE_SIZE } from './pagination';

// Mock Tauri invoke - already mocked in setup.ts but we'll configure it per test
vi.mock('@tauri-apps/api/core');

// ============================================================================
// TEST DATA GENERATORS (with DELETE ME pattern for safety)
// ============================================================================

const TEST_PREFIX = 'PAGTEST_';

const generateTestProject = (index: number): Project => ({
  id: `project:test_${index}`,
  name: `${TEST_PREFIX}Project ${index}`,
  name_short: `PT${index}`,
  status: 'Lead',
  city: 'Test City',
  country: 'Test Country',
  time: {
    created_at: new Date(Date.now() - index * 1000).toISOString(),
    updated_at: new Date(Date.now() - index * 1000).toISOString()
  }
});

const generateTestCompany = (index: number): Company => ({
  id: `company:test_${index}`,
  name: `${TEST_PREFIX}Company ${index}`,
  abbreviation: `TC${index}`,
  city: 'Test City',
  country: 'Test Country',
  time: {
    created_at: new Date(Date.now() - index * 1000).toISOString(),
    updated_at: new Date(Date.now() - index * 1000).toISOString()
  }
});

const generateTestContact = (index: number, companyId?: string): Contact => ({
  id: `contact:test_${index}`,
  first_name: TEST_PREFIX,
  last_name: `Contact ${index}`,
  full_name: `${TEST_PREFIX} Contact ${index}`,
  email: `${TEST_PREFIX.toLowerCase()}${index}@test.local`,
  phone: `+1234567890${index}`,
  position: 'Test Position',
  company: companyId || `company:test_${index}`,
  time: {
    created_at: new Date(Date.now() - index * 1000).toISOString(),
    updated_at: new Date(Date.now() - index * 1000).toISOString()
  }
});

const generateTestFee = (index: number): Fee => ({
  id: `fee:test_${index}`,
  name: `${TEST_PREFIX}Fee ${index}`,
  number: `FEE-${index}`,
  status: 'Draft',
  issue_date: new Date().toISOString(),
  project_id: `project:test_${index}`,
  company_id: `company:test_${index}`,
  contact_id: `contact:test_${index}`,
  revisions: [],
  time: {
    created_at: new Date(Date.now() - index * 1000).toISOString(),
    updated_at: new Date(Date.now() - index * 1000).toISOString()
  }
});

// ============================================================================
// MOCK API RESPONSES
// ============================================================================

/**
 * Simulates paginated API response (matching Rust PaginatedResponse format)
 */
function createPaginatedResponse<T>(
  allItems: T[],
  page: number,
  pageSize: number
): PaginatedResponse<T> {
  const start = (page - 1) * pageSize;
  const end = start + pageSize;
  const items = allItems.slice(start, end);

  return {
    items,
    total: allItems.length,
    page,
    page_size: pageSize,
    has_more: end < allItems.length
  };
}

// ============================================================================
// TC-002: Background Loading - Automatic Fetch
// ============================================================================

describe('TC-002: Background Loading - Automatic Fetch', () => {
  let mockProjects: Project[];

  beforeEach(() => {
    // Generate 200 test projects
    mockProjects = Array.from({ length: 200 }, (_, i) => generateTestProject(i));
    vi.clearAllMocks();
  });

  it('should load initial 50 records', async () => {
    // Mock API to return first page (50 records)
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50));

    // Create a fresh paginated store for testing
    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();

    const state = get(store);

    // TC-002 Assertions: Initial Load
    expect(state.items.length).toBe(50);
    expect(state.pagination.currentPage).toBe(1);
    expect(state.pagination.hasMore).toBe(true);
    expect(state.initialized).toBe(true);
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith('get_projects_page', {
      page: 1,
      page_size: 50
    });
  });

  it('should grow store through sequential page loading: 50 → 100 → 150 → 200', async () => {
    const mockInvoke = vi.mocked(invoke);

    // Setup mock responses for each page
    mockInvoke
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50)) // Initial
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 2, 50)) // Page 2
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 3, 50)) // Page 3
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 4, 50)); // Page 4

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    // Initial load
    await actions.loadInitialPage();
    expect(get(store).items.length).toBe(50);

    // Load remaining pages manually (testing loadNextPage)
    await actions.loadNextPage();
    expect(get(store).items.length).toBe(100);

    await actions.loadNextPage();
    expect(get(store).items.length).toBe(150);

    await actions.loadNextPage();
    expect(get(store).items.length).toBe(200);

    const finalState = get(store);

    // TC-002 Assertions: All Pages Loaded
    expect(finalState.items.length).toBe(200);
    expect(finalState.pagination.hasMore).toBe(false);
    expect(mockInvoke).toHaveBeenCalledTimes(4); // 4 pages total

    // Verify no duplicates
    const uniqueIds = new Set(finalState.items.map(p => extractSurrealId(p.id)));
    expect(uniqueIds.size).toBe(200);
  });

  it('should maintain sort order throughout page loading', async () => {
    const mockInvoke = vi.mocked(invoke);

    mockInvoke
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50))
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 2, 50))
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 3, 50))
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 4, 50));

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();
    await actions.loadNextPage();
    await actions.loadNextPage();
    await actions.loadNextPage();

    const projects = get(store).items;

    // TC-002 Assertion: Sort Order Maintained (newest first based on test data)
    // Test data generates items with timestamps decreasing by 1 second each
    for (let i = 0; i < projects.length - 1; i++) {
      const current = new Date(projects[i].time?.created_at || 0);
      const next = new Date(projects[i + 1].time?.created_at || 0);
      expect(current.getTime()).toBeGreaterThanOrEqual(next.getTime());
    }
  });

  it('should keep UI responsive during initial load', async () => {
    const mockInvoke = vi.mocked(invoke);

    // Simulate slow network
    mockInvoke.mockImplementationOnce(
      () =>
        new Promise(resolve =>
          setTimeout(() => resolve(createPaginatedResponse(mockProjects, 1, 50)), 50)
        )
    );

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    const startTime = performance.now();

    // Load should return after first page, not wait for all data
    await actions.loadInitialPage();

    const loadTime = performance.now() - startTime;

    // Initial load should complete reasonably fast (< 200ms including 50ms mock delay)
    expect(loadTime).toBeLessThan(200);

    // Store should have initial data
    expect(get(store).items.length).toBe(50);
    expect(get(store).initialized).toBe(true);
  });
});

// ============================================================================
// TC-003: Scroll-Triggered Loading (Manual Next Page)
// ============================================================================

describe('TC-003: Scroll-Triggered Loading', () => {
  let mockProjects: Project[];

  beforeEach(() => {
    mockProjects = Array.from({ length: 200 }, (_, i) => generateTestProject(i));
    vi.clearAllMocks();
  });

  it('should load page 2 when loadNextPage is called', async () => {
    const mockInvoke = vi.mocked(invoke);

    mockInvoke
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50))
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 2, 50));

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    // Initial load
    await actions.loadInitialPage();
    expect(get(store).items.length).toBe(50);
    expect(get(store).pagination.currentPage).toBe(1);

    // Simulate scroll trigger by calling loadNextPage
    await actions.loadNextPage();

    const state = get(store);
    expect(state.items.length).toBe(100);
    expect(state.pagination.currentPage).toBe(2);
    expect(state.pagination.hasMore).toBe(true);
  });

  it('should set hasMore to false when all records loaded', async () => {
    const mockInvoke = vi.mocked(invoke);

    // Only 75 records total (2 pages)
    const limitedProjects = mockProjects.slice(0, 75);

    mockInvoke
      .mockResolvedValueOnce(createPaginatedResponse(limitedProjects, 1, 50))
      .mockResolvedValueOnce(createPaginatedResponse(limitedProjects, 2, 50));

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();
    expect(get(store).pagination.hasMore).toBe(true);

    // Load remaining
    await actions.loadNextPage();

    const state = get(store);
    expect(state.items.length).toBe(75);
    expect(state.pagination.hasMore).toBe(false);
  });

  it('should not load when hasMore is false', async () => {
    const mockInvoke = vi.mocked(invoke);

    // Only 30 records (less than one page)
    const smallDataset = mockProjects.slice(0, 30);

    mockInvoke.mockResolvedValueOnce(createPaginatedResponse(smallDataset, 1, 50));

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();
    expect(get(store).pagination.hasMore).toBe(false);

    // Clear mock to track subsequent calls
    vi.clearAllMocks();

    // Try to load more - should be no-op
    await actions.loadNextPage();

    // No additional API call should be made
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});

// ============================================================================
// TC-006: On-Demand Related Record Loading - Company by ID
// ============================================================================

describe('TC-006: On-Demand Company Loading', () => {
  let mockCompanies: Company[];

  beforeEach(() => {
    mockCompanies = Array.from({ length: 10 }, (_, i) => generateTestCompany(i));
    vi.clearAllMocks();
  });

  it('should fetch company when not in cache', async () => {
    const mockInvoke = vi.mocked(invoke);

    const targetCompany = mockCompanies[5];
    mockInvoke.mockResolvedValueOnce(targetCompany);

    const loader = createOnDemandLoader<Company>(
      async id => mockInvoke('get_company_by_id', { id }) as Promise<Company | null>
    );

    // Should not be in cache initially
    expect(loader.has('test_5')).toBe(false);

    // Fetch company
    const company = await loader.getOrFetch('test_5');

    expect(company).toEqual(targetCompany);
    expect(mockInvoke).toHaveBeenCalledWith('get_company_by_id', { id: 'test_5' });

    // Should now be in cache
    expect(loader.has('test_5')).toBe(true);
  });

  it('should use cached company on subsequent requests', async () => {
    const mockInvoke = vi.mocked(invoke);

    const targetCompany = mockCompanies[5];
    mockInvoke.mockResolvedValueOnce(targetCompany);

    const loader = createOnDemandLoader<Company>(
      async id => mockInvoke('get_company_by_id', { id }) as Promise<Company | null>
    );

    // First fetch
    await loader.getOrFetch('test_5');
    expect(mockInvoke).toHaveBeenCalledTimes(1);

    // Clear mock to track subsequent calls
    vi.clearAllMocks();

    // Second fetch - should use cache
    const cached = await loader.getOrFetch('test_5');

    expect(cached).toEqual(targetCompany);
    expect(mockInvoke).not.toHaveBeenCalled(); // No new API call
  });

  it('should handle concurrent requests for same ID', async () => {
    const mockInvoke = vi.mocked(invoke);

    const targetCompany = mockCompanies[5];
    // Slow response to test concurrent handling
    mockInvoke.mockImplementation(
      () => new Promise(resolve => setTimeout(() => resolve(targetCompany), 50))
    );

    const loader = createOnDemandLoader<Company>(
      async id => mockInvoke('get_company_by_id', { id }) as Promise<Company | null>
    );

    // Fire multiple concurrent requests
    const [result1, result2, result3] = await Promise.all([
      loader.getOrFetch('test_5'),
      loader.getOrFetch('test_5'),
      loader.getOrFetch('test_5')
    ]);

    // All should return same result
    expect(result1).toEqual(targetCompany);
    expect(result2).toEqual(targetCompany);
    expect(result3).toEqual(targetCompany);

    // Only ONE API call should be made (others wait on pending promise)
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });

  it('should clear cache when requested', async () => {
    const mockInvoke = vi.mocked(invoke);

    const targetCompany = mockCompanies[5];
    mockInvoke.mockResolvedValue(targetCompany);

    const loader = createOnDemandLoader<Company>(
      async id => mockInvoke('get_company_by_id', { id }) as Promise<Company | null>
    );

    // Populate cache
    await loader.getOrFetch('test_5');
    expect(loader.has('test_5')).toBe(true);

    // Clear cache
    loader.clear();
    expect(loader.has('test_5')).toBe(false);
  });
});

// ============================================================================
// TC-004: Duplicate Prevention During Pagination
// ============================================================================

describe('TC-004: Duplicate Prevention', () => {
  it('should prevent duplicates when server returns overlapping records', async () => {
    const mockInvoke = vi.mocked(invoke);

    // Generate records with IDs 0-74
    const allProjects = Array.from({ length: 75 }, (_, i) => generateTestProject(i));

    // Page 1: records 0-49
    const page1Response: PaginatedResponse<Project> = {
      items: allProjects.slice(0, 50),
      total: 75,
      page: 1,
      page_size: 50,
      has_more: true
    };

    // Page 2: simulate overlap - records 25-74 (items 25-49 are duplicates)
    const page2Response: PaginatedResponse<Project> = {
      items: allProjects.slice(25, 75),
      total: 75,
      page: 2,
      page_size: 50,
      has_more: false
    };

    mockInvoke.mockResolvedValueOnce(page1Response).mockResolvedValueOnce(page2Response);

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    // Load page 1
    await actions.loadInitialPage();
    expect(get(store).items.length).toBe(50);

    // Load page 2 (with overlap)
    await actions.loadNextPage();

    const state = get(store);

    // TC-004 Assertions: No Duplicates
    // Should only have 75 unique items, not 100 (50 + 50 with overlap)
    expect(state.items.length).toBe(75);

    const uniqueIds = new Set(state.items.map(p => extractSurrealId(p.id)));
    expect(uniqueIds.size).toBe(75); // Each ID appears once
  });

  it('should track loaded IDs in pagination state', async () => {
    const mockInvoke = vi.mocked(invoke);

    const mockProjects = Array.from({ length: 60 }, (_, i) => generateTestProject(i));

    mockInvoke.mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50));

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();

    const state = get(store);

    // Should track all 50 loaded IDs
    expect(state.pagination.loadedIds.size).toBe(50);

    // Check specific IDs are tracked (full ID format: project:test_X)
    expect(actions.isLoaded('project:test_0')).toBe(true);
    expect(actions.isLoaded('project:test_49')).toBe(true);
    expect(actions.isLoaded('project:test_50')).toBe(false); // Not loaded yet
  });
});

// ============================================================================
// TC-005: Sort Order Maintenance
// ============================================================================

describe('TC-005: Sort Order Maintenance', () => {
  it('should maintain chronological order across multiple pages', async () => {
    const mockInvoke = vi.mocked(invoke);

    // Generate 150 projects with timestamps (index 0 = newest)
    const allProjects = Array.from({ length: 150 }, (_, i) => ({
      ...generateTestProject(i),
      time: {
        created_at: new Date(Date.now() - i * 1000).toISOString(),
        updated_at: new Date(Date.now() - i * 1000).toISOString()
      }
    }));

    mockInvoke
      .mockResolvedValueOnce(createPaginatedResponse(allProjects, 1, 50))
      .mockResolvedValueOnce(createPaginatedResponse(allProjects, 2, 50))
      .mockResolvedValueOnce(createPaginatedResponse(allProjects, 3, 50));

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();
    await actions.loadNextPage();
    await actions.loadNextPage();

    const projects = get(store).items;

    // TC-005 Assertions: Sort Order (newest first = DESC)
    expect(projects.length).toBe(150);

    // First item should be newer than 50th
    const first = new Date(projects[0].time?.created_at || 0);
    const fiftieth = new Date(projects[49].time?.created_at || 0);
    expect(first.getTime()).toBeGreaterThan(fiftieth.getTime());

    // 50th should be newer than 100th
    const hundredth = new Date(projects[99].time?.created_at || 0);
    expect(fiftieth.getTime()).toBeGreaterThan(hundredth.getTime());

    // 100th should be newer than last
    const last = new Date(projects[149].time?.created_at || 0);
    expect(hundredth.getTime()).toBeGreaterThan(last.getTime());
  });
});

// ============================================================================
// EDGE CASE: Concurrent Load Prevention
// ============================================================================

describe('EC-003: Concurrent Load Prevention', () => {
  it('should ignore concurrent load requests during active load', async () => {
    const mockInvoke = vi.mocked(invoke);

    const mockProjects = Array.from({ length: 50 }, (_, i) => generateTestProject(i));

    // Slow response to test concurrent handling
    mockInvoke.mockImplementation(
      () =>
        new Promise(resolve =>
          setTimeout(() => resolve(createPaginatedResponse(mockProjects, 1, 50)), 100)
        )
    );

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    // Trigger multiple loads concurrently (rapid scroll simulation)
    const load1 = actions.loadInitialPage();
    const load2 = actions.loadInitialPage();
    const load3 = actions.loadInitialPage();

    await Promise.all([load1, load2, load3]);

    // Should only make ONE API call due to isLoading guard
    expect(mockInvoke).toHaveBeenCalledTimes(1);

    // Store should have correct data
    expect(get(store).items.length).toBe(50);
  });

  it('should prevent concurrent loadNextPage calls', async () => {
    const mockInvoke = vi.mocked(invoke);

    const mockProjects = Array.from({ length: 100 }, (_, i) => generateTestProject(i));

    // Fast initial, slow second page
    mockInvoke
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50))
      .mockImplementationOnce(
        () =>
          new Promise(resolve =>
            setTimeout(() => resolve(createPaginatedResponse(mockProjects, 2, 50)), 100)
          )
      );

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();

    // Clear mock to track subsequent calls
    mockInvoke.mockClear();
    mockInvoke.mockImplementation(
      () =>
        new Promise(resolve =>
          setTimeout(() => resolve(createPaginatedResponse(mockProjects, 2, 50)), 100)
        )
    );

    // Trigger multiple next page loads concurrently
    const next1 = actions.loadNextPage();
    const next2 = actions.loadNextPage();
    const next3 = actions.loadNextPage();

    await Promise.all([next1, next2, next3]);

    // Should only make ONE API call
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });
});

// ============================================================================
// EDGE CASE: Empty Page Response
// ============================================================================

describe('EC-005: Empty Page Response', () => {
  it('should handle empty array gracefully', async () => {
    const mockInvoke = vi.mocked(invoke);

    mockInvoke.mockResolvedValueOnce({
      items: [],
      total: 0,
      page: 1,
      page_size: 50,
      has_more: false
    });

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();

    const state = get(store);

    // Should not crash and handle empty state
    expect(state.items).toEqual([]);
    expect(state.pagination.hasMore).toBe(false);
    expect(state.pagination.totalRecords).toBe(0);
    expect(state.initialized).toBe(true);
  });

  it('should handle null/undefined items array', async () => {
    const mockInvoke = vi.mocked(invoke);

    // Malformed response (shouldn't happen but defensive)
    mockInvoke.mockResolvedValueOnce({
      items: undefined,
      total: 0,
      page: 1,
      page_size: 50,
      has_more: false
    });

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      const response = await mockInvoke('get_projects_page', { page, page_size: pageSize });
      // Normalize response
      return {
        ...(response as PaginatedResponse<Project>),
        items: (response as PaginatedResponse<Project>).items || []
      };
    });

    // Should not throw
    await expect(actions.loadInitialPage()).resolves.not.toThrow();
  });
});

// ============================================================================
// Store Operations: Add, Update, Remove
// ============================================================================

describe('Store Operations', () => {
  it('should add item to beginning of store', async () => {
    const mockInvoke = vi.mocked(invoke);

    const mockProjects = Array.from({ length: 10 }, (_, i) => generateTestProject(i));
    mockInvoke.mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50));

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();
    expect(get(store).items.length).toBe(10);

    // Add new item
    const newProject = generateTestProject(999);
    actions.addItem(newProject);

    const state = get(store);
    expect(state.items.length).toBe(11);
    expect(state.items[0]).toEqual(newProject); // Added to beginning
    expect(state.pagination.totalRecords).toBe(11);
    expect(state.pagination.loadedIds.has('project:test_999')).toBe(true);
  });

  it('should update existing item in store', async () => {
    const mockInvoke = vi.mocked(invoke);

    const mockProjects = Array.from({ length: 10 }, (_, i) => generateTestProject(i));
    mockInvoke.mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50));

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();

    // Update project 5 (using full ID format: project:test_5)
    const updatedProject = { ...mockProjects[5], name: 'Updated Name' };
    actions.updateItem('project:test_5', updatedProject);

    const state = get(store);
    const found = state.items.find(p => extractSurrealId(p.id) === 'project:test_5');
    expect(found?.name).toBe('Updated Name');
  });

  it('should remove item from store', async () => {
    const mockInvoke = vi.mocked(invoke);

    const mockProjects = Array.from({ length: 10 }, (_, i) => generateTestProject(i));
    mockInvoke.mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50));

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();
    expect(get(store).items.length).toBe(10);

    // Remove project 5 (using full ID format: project:test_5)
    actions.removeItem('project:test_5');

    const state = get(store);
    expect(state.items.length).toBe(9);
    expect(state.pagination.totalRecords).toBe(9);
    expect(state.pagination.loadedIds.has('project:test_5')).toBe(false);
    expect(state.items.find(p => extractSurrealId(p.id) === 'project:test_5')).toBeUndefined();
  });

  it('should reset store and reload', async () => {
    const mockInvoke = vi.mocked(invoke);

    const mockProjects = Array.from({ length: 100 }, (_, i) => generateTestProject(i));

    mockInvoke
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50))
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 2, 50))
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50)); // Reset

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    await actions.loadInitialPage();
    await actions.loadNextPage();
    expect(get(store).items.length).toBe(100);
    expect(get(store).pagination.currentPage).toBe(2);

    // Reset
    await actions.reset();

    const state = get(store);
    expect(state.items.length).toBe(50);
    expect(state.pagination.currentPage).toBe(1);
    expect(state.pagination.hasMore).toBe(true);
  });
});

// ============================================================================
// Background Loading
// ============================================================================

describe('Background Loading', () => {
  it('should load all pages with loadAllBackground', async () => {
    const mockInvoke = vi.mocked(invoke);

    const mockProjects = Array.from({ length: 120 }, (_, i) => generateTestProject(i));

    mockInvoke
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 1, 50))
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 2, 50))
      .mockResolvedValueOnce(createPaginatedResponse(mockProjects, 3, 50));

    const { store, actions } = createPaginatedStore<Project>(async (page, pageSize) => {
      return mockInvoke('get_projects_page', { page, page_size: pageSize }) as Promise<
        PaginatedResponse<Project>
      >;
    });

    // Initial load
    await actions.loadInitialPage();
    expect(get(store).items.length).toBe(50);

    // Background load all remaining
    await actions.loadAllBackground();

    const state = get(store);
    expect(state.items.length).toBe(120);
    expect(state.pagination.hasMore).toBe(false);
    expect(mockInvoke).toHaveBeenCalledTimes(3);
  });
});

// ============================================================================
// CLEANUP AND SAFETY
// ============================================================================

afterEach(() => {
  vi.clearAllMocks();
});
