import { writable, derived, get } from 'svelte/store';
import type { 
  Project, 
  Company, 
  Contact, 
  Fee, 
  ConnectionStatus 
} from '../types';
import { mockProjects, mockCompanies, mockContacts, mockFees } from './stores/data';
import { 
  getProjects,
  getCompanies,
  getContacts,
  getFees,
  getStats,
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
} from './api';
import { extractSurrealId } from './utils/surrealdb';

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
export const projectsStore = writable<Project[]>(mockProjects);
export const projectsLoading = writable<boolean>(false);
export const projectsError = writable<string | null>(null);


// Companies store
export const companiesStore = writable<Company[]>(mockCompanies);
export const companiesLoading = writable<boolean>(false);
export const companiesError = writable<string | null>(null);


// Contacts store
export const contactsStore = writable<Contact[]>(mockContacts);
export const contactsLoading = writable<boolean>(false);
export const contactsError = writable<string | null>(null);


// Fees store
export const feesStore = writable<Fee[]>(mockFees);
export const feesLoading = writable<boolean>(false);
export const feesError = writable<string | null>(null);



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
    return $fees.filter(fee => new Date(fee.time.created_at) > thirtyDaysAgo);
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
// ACTION CREATORS (ASYNC OPERATIONS)
// ============================================================================

// Projects actions
export const projectsActions = {
  async load() {
    console.log('projectsActions.load() called');
    projectsLoading.set(true);
    projectsError.set(null);
    
    try {
      console.log('Calling getProjects() API...');
      const projects = await getProjects();
      console.log('getProjects() returned:', projects?.length || 0, 'projects');
      projectsStore.set(projects);
      console.log('projectsStore updated with', projects?.length || 0, 'projects');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load projects';
      projectsError.set(errorMessage);
      console.warn('Failed to load projects from database:', error);
      // Clear store when database connection fails
      projectsStore.set([]);
    } finally {
      projectsLoading.set(false);
    }
  },

  async create(project: any) {
    try {
      const newProject = await createProjectWithTemplate(project);
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

  async update(id: string, projectData: Partial<Project>) {
    try {
      const updatedProject = await updateProject(id, projectData);
      if (updatedProject) {
        projectsStore.update(projects => 
          projects.map(project => {
            // Extract project ID for comparison - handle SurrealDB Thing objects
            let projectId = '';
            if (typeof project.id === 'string') {
              projectId = project.id;
            } else if (project.id && typeof project.id === 'object') {
              const thingObj = project.id as any;
              if (thingObj.tb && thingObj.id) {
                if (typeof thingObj.id === 'string') {
                  projectId = thingObj.id;
                } else if (thingObj.id.String) {
                  projectId = thingObj.id.String;
                }
              }
            }
            
            return projectId === id ? updatedProject : project;
          })
        );
        return updatedProject;
      }
      throw new Error('Failed to update project');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to update project';
      projectsError.set(errorMessage);
      console.error('Failed to update project:', error);
      throw error;
    }
  },

  async delete(id: string) {
    try {
      const deletedProject = await deleteProject(id);
      if (deletedProject) {
        projectsStore.update(projects => 
          projects.filter(project => {
            // Extract project ID for comparison - handle SurrealDB Thing objects
            let projectId = '';
            if (typeof project.id === 'string') {
              projectId = project.id;
            } else if (project.id && typeof project.id === 'object') {
              const thingObj = project.id as any;
              if (thingObj.tb && thingObj.id) {
                if (typeof thingObj.id === 'string') {
                  projectId = thingObj.id;
                } else if (thingObj.id.String) {
                  projectId = thingObj.id.String;
                }
              }
            }
            
            return projectId !== id; // Keep projects that don't match the deleted ID
          })
        );
        return deletedProject;
      }
      throw new Error('Failed to delete project');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to delete project';
      projectsError.set(errorMessage);
      console.error('Failed to delete project:', error);
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
      console.warn('Failed to load companies from database:', error);
      // Clear store when database connection fails
      companiesStore.set([]);
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
      const updatedCompany = await updateCompany(id, companyData);
      if (updatedCompany) {
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
      const deletedCompany = await deleteCompany(id);
      if (deletedCompany) {
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
      const contacts = await getContacts();
      contactsStore.set(contacts);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load contacts';
      contactsError.set(errorMessage);
      console.warn('Failed to load contacts from database:', error);
      // Clear store when database connection fails
      contactsStore.set([]);
    } finally {
      contactsLoading.set(false);
    }
  },

  async create(contact: Omit<Contact, 'id'>) {
    try {
      contactsLoading.set(true);
      contactsError.set('');
      
      const newContact = await createContact(contact);
      
      if (newContact) {
        contactsStore.update(contacts => [...contacts, newContact]);
        return newContact;
      }
      throw new Error('Failed to create contact - API returned null');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to create contact';
      contactsError.set(errorMessage);
      throw error;
    } finally {
      contactsLoading.set(false);
    }
  },

  async update(id: string, contactData: Partial<Contact>) {
    try {
      contactsLoading.set(true);
      contactsError.set('');
      
      const updatedContact = await updateContact(id, contactData);
      
      if (updatedContact) {
        contactsStore.update(contacts => 
          contacts.map(contact => 
            extractSurrealId(contact) === extractSurrealId(updatedContact) ? updatedContact : contact
          )
        );
        return updatedContact; // Added return statement
      } else {
        throw new Error('Update returned null - no contact updated');
      }
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to update contact';
      console.error('Contact update error:', errorMessage);
      contactsError.set(errorMessage);
      throw error;
    } finally {
      contactsLoading.set(false);
    }
  },

  async delete(id: string) {
    try {
      contactsLoading.set(true);
      contactsError.set('');
      
      const deletedContact = await deleteContact(id);
      if (deletedContact) {
        contactsStore.update(contacts => 
          contacts.filter(contact => {
            // Extract contact ID for comparison
            let contactId = '';
            if (typeof contact.id === 'string') {
              contactId = contact.id;
            } else if (contact.id && typeof contact.id === 'object') {
              const thingObj = contact.id as any;
              if (thingObj.tb && thingObj.id) {
                if (typeof thingObj.id === 'string') {
                  contactId = thingObj.id;
                } else if (thingObj.id.String) {
                  contactId = thingObj.id.String;
                }
              }
            }
            
            return contactId !== id; // Keep contacts that don't match the deleted ID
          })
        );
        return deletedContact;
      }
      throw new Error('Failed to delete contact');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to delete contact';
      contactsError.set(errorMessage);
      throw error;
    } finally {
      contactsLoading.set(false);
    }
  },

  async refresh() {
    await this.load();
  }
};

// Fees actions
export const feesActions = {
  async load() {
    feesLoading.set(true);
    feesError.set(null);
    
    try {
      const fees = await getFees();
      feesStore.set(fees);
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to load fees';
      feesError.set(errorMessage);
      console.warn('Failed to load fees from database:', error);
      // Clear store when database connection fails
      feesStore.set([]);
    } finally {
      feesLoading.set(false);
    }
  },

  async create(fee: Omit<Fee, 'id'>) {
    try {
      const newFee = await createFee(fee);
      if (newFee) {
        feesStore.update(fees => [...fees, newFee]);
        return newFee;
      }
      throw new Error('Failed to create fee');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to create fee';
      feesError.set(errorMessage);
      console.error('Failed to create fee:', error);
      throw error;
    }
  },

  async update(id: string, feeData: Partial<Fee>) {
    try {
      feesLoading.set(true);
      feesError.set('');
      
      const updatedFee = await updateFee(id, feeData as Omit<Fee, 'id'>);
      if (updatedFee) {
        feesStore.update(fees => 
          fees.map(fee => {
            // Extract Fee ID for comparison - handle SurrealDB Thing objects
            let feeId = '';
            if (typeof fee.id === 'string') {
              feeId = fee.id;
            } else if (fee.id && typeof fee.id === 'object') {
              const thingObj = fee.id as any;
              if (thingObj.tb && thingObj.id) {
                if (typeof thingObj.id === 'string') {
                  feeId = thingObj.id;
                } else if (thingObj.id.String) {
                  feeId = thingObj.id.String;
                }
              }
            }
            
            return feeId === id ? updatedFee : fee;
          })
        );
        return updatedFee;
      } else {
        throw new Error('Update returned null - no fee updated');
      }
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to update fee';
      feesError.set(errorMessage);
      throw error;
    } finally {
      feesLoading.set(false);
    }
  },

  async updateStatus(id: string, newStatus: string) {
    try {
      feesLoading.set(true);
      feesError.set('');
      
      // Get the current fee to preserve other fields  
      const { get } = await import('svelte/store');
      const currentFees = get(feesStore);
      const currentFee = currentFees.find(fee => {
        // Extract Fee ID for comparison
        let feeId = '';
        if (typeof fee.id === 'string') {
          feeId = fee.id;
        } else if (fee.id && typeof fee.id === 'object') {
          const thingObj = fee.id as any;
          if (thingObj.tb && thingObj.id) {
            if (typeof thingObj.id === 'string') {
              feeId = thingObj.id;
            } else if (thingObj.id.String) {
              feeId = thingObj.id.String;
            }
          }
        }
        return feeId === id || `rfp:${feeId}` === id || feeId === `rfp:${id}`;
      });
      
      if (!currentFee) {
        throw new Error(`Fee with ID ${id} not found`);
      }
      
      // Update only the status field
      const updatedFeeData = {
        ...currentFee,
        status: newStatus,
        time: {
          ...currentFee.time,
          updated_at: new Date().toISOString()
        }
      };
      
      // Remove the id field for the update
      delete (updatedFeeData as any).id;
      
      const updatedFee = await updateFee(id, updatedFeeData);
      if (updatedFee) {
        feesStore.update(fees => 
          fees.map(fee => {
            // Extract Fee ID for comparison
            let feeId = '';
            if (typeof fee.id === 'string') {
              feeId = fee.id;
            } else if (fee.id && typeof fee.id === 'object') {
              const thingObj = fee.id as any;
              if (thingObj.tb && thingObj.id) {
                if (typeof thingObj.id === 'string') {
                  feeId = thingObj.id;
                } else if (thingObj.id.String) {
                  feeId = thingObj.id.String;
                }
              }
            }
            
            return feeId === id ? updatedFee : fee;
          })
        );
        return updatedFee;
      } else {
        throw new Error('Status update returned null');
      }
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to update fee status';
      feesError.set(errorMessage);
      throw error;
    } finally {
      feesLoading.set(false);
    }
  },

  async delete(id: string) {
    try {
      feesLoading.set(true);
      feesError.set('');
      
      const deletedFee = await deleteFee(id);
      if (deletedFee) {
        feesStore.update(fees => 
          fees.filter(fee => {
            // Extract Fee ID for comparison - handle SurrealDB Thing objects
            let feeId = '';
            if (typeof fee.id === 'string') {
              feeId = fee.id;
            } else if (fee.id && typeof fee.id === 'object') {
              const thingObj = fee.id as any;
              if (thingObj.tb && thingObj.id) {
                if (typeof thingObj.id === 'string') {
                  feeId = thingObj.id;
                } else if (thingObj.id.String) {
                  feeId = thingObj.id.String;
                }
              }
            }
            
            return feeId !== id; // Keep Fees that don't match the deleted ID
          })
        );
        return deletedFee;
      }
      throw new Error('Failed to delete fee');
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to delete fee';
      feesError.set(errorMessage);
      throw error;
    } finally {
      feesLoading.set(false);
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
  projectsStore.set([]);
  companiesStore.set([]);
  contactsStore.set([]);
  feesStore.set([]);
  
  // Clear errors
  projectsError.set(null);
  companiesError.set(null);
  contactsError.set(null);
  feesError.set(null);
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
// INITIALIZATION
// ============================================================================

// Note: Data loading is now handled by App.svelte onMount to avoid race conditions