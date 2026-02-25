/**
 * Agent API Client Tests
 *
 * Tests for the HTTP-based agent API client that communicates with the
 * local agent server running inside the Tauri app.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  AgentApiClient,
  type AgentHealthResponse,
  type AgentStatsResponse,
  type AgentProjectResponse,
  type AgentFeeResponse,
  type AgentCompanyResponse,
  type AgentContactResponse,
  type AgentListResponse,
  type AgentFeeCreate,
  type AgentContactCreate,
  type AgentExportResponse,
} from './agent';

// Mock global fetch
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

describe('Agent API Client', () => {
  let client: AgentApiClient;

  beforeEach(() => {
    vi.clearAllMocks();
    client = new AgentApiClient('http://localhost:3100');
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('constructor', () => {
    it('should use default base URL when none provided', () => {
      const defaultClient = new AgentApiClient();
      expect(defaultClient.baseUrl).toBe('http://localhost:3100');
    });

    it('should accept custom base URL', () => {
      const customClient = new AgentApiClient('http://10.0.1.5:4000');
      expect(customClient.baseUrl).toBe('http://10.0.1.5:4000');
    });

    it('should strip trailing slash from base URL', () => {
      const slashClient = new AgentApiClient('http://localhost:3100/');
      expect(slashClient.baseUrl).toBe('http://localhost:3100');
    });
  });

  describe('health', () => {
    it('should return health status on success', async () => {
      const mockResponse: AgentHealthResponse = {
        status: 'ok',
        version: '0.10.27',
        db_connected: true,
        uptime_seconds: 120,
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockResponse),
      });

      const result = await client.health();

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/health', {
        headers: { 'Content-Type': 'application/json' },
      });
      expect(result).toEqual(mockResponse);
    });

    it('should throw on connection refused', async () => {
      mockFetch.mockRejectedValueOnce(new TypeError('fetch failed'));

      await expect(client.health()).rejects.toThrow('fetch failed');
    });
  });

  describe('stats', () => {
    it('should return entity counts', async () => {
      const mockStats: AgentStatsResponse = {
        total_projects: 48,
        total_companies: 19,
        total_contacts: 12,
        total_fees: 37,
        active_fees: 15,
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockStats),
      });

      const result = await client.stats();

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/stats', {
        headers: { 'Content-Type': 'application/json' },
      });
      expect(result.total_projects).toBe(48);
      expect(result.active_fees).toBe(15);
    });
  });

  describe('listProjects', () => {
    it('should list all projects', async () => {
      const mockList: AgentListResponse<AgentProjectResponse> = {
        items: [
          {
            id: 'projects:abc123',
            name: 'Project Hittin',
            name_short: 'Hittin',
            status: 'Design',
            country: 'Saudi Arabia',
            city: 'Riyadh',
            area: 'Hittin District',
            number: '25-96601',
            folder: '/Projects/25-96601',
          },
        ],
        total: 1,
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockList),
      });

      const result = await client.listProjects();

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/projects', {
        headers: { 'Content-Type': 'application/json' },
      });
      expect(result.items).toHaveLength(1);
      expect(result.items[0].name).toBe('Project Hittin');
    });
  });

  describe('getProject', () => {
    it('should fetch a project by ID', async () => {
      const mockProject: AgentProjectResponse = {
        id: 'projects:abc123',
        name: 'Project Hittin',
        name_short: 'Hittin',
        status: 'Design',
        country: 'Saudi Arabia',
        city: 'Riyadh',
        area: 'Hittin District',
        number: '25-96601',
        folder: '/Projects/25-96601',
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockProject),
      });

      const result = await client.getProject('projects:abc123');

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/projects/projects:abc123', {
        headers: { 'Content-Type': 'application/json' },
      });
      expect(result.name).toBe('Project Hittin');
    });

    it('should throw on 404', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: () => Promise.resolve({ error: 'Project not found' }),
      });

      await expect(client.getProject('projects:nonexistent')).rejects.toThrow('Project not found');
    });
  });

  describe('createProject', () => {
    it('should create a project with required fields', async () => {
      const newProject = {
        name: 'New Test Project',
        name_short: 'NTP',
        status: 'Draft',
        country: 'United Arab Emirates',
        city: 'Dubai',
        area: 'Downtown',
        number: '26-97101',
        folder: '/Projects/26-97101',
      };
      const mockCreated: AgentProjectResponse = {
        id: 'projects:xyz789',
        ...newProject,
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 201,
        json: () => Promise.resolve(mockCreated),
      });

      const result = await client.createProject(newProject);

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/projects', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newProject),
      });
      expect(result.id).toBe('projects:xyz789');
    });
  });

  describe('listFees', () => {
    it('should list all fees', async () => {
      const mockList: AgentListResponse<AgentFeeResponse> = {
        items: [
          {
            id: 'fee:abc123',
            name: 'Project Hittin Fee Proposal',
            number: '25-96601',
            rev: 1,
            status: 'Draft',
            project_id: 'projects:abc123',
            company_id: 'company:def456',
            total_fee: 960000,
            currency: 'AED',
          },
        ],
        total: 1,
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockList),
      });

      const result = await client.listFees();

      expect(result.items).toHaveLength(1);
      expect(result.items[0].total_fee).toBe(960000);
    });
  });

  describe('getFee', () => {
    it('should fetch a fee by ID', async () => {
      const mockFee: AgentFeeResponse = {
        id: 'fee:abc123_1',
        name: 'WAMI Hotel - Lighting Design',
        number: 'E-26-97101-01',
        rev: 1,
        status: 'Draft',
        project_id: 'projects:abc123',
        company_id: 'company:def456',
        total_fee: 960000,
        currency: 'AED',
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockFee),
      });

      const result = await client.getFee('fee:abc123_1');

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/fees/fee:abc123_1', {
        headers: { 'Content-Type': 'application/json' },
      });
      expect(result.name).toBe('WAMI Hotel - Lighting Design');
      expect(result.total_fee).toBe(960000);
    });

    it('should throw on 404 for non-existent fee', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: () => Promise.resolve({ error: 'Fee not found' }),
      });

      await expect(client.getFee('fee:nonexistent')).rejects.toThrow('Fee not found');
    });
  });

  describe('createFee', () => {
    it('should create a fee with required fields', async () => {
      const newFee: AgentFeeCreate = {
        name: 'WAMI Hotel - Lighting Design',
        number: 'E-26-97101-01',
        project_id: 'projects:abc123',
        company_id: 'company:def456',
        contact_id: 'contacts:ghi789',
      };
      const mockCreated: AgentFeeResponse = {
        id: 'fee:abc123_1',
        name: 'WAMI Hotel - Lighting Design',
        number: 'E-26-97101-01',
        rev: 1,
        status: 'Draft',
        project_id: 'projects:abc123',
        company_id: 'company:def456',
        total_fee: 0,
        currency: 'AED',
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 201,
        json: () => Promise.resolve(mockCreated),
      });

      const result = await client.createFee(newFee);

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/fees', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newFee),
      });
      expect(result.id).toBe('fee:abc123_1');
      expect(result.status).toBe('Draft');
    });
  });

  describe('listFees with filter', () => {
    it('should pass project_id query parameter', async () => {
      const mockList: AgentListResponse<AgentFeeResponse> = {
        items: [],
        total: 0,
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockList),
      });

      await client.listFees('projects:abc123');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3100/api/fees?project_id=projects%3Aabc123',
        { headers: { 'Content-Type': 'application/json' } },
      );
    });

    it('should list all fees when no filter provided', async () => {
      const mockList: AgentListResponse<AgentFeeResponse> = {
        items: [
          {
            id: 'fee:abc123_1',
            name: 'Test Fee',
            number: 'E-26-97101-01',
            rev: 1,
            status: 'Draft',
            project_id: 'projects:abc123',
            company_id: 'company:def456',
            total_fee: 500000,
            currency: 'AED',
          },
        ],
        total: 1,
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockList),
      });

      const result = await client.listFees();

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/fees', {
        headers: { 'Content-Type': 'application/json' },
      });
      expect(result.total).toBe(1);
    });
  });

  describe('listCompanies', () => {
    it('should list all companies', async () => {
      const mockList: AgentListResponse<AgentCompanyResponse> = {
        items: [
          {
            id: 'company:abc123',
            name: 'SAQR Alkhorayef Investment Company',
            name_short: 'SAQR',
            abbreviation: 'SAQR',
            city: 'Riyadh',
            country: 'Saudi Arabia',
          },
        ],
        total: 1,
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockList),
      });

      const result = await client.listCompanies();

      expect(result.items).toHaveLength(1);
      expect(result.items[0].name).toBe('SAQR Alkhorayef Investment Company');
    });
  });

  describe('createCompany', () => {
    it('should create a company', async () => {
      const newCompany = {
        name: 'Test Company',
        name_short: 'TC',
        abbreviation: 'TC',
        city: 'Dubai',
        country: 'United Arab Emirates',
      };
      const mockCreated: AgentCompanyResponse = {
        id: 'company:new123',
        ...newCompany,
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 201,
        json: () => Promise.resolve(mockCreated),
      });

      const result = await client.createCompany(newCompany);

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/companies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newCompany),
      });
      expect(result.id).toBe('company:new123');
    });
  });

  describe('listContacts', () => {
    it('should list all contacts', async () => {
      const mockList: AgentListResponse<AgentContactResponse> = {
        items: [
          {
            id: 'contacts:abc123',
            first_name: 'Ahmed',
            last_name: 'Al Rashid',
            full_name: 'Ahmed Al Rashid',
            email: 'ahmed@example.com',
            phone: '+971501234567',
            position: 'Project Manager',
            company_id: 'company:xyz789',
          },
        ],
        total: 1,
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockList),
      });

      const result = await client.listContacts();

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/contacts', {
        headers: { 'Content-Type': 'application/json' },
      });
      expect(result.items).toHaveLength(1);
      expect(result.items[0].first_name).toBe('Ahmed');
    });

    it('should pass company_id query parameter', async () => {
      const mockList: AgentListResponse<AgentContactResponse> = {
        items: [],
        total: 0,
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockList),
      });

      await client.listContacts('company:xyz789');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3100/api/contacts?company_id=company%3Axyz789',
        { headers: { 'Content-Type': 'application/json' } },
      );
    });
  });

  describe('getContact', () => {
    it('should fetch a contact by ID', async () => {
      const mockContact: AgentContactResponse = {
        id: 'contacts:abc123',
        first_name: 'Ahmed',
        last_name: 'Al Rashid',
        full_name: 'Ahmed Al Rashid',
        email: 'ahmed@example.com',
        phone: '+971501234567',
        position: 'Project Manager',
        company_id: 'company:xyz789',
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockContact),
      });

      const result = await client.getContact('contacts:abc123');

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/contacts/contacts:abc123', {
        headers: { 'Content-Type': 'application/json' },
      });
      expect(result.first_name).toBe('Ahmed');
      expect(result.company_id).toBe('company:xyz789');
    });

    it('should throw on 404 for non-existent contact', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: () => Promise.resolve({ error: 'Contact not found' }),
      });

      await expect(client.getContact('contacts:nonexistent')).rejects.toThrow('Contact not found');
    });
  });

  describe('createContact', () => {
    it('should create a contact', async () => {
      const newContact: AgentContactCreate = {
        first_name: 'Ahmed',
        last_name: 'Al Rashid',
        email: 'ahmed@example.com',
        phone: '+971501234567',
        position: 'Project Manager',
        company_id: 'company:xyz789',
      };
      const mockCreated: AgentContactResponse = {
        id: 'contacts:new123',
        ...newContact,
        full_name: 'Ahmed Al Rashid',
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 201,
        json: () => Promise.resolve(mockCreated),
      });

      const result = await client.createContact(newContact);

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/contacts', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newContact),
      });
      expect(result.id).toBe('contacts:new123');
      expect(result.full_name).toBe('Ahmed Al Rashid');
    });
  });

  describe('exportFeeExcel', () => {
    it('should export fee as Excel file', async () => {
      const mockExport: AgentExportResponse = {
        path: '/tmp/fee-E-26-97101-01-rev1.xlsx',
        filename: 'fee-E-26-97101-01-rev1.xlsx',
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockExport),
      });

      const result = await client.exportFeeExcel('fee:abc123_1');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3100/api/fees/fee:abc123_1/export',
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
        },
      );
      expect(result.filename).toBe('fee-E-26-97101-01-rev1.xlsx');
      expect(result.path).toContain('.xlsx');
    });

    it('should handle fee ID without prefix', async () => {
      const mockExport: AgentExportResponse = {
        path: '/tmp/fee-25-97105-rev2.xlsx',
        filename: 'fee-25-97105-rev2.xlsx',
      };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockExport),
      });

      const result = await client.exportFeeExcel('abc123');

      expect(mockFetch).toHaveBeenCalledWith('http://localhost:3100/api/fees/abc123/export', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      });
      expect(result.filename).toBe('fee-25-97105-rev2.xlsx');
    });

    it('should throw on 404 if fee not found', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: () => Promise.resolve({ error: 'Fee not found' }),
      });

      await expect(client.exportFeeExcel('fee:nonexistent')).rejects.toThrow('Fee not found');
    });
  });

  describe('error handling', () => {
    it('should include status code in error for non-OK responses', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: () => Promise.resolve({ error: 'Internal server error' }),
      });

      await expect(client.listProjects()).rejects.toThrow('Internal server error');
    });

    it('should handle non-JSON error responses', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 502,
        json: () => Promise.reject(new Error('Not JSON')),
      });

      await expect(client.listProjects()).rejects.toThrow('HTTP 502');
    });
  });
});
