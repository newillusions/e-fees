import { invoke } from '@tauri-apps/api/core';
import type { 
  Project, 
  Company, 
  Contact, 
  Proposal, 
  ConnectionStatus 
} from '../types';

// API wrapper for Tauri commands
export class ApiClient {
  
  // Database connection methods
  static async checkDbConnection(): Promise<boolean> {
    try {
      return await invoke<boolean>('check_db_connection');
    } catch (error) {
      console.error('Failed to check database connection:', error);
      return false;
    }
  }

  static async getConnectionStatus(): Promise<ConnectionStatus> {
    try {
      return await invoke<ConnectionStatus>('get_connection_status');
    } catch (error) {
      console.error('Failed to get connection status:', error);
      return {
        is_connected: false,
        last_check: null,
        error_message: error?.toString() || 'Unknown error'
      };
    }
  }

  // Project methods
  static async getProjects(): Promise<Project[]> {
    try {
      const projects = await invoke<Project[]>('get_projects');
      return projects.map(project => ({
        ...project,
        createdAt: new Date(project.created_at),
        updatedAt: new Date(project.updated_at)
      }));
    } catch (error) {
      console.warn('Failed to fetch projects from database, using fallback data:', error);
      // Return mock data as fallback
      const { mockProjects } = await import('./stores/data');
      return mockProjects;
    }
  }

  static async createProject(project: Omit<Project, 'id' | 'createdAt' | 'updatedAt'>): Promise<Project | null> {
    try {
      const newProject = {
        id: null,
        name: project.name,
        description: project.description,
        status: project.status,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      };
      
      const created = await invoke<Project>('create_project', { project: newProject });
      return {
        ...created,
        createdAt: new Date(created.created_at),
        updatedAt: new Date(created.updated_at)
      };
    } catch (error) {
      console.error('Failed to create project:', error);
      return null;
    }
  }

  // Company methods
  static async getCompanies(): Promise<Company[]> {
    try {
      const companies = await invoke<Company[]>('get_companies');
      return companies.map(company => ({
        ...company,
        createdAt: new Date(company.created_at)
      }));
    } catch (error) {
      console.warn('Failed to fetch companies from database, using fallback data:', error);
      // Return mock data as fallback
      const { mockCompanies } = await import('./stores/data');
      return mockCompanies;
    }
  }

  static async createCompany(company: Omit<Company, 'id' | 'createdAt'>): Promise<Company | null> {
    try {
      const newCompany = {
        id: null,
        name: company.name,
        industry: company.industry,
        contact_count: company.contactCount,
        address: company.address || null,
        website: company.website || null,
        created_at: new Date().toISOString()
      };
      
      const created = await invoke<Company>('create_company', { company: newCompany });
      return {
        ...created,
        contactCount: created.contact_count,
        createdAt: new Date(created.created_at)
      };
    } catch (error) {
      console.error('Failed to create company:', error);
      return null;
    }
  }

  // Contact methods
  static async getContacts(): Promise<Contact[]> {
    try {
      const contacts = await invoke<Contact[]>('get_contacts');
      return contacts.map(contact => ({
        ...contact,
        companyId: contact.company_id,
        createdAt: new Date(contact.created_at)
      }));
    } catch (error) {
      console.warn('Failed to fetch contacts from database, using fallback data:', error);
      // Return mock data as fallback
      const { mockContacts } = await import('./stores/data');
      return mockContacts;
    }
  }

  static async createContact(contact: Omit<Contact, 'id' | 'createdAt'>): Promise<Contact | null> {
    try {
      const newContact = {
        id: null,
        name: contact.name,
        email: contact.email,
        phone: contact.phone || null,
        position: contact.position || null,
        company_id: contact.companyId,
        company: contact.company,
        created_at: new Date().toISOString()
      };
      
      const created = await invoke<Contact>('create_contact', { contact: newContact });
      return {
        ...created,
        companyId: created.company_id,
        createdAt: new Date(created.created_at)
      };
    } catch (error) {
      console.error('Failed to create contact:', error);
      return null;
    }
  }

  // Proposal methods
  static async getProposals(): Promise<Proposal[]> {
    try {
      const proposals = await invoke<Proposal[]>('get_proposals');
      return proposals.map(proposal => ({
        ...proposal,
        projectId: proposal.project_id,
        createdAt: new Date(proposal.created_at),
        dueDate: proposal.due_date ? new Date(proposal.due_date) : undefined
      }));
    } catch (error) {
      console.warn('Failed to fetch proposals from database, using fallback data:', error);
      // Return mock data as fallback
      const { mockProposals } = await import('./stores/data');
      return mockProposals;
    }
  }

  static async createProposal(proposal: Omit<Proposal, 'id' | 'createdAt'>): Promise<Proposal | null> {
    try {
      const newProposal = {
        id: null,
        title: proposal.title,
        client: proposal.client,
        amount: proposal.amount,
        status: proposal.status,
        project_id: proposal.projectId || null,
        created_at: new Date().toISOString(),
        due_date: proposal.dueDate ? proposal.dueDate.toISOString() : null
      };
      
      const created = await invoke<Proposal>('create_proposal', { proposal: newProposal });
      return {
        ...created,
        projectId: created.project_id,
        createdAt: new Date(created.created_at),
        dueDate: created.due_date ? new Date(created.due_date) : undefined
      };
    } catch (error) {
      console.error('Failed to create proposal:', error);
      return null;
    }
  }

  // Statistics
  static async getStats(): Promise<{
    totalProjects: number;
    activeProposals: number;
    totalCompanies: number;
    totalContacts: number;
    totalProposals: number;
  }> {
    try {
      return await invoke('get_stats');
    } catch (error) {
      console.warn('Failed to fetch stats from database, using fallback data:', error);
      // Calculate stats from mock data as fallback
      const { mockProjects, mockProposals, mockCompanies, mockContacts } = await import('./stores/data');
      return {
        totalProjects: mockProjects.length,
        activeProposals: mockProposals.filter(p => p.status !== 'rejected').length,
        totalCompanies: mockCompanies.length,
        totalContacts: mockContacts.length,
        totalProposals: mockProposals.length
      };
    }
  }

  // Health check
  static async healthCheck(): Promise<string> {
    try {
      return await invoke<string>('health_check');
    } catch (error) {
      console.error('Health check failed:', error);
      return 'Health check failed';
    }
  }

  // Get detailed database connection info for debugging
  static async getDbInfo(): Promise<any> {
    try {
      return await invoke('get_db_info');
    } catch (error) {
      console.error('Failed to get database info:', error);
      return {
        error: error?.toString() || 'Unknown error',
        timestamp: new Date().toISOString()
      };
    }
  }
}

// Export individual functions for convenience
export const {
  checkDbConnection,
  getConnectionStatus,
  getProjects,
  createProject,
  getCompanies,
  createCompany,
  getContacts,
  createContact,
  getProposals,
  createProposal,
  getStats,
  healthCheck,
  getDbInfo
} = ApiClient;

// Export default
export default ApiClient;