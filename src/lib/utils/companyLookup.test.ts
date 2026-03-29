/**
 * Company Lookup Utilities Test Suite
 *
 * Tests for optimized company lookup with memoization,
 * including the company search text functionality for
 * searching contacts by company code/name.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  createCompanyLookup,
  clearCompanyCache,
  getCompanyCacheStats,
  type CompanyLookup
} from './companyLookup';
import type { Company } from '../../types';

// Test company data — id values use SurrealDB Thing objects to match real DB responses
const mockCompanies = [
  {
    id: { tb: 'company', id: 'PTG' },
    name: 'P&T Group',
    name_short: 'P&T Group',
    abbreviation: 'PTG',
    city: 'Abu Dhabi',
    country: 'UAE'
  },
  {
    id: { tb: 'company', id: 'DIL' },
    name: 'Dubai Islands LLC',
    name_short: 'Dubai Islands',
    abbreviation: 'DIL',
    city: 'Dubai',
    country: 'UAE'
  },
  {
    id: { tb: 'company', id: 'NKLM' },
    name: 'Nakheel PJSC',
    name_short: 'Nakheel',
    abbreviation: 'NKLM',
    city: 'Dubai',
    country: 'UAE'
  },
  {
    id: 'company:PTDUBAI',
    name: 'P&T Dubai',
    name_short: 'P&T Dubai',
    abbreviation: 'PTDUBAI',
    city: 'Dubai',
    country: 'UAE'
  }
] as unknown as Company[];

describe('Company Lookup Utilities', () => {
  let lookup: CompanyLookup;

  beforeEach(() => {
    clearCompanyCache();
    lookup = createCompanyLookup(mockCompanies);
  });

  describe('createCompanyLookup', () => {
    it('should create a lookup from company array', () => {
      expect(lookup).toBeDefined();
      expect(lookup.getCacheSize()).toBeGreaterThan(0);
    });

    it('should handle null companies array', () => {
      const emptyLookup = createCompanyLookup(null as any);
      expect(emptyLookup.getCacheSize()).toBe(0);
    });

    it('should handle undefined companies array', () => {
      const emptyLookup = createCompanyLookup(undefined as any);
      expect(emptyLookup.getCacheSize()).toBe(0);
    });

    it('should handle empty companies array', () => {
      const emptyLookup = createCompanyLookup([]);
      expect(emptyLookup.getCacheSize()).toBe(0);
    });
  });

  describe('getCompanyName', () => {
    it('should return company name for object ID', () => {
      const name = lookup.getCompanyName({ tb: 'company', id: 'PTG' });
      expect(name).toBe('P&T Group');
    });

    it('should return company name for string ID', () => {
      const name = lookup.getCompanyName('company:PTDUBAI');
      expect(name).toBe('P&T Dubai');
    });

    it('should return "Unknown Company" for invalid reference', () => {
      const name = lookup.getCompanyName({ tb: 'company', id: 'NONEXISTENT' });
      expect(name).toBe('Unknown Company');
    });

    it('should return "Unknown Company" for null reference', () => {
      const name = lookup.getCompanyName(null as any);
      expect(name).toBe('Unknown Company');
    });
  });

  describe('getCompanyShortName', () => {
    it('should return short name when available', () => {
      const shortName = lookup.getCompanyShortName({ tb: 'company', id: 'DIL' });
      expect(shortName).toBe('Dubai Islands');
    });

    it('should fall back to full name when short name missing', () => {
      // Create company without name_short
      const companiesNoShort = [
        {
          id: { tb: 'company', id: 'TEST' },
          name: 'Test Company Full Name',
          abbreviation: 'TEST',
          city: 'Test City',
          country: 'Test Country'
        }
      ];
      const testLookup = createCompanyLookup(companiesNoShort as unknown as Company[]);
      const shortName = testLookup.getCompanyShortName({ tb: 'company', id: 'TEST' });
      expect(shortName).toBe('Test Company Full Name');
    });

    it('should return "N/A" for invalid reference', () => {
      const shortName = lookup.getCompanyShortName({ tb: 'company', id: 'NONEXISTENT' });
      expect(shortName).toBe('N/A');
    });
  });

  describe('getCompanyCountry', () => {
    it('should return country for valid company', () => {
      const country = lookup.getCompanyCountry({ tb: 'company', id: 'PTG' });
      expect(country).toBe('UAE');
    });

    it('should return "N/A" for company without country', () => {
      const companiesNoCountry = [
        {
          id: { tb: 'company', id: 'NOCOUNTRY' },
          name: 'No Country Company',
          abbreviation: 'NC',
          city: 'Some City'
          // country intentionally omitted to test fallback
        }
      ];
      const testLookup = createCompanyLookup(companiesNoCountry as unknown as Company[]);
      const country = testLookup.getCompanyCountry({ tb: 'company', id: 'NOCOUNTRY' });
      expect(country).toBe('N/A');
    });
  });

  describe('getCompanyCity', () => {
    it('should return city for valid company', () => {
      const city = lookup.getCompanyCity({ tb: 'company', id: 'DIL' });
      expect(city).toBe('Dubai');
    });

    it('should return "N/A" for invalid reference', () => {
      const city = lookup.getCompanyCity({ tb: 'company', id: 'NONEXISTENT' });
      expect(city).toBe('N/A');
    });
  });

  describe('getCompanyAbbreviation', () => {
    it('should return abbreviation for valid company', () => {
      const abbr = lookup.getCompanyAbbreviation({ tb: 'company', id: 'NKLM' });
      expect(abbr).toBe('NKLM');
    });

    it('should return "N/A" for company without abbreviation', () => {
      const companiesNoAbbr = [
        {
          id: { tb: 'company', id: 'NOABBR' },
          name: 'No Abbreviation Company',
          city: 'Some City',
          country: 'Some Country'
          // abbreviation intentionally omitted to test fallback
        }
      ];
      const testLookup = createCompanyLookup(companiesNoAbbr as unknown as Company[]);
      const abbr = testLookup.getCompanyAbbreviation({ tb: 'company', id: 'NOABBR' });
      expect(abbr).toBe('N/A');
    });
  });

  describe('getCompany', () => {
    it('should return full company object', () => {
      const company = lookup.getCompany({ tb: 'company', id: 'PTG' });
      expect(company).toBeDefined();
      expect(company?.name).toBe('P&T Group');
      expect(company?.abbreviation).toBe('PTG');
    });

    it('should return undefined for invalid reference', () => {
      const company = lookup.getCompany({ tb: 'company', id: 'NONEXISTENT' });
      expect(company).toBeUndefined();
    });
  });

  describe('getCompanySearchText', () => {
    it('should return combined searchable text', () => {
      const searchText = lookup.getCompanySearchText({ tb: 'company', id: 'PTG' });
      expect(searchText).toContain('P&T Group');
      expect(searchText).toContain('PTG');
    });

    it('should include name, name_short, and abbreviation', () => {
      const searchText = lookup.getCompanySearchText({ tb: 'company', id: 'DIL' });
      expect(searchText).toContain('Dubai Islands LLC');
      expect(searchText).toContain('Dubai Islands');
      expect(searchText).toContain('DIL');
    });

    it('should return empty string for invalid reference', () => {
      const searchText = lookup.getCompanySearchText({ tb: 'company', id: 'NONEXISTENT' });
      expect(searchText).toBe('');
    });

    it('should return empty string for null reference', () => {
      const searchText = lookup.getCompanySearchText(null as any);
      expect(searchText).toBe('');
    });

    it('should handle company with partial fields', () => {
      const partialCompanies = [
        {
          id: { tb: 'company', id: 'PARTIAL' },
          name: 'Only Name Company',
          city: 'City',
          country: 'Country'
        }
      ];
      const testLookup = createCompanyLookup(partialCompanies as unknown as Company[]);
      const searchText = testLookup.getCompanySearchText({ tb: 'company', id: 'PARTIAL' });
      expect(searchText).toBe('Only Name Company');
    });

    // Key test: This verifies the bug fix for searching contacts by company code
    it('should allow case-insensitive search by company code', () => {
      const searchText = lookup.getCompanySearchText({ tb: 'company', id: 'PTG' });
      const searchLower = searchText.toLowerCase();

      // User types "ptg" - should find P&T Group
      expect(searchLower.includes('ptg')).toBe(true);
    });

    it('should allow search by partial company name', () => {
      const searchText = lookup.getCompanySearchText({ tb: 'company', id: 'DIL' });
      const searchLower = searchText.toLowerCase();

      // User types "dub" - should find Dubai Islands
      expect(searchLower.includes('dub')).toBe(true);
    });

    it('should distinguish between similar company codes', () => {
      // PTG vs PTDUBAI - both contain "pt"
      const ptgText = lookup.getCompanySearchText({ tb: 'company', id: 'PTG' }).toLowerCase();
      const ptdubaiText = lookup.getCompanySearchText('company:PTDUBAI').toLowerCase();

      // Both should match "pt"
      expect(ptgText.includes('pt')).toBe(true);
      expect(ptdubaiText.includes('pt')).toBe(true);

      // Only PTDUBAI should match "ptdub"
      expect(ptgText.includes('ptdub')).toBe(false);
      expect(ptdubaiText.includes('ptdub')).toBe(true);
    });
  });

  describe('clearCompanyCache', () => {
    it('should clear all cached companies', () => {
      expect(lookup.getCacheSize()).toBeGreaterThan(0);
      clearCompanyCache();
      // After clearing, a new lookup should work but old one still has its closure
      const newLookup = createCompanyLookup([]);
      expect(newLookup.getCacheSize()).toBe(0);
    });
  });

  describe('getCompanyCacheStats', () => {
    it('should return cache statistics', () => {
      const stats = getCompanyCacheStats();
      expect(stats.size).toBeGreaterThan(0);
      expect(Array.isArray(stats.keys)).toBe(true);
    });
  });

  describe('ID format handling', () => {
    it('should handle SurrealDB Thing object format', () => {
      const name = lookup.getCompanyName({ tb: 'company', id: 'PTG' });
      expect(name).toBe('P&T Group');
    });

    it('should handle string ID format with prefix', () => {
      const name = lookup.getCompanyName('company:PTDUBAI');
      expect(name).toBe('P&T Dubai');
    });

    it('should handle plain string ID', () => {
      // The lookup also caches with plain ID
      const name = lookup.getCompanyName('PTG');
      expect(name).toBe('P&T Group');
    });

    it('should handle nested ID structure', () => {
      const name = lookup.getCompanyName({ tb: 'company', id: { String: 'PTG' } });
      expect(name).toBe('P&T Group');
    });
  });
});
