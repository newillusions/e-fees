/**
 * Enhanced CRUD Utilities Test Suite
 * 
 * Comprehensive tests covering all CRUD operations, SurrealDB support,
 * optimistic updates, error handling, and professional logging integration.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import {
  useCrudStore,
  useModalState,
  useOperationState,
  withLoadingState,
  validateSurrealId,
  BaseCrudApi,
  createDebounced,
  createThrottled,
  createEntityStore,
  type CrudApi,
  type CrudStoreOptions,
  type SurrealEntity,
  type CrudState,
  type CrudActions
} from './crud';
import { logger } from '../services/logger';
import type { UnknownSurrealThing } from '../../types';

// Mock the logger
vi.mock('../services/logger', () => ({
  logger: {
    child: vi.fn().mockReturnValue({
      info: vi.fn(),
      warn: vi.fn(),
      error: vi.fn()
    }),
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn()
  },
  logApiError: vi.fn()
}));

// Mock SurrealDB utilities
vi.mock('../utils/surrealdb', () => ({
  extractSurrealId: vi.fn((id: any) => {
    if (typeof id === 'string') return id;
    if (id && typeof id === 'object' && id.id) {
      return typeof id.id === 'string' ? id.id : id.id.String || id.id;
    }
    return null;
  }),
  compareSurrealIds: vi.fn((id1: any, id2: any) => {
    const extractId = (id: any) => {
      if (typeof id === 'string') return id;
      if (id && typeof id === 'object' && id.id) {
        return typeof id.id === 'string' ? id.id : id.id.String || id.id;
      }
      return null;
    };
    return extractId(id1) === extractId(id2);
  })
}));

// Test entity interface
interface TestEntity extends SurrealEntity {
  id: UnknownSurrealThing;
  name: string;
  description?: string;
  status: 'active' | 'inactive';
  createdAt?: string;
}

// Mock API implementation
class MockTestApi implements CrudApi<TestEntity> {
  private items: TestEntity[] = [];
  private nextId = 1;

  async getAll(): Promise<TestEntity[]> {
    return Promise.resolve([...this.items]);
  }

  async search(query: string): Promise<TestEntity[]> {
    const lowerQuery = query.toLowerCase();
    return Promise.resolve(
      this.items.filter(item => 
        item.name.toLowerCase().includes(lowerQuery) ||
        (item.description && item.description.toLowerCase().includes(lowerQuery))
      )
    );
  }

  async filter(criteria: Record<string, unknown>): Promise<TestEntity[]> {
    return Promise.resolve(
      this.items.filter(item => {
        return Object.entries(criteria).every(([key, value]) => {
          const itemValue = (item as any)[key];
          return itemValue === value;
        });
      })
    );
  }

  async create(data: Omit<TestEntity, 'id'>): Promise<TestEntity> {
    const newItem: TestEntity = {
      ...data,
      id: `test_${this.nextId++}`,
      createdAt: new Date().toISOString()
    };
    this.items.push(newItem);
    return Promise.resolve(newItem);
  }

  async update(id: string, data: Partial<TestEntity>): Promise<TestEntity> {
    const index = this.items.findIndex(item => {
      const itemId = typeof item.id === 'string' ? item.id : (item.id as any)?.id || item.id;
      return itemId === id;
    });
    
    if (index === -1) {
      throw new Error(`Item with id ${id} not found`);
    }
    
    this.items[index] = { ...this.items[index], ...data };
    return Promise.resolve(this.items[index]);
  }

  async delete(id: string): Promise<TestEntity> {
    const index = this.items.findIndex(item => {
      const itemId = typeof item.id === 'string' ? item.id : (item.id as any)?.id || item.id;
      return itemId === id;
    });
    
    if (index === -1) {
      throw new Error(`Item with id ${id} not found`);
    }
    
    const deletedItem = this.items[index];
    this.items.splice(index, 1);
    return Promise.resolve(deletedItem);
  }

  // Test helper methods
  reset(): void {
    this.items = [];
    this.nextId = 1;
  }

  seedData(items: TestEntity[]): void {
    this.items = [...items];
  }
}

// Mock error API for testing error scenarios
class MockErrorApi implements CrudApi<TestEntity> {
  async getAll(): Promise<TestEntity[]> {
    throw new Error('Database connection failed');
  }

  async create(): Promise<TestEntity> {
    throw new Error('Validation failed');
  }

  async update(): Promise<TestEntity> {
    throw new Error('Item not found');
  }

  async delete(): Promise<TestEntity> {
    throw new Error('Cannot delete referenced item');
  }
}

describe('Enhanced CRUD Utilities', () => {
  let mockApi: MockTestApi;
  let mockErrorApi: MockErrorApi;

  beforeEach(() => {
    vi.clearAllMocks();
    mockApi = new MockTestApi();
    mockErrorApi = new MockErrorApi();
  });

  afterEach(() => {
    mockApi.reset();
  });

  describe('useCrudStore', () => {
    describe('Basic CRUD Operations', () => {
      it('should initialize with empty state', () => {
        const { store } = useCrudStore(mockApi);
        const state = get(store);

        expect(state.items).toEqual([]);
        expect(state.filteredItems).toEqual([]);
        expect(state.loading).toBe(false);
        expect(state.saving).toBe(false);
        expect(state.error).toBe(null);
        expect(state.searchQuery).toBe('');
        expect(state.filters).toEqual({});
        expect(state.sort).toBe(null);
        expect(state.lastUpdated).toBe(null);
        expect(state.optimisticUpdates.size).toBe(0);
      });

      it('should load items successfully', async () => {
        const testItems: TestEntity[] = [
          { id: 'test_1', name: 'Item 1', status: 'active' },
          { id: 'test_2', name: 'Item 2', status: 'inactive' }
        ];
        mockApi.seedData(testItems);

        const { store, actions } = useCrudStore(mockApi);

        await actions.load();
        const state = get(store);

        expect(state.items).toEqual(testItems);
        expect(state.filteredItems).toEqual(testItems);
        expect(state.loading).toBe(false);
        expect(state.error).toBe(null);
        expect(state.lastUpdated).toBeInstanceOf(Date);
      });

      it('should handle load errors', async () => {
        const { store, actions } = useCrudStore(mockErrorApi);

        await expect(actions.load()).rejects.toThrow('Database connection failed');
        
        const state = get(store);
        expect(state.loading).toBe(false);
        expect(state.error).toBe('Database connection failed');
        expect(state.items).toEqual([]);
      });

      it('should create items successfully', async () => {
        const { store, actions } = useCrudStore(mockApi);
        const newItemData = { name: 'New Item', status: 'active' as const };

        const createdItem = await actions.create(newItemData);
        const state = get(store);

        expect(createdItem.name).toBe('New Item');
        expect(createdItem.id).toBeDefined();
        expect(state.items).toContainEqual(createdItem);
        expect(state.filteredItems).toContainEqual(createdItem);
        expect(state.saving).toBe(false);
      });

      it('should update items successfully', async () => {
        const initialItem: TestEntity = { id: 'test_1', name: 'Item 1', status: 'active' };
        mockApi.seedData([initialItem]);

        const { store, actions } = useCrudStore(mockApi);
        await actions.load();

        const updateData = { name: 'Updated Item' };
        const updatedItem = await actions.update('test_1', updateData);
        const state = get(store);

        expect(updatedItem.name).toBe('Updated Item');
        expect(state.items[0].name).toBe('Updated Item');
        expect(state.filteredItems[0].name).toBe('Updated Item');
        expect(state.saving).toBe(false);
      });

      it('should delete items successfully', async () => {
        const initialItem: TestEntity = { id: 'test_1', name: 'Item 1', status: 'active' };
        mockApi.seedData([initialItem]);

        const { store, actions } = useCrudStore(mockApi);
        await actions.load();

        const deletedItem = await actions.delete('test_1');
        const state = get(store);

        expect(deletedItem).toEqual(initialItem);
        expect(state.items).toEqual([]);
        expect(state.filteredItems).toEqual([]);
        expect(state.saving).toBe(false);
      });
    });

    describe('Optimistic Updates', () => {
      it('should perform optimistic creates', async () => {
        const { store, actions } = useCrudStore(mockApi, { 
          enableOptimistic: true 
        });

        const newItemData = { name: 'Optimistic Item', status: 'active' as const };
        
        // Start create operation but don't await yet
        const createPromise = actions.create(newItemData);
        
        // Check that optimistic item appears immediately
        let state = get(store);
        expect(state.items.length).toBe(1);
        expect(state.items[0].name).toBe('Optimistic Item');
        expect(state.optimisticUpdates.size).toBe(1);
        
        // Wait for completion
        await createPromise;
        
        state = get(store);
        expect(state.items.length).toBe(1);
        expect(state.optimisticUpdates.size).toBe(0);
      });

      it('should rollback optimistic creates on error', async () => {
        const { store, actions } = useCrudStore(mockErrorApi, { 
          enableOptimistic: true 
        });

        const newItemData = { name: 'Failing Item', status: 'active' as const };

        try {
          await actions.create(newItemData);
        } catch (error) {
          // Expected error
        }

        const state = get(store);
        expect(state.items).toEqual([]);
        expect(state.optimisticUpdates.size).toBe(0);
        expect(state.error).toBe('Validation failed');
      });

      it('should perform optimistic updates', async () => {
        const initialItem: TestEntity = { id: 'test_1', name: 'Item 1', status: 'active' };
        mockApi.seedData([initialItem]);

        const { store, actions } = useCrudStore(mockApi, { 
          enableOptimistic: true 
        });
        await actions.load();

        const updateData = { name: 'Optimistic Update' };
        
        // Start update but don't await
        const updatePromise = actions.update('test_1', updateData);
        
        // Check optimistic update
        let state = get(store);
        expect(state.items[0].name).toBe('Optimistic Update');
        expect(state.optimisticUpdates.size).toBe(1);
        
        // Wait for completion
        await updatePromise;
        
        state = get(store);
        expect(state.optimisticUpdates.size).toBe(0);
      });

      it('should rollback optimistic updates on error', async () => {
        const initialItem: TestEntity = { id: 'test_1', name: 'Item 1', status: 'active' };
        mockApi.seedData([initialItem]);

        const { store, actions } = useCrudStore(mockErrorApi, { 
          enableOptimistic: true 
        });
        
        // Manually set initial state since load will fail
        store.update(state => ({
          ...state,
          items: [initialItem],
          filteredItems: [initialItem]
        }));

        const updateData = { name: 'Failing Update' };

        try {
          await actions.update('test_1', updateData);
        } catch (error) {
          // Expected error
        }

        const state = get(store);
        expect(state.items[0].name).toBe('Item 1'); // Rolled back
        expect(state.optimisticUpdates.size).toBe(0);
      });

      it('should support manual rollback', async () => {
        // Create a slower mock API for this test
        const slowMockApi = {
          ...mockApi,
          async create(data: Omit<TestEntity, 'id'>): Promise<TestEntity> {
            // Add delay to simulate slower API
            await new Promise(resolve => setTimeout(resolve, 50));
            return mockApi.create(data);
          }
        };

        const { store, actions } = useCrudStore(slowMockApi, { 
          enableOptimistic: true 
        });

        const newItemData = { name: 'Rollback Test', status: 'active' as const };
        
        // Start optimistic create
        const createPromise = actions.create(newItemData);
        
        // Wait a bit for optimistic update to take effect
        await new Promise(resolve => setTimeout(resolve, 10));
        
        // Rollback before completion
        actions.rollback();
        
        let state = get(store);
        expect(state.items).toEqual([]);
        expect(state.optimisticUpdates.size).toBe(0);
        
        // Wait for actual completion
        const createdItem = await createPromise;
        
        // Should have real item now (rollback only removes optimistic, not real API results)
        state = get(store);
        expect(state.items.length).toBe(1);
        expect(state.items[0]).toEqual(createdItem);
      });
    });

    describe('Search and Filtering', () => {
      beforeEach(async () => {
        const testItems: TestEntity[] = [
          { id: 'test_1', name: 'Apple Product', description: 'Technology device', status: 'active' },
          { id: 'test_2', name: 'Banana Food', description: 'Healthy fruit', status: 'inactive' },
          { id: 'test_3', name: 'Cherry Drink', description: 'Sweet beverage', status: 'active' }
        ];
        mockApi.seedData(testItems);
      });

      it('should perform server-side search when API supports it', async () => {
        const { store, actions } = useCrudStore(mockApi);
        await actions.load();

        await actions.search('Apple');
        const state = get(store);

        expect(state.searchQuery).toBe('Apple');
        expect(state.filteredItems.length).toBe(1);
        expect(state.filteredItems[0].name).toBe('Apple Product');
      });

      it('should fall back to client-side search when API search fails', async () => {
        // Mock API search to fail
        const apiWithFailingSearch = {
          getAll: mockApi.getAll.bind(mockApi),
          create: mockApi.create.bind(mockApi),
          update: mockApi.update.bind(mockApi),
          delete: mockApi.delete.bind(mockApi),
          search: vi.fn().mockRejectedValue(new Error('Search API failed'))
        };

        const { store, actions } = useCrudStore(apiWithFailingSearch);
        await actions.load();

        await actions.search('Technology');
        const state = get(store);

        expect(state.searchQuery).toBe('Technology');
        expect(state.filteredItems.length).toBe(1);
        expect(state.filteredItems[0].description).toContain('Technology');
      });

      it('should filter items by criteria', async () => {
        const { store, actions } = useCrudStore(mockApi);
        await actions.load();

        await actions.applyFilters({ status: 'active' });
        const state = get(store);

        expect(state.filters).toEqual({ status: 'active' });
        expect(state.filteredItems.length).toBe(2);
        expect(state.filteredItems.every(item => item.status === 'active')).toBe(true);
      });

      it('should combine search and filters', async () => {
        const { store, actions } = useCrudStore(mockApi);
        await actions.load();

        await actions.search('Product');
        await actions.applyFilters({ status: 'active' });
        const state = get(store);

        expect(state.filteredItems.length).toBe(1);
        expect(state.filteredItems[0].name).toBe('Apple Product');
        expect(state.filteredItems[0].status).toBe('active');
      });

      it('should reset filters and search', async () => {
        const { store, actions } = useCrudStore(mockApi);
        await actions.load();

        await actions.search('Apple');
        await actions.applyFilters({ status: 'active' });
        
        // Check that we have the expected initial state
        let state = get(store);
        expect(state.items.length).toBe(3); // All items loaded from mock
        
        actions.resetFilters();
        state = get(store);

        expect(state.searchQuery).toBe('');
        expect(state.filters).toEqual({});
        expect(state.filteredItems.length).toBe(state.items.length); // Should match items length
      });
    });

    describe('Sorting', () => {
      beforeEach(async () => {
        const testItems: TestEntity[] = [
          { id: 'test_1', name: 'Charlie', status: 'active' },
          { id: 'test_2', name: 'Alpha', status: 'inactive' },
          { id: 'test_3', name: 'Beta', status: 'active' }
        ];
        mockApi.seedData(testItems);
      });

      it('should sort items ascending', async () => {
        const { store, actions } = useCrudStore(mockApi);
        await actions.load();

        actions.sort('name', 'asc');
        const state = get(store);

        expect(state.sort).toEqual({ field: 'name', direction: 'asc' });
        expect(state.filteredItems[0].name).toBe('Alpha');
        expect(state.filteredItems[1].name).toBe('Beta');
        expect(state.filteredItems[2].name).toBe('Charlie');
      });

      it('should sort items descending', async () => {
        const { store, actions } = useCrudStore(mockApi);
        await actions.load();

        actions.sort('name', 'desc');
        const state = get(store);

        expect(state.sort).toEqual({ field: 'name', direction: 'desc' });
        expect(state.filteredItems[0].name).toBe('Charlie');
        expect(state.filteredItems[1].name).toBe('Beta');
        expect(state.filteredItems[2].name).toBe('Alpha');
      });

      it('should default to ascending when direction not specified', async () => {
        const { store, actions } = useCrudStore(mockApi);
        await actions.load();

        actions.sort('name');
        const state = get(store);

        expect(state.sort?.direction).toBe('asc');
      });
    });

    describe('Utility Methods', () => {
      it('should get item by ID', async () => {
        const testItem: TestEntity = { id: 'test_1', name: 'Test Item', status: 'active' };
        mockApi.seedData([testItem]);

        const { actions } = useCrudStore(mockApi);
        await actions.load();

        const foundItem = actions.getById('test_1');
        expect(foundItem).toEqual(testItem);

        const notFoundItem = actions.getById('nonexistent');
        expect(notFoundItem).toBe(null);
      });

      it('should clear all data', () => {
        const { store, actions } = useCrudStore(mockApi);
        
        // Set some state
        store.update(state => ({
          ...state,
          items: [{ id: 'test_1', name: 'Test', status: 'active' }],
          error: 'Some error',
          searchQuery: 'test'
        }));

        actions.clear();
        const state = get(store);

        expect(state.items).toEqual([]);
        expect(state.error).toBe(null);
        expect(state.searchQuery).toBe('');
      });

      it('should set error messages', () => {
        const { store, actions } = useCrudStore(mockApi);

        actions.setError('Test error');
        const state = get(store);

        expect(state.error).toBe('Test error');
      });
    });

    describe('Configuration Options', () => {
      it('should support custom options', async () => {
        const options: CrudStoreOptions = {
          enableOptimistic: false,
          enableLogging: false,
          component: 'TestComponent'
        };

        const { store, actions } = useCrudStore(mockApi, options);
        
        // Test that optimistic updates are disabled
        const newItemData = { name: 'Non-optimistic Item', status: 'active' as const };
        const createPromise = actions.create(newItemData);
        
        // Should not appear optimistically
        const state = get(store);
        expect(state.items.length).toBe(0);
        
        await createPromise;
        // Should appear only after completion
        const finalState = get(store);
        expect(finalState.items.length).toBe(1);
      });

      it('should support auto-refresh', async () => {
        const autoRefreshOptions: CrudStoreOptions = {
          autoRefresh: 100 // 100ms for testing
        };

        const { actions, destroy } = useCrudStore(mockApi, autoRefreshOptions);
        
        let loadCallCount = 0;
        const originalLoad = actions.load;
        actions.load = vi.fn().mockImplementation(() => {
          loadCallCount++;
          return originalLoad();
        });

        // Wait for auto-refresh to trigger
        await new Promise(resolve => {
          setTimeout(() => {
            expect(loadCallCount).toBeGreaterThan(0);
            destroy(); // Clean up
            resolve(void 0);
          }, 150);
        });
      });
    });

    describe('Cleanup', () => {
      it('should cleanup auto-refresh on destroy', () => {
        const clearIntervalSpy = vi.spyOn(global, 'clearInterval');
        
        const { destroy } = useCrudStore(mockApi, { 
          autoRefresh: 1000 
        });

        destroy();
        expect(clearIntervalSpy).toHaveBeenCalled();
      });
    });
  });

  describe('SurrealDB Utilities', () => {
    describe('validateSurrealId', () => {
      it('should validate string IDs', () => {
        expect(validateSurrealId('test_id')).toBe(true);
        expect(validateSurrealId('')).toBe(false);
        expect(validateSurrealId(null)).toBe(false);
        expect(validateSurrealId(undefined)).toBe(false);
      });

      it('should validate SurrealDB Thing objects', () => {
        expect(validateSurrealId({ tb: 'test', id: 'test_id' })).toBe(true);
        expect(validateSurrealId({ tb: 'test', id: { String: 'test_id' } })).toBe(true);
        expect(validateSurrealId({ tb: 'test' })).toBe(false);
        expect(validateSurrealId({ id: 'test_id' })).toBe(false);
        expect(validateSurrealId({ tb: '', id: 'test_id' })).toBe(false);
        expect(validateSurrealId({ tb: 'test', id: '' })).toBe(false);
      });

      it('should validate other types', () => {
        expect(validateSurrealId(123)).toBe(false);
        expect(validateSurrealId([])).toBe(false);
        expect(validateSurrealId({})).toBe(false);
      });
    });
  });

  describe('BaseCrudApi', () => {
    class TestCrudApi extends BaseCrudApi<TestEntity> {
      private items: TestEntity[] = [];

      async getAll(): Promise<TestEntity[]> {
        this.logSuccess('getAll', this.items);
        return Promise.resolve([...this.items]);
      }

      async create(data: Omit<TestEntity, 'id'>): Promise<TestEntity> {
        const newItem: TestEntity = { ...data, id: `test_${Date.now()}` };
        this.items.push(newItem);
        this.logSuccess('create', newItem);
        return Promise.resolve(newItem);
      }

      async update(id: string, data: Partial<TestEntity>): Promise<TestEntity> {
        const index = this.items.findIndex(item => 
          (typeof item.id === 'string' ? item.id : (item.id as any)?.id) === id
        );
        
        if (index === -1) {
          this.handleError('update', new Error('Item not found'), { id });
        }
        
        this.items[index] = { ...this.items[index], ...data };
        this.logSuccess('update', this.items[index]);
        return Promise.resolve(this.items[index]);
      }

      async delete(id: string): Promise<TestEntity> {
        const index = this.items.findIndex(item => 
          (typeof item.id === 'string' ? item.id : (item.id as any)?.id) === id
        );
        
        if (index === -1) {
          this.handleError('delete', new Error('Item not found'), { id });
        }
        
        const deletedItem = this.items[index];
        this.items.splice(index, 1);
        this.logSuccess('delete', deletedItem);
        return Promise.resolve(deletedItem);
      }

      // Test helper
      seedData(items: TestEntity[]): void {
        this.items = [...items];
      }
    }

    it('should create API with logging', () => {
      const api = new TestCrudApi('test');
      expect(api).toBeDefined();
      expect(logger.child).toHaveBeenCalledWith({ component: 'testApi' });
    });

    it('should handle successful operations with logging', async () => {
      const api = new TestCrudApi('test');
      const newItem = await api.create({ name: 'Test Item', status: 'active' });
      
      expect(newItem.name).toBe('Test Item');
      expect(newItem.id).toBeDefined();
    });

    it('should handle errors with proper logging', async () => {
      const api = new TestCrudApi('test');
      
      await expect(api.update('nonexistent', { name: 'Updated' }))
        .rejects.toThrow('Item not found');
    });

    it('should throw error for unimplemented search', async () => {
      const api = new TestCrudApi('test');
      
      await expect(api.search?.('query'))
        .rejects.toThrow('Search not implemented for test');
    });

    it('should throw error for unimplemented filter', async () => {
      const api = new TestCrudApi('test');
      
      await expect(api.filter?.({ status: 'active' }))
        .rejects.toThrow('Filter not implemented for test');
    });
  });

  describe('Modal State Management', () => {
    it('should initialize modal state', () => {
      const { store } = useModalState<TestEntity>();
      const state = get(store);

      expect(state.isOpen).toBe(false);
      expect(state.mode).toBe('create');
      expect(state.item).toBe(null);
    });

    it('should open create modal', () => {
      const { store, actions } = useModalState<TestEntity>();

      actions.openCreate();
      const state = get(store);

      expect(state.isOpen).toBe(true);
      expect(state.mode).toBe('create');
      expect(state.item).toBe(null);
    });

    it('should open edit modal', () => {
      const { store, actions } = useModalState<TestEntity>();
      const testItem: TestEntity = { id: 'test_1', name: 'Test Item', status: 'active' };

      actions.openEdit(testItem);
      const state = get(store);

      expect(state.isOpen).toBe(true);
      expect(state.mode).toBe('edit');
      expect(state.item).toEqual(testItem);
    });

    it('should close modal', () => {
      const { store, actions } = useModalState<TestEntity>();
      const testItem: TestEntity = { id: 'test_1', name: 'Test Item', status: 'active' };

      actions.openEdit(testItem);
      actions.close();
      const state = get(store);

      expect(state.isOpen).toBe(false);
      expect(state.mode).toBe('create');
      expect(state.item).toBe(null);
    });
  });

  describe('Operation State Management', () => {
    it('should initialize operation state', () => {
      const { store } = useOperationState();
      const state = get(store);

      expect(state.loading).toBe(false);
      expect(state.saving).toBe(false);
      expect(state.deleting).toBe(false);
      expect(state.message).toBe('');
      expect(state.error).toBe(null);
    });

    it('should manage loading states', () => {
      const { store, actions } = useOperationState();

      actions.setLoading(true);
      expect(get(store).loading).toBe(true);

      actions.setSaving(true);
      expect(get(store).saving).toBe(true);

      actions.setDeleting(true);
      expect(get(store).deleting).toBe(true);
    });

    it('should manage messages', () => {
      const { store, actions } = useOperationState();

      actions.setMessage('Success message');
      let state = get(store);
      expect(state.message).toBe('Success message');
      expect(state.error).toBe(null);

      actions.setError('Error message');
      state = get(store);
      expect(state.error).toBe('Error message');
      expect(state.message).toBe('');
    });

    it('should clear messages', () => {
      const { store, actions } = useOperationState();

      actions.setMessage('Test message');
      actions.setError('Test error');
      actions.clearMessages();
      
      const state = get(store);
      expect(state.message).toBe('');
      expect(state.error).toBe(null);
    });

    it('should reset state', () => {
      const { store, actions } = useOperationState();

      actions.setLoading(true);
      actions.setMessage('Test message');
      actions.reset();
      
      const state = get(store);
      expect(state.loading).toBe(false);
      expect(state.message).toBe('');
    });
  });

  describe('withLoadingState', () => {
    it('should wrap operation with loading state', async () => {
      const { actions } = useOperationState();
      const mockOperation = vi.fn().mockResolvedValue('success');

      const result = await withLoadingState(mockOperation, actions, 'loading');

      expect(result).toBe('success');
      expect(mockOperation).toHaveBeenCalled();
    });

    it('should handle operation errors', async () => {
      const { actions } = useOperationState();
      const mockOperation = vi.fn().mockRejectedValue(new Error('Operation failed'));

      await expect(withLoadingState(mockOperation, actions, 'saving'))
        .rejects.toThrow('Operation failed');
    });
  });

  describe('Utility Functions', () => {
    describe('createDebounced', () => {
      it('should debounce function calls', async () => {
        const mockFn = vi.fn();
        const debouncedFn = createDebounced(mockFn, 50);

        debouncedFn('call1');
        debouncedFn('call2');
        debouncedFn('call3');

        // Should not be called immediately
        expect(mockFn).not.toHaveBeenCalled();

        await new Promise(resolve => {
          setTimeout(() => {
            expect(mockFn).toHaveBeenCalledTimes(1);
            expect(mockFn).toHaveBeenCalledWith('call3');
            resolve(void 0);
          }, 60);
        });
      });
    });

    describe('createThrottled', () => {
      it('should throttle function calls', () => {
        const mockFn = vi.fn();
        const throttledFn = createThrottled(mockFn, 50);

        throttledFn('call1');
        throttledFn('call2');
        throttledFn('call3');

        // Should be called immediately for first call
        expect(mockFn).toHaveBeenCalledTimes(1);
        expect(mockFn).toHaveBeenCalledWith('call1');
      });
    });

    describe('createEntityStore', () => {
      it('should create pre-configured store', () => {
        const { store, actions, destroy } = createEntityStore(mockApi, 'TestEntity');

        expect(store).toBeDefined();
        expect(actions).toBeDefined();
        expect(destroy).toBeDefined();
      });

      it('should use custom options', () => {
        const customOptions = { enableOptimistic: false };
        const { store } = createEntityStore(mockApi, 'TestEntity', customOptions);

        expect(store).toBeDefined();
      });
    });
  });
});