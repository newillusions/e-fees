/**
 * Migrated Stores Implementation
 * 
 * This module contains the new store implementations using the enhanced CRUD utilities.
 * It provides identical API contracts to the original stores while reducing code
 * duplication and adding enhanced features like optimistic updates and professional logging.
 */

import { writable, derived, get } from 'svelte/store';
import type { 
  Project, 
  Company, 
  Contact, 
  Fee, 
  ConnectionStatus 
} from '../types';
import { useCrudStore, createEntityStore } from './utils/crud';
import { projectsApi, companiesApi, contactsApi, feesApi } from './stores/adapters';
import { extractSurrealId } from './utils/surrealdb';

// ============================================================================
// CONNECTION STORE (UNCHANGED)
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
// ENTITY STORES - MIGRATED TO CRUD UTILITIES
// ============================================================================

// Projects store using enhanced CRUD utilities
const {
  store: projectsStoreInternal,
  actions: projectsActionsInternal
} = createEntityStore(projectsApi, 'Project', {
  enableOptimistic: true,
  enableLogging: true
});

// Export compatible writable stores for testing and manual updates
export const projectsStore = writable<Project[]>([]);
export const projectsLoading = writable<boolean>(false);
export const projectsError = writable<string | null>(null);

// Auto-sync internal CRUD store with exported stores
projectsStoreInternal.subscribe(state => {
  projectsStore.set(state.items);
  projectsLoading.set(state.loading || state.saving);
  projectsError.set(state.error);
});

// Companies store using enhanced CRUD utilities
const {
  store: companiesStoreInternal,
  actions: companiesActionsInternal
} = createEntityStore(companiesApi, 'Company', {
  enableOptimistic: true,
  enableLogging: true
});

// Export compatible writable stores for testing and manual updates
export const companiesStore = writable<Company[]>([]);
export const companiesLoading = writable<boolean>(false);
export const companiesError = writable<string | null>(null);

// Auto-sync internal CRUD store with exported stores
companiesStoreInternal.subscribe(state => {
  companiesStore.set(state.items);
  companiesLoading.set(state.loading || state.saving);
  companiesError.set(state.error);
});

// Contacts store using enhanced CRUD utilities
const {
  store: contactsStoreInternal,
  actions: contactsActionsInternal
} = createEntityStore(contactsApi, 'Contact', {
  enableOptimistic: true,
  enableLogging: true
});

// Export compatible writable stores for testing and manual updates
export const contactsStore = writable<Contact[]>([]);
export const contactsLoading = writable<boolean>(false);
export const contactsError = writable<string | null>(null);

// Auto-sync internal CRUD store with exported stores
contactsStoreInternal.subscribe(state => {
  contactsStore.set(state.items);
  contactsLoading.set(state.loading || state.saving);
  contactsError.set(state.error);
});

// Fees store using enhanced CRUD utilities
const {
  store: feesStoreInternal,
  actions: feesActionsInternal
} = createEntityStore(feesApi, 'Fee', {
  enableOptimistic: true,
  enableLogging: true
});

// Export compatible writable stores for testing and manual updates
export const feesStore = writable<Fee[]>([]);
export const feesLoading = writable<boolean>(false);
export const feesError = writable<string | null>(null);

// Auto-sync internal CRUD store with exported stores
feesStoreInternal.subscribe(state => {
  feesStore.set(state.items);
  feesLoading.set(state.loading || state.saving);
  feesError.set(state.error);
});



// ============================================================================
// DERIVED STORES (COMPUTED VALUES)
// ============================================================================

// Statistics derived from data
export const statisticsStore = derived(
  [projectsStore, feesStore, companiesStore, contactsStore],
  ([projects, fees, companies, contacts]) => ({
    totalProjects: projects.length,
    activeFees: fees.filter(f => f.status !== 'Lost' && f.status !== 'Cancelled').length,
    totalCompanies: companies.length,
    totalContacts: contacts.length,
    totalFees: fees.length,
  })
);

// Active projects only
export const activeProjectsStore = derived(
  projectsStore,
  $projects => $projects.filter(project => project.status === 'Active')
);

// Recent fees (last 30 days)
export const recentFeesStore = derived(
  feesStore,
  $fees => {
    const thirtyDaysAgo = new Date();
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);
    return $fees.filter(fee => fee.time && new Date(fee.time.created_at) > thirtyDaysAgo);
  }
);


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
  [projectsLoading, companiesLoading, contactsLoading, feesLoading],
  ([projects, companies, contacts, fees]) => 
    projects || companies || contacts || fees
);

// Global error state
export const globalErrorStore = derived(
  [projectsError, companiesError, contactsError, feesError],
  ([projectsErr, companiesErr, contactsErr, feesErr]) => 
    projectsErr || companiesErr || contactsErr || feesErr
);

// ============================================================================
// ACTION CREATORS - MIGRATED TO CRUD UTILITIES
// ============================================================================

// Export compatible actions interface
export const projectsActions = {
  async load() {
    return await projectsActionsInternal.load();
  },

  async create(project: Omit<Project, 'id'>) {
    return await projectsActionsInternal.create(project);
  },

  async update(id: string, projectData: Partial<Project>) {
    return await projectsActionsInternal.update(id, projectData);
  },

  async delete(id: string) {
    return await projectsActionsInternal.delete(id);
  },

  async refresh() {
    return await projectsActionsInternal.refresh();
  }
};

export const companiesActions = {
  async load() {
    return await companiesActionsInternal.load();
  },

  async create(company: Omit<Company, 'id'>) {
    return await companiesActionsInternal.create(company);
  },

  async update(id: string, companyData: Partial<Company>) {
    return await companiesActionsInternal.update(id, companyData);
  },

  async delete(id: string) {
    return await companiesActionsInternal.delete(id);
  },

  async refresh() {
    return await companiesActionsInternal.refresh();
  }
};

export const contactsActions = {
  async load() {
    return await contactsActionsInternal.load();
  },

  async create(contact: Omit<Contact, 'id'>) {
    return await contactsActionsInternal.create(contact);
  },

  async update(id: string, contactData: Partial<Contact>) {
    return await contactsActionsInternal.update(id, contactData);
  },

  async delete(id: string) {
    return await contactsActionsInternal.delete(id);
  },

  async refresh() {
    return await contactsActionsInternal.refresh();
  }
};

// Export compatible actions interface with updateStatus method preserved
export const feesActions = {
  async load() {
    return await feesActionsInternal.load();
  },

  async create(fee: Omit<Fee, 'id'>) {
    return await feesActionsInternal.create(fee);
  },

  async update(id: string, feeData: Partial<Fee>) {
    return await feesActionsInternal.update(id, feeData);
  },

  async delete(id: string) {
    return await feesActionsInternal.delete(id);
  },

  async refresh() {
    return await feesActionsInternal.refresh();
  },

  // Preserved custom method for status updates
  async updateStatus(id: string, newStatus: string) {
    // Get the current fee to preserve other fields  
    const currentFee = feesActionsInternal.getById(id);
    
    if (!currentFee) {
      throw new Error(`Fee with ID ${id} not found`);
    }
    
    // Update only the status field
    const updatedFeeData = {
      ...currentFee,
      status: newStatus as Fee['status'],
      time: {
        ...currentFee.time,
        updated_at: new Date().toISOString(),
        created_at: currentFee.time?.created_at || new Date().toISOString()
      }
    };
    
    // Remove the id field for the update
    delete (updatedFeeData as any).id;
    
    return await feesActionsInternal.update(id, updatedFeeData);
  }
};


// ============================================================================
// GLOBAL ACTIONS
// ============================================================================

// Load all data
export const loadAllData = async () => {
  const { settingsActions } = await import('./stores/settings');
  await Promise.allSettled([
    projectsActions.load(),
    companiesActions.load(),
    contactsActions.load(),
    feesActions.load(),
    settingsActions.load() // Load settings too
  ]);
};

// Convenience functions for individual data loading
export const loadProjects = () => projectsActions.load();
export const loadCompanies = () => companiesActions.load();
export const loadContacts = () => contactsActions.load();
export const loadFees = () => feesActions.load();

// Refresh all data
export const refreshAllData = async () => {
  await loadAllData();
};

// Clear all data (useful for logout)
export const clearAllData = () => {
  projectsActionsInternal.clear();
  companiesActionsInternal.clear();
  contactsActionsInternal.clear();
  feesActionsInternal.clear();
};

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

// Get current values synchronously
export const getCurrentData = () => ({
  projects: get(projectsStore),
  companies: get(companiesStore),
  contacts: get(contactsStore),
  fees: get(feesStore),
  connection: get(connectionStore),
  statistics: get(statisticsStore),
});

// Check if data is loaded
export const isDataLoaded = () => {
  const data = getCurrentData();
  return (
    data.projects.length > 0 ||
    data.companies.length > 0 ||
    data.contacts.length > 0 ||
    data.fees.length > 0
  );
};

// ============================================================================
// SETTINGS EXPORTS
// ============================================================================

// Export settings functionality
export * from './stores/settings';

