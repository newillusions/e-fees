/**
 * API Adapters for CRUD Store Integration
 * 
 * This module provides adapter implementations that bridge the existing API
 * functions with the enhanced CRUD store interface. Each adapter wraps the
 * current API functions to conform to the CrudApi<T> interface.
 */

import type { CrudApi } from '../utils/crud';
import type { Project, Company, Contact, Fee } from '../../types';
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
  deleteFee
} from '../api';

/**
 * Projects API adapter implementing CrudApi interface
 */
export class ProjectsApiAdapter implements CrudApi<Project> {
  async getAll(): Promise<Project[]> {
    return await getProjects();
  }

  async create(data: Omit<Project, 'id'>): Promise<Project> {
    return await createProjectWithTemplate(data);
  }

  async update(id: string, data: Partial<Project>): Promise<Project> {
    return await updateProject(id, data);
  }

  async delete(id: string): Promise<Project> {
    return await deleteProject(id);
  }
}

/**
 * Companies API adapter implementing CrudApi interface
 */
export class CompaniesApiAdapter implements CrudApi<Company> {
  async getAll(): Promise<Company[]> {
    return await getCompanies();
  }

  async create(data: Omit<Company, 'id'>): Promise<Company> {
    const result = await createCompany(data);
    if (!result) throw new Error('Failed to create company');
    return result;
  }

  async update(id: string, data: Partial<Company>): Promise<Company> {
    const result = await updateCompany(id, data);
    if (!result) throw new Error('Failed to update company');
    return result;
  }

  async delete(id: string): Promise<Company> {
    const result = await deleteCompany(id);
    if (!result) throw new Error('Failed to delete company');
    return result;
  }
}

/**
 * Contacts API adapter implementing CrudApi interface
 */
export class ContactsApiAdapter implements CrudApi<Contact> {
  async getAll(): Promise<Contact[]> {
    return await getContacts();
  }

  async create(data: Omit<Contact, 'id'>): Promise<Contact> {
    const result = await createContact(data);
    if (!result) throw new Error('Failed to create contact');
    return result;
  }

  async update(id: string, data: Partial<Contact>): Promise<Contact> {
    const result = await updateContact(id, data);
    if (!result) throw new Error('Failed to update contact');
    return result;
  }

  async delete(id: string): Promise<Contact> {
    const result = await deleteContact(id);
    if (!result) throw new Error('Failed to delete contact');
    return result;
  }
}

/**
 * Fees API adapter implementing CrudApi interface
 */
export class FeesApiAdapter implements CrudApi<Fee> {
  async getAll(): Promise<Fee[]> {
    return await getFees();
  }

  async create(data: Omit<Fee, 'id'>): Promise<Fee> {
    const result = await createFee(data);
    if (!result) throw new Error('Failed to create fee');
    return result;
  }

  async update(id: string, data: Partial<Fee>): Promise<Fee> {
    const result = await updateFee(id, data as Omit<Fee, 'id'>);
    if (!result) throw new Error('Failed to update fee');
    return result;
  }

  async delete(id: string): Promise<Fee> {
    const result = await deleteFee(id);
    if (!result) throw new Error('Failed to delete fee');
    return result;
  }
}

// Pre-instantiated adapters ready for use
export const projectsApi = new ProjectsApiAdapter();
export const companiesApi = new CompaniesApiAdapter();
export const contactsApi = new ContactsApiAdapter();
export const feesApi = new FeesApiAdapter();