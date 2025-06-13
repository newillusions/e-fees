import { writable, derived, get } from 'svelte/store';
import type { 
  Project, 
  Company, 
  Contact, 
  Proposal, 
  ConnectionStatus 
} from '../types';
import { 
  getProjects,
  getCompanies,
  getContacts,
  getProposals,
  getStats,
  createProject,
  createCompany,
  createContact,
  createProposal
} from './api';

// ============================================================================
// CONNECTION STORE
// ============================================================================

export interface ConnectionState {
  isConnected: boolean;
  status: string;
  lastChecked?: Date;
  errorMessage?: string;
}

const initialConnectionState: ConnectionState = {
  isConnected: false,
  status: 'Disconnected'
};

export const connectionStore = writable<ConnectionState>(initialConnectionState);

// ============================================================================
// DATA STORES
// ============================================================================

// Projects store
export const projectsStore = writable<Project[]>([]);
export const projectsLoading = writable<boolean>(false);
export const projectsError = writable<string | null>(null);

// Companies store
export const companiesStore = writable<Company[]>([]);
export const companiesLoading = writable<boolean>(false);
export const companiesError = writable<string | null>(null);

// Contacts store
export const contactsStore = writable<Contact[]>([]);
export const contactsLoading = writable<boolean>(false);
export const contactsError = writable<string | null>(null);

// Proposals store
export const proposalsStore = writable<Proposal[]>([]);
export const proposalsLoading = writable<boolean>(false);
export const proposalsError = writable<string | null>(null);

// ============================================================================
// DERIVED STORES (COMPUTED VALUES)
// ============================================================================

// Statistics derived from data
export const statisticsStore = derived(
  [projectsStore, proposalsStore, companiesStore, contactsStore],
  ([projects, proposals, companies, contacts]) => ({
    totalProjects: projects.length,
    activeProposals: proposals.filter(p => p.status !== 'rejected').length,
    totalCompanies: companies.length,
    totalContacts: contacts.length,
    totalProposals: proposals.length
  })
);

// Active projects only
export const activeProjectsStore = derived(
  projectsStore,
  $projects => $projects.filter(project => project.status === 'active')
);

// Recent proposals (last 30 days)
export const recentProposalsStore = derived(
  proposalsStore,
  $proposals => {
    const thirtyDaysAgo = new Date();
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);
    return $proposals.filter(proposal => proposal.createdAt > thirtyDaysAgo);
  }
);

// Companies with contact counts
export const companiesWithContactsStore = derived(
  [companiesStore, contactsStore],
  ([companies, contacts]) => {
    return companies.map(company => ({
      ...company,
      contactCount: contacts.filter(contact => contact.companyId === company.id).length
    }));
  }
);

// Loading state for any data operation
export const isLoadingStore = derived(
  [projectsLoading, companiesLoading, contactsLoading, proposalsLoading],
  ([projects, companies, contacts, proposals]) => 
    projects || companies || contacts || proposals
);

// Global error state
export const globalErrorStore = derived(
  [projectsError, companiesError, contactsError, proposalsError],
  ([projectsErr, companiesErr, contactsErr, proposalsErr]) => 
    projectsErr || companiesErr || contactsErr || proposalsErr
);

// ============================================================================
// ACTION CREATORS (ASYNC OPERATIONS)
// ============================================================================

// Projects actions
export const projectsActions = {
  async load() {
    projectsLoading.set(true);
    projectsError.set(null);
    
    try {
      const projects = await getProjects();
      projectsStore.set(projects);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load projects';
      projectsError.set(errorMessage);
      console.error('Failed to load projects:', error);
    } finally {
      projectsLoading.set(false);
    }
  },

  async create(project: Omit<Project, 'id' | 'createdAt' | 'updatedAt'>) {
    try {
      const newProject = await createProject(project);
      if (newProject) {
        projectsStore.update(projects => [...projects, newProject]);
        return newProject;
      }
      throw new Error('Failed to create project');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to create project';
      projectsError.set(errorMessage);
      console.error('Failed to create project:', error);
      throw error;
    }
  },

  async refresh() {
    await this.load();
  }
};

// Companies actions
export const companiesActions = {
  async load() {
    companiesLoading.set(true);
    companiesError.set(null);
    
    try {
      const companies = await getCompanies();
      companiesStore.set(companies);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load companies';
      companiesError.set(errorMessage);
      console.error('Failed to load companies:', error);
    } finally {
      companiesLoading.set(false);
    }
  },

  async create(company: Omit<Company, 'id' | 'createdAt'>) {
    try {
      const newCompany = await createCompany(company);
      if (newCompany) {
        companiesStore.update(companies => [...companies, newCompany]);
        return newCompany;
      }
      throw new Error('Failed to create company');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to create company';
      companiesError.set(errorMessage);
      console.error('Failed to create company:', error);
      throw error;
    }
  },

  async refresh() {
    await this.load();
  }
};

// Contacts actions
export const contactsActions = {
  async load() {
    contactsLoading.set(true);
    contactsError.set(null);
    
    try {
      const contacts = await getContacts();
      contactsStore.set(contacts);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load contacts';
      contactsError.set(errorMessage);
      console.error('Failed to load contacts:', error);
    } finally {
      contactsLoading.set(false);
    }
  },

  async create(contact: Omit<Contact, 'id' | 'createdAt'>) {
    try {
      const newContact = await createContact(contact);
      if (newContact) {
        contactsStore.update(contacts => [...contacts, newContact]);
        return newContact;
      }
      throw new Error('Failed to create contact');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to create contact';
      contactsError.set(errorMessage);
      console.error('Failed to create contact:', error);
      throw error;
    }
  },

  async refresh() {
    await this.load();
  }
};

// Proposals actions
export const proposalsActions = {
  async load() {
    proposalsLoading.set(true);
    proposalsError.set(null);
    
    try {
      const proposals = await getProposals();
      proposalsStore.set(proposals);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load proposals';
      proposalsError.set(errorMessage);
      console.error('Failed to load proposals:', error);
    } finally {
      proposalsLoading.set(false);
    }
  },

  async create(proposal: Omit<Proposal, 'id' | 'createdAt'>) {
    try {
      const newProposal = await createProposal(proposal);
      if (newProposal) {
        proposalsStore.update(proposals => [...proposals, newProposal]);
        return newProposal;
      }
      throw new Error('Failed to create proposal');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to create proposal';
      proposalsError.set(errorMessage);
      console.error('Failed to create proposal:', error);
      throw error;
    }
  },

  async refresh() {
    await this.load();
  }
};

// ============================================================================
// GLOBAL ACTIONS
// ============================================================================

// Load all data
export const loadAllData = async () => {
  await Promise.allSettled([
    projectsActions.load(),
    companiesActions.load(),
    contactsActions.load(),
    proposalsActions.load()
  ]);
};

// Refresh all data
export const refreshAllData = async () => {
  await loadAllData();
};

// Clear all data (useful for logout)
export const clearAllData = () => {
  projectsStore.set([]);
  companiesStore.set([]);
  contactsStore.set([]);
  proposalsStore.set([]);
  
  // Clear errors
  projectsError.set(null);
  companiesError.set(null);
  contactsError.set(null);
  proposalsError.set(null);
};

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

// Get current values synchronously
export const getCurrentData = () => ({
  projects: get(projectsStore),
  companies: get(companiesStore),
  contacts: get(contactsStore),
  proposals: get(proposalsStore),
  connection: get(connectionStore),
  statistics: get(statisticsStore)
});

// Check if data is loaded
export const isDataLoaded = () => {
  const data = getCurrentData();
  return (
    data.projects.length > 0 ||
    data.companies.length > 0 ||
    data.contacts.length > 0 ||
    data.proposals.length > 0
  );
};

// ============================================================================
// INITIALIZATION
// ============================================================================

// Note: Data loading is now handled by App.svelte onMount to avoid race conditions