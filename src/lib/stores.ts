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
} from '../types';
import { createEntityStore } from './utils/crud';
import { projectsApi, companiesApi, contactsApi, feesApi } from './stores/adapters';
import { projectLogger, companyLogger, contactLogger, feeLogger } from './services/activityLogger';
import { ACTIVE_PROPOSAL_STATUSES, ACTIVE_PROJECT_STATUSES } from './constants';

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

import type { CrudApi } from './utils/crud';
import type { UnknownSurrealThing } from '../types';

/** Base entity type that has an optional SurrealDB Thing ID */
type EntityWithId = { id?: UnknownSurrealThing };

/**
 * Factory function to create entity stores with auto-sync to external stores.
 * Eliminates code duplication across projects, companies, contacts, and fees.
 */
function createSyncedEntityStore<T extends EntityWithId>(
  api: CrudApi<T>,
  entityName: string
) {
  const { store: internalStore, actions: internalActions } = createEntityStore<T>(api, entityName, {
    enableOptimistic: true,
    enableLogging: true
  });

  const itemsStore = writable<T[]>([]);
  const loadingStore = writable<boolean>(false);
  const errorStore = writable<string | null>(null);

  // Auto-sync internal CRUD store with exported stores
  internalStore.subscribe(state => {
    itemsStore.set(state.items);
    loadingStore.set(state.loading || state.saving);
    errorStore.set(state.error);
  });

  return { internalStore, internalActions, itemsStore, loadingStore, errorStore };
}

// Create all entity stores using the factory
const projectsInternal = createSyncedEntityStore(projectsApi, 'Project');
const companiesInternal = createSyncedEntityStore(companiesApi, 'Company');
const contactsInternal = createSyncedEntityStore(contactsApi, 'Contact');
const feesInternal = createSyncedEntityStore(feesApi, 'Fee');

// Export compatible writable stores for testing and components
export const projectsStore = projectsInternal.itemsStore;
export const projectsLoading = projectsInternal.loadingStore;
export const projectsError = projectsInternal.errorStore;

export const companiesStore = companiesInternal.itemsStore;
export const companiesLoading = companiesInternal.loadingStore;
export const companiesError = companiesInternal.errorStore;

export const contactsStore = contactsInternal.itemsStore;
export const contactsLoading = contactsInternal.loadingStore;
export const contactsError = contactsInternal.errorStore;

export const feesStore = feesInternal.itemsStore;
export const feesLoading = feesInternal.loadingStore;
export const feesError = feesInternal.errorStore;

// Internal references for actions
const projectsActionsInternal = projectsInternal.internalActions;
const companiesActionsInternal = companiesInternal.internalActions;
const contactsActionsInternal = contactsInternal.internalActions;
const feesActionsInternal = feesInternal.internalActions;

// ============================================================================
// DERIVED STORES (COMPUTED VALUES)
// ============================================================================

// Statistics derived from data
export const statisticsStore = derived(
  [projectsStore, feesStore, companiesStore, contactsStore],
  ([projects, fees, companies, contacts]) => ({
    totalProjects: projects.length,
    // Count fees that are current/in-play (Draft, Sent, Negotiation)
    activeFees: fees.filter(f => (ACTIVE_PROPOSAL_STATUSES as readonly string[]).includes(f.status)).length,
    totalCompanies: companies.length,
    totalContacts: contacts.length,
    totalFees: fees.length,
  })
);

// Active projects (currently in progress)
export const activeProjectsStore = derived(
  projectsStore,
  $projects => $projects.filter(project => (ACTIVE_PROJECT_STATUSES as readonly string[]).includes(project.status))
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
// Optimized: O(n+m) instead of O(n*m) by pre-computing contact counts
export const companiesWithContactsStore = derived(
  [companiesStore, contactsStore],
  ([companies, contacts]) => {
    // Build contact count map in O(m)
    const contactCountMap = new Map<string, number>();
    for (const contact of contacts) {
      if (contact.company) {
        const companyId = String(contact.company);
        contactCountMap.set(companyId, (contactCountMap.get(companyId) || 0) + 1);
      }
    }
    // Map companies with counts in O(n)
    return companies.map(company => ({
      ...company,
      contactCount: contactCountMap.get(String(company.id)) || 0
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
    projectLogger.onCreate(result);
    return result;
  },

  async update(id: string, projectData: Partial<Project>) {
    // Get current project for status change detection
    const currentProject = projectsActionsInternal.getById(id);
    const result = await projectsActionsInternal.update(id, projectData);

    // Check if this is a status change
    if (currentProject && projectData.status && currentProject.status !== projectData.status) {
      projectLogger.onStatusChange(result, currentProject.status, projectData.status);
    } else {
      // Log general update with changed fields
      const changedFields = Object.keys(projectData);
      projectLogger.onUpdate(result, changedFields);
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
    companyLogger.onCreate(result);
    return result;
  },

  async update(id: string, companyData: Partial<Company>) {
    const result = await companiesActionsInternal.update(id, companyData);
    // Log update with changed fields
    const changedFields = Object.keys(companyData);
    companyLogger.onUpdate(result, changedFields);
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
    contactLogger.onCreate(result);
    return result;
  },

  async update(id: string, contactData: Partial<Contact>) {
    const result = await contactsActionsInternal.update(id, contactData);
    // Log update with changed fields
    const changedFields = Object.keys(contactData);
    contactLogger.onUpdate(result, changedFields);
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
    feeLogger.onCreate(result);
    return result;
  },

  async update(id: string, feeData: Partial<Fee>) {
    // Get current fee for status change detection
    const currentFee = feesActionsInternal.getById(id);
    const result = await feesActionsInternal.update(id, feeData);

    // Check if this is a status change
    if (currentFee && feeData.status && currentFee.status !== feeData.status) {
      feeLogger.onStatusChange(result, currentFee.status, feeData.status);
    } else {
      // Log general update with changed fields
      const changedFields = Object.keys(feeData);
      feeLogger.onUpdate(result, changedFields);
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

  // Preserved custom method for fee status updates
  async updateStatus(id: string, newStatus: string) {
    // Get the current fee to preserve other fields
    const currentFee = feesActionsInternal.getById(id);

    if (!currentFee) {
      throw new Error(`Fee with ID ${id} not found`);
    }

    const oldStatus = currentFee.status;

    // Update only the status field
    // Destructure to exclude id from update data
    const { id: _id, ...feeWithoutId } = currentFee;
    const updatedFeeData = {
      ...feeWithoutId,
      status: newStatus as Fee['status'],
      time: {
        ...currentFee.time,
        updated_at: new Date().toISOString(),
        created_at: currentFee.time?.created_at || new Date().toISOString()
      }
    };

    const result = await feesActionsInternal.update(id, updatedFeeData);

    // Log status change (fire-and-forget)
    feeLogger.onStatusChange(result, oldStatus, newStatus);

    return result;
  }
};



// ============================================================================
// GLOBAL ACTIONS
// ============================================================================

// Global loading flag to prevent concurrent data loads
let isLoadingData = false;

// Load all data
export const loadAllData = async (): Promise<void> => {
  // Prevent concurrent loads
  if (isLoadingData) {
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
export const loadProjects = (): Promise<void> => projectsActions.load();
export const loadCompanies = (): Promise<void> => companiesActions.load();
export const loadContacts = (): Promise<void> => contactsActions.load();
export const loadFees = (): Promise<void> => feesActions.load();

// Refresh all data
export const refreshAllData = async (): Promise<void> => {
  await loadAllData();
};

// Clear all data (useful for logout)
export const clearAllData = (): void => {
  projectsActionsInternal.clear();
  companiesActionsInternal.clear();
  contactsActionsInternal.clear();
  feesActionsInternal.clear();
};

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/** Statistics snapshot type */
interface StatisticsSnapshot {
  totalProjects: number;
  activeFees: number;
  totalCompanies: number;
  totalContacts: number;
  totalFees: number;
}

/** Current data snapshot type */
interface CurrentDataSnapshot {
  projects: Project[];
  companies: Company[];
  contacts: Contact[];
  fees: Fee[];
  connection: ConnectionState;
  statistics: StatisticsSnapshot;
}

// Get current values synchronously
export const getCurrentData = (): CurrentDataSnapshot => ({
  projects: get(projectsStore),
  companies: get(companiesStore),
  contacts: get(contactsStore),
  fees: get(feesStore),
  connection: get(connectionStore),
  statistics: get(statisticsStore),
});

// Check if data is loaded
export const isDataLoaded = (): boolean => {
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

