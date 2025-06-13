import { writable } from 'svelte/store';
import type { Project, Proposal, Company, Contact } from '../../types';

// Mock data
const mockProjects: Project[] = [
  {
    id: '1',
    name: 'Website Redesign',
    description: 'Complete overhaul of corporate website with modern design',
    status: 'active',
    createdAt: new Date('2024-01-15'),
    updatedAt: new Date('2024-01-20')
  },
  {
    id: '2',
    name: 'Mobile App Development',
    description: 'Native iOS and Android app for customer engagement',
    status: 'active',
    createdAt: new Date('2024-02-01'),
    updatedAt: new Date('2024-02-15')
  }
];

const mockProposals: Proposal[] = [
  {
    id: '1',
    title: 'Website Redesign Proposal',
    client: 'Tech Corp Inc.',
    amount: 15000,
    status: 'sent',
    projectId: '1',
    createdAt: new Date('2024-01-10'),
    dueDate: new Date('2024-02-10')
  },
  {
    id: '2',
    title: 'Mobile App Development Quote',
    client: 'StartupXYZ',
    amount: 25000,
    status: 'draft',
    projectId: '2',
    createdAt: new Date('2024-01-28'),
    dueDate: new Date('2024-03-01')
  }
];

const mockCompanies: Company[] = [
  {
    id: '1',
    name: 'Tech Corp Inc.',
    industry: 'Technology',
    contactCount: 3,
    website: 'https://techcorp.com',
    createdAt: new Date('2023-12-01')
  },
  {
    id: '2',
    name: 'StartupXYZ',
    industry: 'Fintech',
    contactCount: 2,
    website: 'https://startupxyz.com',
    createdAt: new Date('2024-01-15')
  }
];

const mockContacts: Contact[] = [
  {
    id: '1',
    name: 'John Smith',
    email: 'john.smith@techcorp.com',
    phone: '+1 (555) 123-4567',
    position: 'CTO',
    companyId: '1',
    company: 'Tech Corp Inc.',
    createdAt: new Date('2023-12-05')
  },
  {
    id: '2',
    name: 'Sarah Johnson',
    email: 'sarah@startupxyz.com',
    phone: '+1 (555) 987-6543',
    position: 'CEO',
    companyId: '2',
    company: 'StartupXYZ',
    createdAt: new Date('2024-01-20')
  }
];

// Export mock data for fallback use
export { mockProjects, mockProposals, mockCompanies, mockContacts };

// Stores
export const projects = writable<Project[]>(mockProjects);
export const proposals = writable<Proposal[]>(mockProposals);
export const companies = writable<Company[]>(mockCompanies);
export const contacts = writable<Contact[]>(mockContacts);

// Stats computed from data
export const stats = writable({
  totalProjects: mockProjects.length,
  activeProposals: mockProposals.filter(p => p.status !== 'rejected').length,
  totalCompanies: mockCompanies.length,
  totalContacts: mockContacts.length
});