/**
 * Search providers for GlobalSearchModal.
 *
 * Pure functions that search entity arrays and return SearchResult[].
 * Extracted from GlobalSearchModal.performSearch() to enable unit testing.
 */

import type { Project, Company, Contact, Fee } from '../../types';
import type { CompanyLookup } from './companyLookup';
import {
  PROJECT_SEARCH_FIELDS,
  COMPANY_SEARCH_FIELDS,
  CONTACT_SEARCH_FIELDS,
  FEE_SEARCH_FIELDS,
  getProjectDisplayInfo,
  getCompanyDisplayInfo,
  getContactDisplayInfo,
  getFeeDisplayInfo,
} from './search';
import { normalizedMatch } from './searchNormalize';
import { extractId } from './index';

export interface SearchResult {
  type: 'project' | 'company' | 'contact' | 'proposal';
  id: string;
  name: string;
  subtitle: string;
  route: string;
}

export function searchProjects(
  projects: Project[],
  query: string,
  max: number
): SearchResult[] {
  return projects
    .filter((p) => {
      const searchFields = PROJECT_SEARCH_FIELDS.map((field) => String(p[field] || ''));
      searchFields.push(String(p.project_number || ''));
      return searchFields.some((f) => normalizedMatch(f, query));
    })
    .slice(0, max)
    .map((p): SearchResult => {
      const displayInfo = getProjectDisplayInfo(p);
      return {
        type: 'project',
        id: extractId(p.id),
        name: displayInfo.name,
        subtitle: displayInfo.subtitle,
        route: '/projects',
      };
    });
}

export function searchCompanies(
  companies: Company[],
  query: string,
  max: number
): SearchResult[] {
  return companies
    .filter((c) => {
      const searchFields = COMPANY_SEARCH_FIELDS.map((field) => String(c[field] || ''));
      return searchFields.some((f) => normalizedMatch(f, query));
    })
    .slice(0, max)
    .map((c): SearchResult => {
      const displayInfo = getCompanyDisplayInfo(c);
      return {
        type: 'company',
        id: extractId(c.id),
        name: displayInfo.name,
        subtitle: displayInfo.subtitle,
        route: '/companies',
      };
    });
}

export function searchContacts(
  contacts: Contact[],
  companyLookup: CompanyLookup,
  query: string,
  max: number
): SearchResult[] {
  const lookups = { companyLookup };
  return contacts
    .filter((c) => {
      const searchFields = CONTACT_SEARCH_FIELDS.map((field) => String(c[field] || ''));
      const companySearchText = companyLookup.getCompanySearchText(c.company);
      return (
        searchFields.some((f) => normalizedMatch(f, query)) ||
        normalizedMatch(companySearchText, query)
      );
    })
    .slice(0, max)
    .map((c): SearchResult => {
      const displayInfo = getContactDisplayInfo(c, lookups);
      return {
        type: 'contact',
        id: extractId(c.id),
        name: displayInfo.name,
        subtitle: displayInfo.subtitle,
        route: '/contacts',
      };
    });
}

export function searchFees(
  fees: Fee[],
  companyLookup: CompanyLookup,
  projectLookup: Map<string, Project>,
  query: string,
  max: number
): SearchResult[] {
  const lookups = { companyLookup, projectLookup };
  return [...fees]
    .sort((a, b) => {
      const dateA = a.time?.updated_at || a.time?.created_at || '';
      const dateB = b.time?.updated_at || b.time?.created_at || '';
      return dateB.localeCompare(dateA);
    })
    .filter((f) => {
      const feeFields = FEE_SEARCH_FIELDS.map((field) => String(f[field] || ''));
      const companySearchText = companyLookup.getCompanySearchText(f.company_id);
      const projectId = extractId(f.project_id);
      const project = projectLookup.get(projectId);
      const projectSearchText = [project?.name, project?.name_short]
        .filter(Boolean)
        .join(' ');
      return (
        feeFields.some((field) => normalizedMatch(field, query)) ||
        normalizedMatch(companySearchText, query) ||
        normalizedMatch(projectSearchText, query)
      );
    })
    .slice(0, max)
    .map((f): SearchResult => {
      const displayInfo = getFeeDisplayInfo(f, lookups);
      return {
        type: 'proposal',
        id: extractId(f.id),
        name: displayInfo.name,
        subtitle: displayInfo.subtitle,
        route: '/proposals',
      };
    });
}
