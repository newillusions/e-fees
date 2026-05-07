/**
 * Revisions API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { exportIndesignWorkbook, exportFeeTemplate } from './revisions';

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
