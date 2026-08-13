/**
 * Migrated Stores Implementation
 *
 * This module contains the new store implementations using the enhanced CRUD utilities.
 * It provides identical API contracts to the original stores while reducing code
 * duplication and adding enhanced features like optimistic updates and professional logging.
 */

import { writable, derived, get } from 'svelte/store';
import type { Project, Company, Contact, Fee } from '../types';
import { createEntityStore } from './utils/crud';
import { projectsApi, companiesApi, contactsApi, feesApi } from './stores/adapters';
import { projectLogger, companyLogger, contactLogger, feeLogger } from './services/activityLogger';
import { ACTIVE_PROPOSAL_STATUSES, ACTIVE_PROJECT_STATUSES } from './constants';
import { batchDeleteEntities, batchUpdateStatus } from './api/batch';
import { extractIdFromRelation } from './utils/surrealdb';

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
function createSyncedEntityStore<T extends EntityWithId>(api: CrudApi<T>, entityName: string) {
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
    activeFees: fees.filter(f => (ACTIVE_PROPOSAL_STATUSES as readonly string[]).includes(f.status))
      .length,
    totalCompanies: companies.length,
    totalContacts: contacts.length,
    totalFees: fees.length
  })
);

// Active projects (currently in progress)
export const activeProjectsStore = derived(projectsStore, $projects =>
  $projects.filter(project =>
    (ACTIVE_PROJECT_STATUSES as readonly string[]).includes(project.status)
  )
);

// Recent fees (last 30 days)
export const recentFeesStore = derived(feesStore, $fees => {
  const thirtyDaysAgo = new Date();
  thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);
  return $fees.filter(fee => fee.time && new Date(fee.time.created_at) > thirtyDaysAgo);
});

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
  ([projects, companies, contacts, fees]) => projects || companies || contacts || fees
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
    paginatedProjectsStore.actions.addItem(result);
    projectLogger.onCreate(result);
    return result;
  },

  async update(id: string, projectData: Partial<Project>) {
    const currentProject = projectsActionsInternal.getById(id);
    const result = await projectsActionsInternal.update(id, projectData);

    paginatedProjectsStore.actions.updateItem(id, result);

    if (currentProject && projectData.status && currentProject.status !== projectData.status) {
      projectLogger.onStatusChange(result, currentProject.status, projectData.status);
    } else {
      const changedFields = Object.keys(projectData);
      projectLogger.onUpdate(result, changedFields);
    }
    return result;
  },

  async delete(id: string) {
    const project = projectsActionsInternal.getById(id);
    const projectName = project?.name || project?.project_number || 'Unknown Project';
    const result = await projectsActionsInternal.delete(id);
    paginatedProjectsStore.actions.removeItem(id);
    projectLogger.onDelete(id, projectName);
    return result;
  },

  async refresh() {
    return await projectsActionsInternal.refresh();
  },

  /**
   * Update the status of multiple projects in one call (multiselect bulk action).
   * Mirrors update()'s per-item activity logging so bulk status changes are as
   * auditable as single-project edits, and patches the paginated store in place
   * instead of forcing a full refetch.
   *
   * Returns { requested, applied } so callers can detect a partial application
   * (e.g. an id that no longer exists server-side) instead of assuming success.
   */
  async bulkUpdateStatus(
    ids: string[],
    status: Project['status']
  ): Promise<{ requested: number; applied: number }> {
    if (ids.length === 0) return { requested: 0, applied: 0 };

    // Keyed by the BARE record key (matches `ids`, which is bare-key form
    // per the batch API contract). Project.id itself is stored as the full
    // "table:key" string, so it can be passed straight to updateItem() below
    // - that store keys its items by extractSurrealId(item.id), which
    // returns a plain string unchanged (no prefix stripping).
    const before = new Map<string, Project>();
    for (const item of paginatedProjectsStore.actions.getState().items) {
      if (item.id) before.set(extractIdFromRelation(item.id), item);
    }

    const applied = await batchUpdateStatus('projects', ids, status);

    for (const id of ids) {
      const priorProject = before.get(id);
      if (!priorProject || !priorProject.id) continue; // not present locally - nothing to patch/log against

      const updated: Project = {
        ...priorProject,
        status,
        time: {
          created_at: priorProject.time?.created_at ?? new Date().toISOString(),
          updated_at: new Date().toISOString()
        }
      };
      paginatedProjectsStore.actions.updateItem(priorProject.id, updated);

      if (priorProject.status !== status) {
        projectLogger.onStatusChange(updated, priorProject.status, status);
      }
    }

    return { requested: ids.length, applied };
  },

  /**
   * Delete multiple projects in one call (multiselect bulk action).
   * Mirrors delete()'s per-item activity logging and patches the paginated
   * store in place instead of forcing a full refetch.
   *
   * Returns { requested, applied } so callers can detect a partial application.
   */
  async bulkDelete(ids: string[]): Promise<{ requested: number; applied: number }> {
    if (ids.length === 0) return { requested: 0, applied: 0 };

    // Same bare-key lookup convention as bulkUpdateStatus above.
    const before = new Map<string, Project>();
    for (const item of paginatedProjectsStore.actions.getState().items) {
      if (item.id) before.set(extractIdFromRelation(item.id), item);
    }

    const deleted = await batchDeleteEntities('projects', ids);

    for (const id of ids) {
      const priorProject = before.get(id);
      if (priorProject?.id) {
        paginatedProjectsStore.actions.removeItem(priorProject.id);
      }
      const projectName = priorProject?.name || priorProject?.project_number || 'Unknown Project';
      projectLogger.onDelete(id, projectName);
    }

    return { requested: ids.length, applied: deleted.length };
  }
};

export const companiesActions = {
  async load() {
    return await companiesActionsInternal.load();
  },

  async create(company: Omit<Company, 'id'>) {
    const result = await companiesActionsInternal.create(company);
    paginatedCompaniesStore.actions.addItem(result);
    companyLogger.onCreate(result);
    return result;
  },

  async update(id: string, companyData: Partial<Company>) {
    const result = await companiesActionsInternal.update(id, companyData);
    paginatedCompaniesStore.actions.updateItem(id, result);
    const changedFields = Object.keys(companyData);
    companyLogger.onUpdate(result, changedFields);
    return result;
  },

  async delete(id: string) {
    const company = companiesActionsInternal.getById(id);
    const companyName = company?.name || 'Unknown Company';
    const result = await companiesActionsInternal.delete(id);
    paginatedCompaniesStore.actions.removeItem(id);
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
    paginatedContactsStore.actions.addItem(result);
    contactLogger.onCreate(result);
    return result;
  },

  async update(id: string, contactData: Partial<Contact>) {
    const result = await contactsActionsInternal.update(id, contactData);
    paginatedContactsStore.actions.updateItem(id, result);
    const changedFields = Object.keys(contactData);
    contactLogger.onUpdate(result, changedFields);
    return result;
  },

  async delete(id: string) {
    const contact = contactsActionsInternal.getById(id);
    const contactName =
      contact?.full_name ||
      `${contact?.first_name || ''} ${contact?.last_name || ''}`.trim() ||
      'Unknown Contact';
    const result = await contactsActionsInternal.delete(id);
    paginatedContactsStore.actions.removeItem(id);
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
    paginatedFeesStore.actions.addItem(result);
    feeLogger.onCreate(result);
    return result;
  },

  async update(id: string, feeData: Partial<Fee>) {
    const currentFee = feesActionsInternal.getById(id);
    const result = await feesActionsInternal.update(id, feeData);

    paginatedFeesStore.actions.updateItem(id, result);

    if (currentFee && feeData.status && currentFee.status !== feeData.status) {
      feeLogger.onStatusChange(result, currentFee.status, feeData.status);
    } else {
      const changedFields = Object.keys(feeData);
      feeLogger.onUpdate(result, changedFields);
    }
    return result;
  },

  async delete(id: string) {
    const fee = feesActionsInternal.getById(id);
    const feeName = fee?.name || fee?.number || 'Unknown Fee';
    const result = await feesActionsInternal.delete(id);
    paginatedFeesStore.actions.removeItem(id);
    feeLogger.onDelete(id, feeName);
    return result;
  },

  async refresh() {
    return await feesActionsInternal.refresh();
  },

  async updateStatus(id: string, newStatus: string) {
    const currentFee = feesActionsInternal.getById(id);

    if (!currentFee) {
      throw new Error(`Fee with ID ${id} not found`);
    }

    const oldStatus = currentFee.status;

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
    paginatedFeesStore.actions.updateItem(id, result);
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
  statistics: get(statisticsStore)
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
  type PaginatedStoreActions
} from './stores/pagination';

// Export pagination adapters
export {
  projectsPaginationApi,
  companiesPaginationApi,
  contactsPaginationApi,
  feesPaginationApi,
  type PaginationApi
} from './stores/adapters';

// Import for creating paginated stores
import { createPaginatedStore, createOnDemandLoader } from './stores/pagination';
import {
  projectsPaginationApi as _projectsPagApi,
  companiesPaginationApi as _companiesPagApi,
  contactsPaginationApi as _contactsPagApi,
  feesPaginationApi as _feesPagApi
} from './stores/adapters';

// ============================================================================
// PAGINATED STORE INSTANCES
// ============================================================================

/**
 * Paginated projects store with lazy loading support.
 * Use this instead of projectsStore when pagination is needed.
 */
export const paginatedProjectsStore = createPaginatedStore<Project>((page, pageSize) =>
  _projectsPagApi.getPage(page, pageSize)
);

/**
 * Paginated companies store with lazy loading support.
 */
export const paginatedCompaniesStore = createPaginatedStore<Company>((page, pageSize) =>
  _companiesPagApi.getPage(page, pageSize)
);

/**
 * Paginated contacts store with lazy loading support.
 */
export const paginatedContactsStore = createPaginatedStore<Contact>((page, pageSize) =>
  _contactsPagApi.getPage(page, pageSize)
);

/**
 * Paginated fees store with lazy loading support.
 */
export const paginatedFeesStore = createPaginatedStore<Fee>((page, pageSize) =>
  _feesPagApi.getPage(page, pageSize)
);

// ============================================================================
// ON-DEMAND LOADERS (for related record fetching)
// ============================================================================

/**
 * On-demand company loader for fetching companies not in paginated store.
 */
export const companyOnDemandLoader = createOnDemandLoader<Company>(id =>
  _companiesPagApi.getById!(id)
);

/**
 * On-demand contact loader for fetching contacts not in paginated store.
 */
export const contactOnDemandLoader = createOnDemandLoader<Contact>(id =>
  _contactsPagApi.getById!(id)
);

/**
 * On-demand project loader for fetching projects not in paginated store.
 */
export const projectOnDemandLoader = createOnDemandLoader<Project>(id =>
  _projectsPagApi.getById!(id)
);
