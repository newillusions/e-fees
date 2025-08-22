/**
 * Tauri Commands Integration Tests
 * 
 * Tests for the integration between frontend API calls and Tauri backend commands.
 * Ensures proper communication, data transformation, and error handling across
 * the frontend-backend boundary.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { ApiClient } from '../../lib/api';
import type { Project, Company, Contact, Fee } from '../../lib/api';

// Mock Tauri invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('Tauri Commands Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Project Commands', () => {
    const mockProject: Project = {
      id: 'projects:25-97101',
      project_number: '25-97101',
      name: 'Dubai Marina Tower',
      country: 'United Arab Emirates',
      city: 'Dubai',
      client_company: 'Emittiv Engineering',
      status: 'active' as const,
      created_at: '2025-08-21T09:00:00Z',
      updated_at: '2025-08-21T09:00:00Z'
    };

    it('should get all projects', async () => {
      const mockProjects = [mockProject];
      vi.mocked(invoke).mockResolvedValueOnce(mockProjects);

      const result = await ApiClient.getProjects();

      expect(invoke).toHaveBeenCalledWith('get_projects');
      expect(result).toEqual(mockProjects);
    });

    it('should create project with proper data transformation', async () => {
      const projectData = {
        project_number: '25-97102',
        name: 'New Project',
        country: 'United Arab Emirates',
        city: 'Abu Dhabi',
        client_company: 'Test Company',
        status: 'active' as const
      };

      const expectedProjectWithId = {
        ...projectData,
        id: null,
        time: {
          created_at: expect.any(String),
          updated_at: expect.any(String)
        }
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockProject);

      const result = await ApiClient.createProject(projectData);

      expect(invoke).toHaveBeenCalledWith('create_project', { project: expectedProjectWithId });
      expect(result).toEqual(mockProject);
    });

    it('should update project', async () => {
      const updatedProject = { ...mockProject, name: 'Updated Project Name' };
      
      vi.mocked(invoke).mockResolvedValueOnce(updatedProject);

      const result = await ApiClient.updateProject('projects:25-97101', { name: 'Updated Project Name' });

      expect(invoke).toHaveBeenCalledWith('update_project', { 
        id: 'projects:25-97101', 
        projectUpdate: { name: 'Updated Project Name' } 
      });
      expect(result).toEqual(updatedProject);
    });

    it('should delete project', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(true);

      const result = await ApiClient.deleteProject('projects:25-97101');

      expect(invoke).toHaveBeenCalledWith('delete_project', { id: 'projects:25-97101' });
      expect(result).toBe(true);
    });

    it('should handle project creation errors', async () => {
      const error = new Error('Project number already exists');
      vi.mocked(invoke).mockRejectedValueOnce(error);

      const result = await ApiClient.createProject({
        project_number: '25-97101', // Duplicate
        name: 'Test Project',
        country: 'UAE',
        city: 'Dubai',
        client_company: 'Test',
        status: 'active' as const
      });

      expect(result).toBeNull();
      expect(invoke).toHaveBeenCalledWith('create_project', expect.any(Object));
    });
  });

  describe('Company Commands', () => {
    const mockCompany: Company = {
      id: 'company:emt',
      name: 'Emittiv Engineering',
      name_short: 'Emittiv',
      abbreviation: 'emt',
      country: 'United Arab Emirates',
      city: 'Dubai',
      reg_no: 'REG123456',
      tax_no: 'TAX789012',
      time: {
        created_at: '2025-08-21T09:00:00Z',
        updated_at: '2025-08-21T09:00:00Z'
      }
    };

    it('should get all companies', async () => {
      const mockCompanies = [mockCompany];
      vi.mocked(invoke).mockResolvedValueOnce(mockCompanies);

      const result = await ApiClient.getCompanies();

      expect(invoke).toHaveBeenCalledWith('get_companies');
      expect(result).toEqual(mockCompanies);
    });

    it('should create company', async () => {
      const companyData = {
        name: 'New Company',
        name_short: 'NewCo',
        abbreviation: 'NCO',
        country: 'United Arab Emirates',
        city: 'Dubai',
        reg_no: 'REG123456',
        tax_no: 'TAX789012'
      };

      const expectedCompanyCreate = {
        name: 'New Company',
        name_short: 'NewCo',
        abbreviation: 'NCO',
        country: 'United Arab Emirates',
        city: 'Dubai',
        reg_no: 'REG123456',
        tax_no: 'TAX789012'
      };

      const expectedCreatedCompany = {
        id: 'company:nco',
        name: 'New Company',
        name_short: 'NewCo',
        abbreviation: 'nco',
        country: 'United Arab Emirates',
        city: 'Dubai',
        reg_no: 'REG123456',
        tax_no: 'TAX789012',
        time: {
          created_at: '2025-08-21T09:00:00Z',
          updated_at: '2025-08-21T09:00:00Z'
        }
      };

      vi.mocked(invoke).mockResolvedValueOnce(expectedCreatedCompany);

      const result = await ApiClient.createCompany(companyData);

      expect(invoke).toHaveBeenCalledWith('create_company', { company: expectedCompanyCreate });
      expect(result).toEqual(expectedCreatedCompany);
    });

    it('should update company', async () => {
      const updatedCompany = { ...mockCompany, name_short: 'Emittiv Eng' };
      
      vi.mocked(invoke).mockResolvedValueOnce(updatedCompany);

      const result = await ApiClient.updateCompany('company:emt', { name_short: 'Emittiv Eng' });

      expect(invoke).toHaveBeenCalledWith('update_company', { 
        id: 'company:emt', 
        companyUpdate: {
          name: null,
          name_short: 'Emittiv Eng',
          abbreviation: null,
          city: null,
          country: null,
          reg_no: null,
          tax_no: null
        }
      });
      expect(result).toEqual(updatedCompany);
    });

    it('should delete company', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(true);

      const result = await ApiClient.deleteCompany('company:emt');

      expect(invoke).toHaveBeenCalledWith('delete_company', { id: 'company:emt' });
      expect(result).toBe(true);
    });

    it('should handle validation errors', async () => {
      const validationError = new Error('Invalid email format');
      vi.mocked(invoke).mockRejectedValueOnce(validationError);

      const result = await ApiClient.createCompany({
        name: 'Test Company',
        name_short: 'TestCo', 
        abbreviation: 'TC',
        country: 'UAE',
        city: 'Dubai',
        reg_no: 'REG123',
        tax_no: 'TAX456'
      });

      expect(result).toBeNull();
    });
  });

  describe('Contact Commands', () => {
    const mockContact: Contact = {
      id: 'contact:john123',
      first_name: 'John',
      last_name: 'Smith',
      full_name: 'John Smith',
      position: 'Project Manager',
      company: 'company:emt',
      phone: '+971501234567',
      email: 'john.smith@emittiv.com'
    };

    it('should get all contacts', async () => {
      const mockContacts = [mockContact];
      vi.mocked(invoke).mockResolvedValueOnce(mockContacts);

      const result = await ApiClient.getContacts();

      expect(invoke).toHaveBeenCalledWith('get_contacts');
      expect(result).toEqual(mockContacts);
    });

    it('should create contact with full_name generation', async () => {
      const contactData = {
        first_name: 'Jane',
        last_name: 'Doe',
        position: 'Engineer',
        company: 'company:emt',
        phone: '+971501234567',
        email: 'jane.doe@emittiv.com',
        full_name: 'Jane Doe'
      };

      const expectedContactWithId = {
        ...contactData,
        id: null // API adds this
      };

      vi.mocked(invoke).mockResolvedValueOnce({
        ...mockContact,
        first_name: 'Jane',
        last_name: 'Doe',
        full_name: 'Jane Doe'
      });

      const result = await ApiClient.createContact(contactData);

      expect(invoke).toHaveBeenCalledWith('create_contact', { contact: expectedContactWithId });
      expect(result.full_name).toBe('Jane Doe');
    });

    it('should update contact', async () => {
      const updatedContact = { ...mockContact, position: 'Senior Project Manager' };
      
      vi.mocked(invoke).mockResolvedValueOnce(updatedContact);

      const result = await ApiClient.updateContact('contact:john123', { 
        position: 'Senior Project Manager' 
      });

      expect(invoke).toHaveBeenCalledWith('update_contact', { 
        id: 'contact:john123', 
        contactUpdate: { position: 'Senior Project Manager' } 
      });
      expect(result).toEqual(updatedContact);
    });

    it('should delete contact', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(true);

      const result = await ApiClient.deleteContact('contact:john123');

      expect(invoke).toHaveBeenCalledWith('delete_contact', { id: 'contact:john123' });
      expect(result).toBe(true);
    });

    it('should handle foreign key constraint errors', async () => {
      const constraintError = new Error('Company does not exist');
      vi.mocked(invoke).mockRejectedValueOnce(constraintError);

      const result = await ApiClient.createContact({
        first_name: 'Test',
        last_name: 'User',
        email: 'test@example.com',
        phone: '123456789',
        position: 'Engineer',
        company: 'company:nonexistent'
      });

      expect(result).toBeNull();
    });
  });

  describe('Fee Commands', () => {
    const mockFee: Fee = {
      id: 'fee:25-97101-01',
      name: 'Dubai Marina Tower Fee Proposal',
      number: '25-97101-01',
      project_id: 'projects:25-97101',
      company_id: 'company:emt',
      contact_id: 'contact:john123',
      status: 'Draft',
      stage: 'Draft',
      issue_date: '250820',
      activity: 'Design and Consultancy',
      package: 'structural',
      staff_name: 'Martin Engineer',
      revisions: []
    };

    it('should get all fees', async () => {
      const mockFees = [mockFee];
      vi.mocked(invoke).mockResolvedValueOnce(mockFees);

      const result = await ApiClient.getFees();

      expect(invoke).toHaveBeenCalledWith('get_fees');
      expect(result).toEqual(mockFees);
    });

    it('should create fee with relational data', async () => {
      const feeData = {
        name: 'New Fee Proposal',
        number: '25-97101-02',
        project_id: 'projects:25-97101',
        company_id: 'company:emt',
        contact_id: 'contacts:john123',
        status: 'Draft' as const,
        stage: 'Draft' as const,
        issue_date: '250821',
        package: 'structural',
        staff_name: 'John Engineer',
        activity: 'Structural Analysis',
        revisions: []
      };

      // Expected API call with ID prefixes stripped and extra fields added
      const expectedFeeCreate = {
        name: 'New Fee Proposal',
        number: '25-97101-02',
        project_id: '25-97101', // prefix stripped
        company_id: 'emt', // prefix stripped
        contact_id: 'john123', // contacts: prefix stripped
        status: 'Draft' as const,
        issue_date: '250821',
        package: 'structural',
        staff_name: 'John Engineer',
        activity: 'Structural Analysis',
        rev: undefined,
        staff_email: undefined,
        staff_phone: undefined,
        staff_position: undefined,
        strap_line: undefined,
        revisions: []
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockFee);

      const result = await ApiClient.createFee(feeData);

      expect(invoke).toHaveBeenCalledWith('create_fee', { fee: expectedFeeCreate });
      expect(result).toEqual(mockFee);
    });

    it('should update fee', async () => {
      const updatedFee = { ...mockFee, status: 'Active' };
      
      vi.mocked(invoke).mockResolvedValueOnce(updatedFee);

      const result = await ApiClient.updateFee('fee:25-97101-01', { status: 'Active' });

      // The API adds id and updated timestamp to the fee data automatically
      expect(invoke).toHaveBeenCalledWith('update_fee', { 
        id: 'fee:25-97101-01', 
        fee: expect.objectContaining({
          status: 'Active',
          id: null,
          time: expect.objectContaining({
            updated_at: expect.any(String)
          })
        })
      });
      expect(result).toEqual(updatedFee);
    });

    it('should delete fee', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(true);

      const result = await ApiClient.deleteFee('fee:25-97101-01');

      expect(invoke).toHaveBeenCalledWith('delete_fee', { id: 'fee:25-97101-01' });
      expect(result).toBe(true);
    });

    it('should validate enum values', async () => {
      const enumError = new Error('Invalid status value');
      vi.mocked(invoke).mockRejectedValueOnce(enumError);

      const result = await ApiClient.createFee({
        name: 'Invalid Fee',
        number: '25-97101-03',
        rev: 1,
        status: 'invalid_status' as any, // Invalid enum value
        issue_date: '2025-08-21',
        activity: 'Test Activity',
        package: 'structural',
        project_id: '25-97101', // prefix stripped
        company_id: 'emt', // prefix stripped
        contact_id: 'john123', // prefix stripped
        staff_name: 'Test Engineer',
        staff_email: 'test@example.com',
        staff_phone: '123456789',
        staff_position: 'Engineer',
        strap_line: 'Test strap line',
        revisions: []
      });

      expect(result).toBeNull();
    });
  });

  describe('Database Connection Commands', () => {
    it('should check database connection', async () => {
      const mockConnectionStatus = {
        connected: true,
        database_url: 'ws://10.0.1.17:8000',
        namespace: 'emittiv',
        database: 'projects',
        username: 'martin',
        response_time_ms: 45,
        last_check: '2025-08-21T10:30:00Z'
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockConnectionStatus);

      const result = await ApiClient.getConnectionStatus();

      expect(invoke).toHaveBeenCalledWith('get_connection_status');
      expect(result).toEqual(mockConnectionStatus);
    });

    it('should handle connection timeouts', async () => {
      const timeoutError = new Error('Connection timeout');
      vi.mocked(invoke).mockRejectedValueOnce(timeoutError);

      const result = await ApiClient.checkDbConnection();

      expect(result).toBe(false);
      expect(invoke).toHaveBeenCalledWith('check_db_connection');
    });

    it('should handle authentication errors', async () => {
      const authError = new Error('Authentication failed');
      vi.mocked(invoke).mockRejectedValueOnce(authError);

      const result = await ApiClient.getConnectionStatus();

      expect(result).toEqual({
        is_connected: false,
        last_check: undefined,
        error_message: 'Error: Authentication failed'
      });
    });
  });

  describe('Reference Data Commands', () => {
    it('should search countries', async () => {
      const mockCountries = [
        { name: 'United Arab Emirates', code: 'AE', dial_code: 971 },
        { name: 'Saudi Arabia', code: 'SA', dial_code: 966 }
      ];

      vi.mocked(invoke).mockResolvedValueOnce(mockCountries);

      const result = await ApiClient.searchCountries('UAE');

      expect(invoke).toHaveBeenCalledWith('search_countries', { query: 'UAE' });
      expect(result).toEqual(mockCountries);
    });

    it('should handle empty search results', async () => {
      vi.mocked(invoke).mockResolvedValueOnce([]);

      const result = await ApiClient.searchCountries('nonexistent');

      expect(result).toEqual([]);
      expect(invoke).toHaveBeenCalledWith('search_countries', { query: 'nonexistent' });
    });
  });

  describe('Error Handling Across Commands', () => {
    it('should handle network errors uniformly', async () => {
      const networkError = new Error('Network unreachable');
      vi.mocked(invoke).mockRejectedValue(networkError);

      // All commands should handle network errors consistently
      await expect(ApiClient.getProjects()).rejects.toThrow('Network unreachable');
      await expect(ApiClient.getCompanies()).rejects.toThrow('Network unreachable');
      await expect(ApiClient.getContacts()).rejects.toThrow('Network unreachable');
      await expect(ApiClient.getFees()).rejects.toThrow('Network unreachable');
    });

    it('should handle database connection errors', async () => {
      const dbError = new Error('Database connection lost');
      vi.mocked(invoke).mockRejectedValue(dbError);

      const result = await ApiClient.createProject({
        project_number: '25-97103',
        name: 'Test Project',
        country: 'UAE',
        city: 'Dubai',
        client_company: 'Test Company',
        status: 'active' as const
      });

      expect(result).toBeNull();
    });

    it('should handle permission errors', async () => {
      const permissionError = new Error('Insufficient permissions');
      vi.mocked(invoke).mockRejectedValue(permissionError);

      await expect(ApiClient.deleteProject('projects:25-97101')).rejects.toThrow('Insufficient permissions');
    });

    it('should handle malformed data errors', async () => {
      const dataError = new Error('Invalid data format');
      vi.mocked(invoke).mockRejectedValue(dataError);

      const result = await ApiClient.createCompany({
        name: '', // Invalid empty name
        name_short: 'Test',
        abbreviation: 'TST',
        country: 'UAE',
        city: 'Dubai',
        reg_no: 'REG123',
        tax_no: 'TAX456'
      });

      expect(result).toBeNull();
    });
  });

  describe('Data Type Handling', () => {
    it('should handle SurrealDB Thing objects correctly', async () => {
      const mockProjectWithThingId = {
        id: { tb: 'projects', id: '25-97101' },
        project_number: '25-97101',
        name: 'Test Project',
        country: 'UAE',
        city: 'Dubai',
        client_company: 'Test Company',
        status: 'active' as const,
        created_at: '2025-08-21T09:00:00Z',
        updated_at: '2025-08-21T09:00:00Z'
      };

      vi.mocked(invoke).mockResolvedValueOnce([mockProjectWithThingId]);

      const result = await ApiClient.getProjects();

      expect(result).toHaveLength(1);
      // SurrealDB returns Thing objects as-is
      expect(typeof result[0].id).toBe('object');
      expect(result[0].id).toEqual({ tb: 'projects', id: '25-97101' });
    });

    it('should handle date strings correctly', async () => {
      const baseMockFee = {
        id: 'fee:25-97101-01',
        name: 'Test Fee',
        number: '25-97101-01',
        project_id: 'projects:25-97101',
        company_id: 'company:emt',
        contact_id: 'contacts:john123',
        status: 'Draft' as const,
        stage: 'Draft' as const,
        issue_date: '250821',
        revisions: []
      };
      
      const mockFeeWithDates = {
        ...baseMockFee,
        issue_date: '250821', // YYMMDD format for fees
        time: {
          created_at: '2025-08-21T10:00:00Z', // ISO datetime
          updated_at: '2025-08-21T10:00:00Z'
        }
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockFeeWithDates);

      const result = await ApiClient.createFee({
        name: 'Test Fee',
        number: '25-97101-04',
        project_id: 'projects:25-97101',
        company_id: 'company:emt',
        contact_id: 'contacts:john123',
        status: 'Draft' as const,
        stage: 'Draft' as const,
        issue_date: '250821',
        package: 'structural',
        staff_name: 'Test Engineer',
        activity: 'Test Activity',
        revisions: []
      });

      expect(result.issue_date).toBe('250821');
      expect(result.time?.created_at).toMatch(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z/);
    });

    it('should handle null and undefined values', async () => {
      const mockCompany = {
        id: 'company:test123',
        name: 'Test Company',
        name_short: 'Test Co',
        abbreviation: 'TCO',
        country: 'UAE',
        city: 'Dubai',
        reg_no: 'REG123',
        tax_no: 'TAX456',
        time: {
          created_at: '2025-08-21T10:00:00Z',
          updated_at: '2025-08-21T10:00:00Z'
        }
      };

      const mockCompanyWithNulls = {
        ...mockCompany,
        reg_no: null,
        tax_no: undefined
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockCompanyWithNulls);

      const result = await ApiClient.createCompany({
        name: 'Test Company',
        country: 'UAE',
        city: 'Dubai',
        address: '',
        phone: '',
        email: '',
        website: ''
      });

      expect(result).toBeDefined();
      // API should handle null/undefined consistently
    });
  });
});