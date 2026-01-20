/**
 * Optimized company lookup utilities with memoization.
 * 
 * This module provides efficient company lookup functions that use caching
 * to avoid repeated database lookups and complex ID parsing operations.
 * It replaces the duplicate company lookup logic found across components.
 */

import type { Company, UnknownSurrealThing } from '../../types';
import { extractId } from './index';

/**
 * Memoization cache for company lookups.
 * Maps company IDs to company objects for fast retrieval.
 */
const companyCache = new Map<string, Company>();

/**
 * Track the last companies array reference to avoid unnecessary cache rebuilds.
 * PERF-H4: Prevents cache thrashing when reactive system re-evaluates with same data.
 */
let lastCompaniesRef: Company[] | null = null;
let lastCompaniesLength = 0;

/**
 * Company lookup interface providing optimized lookup methods.
 */
export interface CompanyLookup {
  /** Get the full company name */
  getCompanyName: (companyRef: UnknownSurrealThing) => string;
  /** Get the short company name */
  getCompanyShortName: (companyRef: UnknownSurrealThing) => string;
  /** Get the company country */
  getCompanyCountry: (companyRef: UnknownSurrealThing) => string;
  /** Get the company city */
  getCompanyCity: (companyRef: UnknownSurrealThing) => string;
  /** Get the company abbreviation */
  getCompanyAbbreviation: (companyRef: UnknownSurrealThing) => string;
  /** Get the full company object */
  getCompany: (companyRef: UnknownSurrealThing) => Company | undefined;
  /** Get searchable text for a company (name, abbreviation, name_short) */
  getCompanySearchText: (companyRef: UnknownSurrealThing) => string;
  /** Check if cache needs rebuilding */
  getCacheSize: () => number;
}

/**
 * Creates an optimized company lookup system with memoization.
 * 
 * This function builds a cache of companies for fast lookups and provides
 * methods to retrieve company information without repeated ID parsing.
 * The cache is rebuilt when the companies array changes.
 * 
 * @param companies - Array of company objects to cache
 * @returns CompanyLookup interface with optimized lookup methods
 * 
 * @example
 * ```typescript
 * const lookup = createCompanyLookup(companies);
 * const companyName = lookup.getCompanyName(contact.company);
 * const country = lookup.getCompanyCountry(contact.company);
 * ```
 */
export function createCompanyLookup(companies: Company[]): CompanyLookup {
  // Handle case where companies is null, undefined, or not an array
  if (!companies || !Array.isArray(companies)) {
    companies = [];
  }

  // PERF-H4: Only rebuild cache if companies array has actually changed
  // Check both reference equality and length to detect changes
  const needsRebuild =
    lastCompaniesRef !== companies ||
    lastCompaniesLength !== companies.length ||
    companyCache.size === 0;

  if (needsRebuild) {
    companyCache.clear();

    companies.forEach(company => {
      const id = extractId(company.id);
      if (id) {
        companyCache.set(id, company);
        // Also cache with full record ID format for flexibility
        if (typeof company.id === 'object' && company.id) {
          const fullId = `company:${id}`;
          companyCache.set(fullId, company);
        }
      }
    });

    // Update tracking references
    lastCompaniesRef = companies;
    lastCompaniesLength = companies.length;
  }

  return {
    getCompanyName: (companyRef: UnknownSurrealThing): string => {
      const id = extractId(companyRef);
      if (!id) return 'Unknown Company';
      
      const company = companyCache.get(id) || companyCache.get(`company:${id}`);
      return company?.name || 'Unknown Company';
    },
    
    getCompanyShortName: (companyRef: UnknownSurrealThing): string => {
      const id = extractId(companyRef);
      if (!id) return 'N/A';
      
      const company = companyCache.get(id) || companyCache.get(`company:${id}`);
      return company?.name_short || company?.name || 'N/A';
    },
    
    getCompanyCountry: (companyRef: UnknownSurrealThing): string => {
      const id = extractId(companyRef);
      if (!id) return 'N/A';
      
      const company = companyCache.get(id) || companyCache.get(`company:${id}`);
      return company?.country || 'N/A';
    },

    getCompanyCity: (companyRef: UnknownSurrealThing): string => {
      const id = extractId(companyRef);
      if (!id) return 'N/A';
      
      const company = companyCache.get(id) || companyCache.get(`company:${id}`);
      return company?.city || 'N/A';
    },

    getCompanyAbbreviation: (companyRef: UnknownSurrealThing): string => {
      const id = extractId(companyRef);
      if (!id) return 'N/A';
      
      const company = companyCache.get(id) || companyCache.get(`company:${id}`);
      return company?.abbreviation || 'N/A';
    },

    getCompany: (companyRef: UnknownSurrealThing): Company | undefined => {
      const id = extractId(companyRef);
      if (!id) return undefined;

      return companyCache.get(id) || companyCache.get(`company:${id}`);
    },

    getCompanySearchText: (companyRef: UnknownSurrealThing): string => {
      const id = extractId(companyRef);
      if (!id) return '';

      const company = companyCache.get(id) || companyCache.get(`company:${id}`);
      if (!company) return '';

      // Combine all searchable company fields
      return [
        company.name,
        company.name_short,
        company.abbreviation
      ].filter(Boolean).join(' ');
    },

    getCacheSize: (): number => {
      return companyCache.size;
    }
  };
}

/**
 * Clears the company lookup cache.
 *
 * This can be called when companies are updated to ensure
 * the cache doesn't contain stale data.
 */
export function clearCompanyCache(): void {
  companyCache.clear();
  lastCompaniesRef = null;
  lastCompaniesLength = 0;
}

/**
 * Gets cache statistics for debugging and monitoring.
 * 
 * @returns Object containing cache size and hit rate information
 */
export function getCompanyCacheStats(): { size: number; keys: string[] } {
  return {
    size: companyCache.size,
    keys: Array.from(companyCache.keys())
  };
}