/**
 * Test Data Fixtures
 * 
 * Centralized test data for consistent testing across all test suites.
 * Includes mock data for projects, companies, contacts, fees, and reference data.
 */

import type { Project, Company, Contact, Fee } from '../../lib/api';

// Local types for test data
interface Country {
  id?: string;
  name: string;
  code: string;
  dial_code: number;
}

interface Currency {
  id?: string;
  code: string;
  name: string;
  symbol?: string;
}

// Test Countries
export const TEST_COUNTRIES: Country[] = [
  {
    id: '1',
    name: 'United Arab Emirates',
    code: 'AE',
    dial_code: 971
  },
  {
    id: '2', 
    name: 'Saudi Arabia',
    code: 'SA',
    dial_code: 966
  },
  {
    id: '3',
    name: 'United Kingdom',
    code: 'GB',
    dial_code: 44
  },
  {
    id: '4',
    name: 'United States',
    code: 'US',
    dial_code: 1
  }
];

// Test Currencies
export const TEST_CURRENCIES: Currency[] = [
  {
    id: '1',
    code: 'AED',
    name: 'UAE Dirham',
    symbol: 'د.إ'
  },
  {
    id: '2',
    code: 'SAR', 
    name: 'Saudi Riyal',
    symbol: '﷼'
  },
  {
    id: '3',
    code: 'GBP',
    name: 'British Pound',
    symbol: '£'
  },
  {
    id: '4',
    code: 'USD',
    name: 'US Dollar',
    symbol: '$'
  }
];

// Test Companies
export const TEST_COMPANIES: Company[] = [
  {
    id: 'company:emittiv123',
    name: 'Emittiv Engineering Consultancy',
    country: 'United Arab Emirates',
    city: 'Dubai',
    address: 'Business Bay, Dubai Marina Tower, Floor 25',
    phone: '+971501234567',
    email: 'info@emittiv.com',
    website: 'https://emittiv.com',
    // created_at: '2025-08-21T09:00:00Z',
    // updated_at: '2025-08-21T09:00:00Z'
  },
  {
    id: 'company:acme456',
    name: 'ACME Corporation',
    country: 'Saudi Arabia',
    city: 'Riyadh',
    address: 'King Fahd Road, Al Olaya District',
    phone: '+966501234567',
    email: 'contact@acme.com',
    website: 'https://acme.com',
    // created_at: '2025-08-21T09:00:00Z',
    // updated_at: '2025-08-21T09:00:00Z'
  },
  {
    id: 'company:tech789',
    name: 'Tech Solutions Ltd',
    country: 'United Kingdom',
    city: 'London',
    address: 'Canary Wharf, 1 Canada Square',
    phone: '+442071234567',
    email: 'hello@techsolutions.co.uk',
    website: 'https://techsolutions.co.uk',
    // created_at: '2025-08-21T09:00:00Z',
    // updated_at: '2025-08-21T09:00:00Z'
  }
];

// Test Contacts
export const TEST_CONTACTS: Contact[] = [
  {
    id: 'contact:john123',
    first_name: 'John',
    last_name: 'Smith',
    full_name: 'John Smith',
    position: 'Project Manager',
    company: 'company:emittiv123',
    phone: '+971501234567',
    email: 'john.smith@emittiv.com',
    // linkedin: 'https://linkedin.com/in/johnsmith',
    // created_at: '2025-08-21T10:00:00Z',
    // updated_at: '2025-08-21T10:00:00Z'
  },
  {
    id: 'contact:jane456',
    first_name: 'Jane',
    last_name: 'Doe',
    full_name: 'Jane Doe',
    position: 'Engineering Director',
    company: 'company:acme456',
    phone: '+966501234567',
    email: 'jane.doe@acme.com',
    // linkedin: 'https://linkedin.com/in/janedoe',
    // created_at: '2025-08-21T10:00:00Z',
    // updated_at: '2025-08-21T10:00:00Z'
  },
  {
    id: 'contact:mike789',
    first_name: 'Mike',
    last_name: 'Johnson',
    full_name: 'Mike Johnson',
    position: 'Senior Engineer',
    company: 'company:tech789',
    phone: '+442071234567',
    email: 'mike.johnson@techsolutions.co.uk',
    // linkedin: 'https://linkedin.com/in/mikejohnson',
    // created_at: '2025-08-21T10:00:00Z',
    // updated_at: '2025-08-21T10:00:00Z'
  }
];

// Test Projects
export const TEST_PROJECTS: Project[] = [
  {
    id: 'projects:25-97101',
    project_number: '25-97101',
    name: 'Dubai Marina Tower Complex',
    country: 'United Arab Emirates',
    city: 'Dubai',
    client_company: 'Emittiv Engineering Consultancy',
    status: 'active',
    // created_at: '2025-08-21T09:00:00Z',
    // updated_at: '2025-08-21T09:00:00Z'
  },
  {
    id: 'projects:25-96601',
    project_number: '25-96601',
    name: 'Riyadh Business Complex',
    country: 'Saudi Arabia', 
    city: 'Riyadh',
    client_company: 'ACME Corporation',
    status: 'active',
    // created_at: '2025-08-21T09:00:00Z',
    // updated_at: '2025-08-21T09:00:00Z'
  },
  {
    id: 'projects:25-44101',
    project_number: '25-44101',
    name: 'London Office Development',
    country: 'United Kingdom',
    city: 'London',
    client_company: 'Tech Solutions Ltd',
    status: 'Draft',
    // created_at: '2025-08-21T09:00:00Z',
    // updated_at: '2025-08-21T09:00:00Z'
  },
  {
    id: 'projects:24-97199',
    project_number: '24-97199',
    name: 'Legacy Project - Completed',
    country: 'United Arab Emirates',
    city: 'Abu Dhabi',
    client_company: 'Previous Client',
    status: 'Completed',
    // created_at: '2024-12-15T09:00:00Z',
    // updated_at: '2024-12-20T09:00:00Z'
  }
];

// Test Fees
export const TEST_FEES: Fee[] = [
  {
    id: 'rfp:prop123',
    project_id: 'projects:25-97101',
    company: 'company:emittiv123', 
    contact_id: 'contact:john123',
    rfp_number: 'RFP-25-97101-2025',
    issue_date: '2025-08-20',
    status: 'draft',
    stage: 'initial',
    package: 'structural',
    staff_name: 'Martin Engineer',
    activity: 'Structural Design and Analysis for High-rise Building',
    // created_at: '2025-08-21T10:00:00Z',
    // updated_at: '2025-08-21T10:00:00Z'
  },
  {
    id: 'rfp:prop456',
    project_id: 'projects:25-96601',
    company: 'company:acme456',
    contact_id: 'contact:jane456',
    rfp_number: 'RFP-25-96601-2025',
    issue_date: '2025-08-21',
    status: 'submitted',
    stage: 'concept',
    package: 'full_package',
    staff_name: 'Sarah Architect',
    activity: 'Complete Architectural and Engineering Package',
    // created_at: '2025-08-21T10:00:00Z',
    // updated_at: '2025-08-21T11:00:00Z'
  },
  {
    id: 'rfp:prop789',
    project_id: 'projects:25-44101',
    company: 'company:tech789',
    contact_id: 'contact:mike789',
    rfp_number: 'RFP-25-44101-2025',
    issue_date: '2025-08-19',
    status: 'under_review',
    stage: 'design_development',
    package: 'mep',
    staff_name: 'David MEP Engineer',
    activity: 'MEP Design and Coordination',
    // created_at: '2025-08-21T10:00:00Z',
    // updated_at: '2025-08-21T12:00:00Z'
  }
];

// Connection Status Mock Data
export const TEST_CONNECTION_STATUS = {
  connected: true,
  database_url: 'ws://10.0.1.17:8000',
  namespace: 'emittiv',
  database: 'projects',
  username: 'martin',
  response_time_ms: 45,
  last_check: '2025-08-21T10:30:00Z'
};

// Status/Stage/Package Enums for Testing
export const TEST_ENUMS = {
  fee_statuses: ['draft', 'submitted', 'under_review', 'approved', 'rejected'],
  fee_stages: ['initial', 'concept', 'design_development', 'construction_documentation', 'construction_administration'],
  fee_packages: ['structural', 'architectural', 'mep', 'civil', 'full_package'],
  project_statuses: ['planning', 'active', 'on_hold', 'completed', 'cancelled']
};

// Helper functions for test data manipulation
export const TestDataHelpers = {
  // Get company by ID
  getCompanyById: (id: string): Company | undefined => {
    return TEST_COMPANIES.find(company => company.id === id);
  },

  // Get contacts by company ID
  getContactsByCompanyId: (companyId: string): Contact[] => {
    return TEST_CONTACTS.filter(contact => contact.company === companyId);
  },

  // Get project by number
  getProjectByNumber: (projectNumber: string): Project | undefined => {
    return TEST_PROJECTS.find(project => project.project_number === projectNumber);
  },

  // Get fees by project ID
  getFeesByProjectId: (projectId: string): Fee[] => {
    return TEST_FEES.filter(fee => fee.project_id === projectId);
  },

  // Create test project with custom data
  createTestProject: (overrides: Partial<Project> = {}): Project => ({
    id: `projects:test-${Date.now()}`,
    project_number: `25-97${Math.floor(Math.random() * 1000).toString().padStart(3, '0')}`,
    name: 'Test Project',
    country: 'United Arab Emirates',
    city: 'Dubai',
    client_company: 'Test Company',
    status: 'active',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    ...overrides
  }),

  // Create test company with custom data
  createTestCompany: (overrides: Partial<Company> = {}): Company => ({
    id: `company:test-${Date.now()}`,
    name: 'Test Company',
    country: 'United Arab Emirates',
    city: 'Dubai',
    address: 'Test Address',
    phone: '+971501234567',
    email: 'test@company.com',
    website: 'https://testcompany.com',
    ...overrides
  }),

  // Create test contact with custom data
  createTestContact: (overrides: Partial<Contact> = {}): Contact => ({
    id: `contact:test-${Date.now()}`,
    first_name: 'Test',
    last_name: 'User',
    full_name: 'Test User',
    position: 'Test Title',
    company: 'company:emittiv123',
    phone: '+971501234567',
    email: 'test.user@company.com',
    ...overrides
  }),

  // Create test fee with custom data
  createTestFee: (overrides: Partial<Fee> = {}): Fee => ({
    id: `rfp:test-${Date.now()}`,
    project_id: 'projects:25-97101',
    company: 'company:emittiv123',
    contact_id: 'contact:john123',
    rfp_number: `RFP-TEST-${Date.now()}`,
    issue_date: new Date().toISOString().split('T')[0],
    status: 'draft',
    stage: 'initial',
    package: 'structural',
    staff_name: 'Test Engineer',
    activity: 'Test Activity',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    ...overrides
  }),

  // Simulate SurrealDB Thing object format
  withThingId: <T extends { id: string }>(item: T, table: string): T & { id: { tb: string; id: string } } => ({
    ...item,
    id: {
      tb: table,
      id: item.id.split(':')[1] || item.id
    } as any
  }),

  // Extract ID from Thing object (reverse of above)
  extractThingId: (thingId: { tb: string; id: string } | string): string => {
    if (typeof thingId === 'string') return thingId;
    return `${thingId.tb}:${thingId.id}`;
  }
};

// Error scenarios for testing
export const TEST_ERRORS = {
  networkError: new Error('Network unreachable'),
  timeoutError: new Error('Connection timeout'),
  authError: new Error('Authentication failed'),
  validationError: new Error('Validation failed'),
  constraintError: new Error('Database constraint violation'),
  notFoundError: new Error('Record not found'),
  permissionError: new Error('Insufficient permissions')
};

// Mock API responses for different scenarios
export const TEST_API_RESPONSES = {
  success: {
    projects: TEST_PROJECTS,
    companies: TEST_COMPANIES,
    contacts: TEST_CONTACTS,
    fees: TEST_FEES,
    countries: TEST_COUNTRIES,
    currencies: TEST_CURRENCIES,
    connectionStatus: TEST_CONNECTION_STATUS
  },
  empty: {
    projects: [],
    companies: [],
    contacts: [],
    fees: [],
    countries: [],
    currencies: []
  },
  errors: TEST_ERRORS
};

export default {
  TEST_COUNTRIES,
  TEST_CURRENCIES,
  TEST_COMPANIES,
  TEST_CONTACTS,
  TEST_PROJECTS,
  TEST_FEES,
  TEST_CONNECTION_STATUS,
  TEST_ENUMS,
  TestDataHelpers,
  TEST_ERRORS,
  TEST_API_RESPONSES
};