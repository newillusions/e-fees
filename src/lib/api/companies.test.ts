/**
 * Companies API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  getCompanies,
  getCompaniesPage,
  getCompanyById,
  createCompany,
  updateCompany,
  deleteCompany
} from './companies';
import type { Company, CompanyCreate, CompanyUpdate, PaginatedResponse } from '../../types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

const mockCompany: Company = {
  id: 'company:TEST',
  name: 'Test Company',
  name_short: 'Test Co',
  abbreviation: 'TEST',
  city: 'Dubai',
  country: 'U.A.E.',
  time: { created_at: '2025-01-18T00:00:00Z', updated_at: '2025-01-18T00:00:00Z' }
};

describe('Companies API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('getCompanies', () => {
    it('should call invoke with correct command', async () => {
      mockInvoke.mockResolvedValueOnce([mockCompany]);

      const result = await getCompanies();

      expect(mockInvoke).toHaveBeenCalledWith('get_companies');
      expect(result).toEqual([mockCompany]);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database error'));

      await expect(getCompanies()).rejects.toThrow('Database error');
    });
  });

  describe('getCompaniesPage', () => {
    it('should call invoke with pagination params', async () => {
      const mockResponse: PaginatedResponse<Company> = {
        items: [mockCompany],
        total: 1,
        page: 1,
        page_size: 50,
        has_more: false
      };
      mockInvoke.mockResolvedValueOnce(mockResponse);

      const result = await getCompaniesPage(2, 25);

      expect(mockInvoke).toHaveBeenCalledWith('get_companies_page', { page: 2, pageSize: 25 });
      expect(result).toEqual(mockResponse);
    });
  });

  describe('getCompanyById', () => {
    it('should call invoke with company id', async () => {
      mockInvoke.mockResolvedValueOnce(mockCompany);

      const result = await getCompanyById('TEST');

      expect(mockInvoke).toHaveBeenCalledWith('get_company_by_id', { id: 'TEST' });
      expect(result).toEqual(mockCompany);
    });

    it('should return null for non-existent company', async () => {
      mockInvoke.mockResolvedValueOnce(null);

      const result = await getCompanyById('NOTFOUND');

      expect(result).toBeNull();
    });
  });

  describe('createCompany', () => {
    it('should call invoke with sanitized company data', async () => {
      mockInvoke.mockResolvedValueOnce(mockCompany);

      const companyData: CompanyCreate = {
        name: 'New Company',
        name_short: 'New Co',
        abbreviation: 'NEW',
        city: 'Abu Dhabi',
        country: 'U.A.E.'
      };

      const result = await createCompany(companyData);

      expect(mockInvoke).toHaveBeenCalledWith('create_company', {
        company: {
          name: 'New Company',
          name_short: 'New Co',
          abbreviation: 'NEW',
          city: 'Abu Dhabi',
          country: 'U.A.E.',
          reg_no: null,
          tax_no: null
        }
      });
      expect(result).toEqual(mockCompany);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Create failed'));

      await expect(createCompany({} as CompanyCreate)).rejects.toThrow('Create failed');
    });

    it('should pass reg_no and tax_no when provided', async () => {
      mockInvoke.mockResolvedValueOnce(mockCompany);

      const companyData: CompanyCreate = {
        name: 'New Company',
        name_short: 'New Co',
        abbreviation: 'NEW',
        city: 'Dubai',
        country: 'U.A.E.',
        reg_no: 'REG123',
        tax_no: 'TAX456'
      };

      await createCompany(companyData);

      expect(mockInvoke).toHaveBeenCalledWith('create_company', {
        company: expect.objectContaining({
          reg_no: 'REG123',
          tax_no: 'TAX456'
        })
      });
    });
  });

  describe('updateCompany', () => {
    it('should call invoke with id and update data', async () => {
      mockInvoke.mockResolvedValueOnce(mockCompany);

      const update: CompanyUpdate = { name: 'Updated Company' };
      const result = await updateCompany('TEST', update);

      expect(mockInvoke).toHaveBeenCalledWith('update_company', {
        id: 'TEST',
        companyUpdate: expect.objectContaining({
          name: 'Updated Company'
        })
      });
      expect(result).toEqual(mockCompany);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Update failed'));

      await expect(updateCompany('TEST', {})).rejects.toThrow('Update failed');
    });
  });

  describe('deleteCompany', () => {
    it('should call invoke with company id', async () => {
      mockInvoke.mockResolvedValueOnce(mockCompany);

      const result = await deleteCompany('TEST');

      expect(mockInvoke).toHaveBeenCalledWith('delete_company', { id: 'TEST' });
      expect(result).toEqual(mockCompany);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Has associated contacts'));

      await expect(deleteCompany('TEST')).rejects.toThrow('Has associated contacts');
    });
  });
});
