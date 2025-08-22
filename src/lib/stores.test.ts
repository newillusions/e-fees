/**
 * Store Management Tests
 * 
 * Comprehensive test suite for Svelte stores including data stores,
 * derived stores, and store actions.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import {
  projectsStore,
  projectsError,
  projectsActions,
  companiesStore,
  companiesError,
  companiesActions,
  contactsStore,
  contactsError,
  contactsActions,
  feesStore,
  feesError,
  feesActions,
  connectionStore,
  filteredProjectsStore,
  filteredCompaniesStore,
  filteredContactsStore,
  filteredFeesStore,
  companiesWithContactsStore,
  statisticsStore,
  loadAllData
} from './stores';
import type { Project, Company, Contact, Fee, ConnectionState } from '../types';

// Mock the API module
vi.mock('./api', () => ({
  getProjects: vi.fn(),
  getCompanies: vi.fn(),
  getContacts: vi.fn(),
  getFees: vi.fn(),
  getStats: vi.fn(),
  createProjectWithTemplate: vi.fn(),
  updateProject: vi.fn(),
  deleteProject: vi.fn(),
  createCompany: vi.fn(),
  updateCompany: vi.fn(),
  deleteCompany: vi.fn(),
  createContact: vi.fn(),
  updateContact: vi.fn(),
  deleteContact: vi.fn(),
  createFee: vi.fn(),
  updateFee: vi.fn(),
  deleteFee: vi.fn()
}));

// Import mocked functions
import * as api from './api';

// Mock data for testing
const mockProject: Project = {
  id: 'projects:test_project',
  name: 'Test Project',
  name_short: 'Test Proj',
  status: 'Active',
  area: 'Downtown',
  city: 'Abu Dhabi',
  country: 'U.A.E.',
  folder: '25-97101 Test Project',
  number: {
    year: 25,
    country: 971,
    seq: 1,
    id: '25-97101'
  },
  time: {
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString()
  }
};

const mockCompany: Company = {
  id: 'company:TEST',
  name: 'Test Company',
  name_short: 'Test Co',
  abbreviation: 'TEST',
  city: 'Dubai',
  country: 'U.A.E.',
  time: {
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString()
  }
};

const mockContact: Contact = {
  id: 'contacts:test_contact',
  first_name: 'John',
  last_name: 'Doe',
  full_name: 'John Doe',
  email: 'john.doe@example.com',
  phone: '+971501234567',
  position: 'Manager',
  company: 'company:TEST',
  time: {
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString()
  }
};

const mockFee: Fee = {
  id: 'fee:test_fee',
  name: 'Test Fee Proposal',
  number: '25-97101-R1',
  project_id: 'projects:test_project',
  company_id: 'company:TEST',
  contact_id: 'contacts:test_contact',
  status: 'Draft',
  issue_date: '250815',
  rev: 1,
  revisions: [],
  time: {
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString()
  }
};

describe('Store Management', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
    
    // Reset all stores to initial state
    projectsStore.set([]);
    projectsError.set(null);
    companiesStore.set([]);
    companiesError.set(null);
    contactsStore.set([]);
    contactsError.set(null);
    feesStore.set([]);
    feesError.set(null);
    
    const initialConnectionState: ConnectionState = {
      isConnected: false,
      status: 'Disconnected'
    };
    connectionStore.set(initialConnectionState);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Connection Store', () => {
    it('should initialize with disconnected state', () => {
      const state = get(connectionStore);
      
      expect(state.isConnected).toBe(false);
      expect(state.status).toBe('Disconnected');
      expect(state.lastChecked).toBeUndefined();
      expect(state.errorMessage).toBeUndefined();
    });

    it('should update connection state', () => {
      const connectedState: ConnectionState = {
        isConnected: true,
        status: 'Connected',
        lastChecked: new Date(),
        errorMessage: undefined
      };

      connectionStore.set(connectedState);
      const state = get(connectionStore);

      expect(state.isConnected).toBe(true);
      expect(state.status).toBe('Connected');
      expect(state.lastChecked).toBeDefined();
      expect(state.errorMessage).toBeUndefined();
    });

    it('should handle error states', () => {
      const errorState: ConnectionState = {
        isConnected: false,
        status: 'Error',
        lastChecked: new Date(),
        errorMessage: 'Database connection failed'
      };

      connectionStore.set(errorState);
      const state = get(connectionStore);

      expect(state.isConnected).toBe(false);
      expect(state.status).toBe('Error');
      expect(state.errorMessage).toBe('Database connection failed');
    });
  });

  describe('Projects Store', () => {
    describe('Basic Store Operations', () => {
      it('should initialize with empty array', () => {
        const projects = get(projectsStore);
        expect(projects).toEqual([]);
      });

      it('should store projects data', () => {
        const mockProjects = [mockProject];
        projectsStore.set(mockProjects);
        
        const projects = get(projectsStore);
        expect(projects).toEqual(mockProjects);
        expect(projects).toHaveLength(1);
      });

      it('should handle error states', () => {
        const errorMessage = 'Failed to load projects';
        projectsError.set(errorMessage);
        
        const error = get(projectsError);
        expect(error).toBe(errorMessage);
      });
    });

    describe('Projects Actions', () => {
      it('should load projects successfully', async () => {
        const mockProjects = [mockProject];
        vi.mocked(api.getProjects).mockResolvedValueOnce(mockProjects);

        await projectsActions.load();

        expect(api.getProjects).toHaveBeenCalled();
        expect(get(projectsStore)).toEqual(mockProjects);
        expect(get(projectsError)).toBeNull();
      });

      it('should handle load failures', async () => {
        const errorMessage = 'Database connection failed';
        vi.mocked(api.getProjects).mockRejectedValueOnce(new Error(errorMessage));

        await projectsActions.load();

        expect(api.getProjects).toHaveBeenCalled();
        expect(get(projectsStore)).toEqual([]);
        expect(get(projectsError)).toContain(errorMessage);
      });

      it('should create project successfully', async () => {
        const newProject = {
          name: 'New Project',
          name_short: 'New Proj',
          status: 'Draft' as const,
          area: 'Marina',
          city: 'Dubai',
          country: 'U.A.E.',
          folder: '25-97102 New Project',
          number: {
            year: 25,
            country: 971,
            seq: 2,
            id: '25-97102'
          }
        };
        
        const createdProject = { ...mockProject, ...newProject, id: 'projects:new_project' };
        vi.mocked(api.createProjectWithTemplate).mockResolvedValueOnce(createdProject);
        vi.mocked(api.getProjects).mockResolvedValueOnce([mockProject, createdProject]);

        await projectsActions.create(newProject);

        expect(api.createProjectWithTemplate).toHaveBeenCalledWith(newProject);
        expect(get(projectsStore)).toContain(createdProject);
      });

      it('should handle create failures', async () => {
        // Reset stores to ensure clean state
        projectsStore.set([]);
        projectsError.set(null);
        
        const errorMessage = 'Project creation failed';
        const newProject = {
          name: 'Failed Project',
          name_short: 'Failed',
          status: 'Draft' as const,
          area: 'Test Area',
          city: 'Test City',
          country: 'Test Country',
          folder: '25-97199 Failed Project',
          number: {
            year: 25,
            country: 971,
            seq: 99,
            id: '25-97199'
          }
        };

        // Mock the API function to reject with an error
        vi.mocked(api.createProjectWithTemplate).mockRejectedValueOnce(new Error(errorMessage));

        // The create action should throw an error and set the error state
        await expect(projectsActions.create(newProject)).rejects.toThrow(errorMessage);
        
        // Verify that the action was called with the correct parameter
        expect(api.createProjectWithTemplate).toHaveBeenCalledWith(newProject);
        
        // Verify that the error state was set correctly
        expect(get(projectsError)).toContain(errorMessage);
        
        // Verify that the store was not updated with invalid data
        expect(get(projectsStore)).toEqual([]);
      });

      it('should update project successfully', async () => {
        projectsStore.set([mockProject]);
        const updates = { status: 'Completed' as const };
        const updatedProject = { ...mockProject, ...updates };
        
        vi.mocked(api.updateProject).mockResolvedValueOnce(updatedProject);
        vi.mocked(api.getProjects).mockResolvedValueOnce([updatedProject]);

        await projectsActions.update('projects:test_project', updates);

        expect(api.updateProject).toHaveBeenCalledWith('projects:test_project', updates);
        expect(get(projectsStore)).toContain(updatedProject);
      });

      it('should delete project successfully', async () => {
        projectsStore.set([mockProject]);
        
        vi.mocked(api.deleteProject).mockResolvedValueOnce(mockProject);
        vi.mocked(api.getProjects).mockResolvedValueOnce([]);

        await projectsActions.delete('projects:test_project');

        expect(api.deleteProject).toHaveBeenCalledWith('projects:test_project');
        expect(get(projectsStore)).toEqual([]);
      });
    });
  });

  describe('Companies Store', () => {
    describe('Companies Actions', () => {
      it('should load companies successfully', async () => {
        const mockCompanies = [mockCompany];
        vi.mocked(api.getCompanies).mockResolvedValueOnce(mockCompanies);

        await companiesActions.load();

        expect(api.getCompanies).toHaveBeenCalled();
        expect(get(companiesStore)).toEqual(mockCompanies);
        expect(get(companiesError)).toBeNull();
      });

      it('should create company successfully', async () => {
        const newCompany = {
          name: 'New Company',
          name_short: 'New Co',
          abbreviation: 'NEW',
          city: 'Dubai',
          country: 'U.A.E.'
        };
        
        const createdCompany = { ...mockCompany, ...newCompany, id: 'company:NEW' };
        vi.mocked(api.createCompany).mockResolvedValueOnce(createdCompany);
        vi.mocked(api.getCompanies).mockResolvedValueOnce([mockCompany, createdCompany]);

        await companiesActions.create(newCompany);

        expect(api.createCompany).toHaveBeenCalledWith(newCompany);
        expect(get(companiesStore)).toContain(createdCompany);
      });

      it('should update company successfully', async () => {
        companiesStore.set([mockCompany]);
        const updates = { name: 'Updated Company Name' };
        const updatedCompany = { ...mockCompany, ...updates };
        
        vi.mocked(api.updateCompany).mockResolvedValueOnce(updatedCompany);
        vi.mocked(api.getCompanies).mockResolvedValueOnce([updatedCompany]);

        await companiesActions.update('company:TEST', updates);

        expect(api.updateCompany).toHaveBeenCalledWith('company:TEST', updates);
        expect(get(companiesStore)).toContain(updatedCompany);
      });

      it('should delete company successfully', async () => {
        companiesStore.set([mockCompany]);
        
        vi.mocked(api.deleteCompany).mockResolvedValueOnce(mockCompany);
        vi.mocked(api.getCompanies).mockResolvedValueOnce([]);

        await companiesActions.delete('company:TEST');

        expect(api.deleteCompany).toHaveBeenCalledWith('company:TEST');
        expect(get(companiesStore)).toEqual([]);
      });
    });
  });

  describe('Contacts Store', () => {
    describe('Contacts Actions', () => {
      it('should load contacts successfully', async () => {
        const mockContacts = [mockContact];
        vi.mocked(api.getContacts).mockResolvedValueOnce(mockContacts);

        await contactsActions.load();

        expect(api.getContacts).toHaveBeenCalled();
        expect(get(contactsStore)).toEqual(mockContacts);
        expect(get(contactsError)).toBeNull();
      });

      it('should create contact successfully', async () => {
        const newContact = {
          first_name: 'Jane',
          last_name: 'Smith',
          email: 'jane.smith@example.com',
          phone: '+971507654321',
          position: 'Director',
          company: 'company:TEST'
        };
        
        const createdContact = {
          ...mockContact,
          ...newContact,
          full_name: 'Jane Smith',
          id: 'contacts:new_contact'
        };
        vi.mocked(api.createContact).mockResolvedValueOnce(createdContact);
        vi.mocked(api.getContacts).mockResolvedValueOnce([mockContact, createdContact]);

        await contactsActions.create(newContact);

        expect(api.createContact).toHaveBeenCalledWith(newContact);
        expect(get(contactsStore)).toContain(createdContact);
      });

      it('should update contact successfully', async () => {
        contactsStore.set([mockContact]);
        const updates = { position: 'Senior Manager' };
        const updatedContact = { ...mockContact, ...updates };
        
        vi.mocked(api.updateContact).mockResolvedValueOnce(updatedContact);
        vi.mocked(api.getContacts).mockResolvedValueOnce([updatedContact]);

        await contactsActions.update('contacts:test_contact', updates);

        expect(api.updateContact).toHaveBeenCalledWith('contacts:test_contact', updates);
        expect(get(contactsStore)).toContain(updatedContact);
      });

      it('should delete contact successfully', async () => {
        contactsStore.set([mockContact]);
        
        vi.mocked(api.deleteContact).mockResolvedValueOnce(mockContact);
        vi.mocked(api.getContacts).mockResolvedValueOnce([]);

        await contactsActions.delete('contacts:test_contact');

        expect(api.deleteContact).toHaveBeenCalledWith('contacts:test_contact');
        expect(get(contactsStore)).toEqual([]);
      });
    });
  });

  describe('Fees Store', () => {
    describe('Fees Actions', () => {
      it('should load fees successfully', async () => {
        const mockFees = [mockFee];
        vi.mocked(api.getFees).mockResolvedValueOnce(mockFees);

        await feesActions.load();

        expect(api.getFees).toHaveBeenCalled();
        expect(get(feesStore)).toEqual(mockFees);
        expect(get(feesError)).toBeNull();
      });

      it('should create fee successfully', async () => {
        const newFee = {
          name: 'New Fee Proposal',
          number: '25-97102-R1',
          project_id: 'projects:test_project',
          company_id: 'company:TEST',
          contact_id: 'contacts:test_contact',
          status: 'Draft' as const,
          issue_date: '250816'
        };
        
        const createdFee = { ...mockFee, ...newFee, id: 'fee:new_fee' };
        vi.mocked(api.createFee).mockResolvedValueOnce(createdFee);
        vi.mocked(api.getFees).mockResolvedValueOnce([mockFee, createdFee]);

        await feesActions.create(newFee);

        expect(api.createFee).toHaveBeenCalledWith(newFee);
        expect(get(feesStore)).toContain(createdFee);
      });

      it('should update fee successfully', async () => {
        feesStore.set([mockFee]);
        const updates = { status: 'Sent' as const };
        const updatedFee = { ...mockFee, ...updates };
        
        vi.mocked(api.updateFee).mockResolvedValueOnce(updatedFee);
        vi.mocked(api.getFees).mockResolvedValueOnce([updatedFee]);

        await feesActions.update('fee:test_fee', updates);

        expect(api.updateFee).toHaveBeenCalledWith('fee:test_fee', updates);
        expect(get(feesStore)).toContain(updatedFee);
      });

      it('should delete fee successfully', async () => {
        feesStore.set([mockFee]);
        
        vi.mocked(api.deleteFee).mockResolvedValueOnce(mockFee);
        vi.mocked(api.getFees).mockResolvedValueOnce([]);

        await feesActions.delete('fee:test_fee');

        expect(api.deleteFee).toHaveBeenCalledWith('fee:test_fee');
        expect(get(feesStore)).toEqual([]);
      });
    });
  });

  describe('Derived Stores', () => {
    describe('companiesWithContactsStore', () => {
      it('should calculate contact counts for companies', () => {
        companiesStore.set([mockCompany]);
        contactsStore.set([mockContact]);

        const companiesWithContacts = get(companiesWithContactsStore);

        expect(companiesWithContacts).toHaveLength(1);
        expect(companiesWithContacts[0].id).toBe(mockCompany.id);
        expect(companiesWithContacts[0].contactCount).toBe(1);
      });

      it('should handle companies with no contacts', () => {
        const anotherCompany = { ...mockCompany, id: 'company:OTHER', abbreviation: 'OTHER' };
        companiesStore.set([mockCompany, anotherCompany]);
        contactsStore.set([mockContact]); // Only links to TEST company

        const companiesWithContacts = get(companiesWithContactsStore);

        expect(companiesWithContacts).toHaveLength(2);
        expect(companiesWithContacts[0].contactCount).toBe(1); // TEST company
        expect(companiesWithContacts[1].contactCount).toBe(0); // OTHER company
      });

      it('should update when stores change', () => {
        companiesStore.set([mockCompany]);
        contactsStore.set([]);

        let companiesWithContacts = get(companiesWithContactsStore);
        expect(companiesWithContacts[0].contactCount).toBe(0);

        // Add a contact
        contactsStore.set([mockContact]);
        companiesWithContacts = get(companiesWithContactsStore);
        expect(companiesWithContacts[0].contactCount).toBe(1);
      });
    });

    describe('statisticsStore', () => {
      it('should calculate total counts across all stores', () => {
        projectsStore.set([mockProject]);
        companiesStore.set([mockCompany]);
        contactsStore.set([mockContact]);
        feesStore.set([mockFee]);

        const totals = get(statisticsStore);

        expect(totals.totalProjects).toBe(1);
        expect(totals.totalCompanies).toBe(1);
        expect(totals.totalContacts).toBe(1);
        expect(totals.totalFees).toBe(1);
      });

      it('should handle empty stores', () => {
        projectsStore.set([]);
        companiesStore.set([]);
        contactsStore.set([]);
        feesStore.set([]);

        const totals = get(statisticsStore);

        expect(totals.totalProjects).toBe(0);
        expect(totals.totalCompanies).toBe(0);
        expect(totals.totalContacts).toBe(0);
        expect(totals.totalFees).toBe(0);
      });

      it('should update when stores change', () => {
        // Start with empty stores
        projectsStore.set([]);
        companiesStore.set([]);
        contactsStore.set([]);
        feesStore.set([]);

        let totals = get(statisticsStore);
        expect(totals.totalProjects + totals.totalCompanies + totals.totalContacts + totals.totalFees).toBe(0);

        // Add data to stores
        projectsStore.set([mockProject]);
        companiesStore.set([mockCompany]);

        totals = get(statisticsStore);
        expect(totals.totalProjects + totals.totalCompanies).toBe(2);
      });
    });
  });

  describe('Global Data Loading', () => {
    it('should load all data successfully', async () => {
      const mockProjects = [mockProject];
      const mockCompanies = [mockCompany];
      const mockContacts = [mockContact];
      const mockFees = [mockFee];

      vi.mocked(api.getProjects).mockResolvedValueOnce(mockProjects);
      vi.mocked(api.getCompanies).mockResolvedValueOnce(mockCompanies);
      vi.mocked(api.getContacts).mockResolvedValueOnce(mockContacts);
      vi.mocked(api.getFees).mockResolvedValueOnce(mockFees);

      await loadAllData();

      expect(get(projectsStore)).toEqual(mockProjects);
      expect(get(companiesStore)).toEqual(mockCompanies);
      expect(get(contactsStore)).toEqual(mockContacts);
      expect(get(feesStore)).toEqual(mockFees);

      // All error states should be clear
      expect(get(projectsError)).toBeNull();
      expect(get(companiesError)).toBeNull();
      expect(get(contactsError)).toBeNull();
      expect(get(feesError)).toBeNull();
    });

    it('should handle partial failures gracefully', async () => {
      const mockProjects = [mockProject];
      const mockCompanies = [mockCompany];

      vi.mocked(api.getProjects).mockResolvedValueOnce(mockProjects);
      vi.mocked(api.getCompanies).mockResolvedValueOnce(mockCompanies);
      vi.mocked(api.getContacts).mockRejectedValueOnce(new Error('Contacts failed'));
      vi.mocked(api.getFees).mockRejectedValueOnce(new Error('Fees failed'));

      await loadAllData();

      // Successful loads should work
      expect(get(projectsStore)).toEqual(mockProjects);
      expect(get(companiesStore)).toEqual(mockCompanies);

      // Failed loads should result in empty arrays and error messages
      expect(get(contactsStore)).toEqual([]);
      expect(get(feesStore)).toEqual([]);
      expect(get(contactsError)).toContain('Contacts failed');
      expect(get(feesError)).toContain('Fees failed');
    });
  });

  describe('Store Reactivity', () => {
    it('should trigger reactive updates when data changes', () => {
      let updateCount = 0;
      
      // Subscribe to store changes
      const unsubscribe = projectsStore.subscribe(() => {
        updateCount++;
      });

      // Initial subscription triggers once
      expect(updateCount).toBe(1);

      // Set new data should trigger update
      projectsStore.set([mockProject]);
      expect(updateCount).toBe(2);

      // Set same data should still trigger update (Svelte behavior)
      projectsStore.set([mockProject]);
      expect(updateCount).toBe(3);

      unsubscribe();
    });

    it('should handle derived store updates', () => {
      let updateCount = 0;
      
      const unsubscribe = statisticsStore.subscribe(() => {
        updateCount++;
      });

      // Initial subscription
      expect(updateCount).toBe(1);

      // Update base stores should trigger derived store update
      projectsStore.set([mockProject]);
      expect(updateCount).toBe(2);

      companiesStore.set([mockCompany]);
      expect(updateCount).toBe(3);

      unsubscribe();
    });
  });

  describe('Error Handling', () => {
    it('should handle API errors gracefully', async () => {
      const errorMessage = 'Network error';
      vi.mocked(api.getProjects).mockRejectedValueOnce(new Error(errorMessage));

      await projectsActions.load();

      expect(get(projectsStore)).toEqual([]);
      expect(get(projectsError)).toContain(errorMessage);
    });

    it('should reset error states on successful operations', async () => {
      // Set initial error state
      projectsError.set('Previous error');

      // Successful operation should clear error
      const mockProjects = [mockProject];
      vi.mocked(api.getProjects).mockResolvedValueOnce(mockProjects);

      await projectsActions.load();

      expect(get(projectsStore)).toEqual(mockProjects);
      expect(get(projectsError)).toBeNull();
    });

    it('should handle null/undefined API responses', async () => {
      vi.mocked(api.getProjects).mockResolvedValueOnce(null as any);

      await projectsActions.load();

      expect(get(projectsStore)).toBeNull();
    });
  });
});