/**
 * Fees API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  getFees,
  getFeesPage,
  createFee,
  updateFee,
  deleteFee,
  writeFeeToJson,
  writeFeeToJsonSafe
} from './fees';
import type { Fee, FeeCreate, FeeUpdate, PaginatedResponse } from '../../types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

const mockFee: Fee = {
  id: 'fee:test',
  name: 'Test Fee',
  number: '25-97101-R1',
  project_id: 'projects:test',
  company_id: 'company:TEST',
  contact_id: 'contacts:test',
  status: 'Draft',
  issue_date: '250118',
  rev: 1,
  revisions: [],
  time: { created_at: '2025-01-18T00:00:00Z', updated_at: '2025-01-18T00:00:00Z' }
};

describe('Fees API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('getFees', () => {
    it('should call invoke with correct command', async () => {
      mockInvoke.mockResolvedValueOnce([mockFee]);

      const result = await getFees();

      expect(mockInvoke).toHaveBeenCalledWith('get_fees');
      expect(result).toEqual([mockFee]);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database error'));

      await expect(getFees()).rejects.toThrow('Database error');
    });
  });

  describe('getFeesPage', () => {
    it('should call invoke with pagination params', async () => {
      const mockResponse: PaginatedResponse<Fee> = {
        items: [mockFee],
        total: 1,
        page: 1,
        page_size: 50,
        has_more: false
      };
      mockInvoke.mockResolvedValueOnce(mockResponse);

      const result = await getFeesPage(1, 50);

      expect(mockInvoke).toHaveBeenCalledWith('get_fees_page', { page: 1, pageSize: 50 });
      expect(result).toEqual(mockResponse);
    });
  });

  describe('createFee', () => {
    it('should call invoke with processed fee data', async () => {
      mockInvoke.mockResolvedValueOnce(mockFee);

      const feeData: FeeCreate = {
        name: 'New Fee',
        number: '25-97101-R1',
        project_id: 'projects:test_project',
        company_id: 'company:TEST',
        contact_id: 'contacts:test_contact',
        status: 'Draft',
        issue_date: '250118',
        rev: 1,
        revisions: []
      };

      const result = await createFee(feeData);

      expect(mockInvoke).toHaveBeenCalledWith('create_fee', {
        fee: expect.objectContaining({
          name: 'New Fee',
          project_id: 'test_project',
          company_id: 'TEST',
          contact_id: 'test_contact'
        })
      });
      expect(result).toEqual(mockFee);
    });

    it('should strip prefixes from IDs', async () => {
      mockInvoke.mockResolvedValueOnce(mockFee);

      const feeData: FeeCreate = {
        name: 'Test',
        number: '25-97101-R1',
        project_id: 'projects:abc',
        company_id: 'company:xyz',
        contact_id: 'contacts:123',
        status: 'Draft',
        issue_date: '250118',
        rev: 1,
        revisions: []
      };

      await createFee(feeData);

      expect(mockInvoke).toHaveBeenCalledWith('create_fee', {
        fee: expect.objectContaining({
          project_id: 'abc',
          company_id: 'xyz',
          contact_id: '123'
        })
      });
    });

    it('should return null on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Create failed'));

      const result = await createFee({} as FeeCreate);

      expect(result).toBeNull();
    });
  });

  describe('updateFee', () => {
    it('should call invoke with update data and timestamp', async () => {
      mockInvoke.mockResolvedValueOnce(mockFee);

      const update: FeeUpdate = { status: 'Sent' };
      const result = await updateFee('test', update);

      expect(mockInvoke).toHaveBeenCalledWith('update_fee', {
        id: 'test',
        fee: expect.objectContaining({
          status: 'Sent',
          id: null,
          time: expect.objectContaining({
            updated_at: expect.any(String)
          })
        })
      });
      expect(result).toEqual(mockFee);
    });

    it('should return null on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Update failed'));

      const result = await updateFee('test', {});

      expect(result).toBeNull();
    });
  });

  describe('deleteFee', () => {
    it('should call invoke with fee id', async () => {
      mockInvoke.mockResolvedValueOnce(mockFee);

      const result = await deleteFee('test');

      expect(mockInvoke).toHaveBeenCalledWith('delete_fee', { id: 'test' });
      expect(result).toEqual(mockFee);
    });

    it('should return null on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Delete failed'));

      const result = await deleteFee('test');

      expect(result).toBeNull();
    });
  });

  describe('writeFeeToJson', () => {
    it('should call invoke with fee id', async () => {
      mockInvoke.mockResolvedValueOnce('Success: file written');

      const result = await writeFeeToJson('24_96606_1');

      expect(mockInvoke).toHaveBeenCalledWith('write_fee_to_json', { rfpId: '24_96606_1' });
      expect(result).toBe('Success: file written');
    });

    it('should return null on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Write failed'));

      const result = await writeFeeToJson('test');

      expect(result).toBeNull();
    });
  });

  describe('writeFeeToJsonSafe', () => {
    it('should call invoke with fee id', async () => {
      mockInvoke.mockResolvedValueOnce('Success with backup');

      const result = await writeFeeToJsonSafe('fee:123');

      expect(mockInvoke).toHaveBeenCalledWith('write_fee_to_json_safe', { feeId: 'fee:123' });
      expect(result).toBe('Success with backup');
    });

    it('should return null on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Safe write failed'));

      const result = await writeFeeToJsonSafe('test');

      expect(result).toBeNull();
    });
  });
});
