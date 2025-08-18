import { writable } from 'svelte/store';
import type { Project, Fee, Company, Contact } from '../../types';

// Mock data
const mockProjects: Project[] = [
  {
    id: '1',
    name: 'Green Tower Development',
    name_short: 'Green Tower',
    status: 'Active',
    area: 'Commercial',
    city: 'Dubai',
    country: 'UAE',
    folder: 'GT-2024',
    number: {
      year: 2024,
      country: 971,
      seq: 101,
      id: '24-971101'
    },
    time: {
      created_at: '2024-01-15T00:00:00Z',
      updated_at: '2024-01-20T00:00:00Z'
    }
  },
  {
    id: '2',
    name: 'Sustainable Residential Complex',
    name_short: 'SRC',
    status: 'RFP',
    area: 'Residential',
    city: 'Abu Dhabi',
    country: 'UAE',
    folder: 'SRC-2024',
    number: {
      year: 2024,
      country: 971,
      seq: 102,
      id: '24-971102'
    },
    time: {
      created_at: '2024-02-01T00:00:00Z',
      updated_at: '2024-02-15T00:00:00Z'
    }
  }
];

const mockFees: Fee[] = [
  {
    id: '1',
    name: 'Green Tower MEP Design',
    number: 'RFP-24-001',
    rev: 1,
    status: 'Sent',
    issue_date: '240115',
    activity: 'Design and Consultancy',
    package: 'MEP Design Package',
    project_id: '1',
    company_id: '1',
    contact_id: '1',
    staff_name: 'Martin Smith',
    staff_email: 'martin@emittiv.com',
    staff_phone: '+971501234567',
    staff_position: 'Senior Engineer',
    strap_line: 'Sustainable MEP solutions for modern buildings',
    revisions: [
      {
        revision_number: 1,
        revision_date: '240115',
        author_email: 'martin@emittiv.com',
        author_name: 'Martin Smith',
        notes: 'Initial proposal submission'
      }
    ],
    time: {
      created_at: '2024-01-10T00:00:00Z',
      updated_at: '2024-01-15T00:00:00Z'
    }
  },
  {
    id: '2',
    name: 'SRC Construction Management',
    number: 'RFP-24-002',
    rev: 2,
    status: 'Draft',
    issue_date: '240128',
    activity: 'Management',
    package: 'Construction Management',
    project_id: '2',
    company_id: '2',
    contact_id: '2',
    staff_name: 'Sarah Johnson',
    staff_email: 'sarah@emittiv.com',
    staff_phone: '+971509876543',
    staff_position: 'Project Manager',
    strap_line: 'Comprehensive construction management services',
    revisions: [
      {
        revision_number: 1,
        revision_date: '240128',
        author_email: 'sarah@emittiv.com',
        author_name: 'Sarah Johnson',
        notes: 'Draft proposal created'
      },
      {
        revision_number: 2,
        revision_date: '240130',
        author_email: 'sarah@emittiv.com',
        author_name: 'Sarah Johnson',
        notes: 'Updated scope and pricing'
      }
    ],
    time: {
      created_at: '2024-01-28T00:00:00Z',
      updated_at: '2024-01-30T00:00:00Z'
    }
  }
];

const mockCompanies: Company[] = [
  {
    id: '1',
    name: 'Emaar Properties PJSC',
    name_short: 'Emaar',
    abbreviation: 'EMAAR',
    city: 'Dubai',
    country: 'UAE',
    reg_no: '1234567890',
    tax_no: '100123456700003',
    time: {
      created_at: '2024-01-15T08:30:00Z',
      updated_at: '2024-01-15T08:30:00Z'
    }
  },
  {
    id: '2',
    name: 'Dubai Holding Company LLC',
    name_short: 'Dubai Holding',
    abbreviation: 'DH',
    city: 'Dubai',
    country: 'UAE',
    reg_no: '0987654321',
    tax_no: '100987654300003',
    time: {
      created_at: '2024-02-10T10:15:00Z',
      updated_at: '2024-02-10T10:15:00Z'
    }
  }
];

const mockContacts: Contact[] = [
  {
    id: '1',
    first_name: 'Ahmed',
    last_name: 'Al-Rashid',
    full_name: 'Ahmed Al-Rashid',
    email: 'ahmed.alrashid@emaar.com',
    phone: '+971501234567',
    position: 'Senior Project Manager',
    company: '1',
    time: {
      created_at: '2024-01-20T09:00:00Z',
      updated_at: '2024-01-20T09:00:00Z'
    }
  },
  {
    id: '2',
    first_name: 'Fatima',
    last_name: 'Hassan',
    full_name: 'Fatima Hassan',
    email: 'fatima.hassan@dubaiholding.com',
    phone: '+971509876543',
    position: 'Technical Director',
    company: '2',
    time: {
      created_at: '2024-02-15T11:30:00Z',
      updated_at: '2024-02-15T11:30:00Z'
    }
  }
];

// Export mock data for fallback use
export { mockProjects, mockFees, mockCompanies, mockContacts };


// Stores - initialize with empty arrays, not mock data
export const projects = writable<Project[]>([]);
export const fees = writable<Fee[]>([]);
export const companies = writable<Company[]>([]);
export const contacts = writable<Contact[]>([]);

// Alternative export names for consistency
export const projectStore = projects;
export const feeStore = fees;
export const companyStore = companies;
export const contactStore = contacts;

// Stats computed from data - start with zeros
export const stats = writable({
  totalProjects: 0,
  activeFees: 0,
  totalCompanies: 0,
  totalContacts: 0,
  totalFees: 0
});