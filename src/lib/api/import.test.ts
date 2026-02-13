/**
 * Import Wizard API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { importScanDirectory, importExecute } from './import';
import type { ImportPreview, ImportResult } from './import';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

const mockPreview: ImportPreview = {
  staff_rates: {
    count: 16,
    currency: 'AED',
    roles: ['Partner', 'Design Director', 'Designer']
  },
  proposals: [
    {
      source_file: 'hittin-proposal.json',
      project_name: 'Project Hittin',
      project_number: '1735',
      client: 'SAQR Alkhorayef Investment Company',
      total_fee: 960000,
      currency: 'AED',
      stage_type: 'custom_milestones',
      stage_count: 4,
      phase_count: 0,
      add_on_count: 4,
      has_conversion_rate: true,
    },
    {
      source_file: 'thakher-proposal.json',
      project_name: 'Thakher Development Cluster A + Mall',
      project_number: '1734',
      client: 'Khomasiah International Development',
      total_fee: 2969400,
      currency: 'SAR',
      stage_type: 'phased_stages',
      stage_count: 24,
      phase_count: 4,
      add_on_count: 0,
      has_conversion_rate: false,
    },
  ],
  methodology: {
    wht_rate: 0.05,
    aed_to_sar: 1.0211,
    overhead_rate: 0.1,
    margin_rate: 0.05,
    default_stage_count: 6,
  },
  clients: [
    { name: 'SAQR Alkhorayef Investment Company', client_type: 'client', project_count: 1 },
    { name: 'P&T Group, Dubai', client_type: 'consultant', project_count: 2 },
  ],
  warnings: [],
};

const mockImportResult: ImportResult = {
  success: true,
  projects_created: 2,
  fees_created: 2,
  companies_created: 3,
  companies_skipped: 1,
  errors: [],
  warnings: ['Company "Emittiv Lighting Design" already exists, skipping'],
};

describe('Import Wizard API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('importScanDirectory', () => {
    it('should call invoke with correct command and directory', async () => {
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await importScanDirectory('/path/to/rfps/export/output');

      expect(mockInvoke).toHaveBeenCalledWith('import_scan_directory', {
        directory: '/path/to/rfps/export/output'
      });
      expect(result).toEqual(mockPreview);
    });

    it('should return staff rates preview', async () => {
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await importScanDirectory('/any/path');

      expect(result.staff_rates).toBeDefined();
      expect(result.staff_rates?.count).toBe(16);
      expect(result.staff_rates?.currency).toBe('AED');
    });

    it('should return proposal previews with correct stage types', async () => {
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await importScanDirectory('/any/path');

      expect(result.proposals).toHaveLength(2);
      expect(result.proposals[0].stage_type).toBe('custom_milestones');
      expect(result.proposals[1].stage_type).toBe('phased_stages');
    });

    it('should return methodology preview', async () => {
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await importScanDirectory('/any/path');

      expect(result.methodology).toBeDefined();
      expect(result.methodology?.wht_rate).toBe(0.05);
      expect(result.methodology?.aed_to_sar).toBe(1.0211);
    });

    it('should return client previews', async () => {
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await importScanDirectory('/any/path');

      expect(result.clients).toHaveLength(2);
      expect(result.clients[0].client_type).toBe('client');
    });

    it('should throw on invalid directory', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Directory does not exist'));

      await expect(importScanDirectory('/nonexistent')).rejects.toThrow('Directory does not exist');
    });

    it('should throw on no importable files', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('No importable JSON files found'));

      await expect(importScanDirectory('/empty/dir')).rejects.toThrow('No importable JSON files');
    });
  });

  describe('importExecute', () => {
    it('should call invoke with all parameters', async () => {
      mockInvoke.mockResolvedValueOnce(mockImportResult);

      const result = await importExecute('/path/to/dir', true, true);

      expect(mockInvoke).toHaveBeenCalledWith('import_execute', {
        directory: '/path/to/dir',
        importProposals: true,
        importCompanies: true,
      });
      expect(result).toEqual(mockImportResult);
    });

    it('should support importing only companies', async () => {
      const companiesOnly: ImportResult = {
        ...mockImportResult,
        projects_created: 0,
        fees_created: 0,
      };
      mockInvoke.mockResolvedValueOnce(companiesOnly);

      const result = await importExecute('/path/to/dir', false, true);

      expect(mockInvoke).toHaveBeenCalledWith('import_execute', {
        directory: '/path/to/dir',
        importProposals: false,
        importCompanies: true,
      });
      expect(result.projects_created).toBe(0);
      expect(result.companies_created).toBe(3);
    });

    it('should support importing only proposals', async () => {
      const proposalsOnly: ImportResult = {
        ...mockImportResult,
        companies_created: 0,
        companies_skipped: 0,
      };
      mockInvoke.mockResolvedValueOnce(proposalsOnly);

      const result = await importExecute('/path/to/dir', true, false);

      expect(result.fees_created).toBe(2);
      expect(result.companies_created).toBe(0);
    });

    it('should return success with warnings', async () => {
      mockInvoke.mockResolvedValueOnce(mockImportResult);

      const result = await importExecute('/path/to/dir', true, true);

      expect(result.success).toBe(true);
      expect(result.warnings).toHaveLength(1);
      expect(result.warnings[0]).toContain('already exists');
    });

    it('should handle partial failure', async () => {
      const partialFailure: ImportResult = {
        success: true,
        projects_created: 1,
        fees_created: 1,
        companies_created: 2,
        companies_skipped: 0,
        errors: ['Failed to create project "X": duplicate key'],
        warnings: [],
      };
      mockInvoke.mockResolvedValueOnce(partialFailure);

      const result = await importExecute('/path/to/dir', true, true);

      expect(result.success).toBe(true);
      expect(result.errors).toHaveLength(1);
      expect(result.projects_created).toBe(1);
    });

    it('should throw on complete failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database connection failed'));

      await expect(importExecute('/path', true, true)).rejects.toThrow('Database connection failed');
    });
  });
});
