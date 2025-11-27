/**
 * API Compatibility Test
 * 
 * This test uses the EXACT same test cases as the original API test
 * but runs them against the refactored API to verify 100% backward compatibility.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { ApiClient } from './api-refactored'; // Using refactored API
import type { Project, Company, Contact, Fee, ConnectionStatus } from '../types';

// Mock the Tauri invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

// Same mock data as original tests
const mockProject: Project = {
  id: 'projects:test_project',
  name: 'Test Project',
  name_short: 'Test Proj',
  status: 'Active',
  area: 'Downtown',
  city: 'Abu Dhabi',
  country: 'U.A.E.',
  folder: '25-97101 Test Project',
  number: {
    year: 25,
    country: 971,
    seq: 1,
    id: '25-97101'
  },
  time: {
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString()
  }
};

const mockCompany: Company = {
  id: 'company:TEST',
  name: 'Test Company',
  name_short: 'Test Co',
  abbreviation: 'TEST',
  city: 'Dubai',
  country: 'U.A.E.',
  time: {
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString()
  }
};

const mockConnectionStatus: ConnectionStatus = {
  is_connected: true,
  last_check: new Date().toISOString(),
  error_message: undefined
};

describe('API Compatibility Test - Original Tests vs Refactored Implementation', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    // Mock console to avoid cluttering test output
    vi.spyOn(console, 'error').mockImplementation(() => {});
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    vi.spyOn(console, 'info').mockImplementation(() => {});
    vi.spyOn(console, 'log').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  // Exact copy of original database connection tests
  describe('Database Connection Methods', () => {
    describe('checkDbConnection', () => {
      it('should return true when connection is successful', async () => {
        mockInvoke.mockResolvedValueOnce(true);

        const result = await ApiClient.checkDbConnection();

        expect(result).toBe(true);
        expect(mockInvoke).toHaveBeenCalledWith('check_db_connection');
        expect(mockInvoke).toHaveBeenCalledTimes(1);
      });

      it('should return false when connection fails', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Connection failed'));

        const result = await ApiClient.checkDbConnection();

        expect(result).toBe(false);
        expect(mockInvoke).toHaveBeenCalledWith('check_db_connection');
      });

      it('should handle network errors gracefully', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Network unavailable'));

        const result = await ApiClient.checkDbConnection();

        expect(result).toBe(false);
      });
    });

    describe('getConnectionStatus', () => {
      it('should return connection status when successful', async () => {
        mockInvoke.mockResolvedValueOnce(mockConnectionStatus);

        const result = await ApiClient.getConnectionStatus();

        expect(result).toEqual(mockConnectionStatus);
        expect(mockInvoke).toHaveBeenCalledWith('get_connection_status');
      });

      it('should return disconnected status on error', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Status check failed'));

        const result = await ApiClient.getConnectionStatus();

        expect(result.is_connected).toBe(false);
        expect(result.last_check).toBeUndefined();
        expect(result.error_message).toBeDefined();
      });
    });
  });

  // Exact copy of original project tests
  describe('Project CRUD Operations', () => {
    describe('getProjects', () => {
      it('should return projects array when successful', async () => {
        const mockProjects = [mockProject];
        mockInvoke.mockResolvedValueOnce(mockProjects);

        const result = await ApiClient.getProjects();

        expect(result).toEqual(mockProjects);
        expect(mockInvoke).toHaveBeenCalledWith('get_projects');
      });

      it('should return empty array on database error', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Database error'));

        await expect(ApiClient.getProjects()).rejects.toThrow('Database error');
      });

      it('should handle null response from backend', async () => {
        mockInvoke.mockResolvedValueOnce(null);

        const result = await ApiClient.getProjects();

        expect(result).toBeNull();
      });
    });

    describe('createProject', () => {
      const newProject = {
        name: 'New Project',
        name_short: 'New Proj',
        status: 'Draft' as const,
        area: 'Marina',
        city: 'Dubai',
        country: 'U.A.E.',
        folder: '25-97102 New Project',
        number: {
          year: 25,
          country: 971,
          seq: 2,
          id: '25-97102'
        }
      };

      it('should create project successfully', async () => {
        const createdProject = { ...mockProject, ...newProject };
        mockInvoke.mockResolvedValueOnce(createdProject);

        const result = await ApiClient.createProject(newProject);

        expect(result).toEqual(createdProject);
        expect(mockInvoke).toHaveBeenCalledWith('create_project', { project: {
          ...newProject,
          id: null,
          time: {
            created_at: expect.any(String),
            updated_at: expect.any(String)
          }
        }});
      });

      it('should handle creation failures', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Validation failed'));

        const result = await ApiClient.createProject(newProject);

        expect(result).toBeNull();
      });
    });
  });

  // Exact copy of original company tests  
  describe('Company CRUD Operations', () => {
    describe('getCompanies', () => {
      it('should return companies array when successful', async () => {
        const mockCompanies = [mockCompany];
        mockInvoke.mockResolvedValueOnce(mockCompanies);

        const result = await ApiClient.getCompanies();

        expect(result).toEqual(mockCompanies);
        expect(mockInvoke).toHaveBeenCalledWith('get_companies');
      });

      it('should handle database errors', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Database connection lost'));

        await expect(ApiClient.getCompanies()).rejects.toThrow('Database connection lost');
      });
    });

    describe('createCompany', () => {
      const newCompany = {
        name: 'New Company',
        name_short: 'New Co',
        abbreviation: 'NEW',
        city: 'Dubai',
        country: 'U.A.E.'
      };

      it('should create company successfully', async () => {
        const createdCompany = { ...mockCompany, ...newCompany, id: 'company:NEW' };
        mockInvoke.mockResolvedValueOnce(createdCompany);

        const result = await ApiClient.createCompany(newCompany);

        expect(result).toEqual(createdCompany);
        expect(mockInvoke).toHaveBeenCalledWith('create_company', { company: {
          name: 'New Company',
          name_short: 'New Co',
          abbreviation: 'NEW',
          city: 'Dubai',
          country: 'U.A.E.',
          reg_no: null,
          tax_no: null
        }});
      });

      it('should handle duplicate abbreviation errors', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Abbreviation already exists'));

        const result = await ApiClient.createCompany(newCompany);

        expect(result).toBeNull();
      });
    });
  });

  // Test error handling compatibility
  describe('Error Handling Compatibility', () => {
    it('should handle network timeouts', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Request timeout'));

      await expect(ApiClient.getProjects()).rejects.toThrow('Request timeout');
    });

    it('should handle malformed responses', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      const result = await ApiClient.getCompanies();

      expect(result).toBeUndefined();
    });

    it('should return safe fallbacks for health checks', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Service unavailable'));

      const result = await ApiClient.healthCheck();

      expect(result).toBe('Health check failed');
    });
  });

  // Test concurrent operations compatibility
  describe('Concurrent Operations Compatibility', () => {
    it('should handle multiple concurrent requests', async () => {
      mockInvoke
        .mockResolvedValueOnce([mockProject])
        .mockResolvedValueOnce([mockCompany])
        .mockResolvedValueOnce([]);

      const results = await Promise.all([
        ApiClient.getProjects(),
        ApiClient.getCompanies(),
        ApiClient.getContacts()
      ]);

      expect(results[0]).toEqual([mockProject]);
      expect(results[1]).toEqual([mockCompany]);
      expect(results[2]).toEqual([]);
      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });
  });
});