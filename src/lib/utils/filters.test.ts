/**
 * Filter Utilities Test Suite
 *
 * Tests for the filtering system including the customSearchFunctions
 * feature that enables searching contacts by company code/name.
 */

import { describe, it, expect } from 'vitest';
import {
  createFilterFunction,
  getUniqueFieldValues,
  hasActiveFilters,
  clearAllFilters,
  getFilterCounts,
  type FilterConfig
} from './filters';

// Test entity interface
interface TestContact {
  id: string;
  full_name: string;
  first_name: string;
  last_name: string;
  email: string;
  phone: string;
  position: string;
  company: string; // Company reference ID
  time?: {
    updated_at?: string;
    created_at?: string;
  };
}

// Mock contacts data - simulates real contact records
const mockContacts: TestContact[] = [
  {
    id: 'contact_1',
    full_name: 'John Smith',
    first_name: 'John',
    last_name: 'Smith',
    email: 'john@ptg.com',
    phone: '+971501234567',
    position: 'Project Manager',
    company: 'company:PTG',
    time: { updated_at: '2025-01-15T10:00:00Z' }
  },
  {
    id: 'contact_2',
    full_name: 'Jane Doe',
    first_name: 'Jane',
    last_name: 'Doe',
    email: 'jane@dubaiislands.com',
    phone: '+971502345678',
    position: 'Senior Architect',
    company: 'company:DIL',
    time: { updated_at: '2025-01-14T10:00:00Z' }
  },
  {
    id: 'contact_3',
    full_name: 'Ryan Marginson',
    first_name: 'Ryan',
    last_name: 'Marginson',
    email: 'ryan@dubaiislands.com',
    phone: '+971503456789',
    position: 'Director',
    company: 'company:DIL',
    time: { updated_at: '2025-01-13T10:00:00Z' }
  },
  {
    id: 'contact_4',
    full_name: 'Ahmed Hassan',
    first_name: 'Ahmed',
    last_name: 'Hassan',
    email: 'ahmed@nakheel.com',
    phone: '+971504567890',
    position: 'Engineer',
    company: 'company:NKLM',
    time: { updated_at: '2025-01-12T10:00:00Z' }
  },
  {
    id: 'contact_5',
    full_name: 'Sarah Connor',
    first_name: 'Sarah',
    last_name: 'Connor',
    email: 'sarah@ptdubai.com',
    phone: '+971505678901',
    position: 'Manager',
    company: 'company:PTDUBAI',
    time: { updated_at: '2025-01-11T10:00:00Z' }
  }
];

// Mock company lookup function - simulates the real companyLookup utility
function mockGetCompanySearchText(companyRef: string): string {
  const companyData: Record<string, string> = {
    'company:PTG': 'P&T Group P&T Group PTG',
    'company:DIL': 'Dubai Islands LLC Dubai Islands DIL',
    'company:NKLM': 'Nakheel PJSC Nakheel NKLM',
    'company:PTDUBAI': 'P&T Dubai P&T Dubai PTDUBAI'
  };
  return companyData[companyRef] || '';
}

function mockGetCompanyShortName(companyRef: string): string {
  const shortNames: Record<string, string> = {
    'company:PTG': 'P&T Group',
    'company:DIL': 'Dubai Islands',
    'company:NKLM': 'Nakheel',
    'company:PTDUBAI': 'P&T Dubai'
  };
  return shortNames[companyRef] || 'N/A';
}

describe('Filter Utilities', () => {
  describe('createFilterFunction', () => {
    describe('Basic Search', () => {
      const basicConfig: FilterConfig<TestContact> = {
        searchFields: ['full_name', 'first_name', 'last_name', 'email', 'phone', 'position']
      };

      it('should filter by direct field search', () => {
        const filtered = createFilterFunction(mockContacts, 'john', {}, basicConfig);
        expect(filtered.length).toBe(1);
        expect(filtered[0].full_name).toBe('John Smith');
      });

      it('should be case-insensitive', () => {
        const filtered = createFilterFunction(mockContacts, 'JOHN', {}, basicConfig);
        expect(filtered.length).toBe(1);
        expect(filtered[0].full_name).toBe('John Smith');
      });

      it('should search across multiple fields', () => {
        // Search by email domain
        const filtered = createFilterFunction(mockContacts, 'dubaiislands.com', {}, basicConfig);
        expect(filtered.length).toBe(2);
        expect(filtered.map(c => c.full_name)).toContain('Jane Doe');
        expect(filtered.map(c => c.full_name)).toContain('Ryan Marginson');
      });

      it('should return all items when search is empty', () => {
        const filtered = createFilterFunction(mockContacts, '', {}, basicConfig);
        expect(filtered.length).toBe(mockContacts.length);
      });

      it('should return empty array when no matches', () => {
        const filtered = createFilterFunction(mockContacts, 'nonexistent', {}, basicConfig);
        expect(filtered.length).toBe(0);
      });
    });

    describe('Custom Search Functions', () => {
      // This tests the key feature added for company code search
      const configWithCompanySearch: FilterConfig<TestContact> = {
        searchFields: ['full_name', 'first_name', 'last_name', 'email', 'phone', 'position'],
        customSearchFunctions: [
          (contact) => mockGetCompanySearchText(contact.company)
        ]
      };

      it('should search by company code (PTG)', () => {
        const filtered = createFilterFunction(mockContacts, 'ptg', {}, configWithCompanySearch);
        expect(filtered.length).toBe(1);
        expect(filtered[0].full_name).toBe('John Smith');
        expect(filtered[0].company).toBe('company:PTG');
      });

      it('should search by company code (DIL)', () => {
        const filtered = createFilterFunction(mockContacts, 'dil', {}, configWithCompanySearch);
        expect(filtered.length).toBe(2);
        expect(filtered.map(c => c.full_name)).toContain('Jane Doe');
        expect(filtered.map(c => c.full_name)).toContain('Ryan Marginson');
      });

      it('should search by partial company name (dub)', () => {
        // This was the bug: searching "dub" should find both Dubai Islands AND P&T Dubai contacts
        const filtered = createFilterFunction(mockContacts, 'dub', {}, configWithCompanySearch);

        // Should find: DIL (Dubai Islands - 2 contacts) and PTDUBAI (1 contact)
        expect(filtered.length).toBe(3);
        expect(filtered.map(c => c.full_name)).toContain('Jane Doe'); // DIL
        expect(filtered.map(c => c.full_name)).toContain('Ryan Marginson'); // DIL
        expect(filtered.map(c => c.full_name)).toContain('Sarah Connor'); // PTDUBAI
      });

      it('should search by full company name', () => {
        const filtered = createFilterFunction(mockContacts, 'nakheel', {}, configWithCompanySearch);
        expect(filtered.length).toBe(1);
        expect(filtered[0].full_name).toBe('Ahmed Hassan');
      });

      it('should combine direct field and custom search matches', () => {
        // Search for "dubai" - should match:
        // 1. DIL contacts (company name contains "Dubai")
        // 2. PTDUBAI contact (company name contains "Dubai")
        // 3. Any contact with "dubai" in their direct fields
        const filtered = createFilterFunction(mockContacts, 'dubai', {}, configWithCompanySearch);
        expect(filtered.length).toBe(3);
      });

      it('should support multiple custom search functions', () => {
        const configWithMultipleCustom: FilterConfig<TestContact> = {
          searchFields: ['full_name'],
          customSearchFunctions: [
            (contact) => mockGetCompanySearchText(contact.company),
            (contact) => `extra_${contact.position}` // Additional searchable field
          ]
        };

        const filtered = createFilterFunction(mockContacts, 'extra_director', {}, configWithMultipleCustom);
        expect(filtered.length).toBe(1);
        expect(filtered[0].full_name).toBe('Ryan Marginson');
      });
    });

    describe('Filter Fields', () => {
      const configWithFilters: FilterConfig<TestContact> = {
        searchFields: ['full_name'],
        filterFields: {
          company: (contact) => mockGetCompanyShortName(contact.company),
          position: (contact) => contact.position
        }
      };

      it('should filter by company dropdown', () => {
        const filtered = createFilterFunction(
          mockContacts,
          '',
          { company: 'Dubai Islands' },
          configWithFilters
        );
        expect(filtered.length).toBe(2);
        expect(filtered.map(c => c.full_name)).toContain('Jane Doe');
        expect(filtered.map(c => c.full_name)).toContain('Ryan Marginson');
      });

      it('should filter by position dropdown', () => {
        const filtered = createFilterFunction(
          mockContacts,
          '',
          { position: 'Director' },
          configWithFilters
        );
        expect(filtered.length).toBe(1);
        expect(filtered[0].full_name).toBe('Ryan Marginson');
      });

      it('should combine multiple filters', () => {
        const filtered = createFilterFunction(
          mockContacts,
          '',
          { company: 'Dubai Islands', position: 'Senior Architect' },
          configWithFilters
        );
        expect(filtered.length).toBe(1);
        expect(filtered[0].full_name).toBe('Jane Doe');
      });
    });

    describe('Combined Search and Filters', () => {
      const fullConfig: FilterConfig<TestContact> = {
        searchFields: ['full_name', 'first_name', 'last_name', 'email', 'position'],
        filterFields: {
          company: (contact) => mockGetCompanyShortName(contact.company)
        },
        customSearchFunctions: [
          (contact) => mockGetCompanySearchText(contact.company)
        ]
      };

      it('should apply both search and filters', () => {
        // Search for people at Dubai Islands (DIL) who are Directors
        const filtered = createFilterFunction(
          mockContacts,
          'director',
          { company: 'Dubai Islands' },
          fullConfig
        );
        expect(filtered.length).toBe(1);
        expect(filtered[0].full_name).toBe('Ryan Marginson');
      });

      it('should narrow results when combining company code search with filters', () => {
        // Search by company code, filter by position
        const filtered = createFilterFunction(
          mockContacts,
          'dil', // Company code
          { position: 'Senior Architect' },
          { ...fullConfig, filterFields: { position: (c) => c.position } }
        );
        expect(filtered.length).toBe(1);
        expect(filtered[0].full_name).toBe('Jane Doe');
      });
    });

    describe('Sorting', () => {
      const configWithSort: FilterConfig<TestContact> = {
        searchFields: ['full_name']
      };

      it('should sort by updated_at descending by default', () => {
        const filtered = createFilterFunction(mockContacts, '', {}, configWithSort);
        // Most recently updated first
        expect(filtered[0].full_name).toBe('John Smith');
        expect(filtered[filtered.length - 1].full_name).toBe('Sarah Connor');
      });

      it('should use custom sort function when provided', () => {
        const customSortConfig: FilterConfig<TestContact> = {
          searchFields: ['full_name'],
          sortFunction: (a, b) => a.full_name.localeCompare(b.full_name)
        };

        const filtered = createFilterFunction(mockContacts, '', {}, customSortConfig);
        expect(filtered[0].full_name).toBe('Ahmed Hassan');
        expect(filtered[filtered.length - 1].full_name).toBe('Sarah Connor');
      });
    });
  });

  describe('getUniqueFieldValues', () => {
    it('should extract unique values', () => {
      const positions = getUniqueFieldValues(mockContacts, (c) => c.position);
      expect(positions).toContain('Project Manager');
      expect(positions).toContain('Senior Architect');
      expect(positions).toContain('Director');
      expect(positions).toContain('Engineer');
      expect(positions).toContain('Manager');
      expect(positions.length).toBe(5);
    });

    it('should return sorted values', () => {
      const positions = getUniqueFieldValues(mockContacts, (c) => c.position);
      const sorted = [...positions].sort();
      expect(positions).toEqual(sorted);
    });

    it('should filter out empty values', () => {
      const contactsWithEmpty: TestContact[] = [
        ...mockContacts,
        { ...mockContacts[0], id: 'empty', position: '' }
      ];
      const positions = getUniqueFieldValues(contactsWithEmpty, (c) => c.position);
      expect(positions).not.toContain('');
    });

    it('should return unique company names', () => {
      const companies = getUniqueFieldValues(
        mockContacts,
        (c) => mockGetCompanyShortName(c.company)
      );
      expect(companies.length).toBe(4);
      expect(companies).toContain('P&T Group');
      expect(companies).toContain('Dubai Islands');
      expect(companies).toContain('Nakheel');
      expect(companies).toContain('P&T Dubai');
    });
  });

  describe('hasActiveFilters', () => {
    it('should return false when no filters active', () => {
      expect(hasActiveFilters({}, '')).toBe(false);
      expect(hasActiveFilters({ company: '', position: '' }, '')).toBe(false);
    });

    it('should return true when search query is active', () => {
      expect(hasActiveFilters({}, 'test')).toBe(true);
    });

    it('should return true when filter is active', () => {
      expect(hasActiveFilters({ company: 'P&T Group' }, '')).toBe(true);
    });

    it('should return true when both search and filter active', () => {
      expect(hasActiveFilters({ company: 'P&T Group' }, 'john')).toBe(true);
    });

    it('should ignore whitespace-only values', () => {
      expect(hasActiveFilters({ company: '   ' }, '   ')).toBe(false);
    });
  });

  describe('clearAllFilters', () => {
    it('should clear all filter values', () => {
      const filters = { company: 'P&T Group', position: 'Director' };
      const result = clearAllFilters(filters);

      expect(result).toBe('');
      expect(filters.company).toBe('');
      expect(filters.position).toBe('');
    });

    it('should mutate the original filters object', () => {
      const filters = { company: 'Test' };
      clearAllFilters(filters);
      expect(filters.company).toBe('');
    });
  });

  describe('getFilterCounts', () => {
    const config: FilterConfig<TestContact> = {
      searchFields: ['full_name'],
      customSearchFunctions: [
        (contact) => mockGetCompanySearchText(contact.company)
      ]
    };

    it('should return total and filtered counts', () => {
      const counts = getFilterCounts(mockContacts, 'dil', {}, config);
      expect(counts.total).toBe(5);
      expect(counts.filtered).toBe(2);
    });

    it('should return equal counts when no filters', () => {
      const counts = getFilterCounts(mockContacts, '', {}, config);
      expect(counts.total).toBe(counts.filtered);
    });
  });
});

describe('Integration: Contact Search by Company Code', () => {
  // This test suite specifically validates the user's reported bug fix
  // Bug: Searching "dub" in contacts caused Ryan Marginson from Dubai Islands to disappear

  const realWorldConfig: FilterConfig<TestContact> = {
    searchFields: ['full_name', 'first_name', 'last_name', 'email', 'phone', 'position'],
    filterFields: {
      company: (contact) => mockGetCompanyShortName(contact.company),
      position: (contact) => contact.position
    },
    customSearchFunctions: [
      (contact) => mockGetCompanySearchText(contact.company)
    ]
  };

  it('should find Ryan Marginson when searching "dub"', () => {
    const filtered = createFilterFunction(mockContacts, 'dub', {}, realWorldConfig);

    // Ryan should appear because his company (DIL = Dubai Islands) contains "dub"
    const ryanFound = filtered.some(c => c.full_name === 'Ryan Marginson');
    expect(ryanFound).toBe(true);
  });

  it('should find all contacts at companies containing "dub"', () => {
    const filtered = createFilterFunction(mockContacts, 'dub', {}, realWorldConfig);

    // Should find 3 contacts:
    // - Jane Doe (DIL = Dubai Islands)
    // - Ryan Marginson (DIL = Dubai Islands)
    // - Sarah Connor (PTDUBAI = P&T Dubai)
    expect(filtered.length).toBe(3);
  });

  it('should find contacts when searching by exact company code', () => {
    const filtered = createFilterFunction(mockContacts, 'ptg', {}, realWorldConfig);
    expect(filtered.length).toBe(1);
    expect(filtered[0].company).toBe('company:PTG');
  });

  it('should differentiate PTG from PTDUBAI', () => {
    // Searching "ptg" should only find P&T Group, not P&T Dubai
    const ptgFiltered = createFilterFunction(mockContacts, 'ptg', {}, realWorldConfig);
    expect(ptgFiltered.length).toBe(1);
    expect(ptgFiltered[0].company).toBe('company:PTG');

    // Searching "ptdubai" should only find P&T Dubai
    const ptdubaiFiltered = createFilterFunction(mockContacts, 'ptdubai', {}, realWorldConfig);
    expect(ptdubaiFiltered.length).toBe(1);
    expect(ptdubaiFiltered[0].company).toBe('company:PTDUBAI');
  });
});
