/**
 * Revisions API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { exportIndesignWorkbook, exportFeeTemplate, cloneFeeRevision } from './revisions';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('Revisions API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('exportIndesignWorkbook', () => {
    it('should call invoke with fee id and undefined output path', async () => {
      mockInvoke.mockResolvedValueOnce('/tmp/25-97101-IDW Pricing.xlsx');

      const result = await exportIndesignWorkbook('fee:abc123');

      expect(mockInvoke).toHaveBeenCalledWith('export_indesign_workbook', {
        feeId: 'fee:abc123',
        outputPath: undefined
      });
      expect(result).toBe('/tmp/25-97101-IDW Pricing.xlsx');
    });

    it('should pass through optional output path', async () => {
      mockInvoke.mockResolvedValueOnce('/Users/me/Pricing.xlsx');

      await exportIndesignWorkbook('fee:abc123', '/Users/me/Pricing.xlsx');

      expect(mockInvoke).toHaveBeenCalledWith('export_indesign_workbook', {
        feeId: 'fee:abc123',
        outputPath: '/Users/me/Pricing.xlsx'
      });
    });

    it('should propagate errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Fee not found'));

      await expect(exportIndesignWorkbook('fee:missing')).rejects.toThrow('Fee not found');
    });
  });

  describe('cloneFeeRevision', () => {
    it('should call invoke with feeId and the audit-trail fields', async () => {
      mockInvoke.mockResolvedValueOnce({ id: 'fee:abc123', rev: 2 });

      const result = await cloneFeeRevision(
        'fee:abc123',
        'reviewer@emittiv.com',
        'Reviewer Name',
        'Reduced scope per client meeting'
      );

      expect(mockInvoke).toHaveBeenCalledWith('clone_fee_revision', {
        feeId: 'fee:abc123',
        authorEmail: 'reviewer@emittiv.com',
        authorName: 'Reviewer Name',
        notes: 'Reduced scope per client meeting'
      });
      expect(result).toEqual({ id: 'fee:abc123', rev: 2 });
    });

    it('should propagate errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('source fee not found'));

      await expect(cloneFeeRevision('fee:missing', '', '', '')).rejects.toThrow(
        'source fee not found'
      );
    });
  });

  describe('exportFeeTemplate', () => {
    it('should still wire the existing template export', async () => {
      mockInvoke.mockResolvedValueOnce('/tmp/template.xlsx');

      const result = await exportFeeTemplate('fee:abc123');

      expect(mockInvoke).toHaveBeenCalledWith('export_fee_template', {
        feeId: 'fee:abc123',
        outputPath: undefined
      });
      expect(result).toBe('/tmp/template.xlsx');
    });
  });
});
