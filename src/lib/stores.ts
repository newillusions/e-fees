import { writable, derived, get } from 'svelte/store';
import type { 
  Project, 
  Company, 
  Contact, 
  Rfp, 
  ConnectionStatus 
} from '../types';
import { 
  getProjects,
  getCompanies,
  getContacts,
  getRfps,
  getStats,
  createProject,
  createCompany,
  updateCompany,
  deleteCompany,
  createContact,
  createRfp
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

// Debug: Log when stores are updated
projectsStore.subscribe(value => {
  console.log('STORE UPDATE - Projects:', value.length, 'items');
});

// Companies store
export const companiesStore = writable<Company[]>([]);
export const companiesLoading = writable<boolean>(false);
export const companiesError = writable<string | null>(null);

// Debug: Log when stores are updated
companiesStore.subscribe(value => {
  console.log('STORE UPDATE - Companies:', value.length, 'items');
});

// Contacts store
export const contactsStore = writable<Contact[]>([]);
export const contactsLoading = writable<boolean>(false);
export const contactsError = writable<string | null>(null);

// Debug: Log when stores are updated
contactsStore.subscribe(value => {
  console.log('STORE UPDATE - Contacts:', value.length, 'items');
});

// RFPs store
export const rfpsStore = writable<Rfp[]>([]);
export const rfpsLoading = writable<boolean>(false);
export const rfpsError = writable<string | null>(null);

// Debug: Log when stores are updated
rfpsStore.subscribe(value => {
  console.log('STORE UPDATE - RFPs:', value.length, 'items');
});

// Legacy aliases for backwards compatibility
export const proposalsStore = rfpsStore;
export const proposalsLoading = rfpsLoading;
export const proposalsError = rfpsError;

// ============================================================================
// DERIVED STORES (COMPUTED VALUES)
// ============================================================================

// Statistics derived from data
export const statisticsStore = derived(
  [projectsStore, rfpsStore, companiesStore, contactsStore],
  ([projects, rfps, companies, contacts]) => ({
    totalProjects: projects.length,
    activeRfps: rfps.filter(r => r.status !== 'Lost' && r.status !== 'Cancelled').length,
    totalCompanies: companies.length,
    totalContacts: contacts.length,
    totalRfps: rfps.length,
    // Legacy fields for backwards compatibility
    activeProposals: rfps.filter(r => r.status !== 'Lost' && r.status !== 'Cancelled').length,
    totalProposals: rfps.length
  })
);

// Active projects only
export const activeProjectsStore = derived(
  projectsStore,
  $projects => $projects.filter(project => project.status === 'Active')
);

// Recent RFPs (last 30 days)
export const recentRfpsStore = derived(
  rfpsStore,
  $rfps => {
    const thirtyDaysAgo = new Date();
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);
    return $rfps.filter(rfp => new Date(rfp.time.created_at) > thirtyDaysAgo);
  }
);

// Legacy alias
export const recentProposalsStore = recentRfpsStore;

// Companies with contact counts
export const companiesWithContactsStore = derived(
  [companiesStore, contactsStore],
  ([companies, contacts]) => {
    return companies.map(company => ({
      ...company,
      contactCount: contacts.filter(contact => contact.company === company.id).length
    }));
  }
);

// Loading state for any data operation
export const isLoadingStore = derived(
  [projectsLoading, companiesLoading, contactsLoading, rfpsLoading],
  ([projects, companies, contacts, rfps]) => 
    projects || companies || contacts || rfps
);

// Global error state
export const globalErrorStore = derived(
  [projectsError, companiesError, contactsError, rfpsError],
  ([projectsErr, companiesErr, contactsErr, rfpsErr]) => 
    projectsErr || companiesErr || contactsErr || rfpsErr
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
      console.log('Loading projects from database...');
      const projects = await getProjects();
      console.log('Projects loaded successfully:', projects.length, 'items');
      projectsStore.set(projects);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load projects';
      projectsError.set(errorMessage);
      console.warn('Failed to load projects from database, using mock data:', error);
      // Keep mock data in store - don't overwrite with empty array
    } finally {
      projectsLoading.set(false);
    }
  },

  async create(project: Omit<Project, 'id'>) {
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
      console.log('Loading companies from database...');
      const companies = await getCompanies();
      console.log('Companies loaded successfully:', companies.length, 'items');
      companiesStore.set(companies);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load companies';
      companiesError.set(errorMessage);
      console.warn('Failed to load companies from database, using mock data:', error);
    } finally {
      companiesLoading.set(false);
    }
  },

  async create(company: Omit<Company, 'id'>) {
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

  async update(id: string, companyData: Partial<Company>) {
    try {
      console.log('Store update called with ID:', id);
      const updatedCompany = await updateCompany(id, companyData);
      if (updatedCompany) {
        console.log('Update successful, updating store');
        companiesStore.update(companies => 
          companies.map(company => {
            // Extract company ID for comparison
            let companyId = '';
            if (typeof company.id === 'string') {
              companyId = company.id;
            } else if (company.id && typeof company.id === 'object') {
              const thingObj = company.id as any;
              if (thingObj.tb && thingObj.id) {
                if (typeof thingObj.id === 'string') {
                  companyId = thingObj.id;
                } else if (thingObj.id.String) {
                  companyId = thingObj.id.String;
                }
              }
            }
            
            console.log('Comparing:', companyId, 'with', id);
            return companyId === id ? updatedCompany : company;
          })
        );
        return updatedCompany;
      }
      throw new Error('Failed to update company');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to update company';
      companiesError.set(errorMessage);
      console.error('Failed to update company:', error);
      throw error;
    }
  },

  async delete(id: string) {
    try {
      console.log('Store delete called with ID:', id);
      const deletedCompany = await deleteCompany(id);
      if (deletedCompany) {
        console.log('Delete successful, updating store');
        companiesStore.update(companies => 
          companies.filter(company => {
            // Extract company ID for comparison
            let companyId = '';
            if (typeof company.id === 'string') {
              companyId = company.id;
            } else if (company.id && typeof company.id === 'object') {
              const thingObj = company.id as any;
              if (thingObj.tb && thingObj.id) {
                if (typeof thingObj.id === 'string') {
                  companyId = thingObj.id;
                } else if (thingObj.id.String) {
                  companyId = thingObj.id.String;
                }
              }
            }
            
            console.log('Delete filter comparing:', companyId, 'with', id);
            return companyId !== id; // Keep companies that don't match the deleted ID
          })
        );
        return deletedCompany;
      }
      throw new Error('Failed to delete company');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to delete company';
      companiesError.set(errorMessage);
      console.error('Failed to delete company:', error);
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
      console.log('Loading contacts from database...');
      const contacts = await getContacts();
      console.log('Contacts loaded successfully:', contacts.length, 'items');
      contactsStore.set(contacts);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load contacts';
      contactsError.set(errorMessage);
      console.warn('Failed to load contacts from database, using mock data:', error);
    } finally {
      contactsLoading.set(false);
    }
  },

  async create(contact: Omit<Contact, 'id'>) {
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

// RFPs actions
export const rfpsActions = {
  async load() {
    rfpsLoading.set(true);
    rfpsError.set(null);
    
    try {
      console.log('Loading RFPs from database...');
      const rfps = await getRfps();
      console.log('RFPs loaded successfully:', rfps.length, 'items');
      rfpsStore.set(rfps);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load rfps';
      rfpsError.set(errorMessage);
      console.warn('Failed to load RFPs from database, using mock data:', error);
    } finally {
      rfpsLoading.set(false);
    }
  },

  async create(rfp: Omit<Rfp, 'id'>) {
    try {
      const newRfp = await createRfp(rfp);
      if (newRfp) {
        rfpsStore.update(rfps => [...rfps, newRfp]);
        return newRfp;
      }
      throw new Error('Failed to create rfp');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to create rfp';
      rfpsError.set(errorMessage);
      console.error('Failed to create rfp:', error);
      throw error;
    }
  },

  async refresh() {
    await this.load();
  }
};

// Legacy alias for backwards compatibility
export const proposalsActions = rfpsActions;

// ============================================================================
// GLOBAL ACTIONS
// ============================================================================

// Load all data
export const loadAllData = async () => {
  console.log('🔄 STARTING loadAllData...');
  const results = await Promise.allSettled([
    projectsActions.load(),
    companiesActions.load(),
    contactsActions.load(),
    rfpsActions.load()
  ]);
  
  console.log('✅ COMPLETED loadAllData, results:', results.map(r => r.status));
};

// Convenience functions for individual data loading
export const loadProjects = () => projectsActions.load();
export const loadCompanies = () => companiesActions.load();
export const loadContacts = () => contactsActions.load();
export const loadRfps = () => rfpsActions.load();

// Refresh all data
export const refreshAllData = async () => {
  await loadAllData();
};

// Clear all data (useful for logout)
export const clearAllData = () => {
  projectsStore.set([]);
  companiesStore.set([]);
  contactsStore.set([]);
  rfpsStore.set([]);
  
  // Clear errors
  projectsError.set(null);
  companiesError.set(null);
  contactsError.set(null);
  rfpsError.set(null);
};

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

// Get current values synchronously
export const getCurrentData = () => ({
  projects: get(projectsStore),
  companies: get(companiesStore),
  contacts: get(contactsStore),
  rfps: get(rfpsStore),
  connection: get(connectionStore),
  statistics: get(statisticsStore),
  // Legacy alias
  proposals: get(rfpsStore)
});

// Check if data is loaded
export const isDataLoaded = () => {
  const data = getCurrentData();
  return (
    data.projects.length > 0 ||
    data.companies.length > 0 ||
    data.contacts.length > 0 ||
    data.rfps.length > 0
  );
};

// ============================================================================
// SETTINGS EXPORTS
// ============================================================================

// Export settings functionality
export * from './stores/settings';

// ============================================================================
// INITIALIZATION
// ============================================================================

// Note: Data loading is now handled by App.svelte onMount to avoid race conditions