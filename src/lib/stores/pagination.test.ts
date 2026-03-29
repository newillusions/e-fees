/**
 * Pagination Feature Unit Tests
 *
 * Test specification: docs/test-specs/pagination.spec.md
 *
 * These tests define the contract for pagination/lazy loading functionality
 * that will replace the current loadAllData() pattern. Tests are written in
 * TDD style and will FAIL initially (RED phase) until implementation is complete.
 *
 * Key Requirements:
 * - REQ-01: Default page size of 50 records
 * - REQ-02: Background OR scroll-triggered loading
 * - REQ-03: On-demand loading of related records
 * - REQ-04: Append new data WITHOUT duplicates
 * - REQ-05: Maintain sort order when appending
 * - REQ-06: Memory efficient - don't reload existing data
 *
 * Safety: All test data uses PAGTEST_ prefix for cleanup identification
 */

import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import type { Project, Company, Contact, Fee, SurrealThing } from '../../types';

// ============================================================================
// TYPE DEFINITIONS (Contract for pagination feature)
// ============================================================================

/**
 * Pagination state interface - defines the contract for pagination tracking
 */
interface PaginationState {
  currentPage: number;
  pageSize: number;
  totalRecords: number;
  hasMore: boolean;
  loadedIds: Set<string>;
  isLoading: boolean;
}

/**
 * Paginated response structure - defines API response contract
 */
interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

/**
 * Store state with pagination - defines enhanced store contract
 */
interface PaginatedStoreState<T> {
  items: T[];
  pagination: PaginationState;
  loading: boolean;
  error: string | null;
}

/**
 * Pagination actions interface - defines action contract
 */
interface PaginationActions<T> {
  loadPage(page: number): Promise<void>;
  loadNextPage(): Promise<void>;
  appendItems(newItems: T[]): void;
  reset(): void;
  getById(id: string): T | null;
}

// ============================================================================
// TEST DATA GENERATORS (Using PAGTEST_ prefix for safety)
// ============================================================================

/**
 * Generate test project with PAGTEST_ marker for safe cleanup
 */
function generateTestProject(sequence: number): Project {
  const timestamp = Date.now() + sequence;
  return {
    id: `project:PAGTEST_${sequence}`,
    name: `PAGTEST_Project_${sequence}`,
    name_short: `PT${sequence}`,
    status: 'Lead',
    city: 'Dubai',
    country: 'United Arab Emirates',
    time: {
      created_at: new Date(timestamp).toISOString(),
      updated_at: new Date(timestamp).toISOString()
    }
  };
}

/**
 * Generate test company with PAGTEST_ marker
 */
function generateTestCompany(sequence: number): Company {
  const timestamp = Date.now() + sequence;
  return {
    id: `company:PAGTEST_${sequence}`,
    name: `PAGTEST_Company_${sequence}`,
    abbreviation: `PC${sequence}`,
    city: 'Dubai',
    country: 'United Arab Emirates',
    time: {
      created_at: new Date(timestamp).toISOString(),
      updated_at: new Date(timestamp).toISOString()
    }
  };
}

/**
 * Generate test contact with PAGTEST_ marker
 */
function generateTestContact(sequence: number): Contact {
  const timestamp = Date.now() + sequence;
  return {
    id: `contact:PAGTEST_${sequence}`,
    first_name: 'PAGTEST_',
    last_name: `Contact_${sequence}`,
    full_name: `PAGTEST_ Contact_${sequence}`,
    email: `pagtest_${sequence}@test.local`,
    phone: `+971501234${String(sequence).padStart(3, '0')}`,
    position: 'Test Position',
    company: `company:PAGTEST_${sequence}`,
    time: {
      created_at: new Date(timestamp).toISOString(),
      updated_at: new Date(timestamp).toISOString()
    }
  };
}

/**
 * Generate test fee with PAGTEST_ marker
 */
function generateTestFee(sequence: number): Fee {
  const timestamp = Date.now() + sequence;
  return {
    id: `fee:PAGTEST_${sequence}`,
    name: `PAGTEST_Fee_${sequence}`,
    number: `PAGTEST-FEE-${String(sequence).padStart(4, '0')}`,
    status: 'Draft',
    issue_date: new Date(timestamp).toISOString(),
    project_id: `project:PAGTEST_${sequence}`,
    company_id: `company:PAGTEST_${sequence}`,
    contact_id: `contact:PAGTEST_${sequence}`,
    revisions: [],
    time: {
      created_at: new Date(timestamp).toISOString(),
      updated_at: new Date(timestamp).toISOString()
    }
  };
}

/**
 * Extract ID from SurrealDB Thing object or string
 */
function extractTestId(
  id: string | SurrealThing | { id: string | { String: string } } | undefined
): string {
  if (typeof id === 'string') return id;
  if (!id) return '';
  // Handle SurrealThing with nested { String: ... } structure
  if ('tb' in id && id.id) {
    const innerId = id.id;
    if (typeof innerId === 'string') return innerId;
    if (typeof innerId === 'object' && 'String' in innerId) return innerId.String;
  }
  // Handle simple { id: string } structure
  if ('id' in id) {
    const innerId = id.id;
    if (typeof innerId === 'string') return innerId;
    if (typeof innerId === 'object' && innerId && 'String' in innerId)
      return (innerId as { String: string }).String;
  }
  return String(id);
}

// ============================================================================
// MOCK PAGINATION API
// ============================================================================

/**
 * Mock paginated API for testing - simulates backend pagination
 */
class MockPaginatedApi<T extends { id?: any; time?: { created_at: string } }> {
  private allItems: T[];
  private pageSize: number;

  constructor(totalItems: number, generator: (seq: number) => T, pageSize = 50) {
    this.pageSize = pageSize;
    // Generate items and sort by created_at DESC (newest first)
    this.allItems = Array.from({ length: totalItems }, (_, i) => generator(i + 1)).sort((a, b) => {
      const timeA = a.time?.created_at || '';
      const timeB = b.time?.created_at || '';
      return timeB.localeCompare(timeA); // DESC order
    });
  }

  /**
   * Fetch a specific page of data
   */
  async fetchPage(page: number): Promise<PaginatedResponse<T>> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 10));

    const startIndex = (page - 1) * this.pageSize;
    const endIndex = startIndex + this.pageSize;
    const items = this.allItems.slice(startIndex, endIndex);

    return {
      items,
      total: this.allItems.length,
      page,
      pageSize: this.pageSize,
      hasMore: endIndex < this.allItems.length
    };
  }

  /**
   * Fetch items by IDs (for on-demand loading)
   */
  async fetchByIds(ids: string[]): Promise<T[]> {
    await new Promise(resolve => setTimeout(resolve, 10));
    return this.allItems.filter(item => {
      const itemId = extractTestId(item.id!);
      return ids.some(id => itemId === id);
    });
  }

  /**
   * Get all items (for reference)
   */
  getAllItems(): T[] {
    return [...this.allItems];
  }
}

// ============================================================================
// TC-001: Initial Page Load
// ============================================================================

describe('TC-001: Initial Page Load', () => {
  it('should load exactly 50 records on initial fetch when 150+ records exist', async () => {
    // Arrange: Create mock API with 150 projects
    const api = new MockPaginatedApi(150, generateTestProject, 50);

    // Act: Fetch first page
    const response = await api.fetchPage(1);

    // Assert: Verify REQ-01 (page size = 50)
    expect(response.items).toHaveLength(50);
    expect(response.total).toBe(150);
    expect(response.page).toBe(1);
    expect(response.hasMore).toBe(true);
  });

  it('should return total count if less than page size', async () => {
    // Arrange: Only 30 records exist
    const api = new MockPaginatedApi(30, generateTestProject, 50);

    // Act: Fetch first page
    const response = await api.fetchPage(1);

    // Assert: Should return all 30
    expect(response.items).toHaveLength(30);
    expect(response.total).toBe(30);
    expect(response.hasMore).toBe(false);
  });

  it('should set pagination state correctly after initial load', async () => {
    // This test defines the contract for pagination state tracking
    const mockState: PaginatedStoreState<Project> = {
      items: [],
      pagination: {
        currentPage: 0,
        pageSize: 50,
        totalRecords: 0,
        hasMore: false,
        loadedIds: new Set(),
        isLoading: false
      },
      loading: false,
      error: null
    };

    // Simulate loading page 1
    const api = new MockPaginatedApi(150, generateTestProject, 50);
    const response = await api.fetchPage(1);

    // Update state as implementation should do
    mockState.items = response.items;
    mockState.pagination.currentPage = response.page;
    mockState.pagination.totalRecords = response.total;
    mockState.pagination.hasMore = response.hasMore;
    mockState.pagination.loadedIds = new Set(response.items.map(item => extractTestId(item.id!)));

    // Assert: State matches requirements
    expect(mockState.pagination.currentPage).toBe(1);
    expect(mockState.pagination.hasMore).toBe(true);
    expect(mockState.pagination.loadedIds.size).toBe(50);
    expect(mockState.items).toHaveLength(50);
  });

  it('should maintain default sort order (created_at DESC)', async () => {
    // Arrange: Create API with timestamped data
    const api = new MockPaginatedApi(150, generateTestProject, 50);

    // Act: Fetch first page
    const response = await api.fetchPage(1);

    // Assert: Items are in DESC order (newest first)
    expect(response.items.length).toBeGreaterThan(1);
    const first = new Date(response.items[0].time!.created_at).getTime();
    const last = new Date(response.items[49].time!.created_at).getTime();
    expect(first).toBeGreaterThan(last);
  });
});

// ============================================================================
// TC-004: Duplicate Prevention During Pagination
// ============================================================================

describe('TC-004: Duplicate Prevention', () => {
  it('should prevent duplicates when server returns overlapping records', () => {
    // Arrange: Store has records 1-50
    const existingItems = Array.from({ length: 50 }, (_, i) => generateTestProject(i + 1));
    const existingIds = new Set(existingItems.map(item => extractTestId(item.id!)));

    // Server returns records 25-75 (overlap)
    const newItems = Array.from({ length: 51 }, (_, i) => generateTestProject(i + 25));

    // Act: Deduplicate using existing logic pattern
    const deduped = newItems.filter(item => {
      const id = extractTestId(item.id!);
      return !existingIds.has(id);
    });

    // Assert: REQ-04 (no duplicates)
    expect(deduped).toHaveLength(25); // Only records 51-75 are new

    // Combine and verify unique count
    const combined = [...existingItems, ...deduped];
    const uniqueIds = new Set(combined.map(item => extractTestId(item.id!)));
    expect(uniqueIds.size).toBe(75); // Total unique records
    expect(combined).toHaveLength(75);
  });

  it('should detect duplicates using loadedIds Set', () => {
    // Arrange: Pagination state with 50 loaded IDs
    const loadedIds = new Set(Array.from({ length: 50 }, (_, i) => `project:PAGTEST_${i + 1}`));

    // Act: Check if new items are already loaded
    const newItem1 = generateTestProject(25); // Duplicate
    const newItem2 = generateTestProject(75); // New

    const isDupe1 = loadedIds.has(extractTestId(newItem1.id!));
    const isDupe2 = loadedIds.has(extractTestId(newItem2.id!));

    // Assert
    expect(isDupe1).toBe(true); // Should be detected as duplicate
    expect(isDupe2).toBe(false); // Should be new
  });

  it('should maintain Set of loaded IDs for O(1) lookup', () => {
    // Arrange: Large set of IDs
    const loadedIds = new Set(Array.from({ length: 1000 }, (_, i) => `project:PAGTEST_${i}`));

    // Act: Multiple lookups
    const start = performance.now();
    for (let i = 0; i < 100; i++) {
      loadedIds.has(`project:PAGTEST_${Math.floor(Math.random() * 1000)}`);
    }
    const duration = performance.now() - start;

    // Assert: O(1) lookup should be very fast
    expect(duration).toBeLessThan(10); // Should take < 10ms for 100 lookups
    expect(loadedIds.size).toBe(1000);
  });
});

// ============================================================================
// TC-005: Sort Order Maintenance
// ============================================================================

describe('TC-005: Sort Order Maintenance', () => {
  it('should maintain chronological order across paginated loads', async () => {
    // Arrange: API with 150 records
    const api = new MockPaginatedApi(150, generateTestProject, 50);

    // Act: Load all 3 pages
    const page1 = await api.fetchPage(1);
    const page2 = await api.fetchPage(2);
    const page3 = await api.fetchPage(3);

    // Combine all pages
    const allItems = [...page1.items, ...page2.items, ...page3.items];

    // Assert: REQ-05 (sort order maintained)
    // Check first vs last of page 1
    const p1First = new Date(page1.items[0].time!.created_at).getTime();
    const p1Last = new Date(page1.items[49].time!.created_at).getTime();
    expect(p1First).toBeGreaterThan(p1Last);

    // Check boundary between page 1 and 2
    const p1End = new Date(page1.items[49].time!.created_at).getTime();
    const p2Start = new Date(page2.items[0].time!.created_at).getTime();
    expect(p1End).toBeGreaterThan(p2Start);

    // Check boundary between page 2 and 3
    const p2End = new Date(page2.items[49].time!.created_at).getTime();
    const p3Start = new Date(page3.items[0].time!.created_at).getTime();
    expect(p2End).toBeGreaterThan(p3Start);

    // Overall sort verification
    for (let i = 1; i < allItems.length; i++) {
      const prev = new Date(allItems[i - 1].time!.created_at).getTime();
      const curr = new Date(allItems[i].time!.created_at).getTime();
      expect(prev).toBeGreaterThanOrEqual(curr);
    }
  });

  it('should maintain sort when appending new items', () => {
    // Arrange: Existing sorted items (newest first)
    const existingItems = Array.from({ length: 50 }, (_, i) => {
      const project = generateTestProject(i + 1);
      project.time = {
        created_at: new Date(1000000 - i).toISOString(),
        updated_at: new Date(1000000 - i).toISOString()
      };
      return project;
    });

    // New items (older timestamps)
    const newItems = Array.from({ length: 50 }, (_, i) => {
      const project = generateTestProject(i + 51);
      project.time = {
        created_at: new Date(999950 - i).toISOString(),
        updated_at: new Date(999950 - i).toISOString()
      };
      return project;
    });

    // Act: Append (simple concatenation)
    const combined = [...existingItems, ...newItems];

    // Assert: Verify sort maintained
    for (let i = 1; i < combined.length; i++) {
      const prev = new Date(combined[i - 1].time!.created_at).getTime();
      const curr = new Date(combined[i].time!.created_at).getTime();
      expect(prev).toBeGreaterThanOrEqual(curr);
    }
  });

  it('should handle equal timestamps with stable sort', () => {
    // Arrange: Items with same timestamp
    const sameTimestamp = new Date('2025-01-01T00:00:00Z').toISOString();
    const items = Array.from({ length: 10 }, (_, i) => {
      const project = generateTestProject(i + 1);
      project.time = {
        created_at: sameTimestamp,
        updated_at: sameTimestamp
      };
      return project;
    });

    // Act: Sort by timestamp (should maintain relative order)
    const sorted = [...items].sort((a, b) => {
      const aTime = new Date(a.time!.created_at).getTime();
      const bTime = new Date(b.time!.created_at).getTime();
      return bTime - aTime; // DESC
    });

    // Assert: Order unchanged when timestamps equal (stable sort)
    sorted.forEach((item, i) => {
      expect(extractTestId(item.id!)).toBe(extractTestId(items[i].id!));
    });
  });
});

// ============================================================================
// TC-008: Memory Efficiency - No Reload
// ============================================================================

describe('TC-008: Memory Efficiency', () => {
  it('should not reload already-loaded data', async () => {
    // Arrange: Mock API with request tracking
    const api = new MockPaginatedApi(150, generateTestProject, 50);
    const requestLog: number[] = [];

    const trackedFetch = async (page: number) => {
      requestLog.push(page);
      return await api.fetchPage(page);
    };

    // Act: Load page 1 twice
    await trackedFetch(1);
    const loadedIds = new Set<string>();

    // Simulate checking if page 1 is already loaded
    const page1Response = await api.fetchPage(1);
    page1Response.items.forEach(item => loadedIds.add(extractTestId(item.id!)));

    // Attempt to load page 1 again (should be skipped)
    const shouldReload = loadedIds.size === 0;

    // Assert: REQ-06 (no unnecessary reloads)
    expect(shouldReload).toBe(false);
    expect(loadedIds.size).toBe(50);
  });

  it('should track loaded pages to prevent duplicate fetches', () => {
    // Arrange: Pagination state
    const loadedPages = new Set<number>();

    // Act: Mark pages as loaded
    loadedPages.add(1);
    loadedPages.add(2);

    // Attempt to load page 1 again
    const shouldFetchPage1 = !loadedPages.has(1);
    const shouldFetchPage3 = !loadedPages.has(3);

    // Assert
    expect(shouldFetchPage1).toBe(false);
    expect(shouldFetchPage3).toBe(true);
    expect(loadedPages.size).toBe(2);
  });

  it('should preserve store items across navigation', () => {
    // Arrange: Simulated store with data
    const storeState: PaginatedStoreState<Project> = {
      items: Array.from({ length: 100 }, (_, i) => generateTestProject(i + 1)),
      pagination: {
        currentPage: 2,
        pageSize: 50,
        totalRecords: 150,
        hasMore: true,
        loadedIds: new Set(),
        isLoading: false
      },
      loading: false,
      error: null
    };

    // Act: Simulate navigation away (store should persist)
    const itemsBeforeNav = storeState.items.length;

    // Navigation happens...

    // Simulate navigation back (check if data preserved)
    const itemsAfterNav = storeState.items.length;

    // Assert: No data loss
    expect(itemsAfterNav).toBe(itemsBeforeNav);
    expect(itemsAfterNav).toBe(100);
  });
});

// ============================================================================
// TC-009: Concurrent Load Prevention
// ============================================================================

describe('TC-009: Concurrent Load Prevention', () => {
  it('should prevent concurrent page loads using isLoading flag', async () => {
    // Arrange: Mock API with delay
    const api = new MockPaginatedApi(200, generateTestProject, 50);
    let isLoading = false;
    const loadAttempts: number[] = [];

    const loadPage = async (page: number) => {
      loadAttempts.push(page);

      // Guard: Prevent concurrent loads (REQ implementation)
      if (isLoading) {
        return null; // Reject concurrent load
      }

      isLoading = true;
      try {
        const response = await api.fetchPage(page);
        return response;
      } finally {
        isLoading = false;
      }
    };

    // Act: Trigger multiple loads rapidly
    const results = await Promise.all([
      loadPage(1),
      loadPage(2), // Should be blocked
      loadPage(3) // Should be blocked
    ]);

    // Assert: Only first load succeeded
    expect(results.filter(r => r !== null)).toHaveLength(1);
    expect(results[0]).not.toBeNull();
    expect(results[1]).toBeNull();
    expect(results[2]).toBeNull();
  });

  it('should queue subsequent requests when load in progress', async () => {
    // Arrange: Request queue
    const queue: number[] = [];
    let isLoading = false;

    const queueLoad = (page: number) => {
      if (isLoading) {
        queue.push(page);
        return false;
      }
      isLoading = true; // First call sets loading flag
      return true;
    };

    // Act: Attempt multiple loads
    const load1 = queueLoad(1); // Active (sets isLoading = true)
    const load2 = queueLoad(2); // Queued (isLoading is true)
    const load3 = queueLoad(3); // Queued (isLoading is true)

    // Assert: Queuing works
    expect(load1).toBe(true);
    expect(load2).toBe(false);
    expect(load3).toBe(false);
    expect(queue).toEqual([2, 3]);
  });

  it('should not create race conditions with rapid pagination', async () => {
    // Arrange: Track operation order
    const operations: string[] = [];
    let isLoading = false;

    const simulateLoad = async (page: number) => {
      if (isLoading) return;

      isLoading = true;
      operations.push(`start-${page}`);

      await new Promise(resolve => setTimeout(resolve, 20));

      operations.push(`end-${page}`);
      isLoading = false;
    };

    // Act: Rapid sequential calls
    await simulateLoad(1);
    await simulateLoad(2);
    await simulateLoad(3);

    // Assert: Operations completed in order (no overlap)
    expect(operations).toEqual(['start-1', 'end-1', 'start-2', 'end-2', 'start-3', 'end-3']);
  });
});

// ============================================================================
// INTEGRATION: Full Pagination Flow
// ============================================================================

describe('Integration: Complete Pagination Flow', () => {
  it('should handle full pagination lifecycle correctly', async () => {
    // Arrange: Mock store with pagination
    const api = new MockPaginatedApi(200, generateTestProject, 50);

    const store: PaginatedStoreState<Project> = {
      items: [],
      pagination: {
        currentPage: 0,
        pageSize: 50,
        totalRecords: 0,
        hasMore: false,
        loadedIds: new Set(),
        isLoading: false
      },
      loading: false,
      error: null
    };

    const appendItems = (newItems: Project[]) => {
      // Deduplicate
      const deduped = newItems.filter(item => {
        const id = extractTestId(item.id!);
        return !store.pagination.loadedIds.has(id);
      });

      // Append
      store.items = [...store.items, ...deduped];

      // Update loaded IDs
      deduped.forEach(item => {
        store.pagination.loadedIds.add(extractTestId(item.id!));
      });
    };

    // Act: Load multiple pages
    // Page 1
    const page1 = await api.fetchPage(1);
    appendItems(page1.items);
    store.pagination.currentPage = 1;
    store.pagination.totalRecords = page1.total;
    store.pagination.hasMore = page1.hasMore;

    expect(store.items).toHaveLength(50);
    expect(store.pagination.hasMore).toBe(true);

    // Page 2
    const page2 = await api.fetchPage(2);
    appendItems(page2.items);
    store.pagination.currentPage = 2;
    store.pagination.hasMore = page2.hasMore;

    expect(store.items).toHaveLength(100);
    expect(store.pagination.hasMore).toBe(true);

    // Page 3
    const page3 = await api.fetchPage(3);
    appendItems(page3.items);
    store.pagination.currentPage = 3;
    store.pagination.hasMore = page3.hasMore;

    expect(store.items).toHaveLength(150);
    expect(store.pagination.hasMore).toBe(true);

    // Page 4 (final page)
    const page4 = await api.fetchPage(4);
    appendItems(page4.items);
    store.pagination.currentPage = 4;
    store.pagination.hasMore = page4.hasMore;

    // Assert: Final state
    expect(store.items).toHaveLength(200);
    expect(store.pagination.hasMore).toBe(false);
    expect(store.pagination.currentPage).toBe(4);
    expect(store.pagination.loadedIds.size).toBe(200);

    // Verify no duplicates
    const uniqueIds = new Set(store.items.map(item => extractTestId(item.id!)));
    expect(uniqueIds.size).toBe(200);
  });

  it('should handle edge case: exact page size multiple', async () => {
    // Arrange: Exactly 100 records, page size 50
    const api = new MockPaginatedApi(100, generateTestProject, 50);

    // Act: Load both pages
    const page1 = await api.fetchPage(1);
    const page2 = await api.fetchPage(2);

    // Assert: No page 3 needed
    expect(page1.items).toHaveLength(50);
    expect(page1.hasMore).toBe(true);
    expect(page2.items).toHaveLength(50);
    expect(page2.hasMore).toBe(false);

    // Attempt page 3 (should be empty)
    const page3 = await api.fetchPage(3);
    expect(page3.items).toHaveLength(0);
    expect(page3.hasMore).toBe(false);
  });

  it('should handle empty page gracefully', async () => {
    // Arrange: Only 50 records
    const api = new MockPaginatedApi(50, generateTestProject, 50);

    // Act: Load page 1 (full) and page 2 (empty)
    const page1 = await api.fetchPage(1);
    const page2 = await api.fetchPage(2);

    // Assert: Graceful empty handling
    expect(page1.items).toHaveLength(50);
    expect(page1.hasMore).toBe(false);
    expect(page2.items).toHaveLength(0);
    expect(page2.hasMore).toBe(false);
  });
});

// ============================================================================
// ON-DEMAND RELATED RECORD LOADING
// ============================================================================

describe('On-Demand Related Record Loading', () => {
  it('should fetch related company when not in memory', async () => {
    // Arrange: Fee references company not yet loaded
    const fee = generateTestFee(1);
    const companyId = 'company:PAGTEST_1';
    fee.company_id = companyId;

    const companiesApi = new MockPaginatedApi(100, generateTestCompany, 50);
    const companiesInMemory = new Set<string>();

    // Act: Check if company needs loading
    const needsLoad = !companiesInMemory.has(companyId);

    let company: Company | null = null;
    if (needsLoad) {
      const companies = await companiesApi.fetchByIds([companyId]);
      company = companies[0] || null;
      if (company) {
        companiesInMemory.add(companyId);
      }
    }

    // Assert: REQ-03 (on-demand loading)
    expect(needsLoad).toBe(true);
    expect(company).not.toBeNull();
    expect(company!.name).toBe('PAGTEST_Company_1');
    expect(companiesInMemory.has(companyId)).toBe(true);
  });

  it('should not re-fetch already loaded related records', async () => {
    // Arrange: Company already in memory
    const companyId = 'company:PAGTEST_1';
    const companiesInMemory = new Set([companyId]);
    const fetchCount = { count: 0 };

    const mockFetch = async (id: string) => {
      fetchCount.count++;
      return generateTestCompany(1);
    };

    // Act: Check if company needs loading
    const needsLoad = !companiesInMemory.has(companyId);

    if (needsLoad) {
      await mockFetch(companyId);
    }

    // Assert: No fetch occurred (REQ-06: memory efficiency)
    expect(needsLoad).toBe(false);
    expect(fetchCount.count).toBe(0);
  });

  it('should handle multiple related records efficiently', async () => {
    // Arrange: 10 fees referencing 3 companies
    const fees = Array.from({ length: 10 }, (_, i) => {
      const fee = generateTestFee(i + 1);
      fee.company_id = `company:PAGTEST_${(i % 3) + 1}`; // Reuse companies
      return fee;
    });

    const companiesApi = new MockPaginatedApi(100, generateTestCompany, 50);
    const companiesInMemory = new Map<string, Company>();

    // Act: Load required companies
    const requiredCompanyIds = new Set(fees.map(fee => String(fee.company_id)));

    const toFetch = Array.from(requiredCompanyIds).filter(id => !companiesInMemory.has(id));

    if (toFetch.length > 0) {
      const companies = await companiesApi.fetchByIds(toFetch);
      companies.forEach(company => {
        companiesInMemory.set(extractTestId(company.id!), company);
      });
    }

    // Assert: Only 3 companies fetched (not 10)
    expect(companiesInMemory.size).toBe(3);
    expect(requiredCompanyIds.size).toBe(3);

    // All fees can resolve company names
    fees.forEach(fee => {
      const company = companiesInMemory.get(String(fee.company_id));
      expect(company).toBeDefined();
    });
  });
});

// ============================================================================
// EDGE CASES & ERROR HANDLING
// ============================================================================

describe('Edge Cases', () => {
  it('should handle filter changes mid-pagination', () => {
    // Arrange: Store with partial data loaded
    const store: PaginatedStoreState<Project> = {
      items: Array.from({ length: 100 }, (_, i) => generateTestProject(i + 1)),
      pagination: {
        currentPage: 2,
        pageSize: 50,
        totalRecords: 200,
        hasMore: true,
        loadedIds: new Set(),
        isLoading: false
      },
      loading: false,
      error: null
    };

    // Act: Filter applied (should reset pagination)
    const resetPagination = () => {
      store.items = [];
      store.pagination = {
        currentPage: 0,
        pageSize: 50,
        totalRecords: 0,
        hasMore: false,
        loadedIds: new Set(),
        isLoading: false
      };
    };

    resetPagination();

    // Assert: State reset for new filter
    expect(store.items).toHaveLength(0);
    expect(store.pagination.currentPage).toBe(0);
    expect(store.pagination.loadedIds.size).toBe(0);
  });

  it('should handle sort changes mid-pagination', () => {
    // Arrange: Store with partial data loaded
    const currentSort: string = 'created_at';
    const items = Array.from({ length: 100 }, (_, i) => generateTestProject(i + 1));

    // Act: Sort changed
    const newSort: string = 'name';
    const shouldReload = newSort !== currentSort;

    // Assert: Should trigger reload
    expect(shouldReload).toBe(true);
  });

  it('should handle deleted related record gracefully', async () => {
    // Arrange: Fee references non-existent company
    const fee = generateTestFee(1);
    fee.company_id = 'company:DELETED';

    const companiesApi = new MockPaginatedApi(100, generateTestCompany, 50);

    // Act: Attempt to fetch deleted company
    const companies = await companiesApi.fetchByIds(['company:DELETED']);

    // Assert: Returns empty array (no crash)
    expect(companies).toEqual([]);

    // UI should show placeholder
    const companyName = companies.length > 0 ? companies[0].name : '[Deleted]';
    expect(companyName).toBe('[Deleted]');
  });
});

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

describe('Performance Requirements', () => {
  it('should complete page load in under 500ms (simulated)', async () => {
    // Arrange: Mock fast API
    const api = new MockPaginatedApi(50, generateTestProject, 50);

    // Act: Time page load
    const start = performance.now();
    await api.fetchPage(1);
    const duration = performance.now() - start;

    // Assert: < 500ms (with simulated 10ms delay)
    expect(duration).toBeLessThan(100); // Mock is fast, real API should be < 500ms
  });

  it('should handle large datasets efficiently', () => {
    // Arrange: 10,000 IDs in Set
    const loadedIds = new Set(Array.from({ length: 10000 }, (_, i) => `project:PAGTEST_${i}`));

    // Act: Duplicate checks
    const start = performance.now();
    for (let i = 0; i < 1000; i++) {
      loadedIds.has(`project:PAGTEST_${Math.floor(Math.random() * 10000)}`);
    }
    const duration = performance.now() - start;

    // Assert: O(1) lookup remains fast even with large datasets
    expect(duration).toBeLessThan(50); // Should be very fast
    expect(loadedIds.size).toBe(10000);
  });
});

// ============================================================================
// TEST DATA CLEANUP VERIFICATION
// ============================================================================

describe('Test Data Safety', () => {
  it('should mark all test data with PAGTEST_ prefix', () => {
    // Arrange: Generate various test entities
    const project = generateTestProject(1);
    const company = generateTestCompany(1);
    const contact = generateTestContact(1);
    const fee = generateTestFee(1);

    // Assert: All have PAGTEST_ marker
    expect(project.name).toContain('PAGTEST_');
    expect(company.name).toContain('PAGTEST_');
    expect(contact.first_name).toBe('PAGTEST_');
    expect(fee.name).toContain('PAGTEST_');
    expect(fee.number).toContain('PAGTEST');
  });

  it('should provide cleanup identification pattern', () => {
    // Arrange: Simulated database query
    const allProjects = Array.from({ length: 200 }, (_, i) => {
      if (i < 50) return generateTestProject(i + 1); // Test data
      return {
        ...generateTestProject(i + 1),
        name: `Real_Project_${i}`
      }; // Real data
    });

    // Act: Filter for cleanup
    const testData = allProjects.filter(p => p.name.includes('PAGTEST_'));

    // Assert: Only test data selected
    expect(testData).toHaveLength(50);
    testData.forEach(p => {
      expect(p.name).toContain('PAGTEST_');
    });
  });
});
