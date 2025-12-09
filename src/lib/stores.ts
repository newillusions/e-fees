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
import { projectLogger, companyLogger, contactLogger, feeLogger } from './services/activityLogger';

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
    // Only count fees truly in-progress: Draft, Sent, Negotiation
    // Exclude: Awarded, Completed, Lost, Cancelled, Revised, On Hold, Active
    activeFees: fees.filter(f => ['Draft', 'Sent', 'Negotiation'].includes(f.status)).length,
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

// Export compatible actions interface with activity logging
export const projectsActions = {
  async load() {
    return await projectsActionsInternal.load();
  },

  async create(project: Omit<Project, 'id'>) {
    const result = await projectsActionsInternal.create(project);
    // Log activity (fire-and-forget)
    projectLogger.onCreate(result as unknown as Record<string, unknown>);
    return result;
  },

  async update(id: string, projectData: Partial<Project>) {
    // Get current project for status change detection
    const currentProject = projectsActionsInternal.getById(id);
    const result = await projectsActionsInternal.update(id, projectData);

    // Check if this is a status change
    if (currentProject && projectData.status && currentProject.status !== projectData.status) {
      projectLogger.onStatusChange(
        result as unknown as Record<string, unknown>,
        currentProject.status,
        projectData.status
      );
    } else {
      // Log general update with changed fields
      const changedFields = Object.keys(projectData);
      projectLogger.onUpdate(result as unknown as Record<string, unknown>, changedFields);
    }
    return result;
  },

  async delete(id: string) {
    // Get project name before deletion
    const project = projectsActionsInternal.getById(id);
    const projectName = project?.name || project?.project_number || 'Unknown Project';
    const result = await projectsActionsInternal.delete(id);
    // Log activity (fire-and-forget)
    projectLogger.onDelete(id, projectName);
    return result;
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
    const result = await companiesActionsInternal.create(company);
    // Log activity (fire-and-forget)
    companyLogger.onCreate(result as unknown as Record<string, unknown>);
    return result;
  },

  async update(id: string, companyData: Partial<Company>) {
    const result = await companiesActionsInternal.update(id, companyData);
    // Log update with changed fields
    const changedFields = Object.keys(companyData);
    companyLogger.onUpdate(result as unknown as Record<string, unknown>, changedFields);
    return result;
  },

  async delete(id: string) {
    // Get company name before deletion
    const company = companiesActionsInternal.getById(id);
    const companyName = company?.name || 'Unknown Company';
    const result = await companiesActionsInternal.delete(id);
    // Log activity (fire-and-forget)
    companyLogger.onDelete(id, companyName);
    return result;
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
    const result = await contactsActionsInternal.create(contact);
    // Log activity (fire-and-forget)
    contactLogger.onCreate(result as unknown as Record<string, unknown>);
    return result;
  },

  async update(id: string, contactData: Partial<Contact>) {
    const result = await contactsActionsInternal.update(id, contactData);
    // Log update with changed fields
    const changedFields = Object.keys(contactData);
    contactLogger.onUpdate(result as unknown as Record<string, unknown>, changedFields);
    return result;
  },

  async delete(id: string) {
    // Get contact name before deletion
    const contact = contactsActionsInternal.getById(id);
    const contactName = contact?.full_name ||
      `${contact?.first_name || ''} ${contact?.last_name || ''}`.trim() ||
      'Unknown Contact';
    const result = await contactsActionsInternal.delete(id);
    // Log activity (fire-and-forget)
    contactLogger.onDelete(id, contactName);
    return result;
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
    const result = await feesActionsInternal.create(fee);
    // Log activity (fire-and-forget)
    feeLogger.onCreate(result as unknown as Record<string, unknown>);
    return result;
  },

  async update(id: string, feeData: Partial<Fee>) {
    // Get current fee for status change detection
    const currentFee = feesActionsInternal.getById(id);
    const result = await feesActionsInternal.update(id, feeData);

    // Check if this is a status change
    if (currentFee && feeData.status && currentFee.status !== feeData.status) {
      feeLogger.onStatusChange(
        result as unknown as Record<string, unknown>,
        currentFee.status,
        feeData.status
      );
    } else {
      // Log general update with changed fields
      const changedFields = Object.keys(feeData);
      feeLogger.onUpdate(result as unknown as Record<string, unknown>, changedFields);
    }
    return result;
  },

  async delete(id: string) {
    // Get fee name before deletion
    const fee = feesActionsInternal.getById(id);
    const feeName = fee?.name || fee?.number || 'Unknown Fee';
    const result = await feesActionsInternal.delete(id);
    // Log activity (fire-and-forget)
    feeLogger.onDelete(id, feeName);
    return result;
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

    const oldStatus = currentFee.status;

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

    const result = await feesActionsInternal.update(id, updatedFeeData);

    // Log status change (fire-and-forget)
    feeLogger.onStatusChange(
      result as unknown as Record<string, unknown>,
      oldStatus,
      newStatus
    );

    return result;
  }
};


// ============================================================================
// GLOBAL ACTIONS
// ============================================================================

// Global loading flag to prevent concurrent data loads
let isLoadingData = false;

// Load all data
export const loadAllData = async () => {
  // Prevent concurrent loads
  if (isLoadingData) {
    console.log('[loadAllData] Already loading, skipping duplicate call');
    return;
  }

  isLoadingData = true;
  try {
    const { settingsActions } = await import('./stores/settings');
    await Promise.allSettled([
      projectsActions.load(),
      companiesActions.load(),
      contactsActions.load(),
      feesActions.load(),
      settingsActions.load() // Load settings too
    ]);
  } finally {
    isLoadingData = false;
  }
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

// ============================================================================
// PAGINATION EXPORTS
// ============================================================================

// Export pagination utilities and types
export {
  createPaginatedStore,
  createOnDemandLoader,
  createScrollTrigger,
  sortByCreatedAt,
  DEFAULT_PAGE_SIZE,
  BACKGROUND_LOAD_DELAY,
  type PaginatedStoreState,
  type PaginatedStoreActions,
} from './stores/pagination';

// Export pagination adapters
export {
  projectsPaginationApi,
  companiesPaginationApi,
  contactsPaginationApi,
  feesPaginationApi,
  type PaginationApi,
} from './stores/adapters';

// Import for creating paginated stores
import { createPaginatedStore, createOnDemandLoader } from './stores/pagination';
import {
  projectsPaginationApi as _projectsPagApi,
  companiesPaginationApi as _companiesPagApi,
  contactsPaginationApi as _contactsPagApi,
  feesPaginationApi as _feesPagApi,
} from './stores/adapters';

// ============================================================================
// PAGINATED STORE INSTANCES
// ============================================================================

/**
 * Paginated projects store with lazy loading support.
 * Use this instead of projectsStore when pagination is needed.
 */
export const paginatedProjectsStore = createPaginatedStore<Project>(
  (page, pageSize) => _projectsPagApi.getPage(page, pageSize)
);

/**
 * Paginated companies store with lazy loading support.
 */
export const paginatedCompaniesStore = createPaginatedStore<Company>(
  (page, pageSize) => _companiesPagApi.getPage(page, pageSize)
);

/**
 * Paginated contacts store with lazy loading support.
 */
export const paginatedContactsStore = createPaginatedStore<Contact>(
  (page, pageSize) => _contactsPagApi.getPage(page, pageSize)
);

/**
 * Paginated fees store with lazy loading support.
 */
export const paginatedFeesStore = createPaginatedStore<Fee>(
  (page, pageSize) => _feesPagApi.getPage(page, pageSize)
);

// ============================================================================
// ON-DEMAND LOADERS (for related record fetching)
// ============================================================================

/**
 * On-demand company loader for fetching companies not in paginated store.
 */
export const companyOnDemandLoader = createOnDemandLoader<Company>(
  (id) => _companiesPagApi.getById!(id)
);

/**
 * On-demand contact loader for fetching contacts not in paginated store.
 */
export const contactOnDemandLoader = createOnDemandLoader<Contact>(
  (id) => _contactsPagApi.getById!(id)
);

/**
 * On-demand project loader for fetching projects not in paginated store.
 */
export const projectOnDemandLoader = createOnDemandLoader<Project>(
  (id) => _projectsPagApi.getById!(id)
);

