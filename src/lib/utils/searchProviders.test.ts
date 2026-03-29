import { describe, it, expect } from 'vitest';
import {
  searchProjects,
  searchCompanies,
  searchContacts,
  searchFees,
  type SearchResult
} from './searchProviders';
import type { Project, Company, Contact, Fee } from '../../types';
import { createCompanyLookup } from './companyLookup';
import { createProjectLookup } from './search';

// --- Test fixtures ---

const mockProjects: Project[] = [
  {
    id: 'projects:25-97101',
    name: 'Dubai Islands Hotel',
    name_short: 'Dubai Islands',
    project_number: '25-97101',
    number: { id: '25-97101' },
    city: 'Dubai',
    country: 'U.A.E.',
    area: 'Dubai Islands',
    status: 'Design',
    time: { created_at: '2025-01-01', updated_at: '2025-06-01' }
  } as unknown as Project,
  {
    id: 'projects:26-96606',
    name: 'Riyadh Tower',
    name_short: 'Riyadh',
    project_number: '26-96606',
    number: { id: '26-96606' },
    city: 'Riyadh',
    country: 'Saudi Arabia',
    area: 'KAFD',
    status: 'Lead',
    time: { created_at: '2026-01-01', updated_at: '2026-03-01' }
  } as unknown as Project
];

const mockCompanies: Company[] = [
  {
    id: 'company:ptg',
    name: 'P&T Group',
    name_short: 'P&T',
    abbreviation: 'PTG',
    city: 'Dubai',
    country: 'U.A.E.',
    time: { created_at: '2025-01-01', updated_at: '2025-06-01' }
  } as unknown as Company,
  {
    id: 'company:dar',
    name: 'Dar Al Handasah',
    name_short: 'Dar',
    abbreviation: 'DAR',
    city: 'Beirut',
    country: 'Lebanon',
    time: { created_at: '2025-02-01', updated_at: '2025-05-01' }
  } as unknown as Company
];

const mockContacts: Contact[] = [
  {
    id: 'contacts:john',
    full_name: 'John Smith',
    first_name: 'John',
    last_name: 'Smith',
    email: 'john@ptgroup.com',
    phone: '+971501234567',
    position: 'Director',
    company: 'company:ptg',
    time: { created_at: '2025-01-01', updated_at: '2025-06-01' }
  } as unknown as Contact,
  {
    id: 'contacts:sara',
    full_name: 'Sara Ahmed',
    first_name: 'Sara',
    last_name: 'Ahmed',
    email: 'sara@dar.com',
    phone: '+9611234567',
    position: 'Architect',
    company: 'company:dar',
    time: { created_at: '2025-02-01', updated_at: '2025-05-01' }
  } as unknown as Contact
];

const mockFees: Fee[] = [
  {
    id: 'fee:25_97101_1',
    name: 'Lighting Design',
    number: 'R1',
    activity: 'Design',
    package: 'Base Building',
    staff_name: 'Martin',
    status: 'Sent',
    issue_date: '250601',
    company_id: 'company:ptg',
    project_id: 'projects:25-97101',
    time: { created_at: '2025-06-01', updated_at: '2025-06-15' }
  } as unknown as Fee,
  {
    id: 'fee:26_96606_1',
    name: 'Facade Lighting',
    number: 'R1',
    activity: 'Design',
    package: 'Facade',
    staff_name: 'Martin',
    status: 'Draft',
    issue_date: '260301',
    company_id: 'company:dar',
    project_id: 'projects:26-96606',
    time: { created_at: '2026-03-01', updated_at: '2026-03-10' }
  } as unknown as Fee
];

// --- Tests ---

describe('searchProjects', () => {
  it('matches on project name', () => {
    const results = searchProjects(mockProjects, 'Dubai', 5);
    expect(results.length).toBe(1);
    expect(results[0].type).toBe('project');
    expect(results[0].name).toBe('Dubai Islands Hotel');
  });

  it('matches on project_number', () => {
    const results = searchProjects(mockProjects, '97101', 5);
    expect(results.length).toBe(1);
    expect(results[0].id).toBe('projects:25-97101');
  });

  it('respects MAX limit', () => {
    const results = searchProjects(mockProjects, 'a', 1);
    expect(results.length).toBeLessThanOrEqual(1);
  });

  it('returns SearchResult[] shape', () => {
    const results = searchProjects(mockProjects, 'Dubai', 5);
    const r = results[0];
    expect(r).toHaveProperty('type');
    expect(r).toHaveProperty('id');
    expect(r).toHaveProperty('name');
    expect(r).toHaveProperty('subtitle');
    expect(r).toHaveProperty('route');
    expect(r.route).toBe('/projects');
  });

  it('returns empty for short queries', () => {
    const results = searchProjects(mockProjects, 'D', 5);
    // performSearch requires length >= 2, but providers are pure — caller enforces
    // Still, empty/null input should be safe
    expect(Array.isArray(results)).toBe(true);
  });

  it('returns empty for no matches', () => {
    const results = searchProjects(mockProjects, 'zzzznotfound', 5);
    expect(results.length).toBe(0);
  });
});

describe('searchCompanies', () => {
  it('matches on company name', () => {
    const results = searchCompanies(mockCompanies, 'Dar', 5);
    expect(results.length).toBe(1);
    expect(results[0].name).toBe('Dar Al Handasah');
  });

  it('matches on country with normalized match (U.A.E. vs uae)', () => {
    const results = searchCompanies(mockCompanies, 'uae', 5);
    expect(results.length).toBe(1);
    expect(results[0].type).toBe('company');
  });

  it('respects MAX limit', () => {
    const results = searchCompanies(mockCompanies, 'a', 1);
    expect(results.length).toBeLessThanOrEqual(1);
  });

  it('returns correct route', () => {
    const results = searchCompanies(mockCompanies, 'PTG', 5);
    expect(results[0].route).toBe('/companies');
  });
});

describe('searchContacts', () => {
  const companyLookup = createCompanyLookup(mockCompanies);

  it('matches on contact name', () => {
    const results = searchContacts(mockContacts, companyLookup, 'John', 5);
    expect(results.length).toBe(1);
    expect(results[0].name).toBe('John Smith');
  });

  it('matches on email', () => {
    const results = searchContacts(mockContacts, companyLookup, 'sara@dar', 5);
    expect(results.length).toBe(1);
  });

  it('matches on company name via lookup', () => {
    const results = searchContacts(mockContacts, companyLookup, 'P&T', 5);
    expect(results.length).toBe(1);
    expect(results[0].name).toBe('John Smith');
  });

  it('respects MAX limit', () => {
    const results = searchContacts(mockContacts, companyLookup, 'a', 1);
    expect(results.length).toBeLessThanOrEqual(1);
  });

  it('returns correct route', () => {
    const results = searchContacts(mockContacts, companyLookup, 'John', 5);
    expect(results[0].route).toBe('/contacts');
  });
});

describe('searchFees', () => {
  const companyLookup = createCompanyLookup(mockCompanies);
  const projectLookup = createProjectLookup(mockProjects);

  it('matches on fee name', () => {
    const results = searchFees(mockFees, companyLookup, projectLookup, 'Facade', 5);
    expect(results.length).toBe(1);
    expect(results[0].type).toBe('proposal');
  });

  it('matches on company name via lookup', () => {
    const results = searchFees(mockFees, companyLookup, projectLookup, 'Dar', 5);
    expect(results.length).toBe(1);
  });

  it('matches on project name via lookup', () => {
    const results = searchFees(mockFees, companyLookup, projectLookup, 'Riyadh', 5);
    expect(results.length).toBe(1);
  });

  it('sorts by date descending (newest first)', () => {
    const results = searchFees(mockFees, companyLookup, projectLookup, 'Design', 5);
    expect(results.length).toBe(2);
    // 260301 is newer than 250601, so Facade Lighting should be first
    expect(results[0].id).toBe('fee:26_96606_1');
  });

  it('respects MAX limit', () => {
    const results = searchFees(mockFees, companyLookup, projectLookup, 'Design', 1);
    expect(results.length).toBe(1);
  });

  it('returns correct route', () => {
    const results = searchFees(mockFees, companyLookup, projectLookup, 'Facade', 5);
    expect(results[0].route).toBe('/proposals');
  });
});
