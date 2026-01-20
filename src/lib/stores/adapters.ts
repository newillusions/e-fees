/**
 * API Adapters for CRUD Store Integration
 *
 * This module provides adapter implementations that bridge the existing API
 * functions with the enhanced CRUD store interface. Each adapter wraps the
 * current API functions to conform to the CrudApi<T> interface.
 *
 * Uses object literals instead of classes since adapters are stateless.
 */

import type { CrudApi } from '../utils/crud';
import type { Project, Company, Contact, Fee, PaginatedResponse } from '../../types';
import {
  getProjects,
  getCompanies,
  getContacts,
  getFees,
  createProjectWithTemplate,
  updateProject,
  deleteProject,
  createCompany,
  updateCompany,
  deleteCompany,
  createContact,
  updateContact,
  deleteContact,
  createFee,
  updateFee,
  deleteFee,
  getProjectsPage,
  getCompaniesPage,
  getContactsPage,
  getFeesPage,
  getProjectById,
  getCompanyById,
  getContactById
} from '../api';

/**
 * Pagination API interface for paginated data fetching.
 */
export interface PaginationApi<T> {
  /** Fetch a page of data */
  getPage(page: number, pageSize: number): Promise<PaginatedResponse<T>>;
  /** Fetch a single item by ID (for on-demand loading) */
  getById?(id: string): Promise<T | null>;
}

// ============================================================================
// CRUD API ADAPTERS
// ============================================================================

/** Projects API adapter implementing CrudApi interface */
export const projectsApi: CrudApi<Project> = {
  getAll: () => getProjects(),
  create: (data) => createProjectWithTemplate(data),
  update: (id, data) => updateProject(id, data),
  delete: (id) => deleteProject(id)
};

/** Companies API adapter implementing CrudApi interface */
export const companiesApi: CrudApi<Company> = {
  getAll: () => getCompanies(),
  create: (data) => createCompany(data),
  update: async (id, data) => {
    const result = await updateCompany(id, data);
    if (!result) throw new Error(`Company ${id} not found`);
    return result;
  },
  delete: async (id) => {
    const result = await deleteCompany(id);
    if (!result) throw new Error(`Company ${id} not found`);
    return result;
  }
};

/** Contacts API adapter implementing CrudApi interface */
export const contactsApi: CrudApi<Contact> = {
  getAll: () => getContacts(),
  create: (data) => createContact(data),
  update: (id, data) => updateContact(id, data),
  delete: (id) => deleteContact(id)
};

/** Fees API adapter implementing CrudApi interface */
export const feesApi: CrudApi<Fee> = {
  getAll: () => getFees(),
  create: (data) => createFee(data),
  update: (id, data) => updateFee(id, data as Omit<Fee, 'id'>),
  delete: (id) => deleteFee(id)
};

// ============================================================================
// PAGINATION API ADAPTERS
// ============================================================================

/** Projects pagination API adapter */
export const projectsPaginationApi: PaginationApi<Project> = {
  getPage: (page, pageSize) => getProjectsPage(page, pageSize),
  getById: (id) => getProjectById(id)
};

/** Companies pagination API adapter */
export const companiesPaginationApi: PaginationApi<Company> = {
  getPage: (page, pageSize) => getCompaniesPage(page, pageSize),
  getById: (id) => getCompanyById(id)
};

/** Contacts pagination API adapter */
export const contactsPaginationApi: PaginationApi<Contact> = {
  getPage: (page, pageSize) => getContactsPage(page, pageSize),
  getById: (id) => getContactById(id)
};

/** Fees pagination API adapter */
export const feesPaginationApi: PaginationApi<Fee> = {
  getPage: (page, pageSize) => getFeesPage(page, pageSize)
};
