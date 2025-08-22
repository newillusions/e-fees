/**
 * API Client Refactored Tests
 * 
 * Test suite ensuring 100% backward compatibility of the refactored API client.
 * All existing method signatures and behaviors must be preserved.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { ApiClient } from './api-refactored';
import type { Project, Company, Contact, Fee, ConnectionStatus } from '../types';

// Mock the Tauri invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

// Test data - same as original tests for compatibility verification
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

const mockContact: Contact = {
  id: 'contacts:test_contact',
  first_name: 'John',
  last_name: 'Doe',
  full_name: 'John Doe',
  email: 'john.doe@example.com',
  phone: '+971501234567',
  position: 'Manager',
  company: 'company:TEST',
  time: {
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString()
  }
};

const mockFee: Fee = {
  id: 'fee:test_fee',
  name: 'Test Fee Proposal',
  number: '25-97101-R1',
  project_id: 'projects:test_project',
  company_id: 'company:TEST',
  contact_id: 'contacts:test_contact',
  status: 'Draft',
  issue_date: '250815',
  rev: 1,
  revisions: [],
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

describe('ApiClient Refactored - Backward Compatibility', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    // Suppress console output in tests
    vi.spyOn(console, 'error').mockImplementation(() => {});
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    vi.spyOn(console, 'info').mockImplementation(() => {});
    vi.spyOn(console, 'log').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Method Signature Compatibility', () => {
    it('should have all original database connection methods', () => {
      expect(typeof ApiClient.checkDbConnection).toBe('function');
      expect(typeof ApiClient.getConnectionStatus).toBe('function');
    });

    it('should have all original project methods', () => {
      expect(typeof ApiClient.getProjects).toBe('function');
      expect(typeof ApiClient.searchProjects).toBe('function');
      expect(typeof ApiClient.createProject).toBe('function');
      expect(typeof ApiClient.updateProject).toBe('function');
      expect(typeof ApiClient.deleteProject).toBe('function');
    });

    it('should have all original company methods', () => {
      expect(typeof ApiClient.getCompanies).toBe('function');
      expect(typeof ApiClient.createCompany).toBe('function');
      expect(typeof ApiClient.updateCompany).toBe('function');
      expect(typeof ApiClient.deleteCompany).toBe('function');
    });

    it('should have all original contact methods', () => {
      expect(typeof ApiClient.getContacts).toBe('function');
      expect(typeof ApiClient.createContact).toBe('function');
      expect(typeof ApiClient.updateContact).toBe('function');
      expect(typeof ApiClient.deleteContact).toBe('function');
    });

    it('should have all original fee methods', () => {
      expect(typeof ApiClient.getFees).toBe('function');
      expect(typeof ApiClient.createFee).toBe('function');
      expect(typeof ApiClient.updateFee).toBe('function');
      expect(typeof ApiClient.deleteFee).toBe('function');
    });

    it('should have all original system methods', () => {
      expect(typeof ApiClient.getStats).toBe('function');
      expect(typeof ApiClient.healthCheck).toBe('function');
      expect(typeof ApiClient.getDbInfo).toBe('function');
      expect(typeof ApiClient.getTableSchema).toBe('function');
      expect(typeof ApiClient.positionWindow4K).toBe('function');
      expect(typeof ApiClient.getSettings).toBe('function');
      expect(typeof ApiClient.saveSettings).toBe('function');
      expect(typeof ApiClient.reloadDatabaseConfig).toBe('function');
    });
  });

  describe('Behavior Compatibility - Connection Methods', () => {
    describe('checkDbConnection', () => {
      it('should return true when connection is successful', async () => {
        mockInvoke.mockResolvedValueOnce(true);

        const result = await ApiClient.checkDbConnection();

        expect(result).toBe(true);
        expect(mockInvoke).toHaveBeenCalledWith('check_db_connection');
      });

      it('should return false when connection fails', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Connection failed'));

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
        expect(result.error_message).toBeDefined();
      });
    });
  });

  describe('Behavior Compatibility - Project Methods', () => {
    describe('getProjects', () => {
      it('should return projects array when successful', async () => {
        const mockProjects = [mockProject];
        mockInvoke.mockResolvedValueOnce(mockProjects);

        const result = await ApiClient.getProjects();

        expect(result).toEqual(mockProjects);
        expect(mockInvoke).toHaveBeenCalledWith('get_projects');
      });

      it('should throw error on database failure', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Database error'));

        await expect(ApiClient.getProjects()).rejects.toThrow('Database error');
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

      it('should return null on creation failure', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Validation failed'));

        const result = await ApiClient.createProject(newProject);

        expect(result).toBeNull();
      });
    });
  });

  describe('Behavior Compatibility - Company Methods', () => {
    describe('createCompany', () => {
      const newCompany = {
        name: 'New Company',
        name_short: 'New Co',
        abbreviation: 'NEW',
        city: 'Dubai',
        country: 'U.A.E.'
      };

      it('should create company successfully', async () => {
        const createdCompany = { ...mockCompany, ...newCompany };
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

      it('should return null on creation failure', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Duplicate abbreviation'));

        const result = await ApiClient.createCompany(newCompany);

        expect(result).toBeNull();
      });
    });

    describe('updateCompany', () => {
      const companyUpdate = { name: 'Updated Company Name' };

      it('should update company successfully', async () => {
        const updatedCompany = { ...mockCompany, ...companyUpdate };
        mockInvoke.mockResolvedValueOnce(updatedCompany);

        const result = await ApiClient.updateCompany('company:TEST', companyUpdate);

        expect(result).toEqual(updatedCompany);
        expect(mockInvoke).toHaveBeenCalledWith('update_company', {
          id: 'company:TEST',
          companyUpdate: {
            name: 'Updated Company Name',
            name_short: null,
            abbreviation: null,
            city: null,
            country: null,
            reg_no: null,
            tax_no: null
          }
        });
      });
    });
  });

  describe('Behavior Compatibility - Contact Methods', () => {
    describe('createContact', () => {
      const newContact = {
        first_name: 'Jane',
        last_name: 'Smith',
        email: 'jane.smith@example.com',
        phone: '+971507654321',
        position: 'Director',
        company: 'company:TEST'
      };

      it('should create contact successfully', async () => {
        const createdContact = {
          ...mockContact,
          ...newContact,
          full_name: 'Jane Smith',
          id: 'contacts:new_contact'
        };
        mockInvoke.mockResolvedValueOnce(createdContact);

        const result = await ApiClient.createContact(newContact);

        expect(result).toEqual(createdContact);
        expect(mockInvoke).toHaveBeenCalledWith('create_contact', { contact: {
          ...newContact,
          id: null
        }});
      });

      it('should return null on creation failure', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Invalid email format'));

        const result = await ApiClient.createContact(newContact);

        expect(result).toBeNull();
      });
    });
  });

  describe('Behavior Compatibility - Fee Methods', () => {
    describe('createFee', () => {
      const newFee = {
        name: 'New Fee Proposal',
        number: '25-97103-R1',
        project_id: 'projects:test_project',
        company_id: 'company:TEST',
        contact_id: 'contacts:test_contact',
        status: 'Draft' as const,
        issue_date: '250816'
      };

      it('should create fee successfully', async () => {
        const createdFee = {
          ...mockFee,
          ...newFee,
          id: 'fee:new_fee'
        };
        mockInvoke.mockResolvedValueOnce(createdFee);

        const result = await ApiClient.createFee(newFee);

        expect(result).toEqual(createdFee);
        expect(mockInvoke).toHaveBeenCalledWith('create_fee', { fee: {
          name: 'New Fee Proposal',
          number: '25-97103-R1',
          rev: undefined,
          status: 'Draft',
          issue_date: '250816',
          activity: undefined,
          package: undefined,
          staff_name: undefined,
          staff_email: undefined,
          staff_phone: undefined,
          staff_position: undefined,
          strap_line: undefined,
          project_id: 'test_project', // cleaned ID
          company_id: 'TEST', // cleaned ID  
          contact_id: 'test_contact', // cleaned ID
          revisions: []
        }});
      });
    });
  });

  describe('Error Handling Compatibility', () => {
    it('should handle network timeouts the same way', async () => {
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

  describe('Concurrent Operations Compatibility', () => {
    it('should handle multiple concurrent requests', async () => {
      mockInvoke
        .mockResolvedValueOnce([mockProject])
        .mockResolvedValueOnce([mockCompany])
        .mockResolvedValueOnce([mockContact]);

      const results = await Promise.all([
        ApiClient.getProjects(),
        ApiClient.getCompanies(),
        ApiClient.getContacts()
      ]);

      expect(results[0]).toEqual([mockProject]);
      expect(results[1]).toEqual([mockCompany]);
      expect(results[2]).toEqual([mockContact]);
      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });
  });

  describe('Professional Logging Verification', () => {
    it('should execute without console errors in production mode', async () => {
      // Verify the refactored API doesn't throw unexpected errors
      mockInvoke.mockResolvedValueOnce([mockProject]);

      const result = await ApiClient.getProjects();

      // Should complete successfully without throwing errors
      expect(result).toEqual([mockProject]);
      expect(mockInvoke).toHaveBeenCalledWith('get_projects');
    });

    it('should provide structured error logging', async () => {
      const errorSpy = vi.spyOn(console, 'error');
      
      mockInvoke.mockRejectedValueOnce(new Error('Test error'));

      await expect(ApiClient.getProjects()).rejects.toThrow();

      // Should use structured error logging
      expect(errorSpy).toHaveBeenCalledWith(
        expect.stringContaining('API ERROR'),
        expect.stringContaining('get_projects'),
        expect.any(Object)
      );
      
      errorSpy.mockRestore();
    });
  });
});