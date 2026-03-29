/**
 * Unified Search Module
 *
 * Provides consistent search configurations and utilities across all entity types.
 * This module ensures that searching for "ptg" (a company code) works the same way
 * whether you're on the Proposals page, Contacts page, or using Global Search.
 */

import type { Company, Contact, Fee, Project, UnknownSurrealThing } from '../../types';
import type { CompanyLookup } from './companyLookup';
import type { FilterConfig } from './filters';
import { extractId } from './index';

/**
 * Extracts the project number ID string from a Project's number field.
 * Handles both object format (ProjectNumber) and string format.
 */
function getProjectNumberId(project: Project): string {
  if (!project.number) return '';
  if (typeof project.number === 'object' && 'id' in project.number) {
    return project.number.id || '';
  }
  return String(project.number);
}

// Import and re-export normalization utilities
import { normalizeForSearch, normalizedMatch } from './searchNormalize';
export { normalizeForSearch, normalizedMatch };

/**
 * Lookup context for related entity searches.
 * Pass these to enable searching by company name/code or project name.
 */
export interface SearchLookups {
  companyLookup?: CompanyLookup;
  projectLookup?: Map<string, Project>;
}

/**
 * Creates a project lookup map for efficient project searches.
 * Uses extractId for consistent ID handling.
 *
 * @param projects - Array of projects to cache
 * @returns Map of project IDs to project objects
 */
export function createProjectLookup(projects: Project[]): Map<string, Project> {
  const lookup = new Map<string, Project>();
  if (!projects || !Array.isArray(projects)) return lookup;

  projects.forEach(p => {
    const id = extractId(p.id);
    if (id) {
      lookup.set(id, p);
    }
  });

  return lookup;
}

/**
 * Gets searchable text for a project (name, name_short, number).
 *
 * @param projectRef - Project ID reference
 * @param lookup - Project lookup map
 * @returns Combined searchable text or empty string
 */
export function getProjectSearchText(
  projectRef: UnknownSurrealThing | undefined,
  lookup: Map<string, Project>
): string {
  if (!projectRef || !lookup) return '';

  const id = extractId(projectRef);
  if (!id) return '';

  const project = lookup.get(id);
  if (!project) return '';

  return [project.name, project.name_short, project.number?.id || project.number]
    .filter(Boolean)
    .join(' ');
}

// ============================================================================
// SEARCH FIELD DEFINITIONS
// ============================================================================

/**
 * Project search fields - what fields to search when querying projects.
 */
export const PROJECT_SEARCH_FIELDS: (keyof Project)[] = [
  'name',
  'name_short',
  'area',
  'city',
  'country',
  'folder'
];

/**
 * Company search fields - what fields to search when querying companies.
 */
export const COMPANY_SEARCH_FIELDS: (keyof Company)[] = [
  'name',
  'name_short',
  'abbreviation',
  'city',
  'country'
];

/**
 * Contact search fields - what fields to search when querying contacts.
 */
export const CONTACT_SEARCH_FIELDS: (keyof Contact)[] = [
  'full_name',
  'first_name',
  'last_name',
  'email',
  'phone',
  'position'
];

/**
 * Fee/Proposal search fields - what fields to search when querying proposals.
 */
export const FEE_SEARCH_FIELDS: (keyof Fee)[] = [
  'name',
  'number',
  'activity',
  'package',
  'staff_name'
];

// ============================================================================
// FILTER CONFIG FACTORIES
// ============================================================================

/**
 * Creates a filter config for Projects.
 * Projects don't have company references, so no related search needed.
 */
export function createProjectFilterConfig(): FilterConfig<Project> {
  return {
    searchFields: PROJECT_SEARCH_FIELDS,
    sortFunction: (a, b) => {
      // Sort by project number descending (newest first)
      const numA = getProjectNumberId(a);
      const numB = getProjectNumberId(b);
      return numB.localeCompare(numA);
    }
  };
}

/**
 * Creates a filter config for Companies.
 * Companies don't have related entities for search.
 */
export function createCompanyFilterConfig(): FilterConfig<Company> {
  return {
    searchFields: COMPANY_SEARCH_FIELDS
    // Uses default sort (updated_at descending)
  };
}

/**
 * Creates a filter config for Contacts with company search support.
 *
 * @param lookups - Optional lookups for related entity search
 * @returns FilterConfig for contacts
 *
 * @example
 * ```typescript
 * const companyLookup = createCompanyLookup(companies);
 * const filterConfig = createContactFilterConfig({ companyLookup });
 * // Now searching "ptg" will find contacts at P&T Group
 * ```
 */
export function createContactFilterConfig(lookups?: SearchLookups): FilterConfig<Contact> {
  const config: FilterConfig<Contact> = {
    searchFields: CONTACT_SEARCH_FIELDS,
    filterFields: {
      company: contact => lookups?.companyLookup?.getCompanyShortName(contact.company) || '',
      country: contact => lookups?.companyLookup?.getCompanyCountry(contact.company) || '',
      position: contact => contact.position || ''
    }
  };

  // Add company search if lookup is available
  if (lookups?.companyLookup) {
    config.customSearchFunctions = [
      contact => lookups.companyLookup!.getCompanySearchText(contact.company)
    ];
  }

  return config;
}

/**
 * Creates a filter config for Fees/Proposals with company and project search support.
 *
 * @param lookups - Optional lookups for related entity search
 * @returns FilterConfig for fees
 *
 * @example
 * ```typescript
 * const companyLookup = createCompanyLookup(companies);
 * const projectLookup = createProjectLookup(projects);
 * const filterConfig = createFeeFilterConfig({ companyLookup, projectLookup });
 * // Now searching "ptg" will find proposals for P&T Group
 * // And searching "dubai islands" will find proposals for that project
 * ```
 */
export function createFeeFilterConfig(lookups?: SearchLookups): FilterConfig<Fee> {
  const customSearchFunctions: ((item: Fee) => string)[] = [];

  // Add company search if lookup is available
  if (lookups?.companyLookup) {
    customSearchFunctions.push(fee => lookups.companyLookup!.getCompanySearchText(fee.company_id));
  }

  // Add project search if lookup is available
  if (lookups?.projectLookup) {
    customSearchFunctions.push(fee => getProjectSearchText(fee.project_id, lookups.projectLookup!));
  }

  return {
    searchFields: FEE_SEARCH_FIELDS,
    customSearchFunctions: customSearchFunctions.length > 0 ? customSearchFunctions : undefined,
    sortFunction: (a, b) => {
      // Primary sort: issue_date descending (most recent first)
      // issue_date is in YYMMDD format
      const parseIssueDate = (issueDate: string) => {
        if (!issueDate || issueDate.length !== 6) return 0;
        const year = 2000 + parseInt(issueDate.substring(0, 2));
        const month = parseInt(issueDate.substring(2, 4)) - 1;
        const day = parseInt(issueDate.substring(4, 6));
        return new Date(year, month, day).getTime();
      };

      const aIssueDate = parseIssueDate(a.issue_date);
      const bIssueDate = parseIssueDate(b.issue_date);

      if (bIssueDate !== aIssueDate) {
        return bIssueDate - aIssueDate;
      }

      // Secondary sort: updated_at descending
      const aUpdated = new Date(a.time?.updated_at || a.time?.created_at || 0).getTime();
      const bUpdated = new Date(b.time?.updated_at || b.time?.created_at || 0).getTime();
      return bUpdated - aUpdated;
    }
  };
}

// ============================================================================
// SEARCH RESULT HELPERS
// ============================================================================

/**
 * Gets display information for a fee/proposal in search results.
 * Returns project name + package as the title.
 *
 * @param fee - The fee to get display info for
 * @param lookups - Lookups for related entities
 * @returns Object with name and subtitle
 */
export function getFeeDisplayInfo(
  fee: Fee,
  lookups?: SearchLookups
): { name: string; subtitle: string } {
  let projectName: string | undefined;

  if (lookups?.projectLookup && fee.project_id) {
    const projectId = extractId(fee.project_id);
    const project = lookups.projectLookup.get(projectId);
    projectName = project?.name_short || project?.name;
  }

  // Build display name: Project Name - Package (if both exist)
  const displayName =
    [projectName, fee.package].filter(Boolean).join(' - ') ||
    fee.name ||
    fee.number ||
    'Unnamed Proposal';

  // Subtitle: fee number + status
  const subtitle = [fee.number, fee.status].filter(Boolean).join(' • ');

  return { name: displayName, subtitle };
}

/**
 * Gets display information for a contact in search results.
 *
 * @param contact - The contact to get display info for
 * @param lookups - Lookups for related entities
 * @returns Object with name and subtitle
 */
export function getContactDisplayInfo(
  contact: Contact,
  lookups?: SearchLookups
): { name: string; subtitle: string } {
  const name =
    contact.full_name ||
    `${contact.first_name || ''} ${contact.last_name || ''}`.trim() ||
    'Unnamed Contact';

  const companyName = lookups?.companyLookup?.getCompanyShortName(contact.company);

  const subtitle = [contact.position, companyName !== 'N/A' ? companyName : null, contact.email]
    .filter(Boolean)
    .join(' • ');

  return { name, subtitle };
}

/**
 * Gets display information for a project in search results.
 *
 * @param project - The project to get display info for
 * @returns Object with name and subtitle
 */
export function getProjectDisplayInfo(project: Project): { name: string; subtitle: string } {
  const name = project.name || 'Unnamed Project';
  const projectNumber = getProjectNumberId(project);

  const subtitle = [projectNumber, project.city, project.country].filter(Boolean).join(' • ');

  return { name, subtitle };
}

/**
 * Gets display information for a company in search results.
 *
 * @param company - The company to get display info for
 * @returns Object with name and subtitle
 */
export function getCompanyDisplayInfo(company: Company): { name: string; subtitle: string } {
  const name = company.name || 'Unnamed Company';
  const subtitle = [company.abbreviation, company.city, company.country]
    .filter(Boolean)
    .join(' • ');

  return { name, subtitle };
}

// ============================================================================
// TYPEAHEAD SEARCH UTILITIES
// ============================================================================

/**
 * Configuration for typeahead search.
 */
export interface TypeaheadSearchConfig<T, R> {
  /** Fields to search within each item */
  searchFields: (keyof T)[];
  /** Function to extract nested field values (e.g., project.number.id) */
  nestedFieldExtractor?: (item: T) => string[];
  /** Function to transform item to result format */
  mapToResult: (item: T) => R;
  /** Maximum results to return (default: 10) */
  maxResults?: number;
}

/**
 * Creates a typeahead search function for a given entity type.
 *
 * Performance optimized:
 * - Normalizes search query once (not per item)
 * - Uses pre-defined search fields
 * - Limits results early
 *
 * @param config - Search configuration
 * @returns A search function that takes items and search text
 *
 * @example
 * ```typescript
 * const searchProjects = createTypeaheadSearch({
 *   searchFields: ['name', 'name_short'],
 *   nestedFieldExtractor: (p) => [p.number?.id || ''],
 *   mapToResult: (p) => ({ id: extractId(p.id), name: p.name })
 * });
 *
 * const results = searchProjects(projects, 'dubai');
 * ```
 */
export function createTypeaheadSearch<T, R>(
  config: TypeaheadSearchConfig<T, R>
): (items: T[], searchText: string) => R[] {
  const { searchFields, nestedFieldExtractor, mapToResult, maxResults = 10 } = config;

  return (items: T[], searchText: string): R[] => {
    if (!searchText || searchText.length < 1 || !items?.length) {
      return [];
    }

    // Normalize search query once (PERF-H5 fix)
    const normalizedQuery = normalizeForSearch(searchText);
    const results: R[] = [];

    for (const item of items) {
      // Check standard fields
      let matched = false;
      for (const field of searchFields) {
        const value = item[field];
        if (value && typeof value === 'string' && normalizedMatch(value, normalizedQuery)) {
          matched = true;
          break;
        }
      }

      // Check nested fields if not matched yet
      if (!matched && nestedFieldExtractor) {
        const nestedValues = nestedFieldExtractor(item);
        for (const value of nestedValues) {
          if (value && normalizedMatch(value, normalizedQuery)) {
            matched = true;
            break;
          }
        }
      }

      if (matched) {
        results.push(mapToResult(item));
        if (results.length >= maxResults) {
          break; // Early exit once we have enough results
        }
      }
    }

    return results;
  };
}

/**
 * Pre-configured typeahead search for Projects.
 */
export function createProjectTypeaheadSearch(
  extractIdFn: (item: UnknownSurrealThing) => string | null
): (
  projects: Project[],
  searchText: string
) => Array<{ id: string; name: string; name_short: string; number: string }> {
  return createTypeaheadSearch<
    Project,
    { id: string; name: string; name_short: string; number: string }
  >({
    searchFields: ['name', 'name_short'],
    nestedFieldExtractor: p => [p.number?.id || ''],
    mapToResult: p => ({
      id: extractIdFn(p.id) || extractIdFn(p) || '',
      name: p.name || '',
      name_short: p.name_short || '',
      number: p.number?.id || 'No Number'
    })
  });
}

/**
 * Pre-configured typeahead search for Companies.
 */
export function createCompanyTypeaheadSearch(
  extractIdFn: (item: UnknownSurrealThing) => string | null
): (
  companies: Company[],
  searchText: string
) => Array<{ id: string; name: string; name_short: string; abbreviation: string }> {
  return createTypeaheadSearch<
    Company,
    { id: string; name: string; name_short: string; abbreviation: string }
  >({
    searchFields: ['name', 'name_short', 'abbreviation'],
    mapToResult: c => ({
      id: extractIdFn(c.id) || extractIdFn(c) || '',
      name: c.name || '',
      name_short: c.name_short || '',
      abbreviation: c.abbreviation || ''
    })
  });
}

/**
 * Pre-configured typeahead search for Contacts.
 */
export function createContactTypeaheadSearch(
  extractIdFn: (item: UnknownSurrealThing) => string | null
): (
  contacts: Contact[],
  searchText: string
) => Array<{ id: string; name: string; full_name: string; email: string }> {
  return createTypeaheadSearch<
    Contact,
    { id: string; name: string; full_name: string; email: string }
  >({
    searchFields: ['full_name', 'email'],
    mapToResult: c => ({
      id: extractIdFn(c.id) || extractIdFn(c) || '',
      name: c.full_name || '',
      full_name: c.full_name || '',
      email: c.email || ''
    })
  });
}
