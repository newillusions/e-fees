/**
 * API Client Tests
 * 
 * Comprehensive test suite for the ApiClient class covering all database operations,
 * connection management, and file system interactions.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { ApiClient } from './api';
import type { Project, Company, Contact, Fee, ConnectionStatus } from '../types';

// Mock the Tauri invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

// Mock data for testing
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

describe('ApiClient', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    // Mock console.error to avoid cluttering test output
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

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
        expect(console.error).toHaveBeenCalledWith(
          'Failed to check database connection:',
          expect.any(Error)
        );
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
        expect(console.error).toHaveBeenCalledWith(
          'Failed to fetch projects from database:',
          expect.any(Error)
        );
      });

      it('should handle null response from backend', async () => {
        mockInvoke.mockResolvedValueOnce(null);

        const result = await ApiClient.getProjects();

        expect(result).toBeNull();
      });
    });

    describe('searchProjects', () => {
      it('should return filtered projects for valid query', async () => {
        const searchResults = [mockProject];
        mockInvoke.mockResolvedValueOnce(searchResults);

        const result = await ApiClient.searchProjects('test');

        expect(result).toEqual(searchResults);
        expect(mockInvoke).toHaveBeenCalledWith('search_projects', { query: 'test' });
      });

      it('should return empty array for empty query', async () => {
        mockInvoke.mockResolvedValueOnce([]);
        
        const result = await ApiClient.searchProjects('');

        expect(result).toEqual([]);
        expect(mockInvoke).toHaveBeenCalledWith('search_projects', { query: '' });
      });

      it('should handle search errors gracefully', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Search failed'));

        await expect(ApiClient.searchProjects('test')).rejects.toThrow('Search failed');
        expect(console.error).toHaveBeenCalledWith(
          'Failed to search projects:',
          expect.any(Error)
        );
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
        expect(console.error).toHaveBeenCalledWith(
          'Failed to create project:',
          expect.any(Error)
        );
      });

      it('should validate required fields', async () => {
        const invalidProject = { name: '' } as any;
        mockInvoke.mockRejectedValueOnce(new Error('Validation failed'));

        const result = await ApiClient.createProject(invalidProject);

        expect(result).toBeNull();
        expect(console.error).toHaveBeenCalledWith(
          'Failed to create project:',
          expect.any(Error)
        );
      });
    });
  });

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
        expect(console.error).toHaveBeenCalledWith(
          'Failed to fetch companies from database:',
          expect.any(Error)
        );
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
        expect(console.error).toHaveBeenCalledWith(
          'Failed to create company:',
          expect.any(Error)
        );
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

      it('should handle non-existent company', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Company not found'));

        await expect(ApiClient.updateCompany('company:NONEXISTENT', companyUpdate)).rejects.toThrow('Company not found');
        expect(console.error).toHaveBeenCalledWith(
          'Failed to update company:',
          expect.any(Error)
        );
      });
    });

    describe('deleteCompany', () => {
      it('should delete company successfully', async () => {
        mockInvoke.mockResolvedValueOnce(mockCompany);

        const result = await ApiClient.deleteCompany('company:TEST');

        expect(result).toEqual(mockCompany);
        expect(mockInvoke).toHaveBeenCalledWith('delete_company', { id: 'company:TEST' });
      });

      it('should handle deletion errors', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Company has related records'));

        await expect(ApiClient.deleteCompany('company:TEST')).rejects.toThrow('Company has related records');
        expect(console.error).toHaveBeenCalledWith(
          'Failed to delete company:',
          expect.any(Error)
        );
      });
    });
  });

  describe('Contact CRUD Operations', () => {
    describe('getContacts', () => {
      it('should return contacts array', async () => {
        const mockContacts = [mockContact];
        mockInvoke.mockResolvedValueOnce(mockContacts);

        const result = await ApiClient.getContacts();

        expect(result).toEqual(mockContacts);
        expect(mockInvoke).toHaveBeenCalledWith('get_contacts');
      });

      it('should handle empty contacts list', async () => {
        mockInvoke.mockResolvedValueOnce([]);

        const result = await ApiClient.getContacts();

        expect(result).toEqual([]);
      });
    });

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
          first_name: 'Jane',
          last_name: 'Smith',
          email: 'jane.smith@example.com',
          phone: '+971507654321',
          position: 'Director',
          company: 'company:TEST',
          id: null
        }});
      });

      it('should handle email validation errors', async () => {
        const invalidContact = { ...newContact, email: 'invalid-email' };
        mockInvoke.mockRejectedValueOnce(new Error('Invalid email format'));

        const result = await ApiClient.createContact(invalidContact);

        expect(result).toBeNull();
      });
    });

    describe('updateContact', () => {
      const contactUpdate = { position: 'Senior Director' };

      it('should update contact successfully', async () => {
        const updatedContact = { ...mockContact, ...contactUpdate };
        mockInvoke.mockResolvedValueOnce(updatedContact);

        const result = await ApiClient.updateContact('contacts:test_contact', contactUpdate);

        expect(result).toEqual(updatedContact);
        expect(mockInvoke).toHaveBeenCalledWith('update_contact', {
          id: 'contacts:test_contact',
          contactUpdate: contactUpdate
        });
      });
    });

    describe('deleteContact', () => {
      it('should delete contact successfully', async () => {
        mockInvoke.mockResolvedValueOnce(mockContact);

        const result = await ApiClient.deleteContact('contacts:test_contact');

        expect(result).toEqual(mockContact);
        expect(mockInvoke).toHaveBeenCalledWith('delete_contact', { id: 'contacts:test_contact' });
      });
    });
  });

  describe('Fee CRUD Operations', () => {
    describe('getFees', () => {
      it('should return fees array', async () => {
        const mockFees = [mockFee];
        mockInvoke.mockResolvedValueOnce(mockFees);

        const result = await ApiClient.getFees();

        expect(result).toEqual(mockFees);
        expect(mockInvoke).toHaveBeenCalledWith('get_fees');
      });

      it('should handle database connection issues', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Connection timeout'));

        await expect(ApiClient.getFees()).rejects.toThrow('Connection timeout');
        expect(console.error).toHaveBeenCalledWith(
          'Failed to fetch fees from database:',
          expect.any(Error)
        );
      });
    });

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

      it('should handle foreign key constraint errors', async () => {
        const invalidFee = { ...newFee, project_id: 'projects:nonexistent' };
        mockInvoke.mockRejectedValueOnce(new Error('Invalid project reference'));

        const result = await ApiClient.createFee(invalidFee);

        expect(result).toBeNull();
      });
    });

    describe('updateFee', () => {
      const feeUpdate = { status: 'Sent' as const };

      it('should update fee successfully', async () => {
        const updatedFee = { ...mockFee, ...feeUpdate };
        mockInvoke.mockResolvedValueOnce(updatedFee);

        const result = await ApiClient.updateFee('fee:test_fee', feeUpdate);

        expect(result).toEqual(updatedFee);
        expect(mockInvoke).toHaveBeenCalledWith('update_fee', {
          id: 'fee:test_fee',
          fee: {
            status: 'Sent',
            id: null,
            time: {
              updated_at: expect.any(String) // Any ISO string
            }
          }
        });
      });
    });

    describe('deleteFee', () => {
      it('should delete fee successfully', async () => {
        mockInvoke.mockResolvedValueOnce(mockFee);

        const result = await ApiClient.deleteFee('fee:test_fee');

        expect(result).toEqual(mockFee);
        expect(mockInvoke).toHaveBeenCalledWith('delete_fee', { id: 'fee:test_fee' });
      });
    });
  });

  describe('File Operations', () => {
    describe('writeFeeToJson', () => {
      it('should write fee to JSON successfully', async () => {
        const jsonPath = '/path/to/fee.json';
        mockInvoke.mockResolvedValueOnce(jsonPath);

        const result = await ApiClient.writeFeeToJson('fee:test_fee');

        expect(result).toBe(jsonPath);
        expect(mockInvoke).toHaveBeenCalledWith('write_fee_to_json', { rfpId: 'fee:test_fee' });
      });

      it('should handle file write errors', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('Permission denied'));

        const result = await ApiClient.writeFeeToJson('fee:test_fee');

        expect(result).toBeNull();
        expect(console.error).toHaveBeenCalledWith(
          'Failed to write RFP data to JSON:',
          expect.any(Error)
        );
      });
    });

    describe('checkProjectFolderExists', () => {
      it('should return true for existing folder', async () => {
        mockInvoke.mockResolvedValueOnce(true);

        const result = await ApiClient.checkProjectFolderExists('25-97101', 'Test Project');

        expect(result).toBe(true);
        expect(mockInvoke).toHaveBeenCalledWith('check_project_folder_exists', {
          projectNumber: '25-97101',
          projectShortName: 'Test Project'
        });
      });

      it('should return false for non-existent folder', async () => {
        mockInvoke.mockResolvedValueOnce(false);

        const result = await ApiClient.checkProjectFolderExists('25-97999', 'Nonexistent');

        expect(result).toBe(false);
      });

      it('should handle file system errors', async () => {
        mockInvoke.mockRejectedValueOnce(new Error('File system error'));

        const result = await ApiClient.checkProjectFolderExists('25-97101', 'Test Project');

        expect(result).toBe(false);
        expect(console.error).toHaveBeenCalledWith(
          'Failed to check project folder existence:',
          expect.any(Error)
        );
      });
    });
  });

  describe('Error Handling', () => {
    it('should handle network timeouts', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Request timeout'));

      await expect(ApiClient.getProjects()).rejects.toThrow('Request timeout');
      expect(console.error).toHaveBeenCalledWith(
        'Failed to fetch projects from database:',
        expect.any(Error)
      );
    });

    it('should handle malformed responses', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      const result = await ApiClient.getCompanies();

      expect(result).toBeUndefined();
    });

    it('should handle JSON parsing errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid JSON response'));

      await expect(ApiClient.getContacts()).rejects.toThrow('Invalid JSON response');
      expect(console.error).toHaveBeenCalledWith(
        'Failed to fetch contacts from database:',
        expect.any(Error)
      );
    });
  });

  describe('Concurrent Operations', () => {
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

    it('should handle partial failures in concurrent operations', async () => {
      mockInvoke
        .mockResolvedValueOnce([mockProject])
        .mockRejectedValueOnce(new Error('Database error'))
        .mockResolvedValueOnce([mockContact]);

      // Promise.all rejects if any promise rejects, so we expect the whole operation to throw
      await expect(Promise.all([
        ApiClient.getProjects(),
        ApiClient.getCompanies(),
        ApiClient.getContacts()
      ])).rejects.toThrow('Database error');
    });
  });
});