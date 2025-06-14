import { invoke } from '@tauri-apps/api/core';
import type { 
  Project, 
  Company, 
  Contact, 
  Rfp, 
  ConnectionStatus 
} from '../types';

// API wrapper for Tauri commands
export class ApiClient {
  
  // Database connection methods
  static async checkDbConnection(): Promise<boolean> {
    try {
      return await invoke<boolean>('check_db_connection');
    } catch (error) {
      console.error('Failed to check database connection:', error);
      return false;
    }
  }

  static async getConnectionStatus(): Promise<ConnectionStatus> {
    try {
      return await invoke<ConnectionStatus>('get_connection_status');
    } catch (error) {
      console.error('Failed to get connection status:', error);
      return {
        is_connected: false,
        last_check: undefined,
        error_message: error?.toString() || 'Unknown error'
      };
    }
  }

  // Project methods
  static async getProjects(): Promise<Project[]> {
    try {
      console.log('Invoking get_projects command...');
      const projects = await invoke<Project[]>('get_projects');
      console.log('get_projects response:', projects);
      return projects;
    } catch (error) {
      console.error('Failed to fetch projects from database:', error);
      throw error;
    }
  }

  static async searchProjects(query: string): Promise<Project[]> {
    try {
      console.log('Searching projects with query:', query);
      const projects = await invoke<Project[]>('search_projects', { query });
      console.log('search_projects response:', projects);
      return projects;
    } catch (error) {
      console.error('Failed to search projects:', error);
      throw error;
    }
  }

  static async createProject(project: Omit<Project, 'id'>): Promise<Project | null> {
    try {
      const newProject = {
        ...project,
        id: null,
        time: {
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        }
      };
      
      const created = await invoke<Project>('create_project', { project: newProject });
      return created;
    } catch (error) {
      console.error('Failed to create project:', error);
      return null;
    }
  }

  // Company methods
  static async getCompanies(): Promise<Company[]> {
    try {
      console.log('Invoking get_companies command...');
      const companies = await invoke<Company[]>('get_companies');
      console.log('get_companies response:', companies);
      return companies;
    } catch (error) {
      console.error('Failed to fetch companies from database:', error);
      throw error;
    }
  }

  static async createCompany(company: Omit<Company, 'id'>): Promise<Company | null> {
    try {
      const newCompany = {
        ...company,
        id: null
      };
      
      const created = await invoke<Company>('create_company', { company: newCompany });
      return created;
    } catch (error) {
      console.error('Failed to create company:', error);
      return null;
    }
  }

  // Contact methods
  static async getContacts(): Promise<Contact[]> {
    try {
      console.log('Invoking get_contacts command...');
      const contacts = await invoke<Contact[]>('get_contacts');
      console.log('get_contacts response:', contacts);
      return contacts;
    } catch (error) {
      console.error('Failed to fetch contacts from database:', error);
      throw error;
    }
  }

  static async createContact(contact: Omit<Contact, 'id'>): Promise<Contact | null> {
    try {
      const newContact = {
        ...contact,
        id: null
      };
      
      const created = await invoke<Contact>('create_contact', { contact: newContact });
      return created;
    } catch (error) {
      console.error('Failed to create contact:', error);
      return null;
    }
  }

  // RFP methods
  static async getRfps(): Promise<Rfp[]> {
    try {
      console.log('Invoking get_rfps command...');
      const rfps = await invoke<Rfp[]>('get_rfps');
      console.log('get_rfps response:', rfps);
      return rfps;
    } catch (error) {
      console.error('Failed to fetch rfps from database:', error);
      throw error;
    }
  }

  static async createRfp(rfp: Omit<Rfp, 'id'>): Promise<Rfp | null> {
    try {
      const newRfp = {
        ...rfp,
        id: null,
        time: {
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        }
      };
      
      const created = await invoke<Rfp>('create_rfp', { rfp: newRfp });
      return created;
    } catch (error) {
      console.error('Failed to create rfp:', error);
      return null;
    }
  }

  // Statistics
  static async getStats(): Promise<{
    totalProjects: number;
    activeRfps: number;
    totalCompanies: number;
    totalContacts: number;
    totalRfps: number;
  }> {
    try {
      console.log('Invoking get_stats command...');
      const stats = await invoke('get_stats');
      console.log('get_stats response:', stats);
      return stats as any;
    } catch (error) {
      console.error('Failed to fetch stats from database:', error);
      throw error;
    }
  }

  // Health check
  static async healthCheck(): Promise<string> {
    try {
      return await invoke<string>('health_check');
    } catch (error) {
      console.error('Health check failed:', error);
      return 'Health check failed';
    }
  }

  // Get detailed database connection info for debugging
  static async getDbInfo(): Promise<any> {
    try {
      return await invoke('get_db_info');
    } catch (error) {
      console.error('Failed to get database info:', error);
      return {
        error: error?.toString() || 'Unknown error',
        timestamp: new Date().toISOString()
      };
    }
  }

  // Get table schema information
  static async getTableSchema(tableName: string): Promise<any> {
    try {
      return await invoke('get_table_schema', { tableName });
    } catch (error) {
      console.error(`Failed to get schema for table ${tableName}:`, error);
      return {
        error: error?.toString() || 'Unknown error',
        table: tableName,
        timestamp: new Date().toISOString()
      };
    }
  }

  // Position window for 4K monitor
  static async positionWindow4K(): Promise<string> {
    try {
      return await invoke<string>('position_window_4k');
    } catch (error) {
      console.error('Failed to position window:', error);
      return 'Failed to position window';
    }
  }

  // File system operations
  static async selectFolder(): Promise<string | null> {
    try {
      console.log('Opening folder picker dialog...');
      const folder = await invoke<string | null>('select_folder');
      console.log('Selected folder:', folder);
      return folder;
    } catch (error) {
      console.error('Failed to open folder picker:', error);
      return null;
    }
  }

  static async openFolderInExplorer(folderPath: string): Promise<string> {
    try {
      console.log('Opening folder in explorer:', folderPath);
      const result = await invoke<string>('open_folder_in_explorer', { folderPath });
      console.log('Folder opened successfully');
      return result;
    } catch (error) {
      console.error('Failed to open folder in explorer:', error);
      return 'Failed to open folder';
    }
  }
}

// Export individual functions for convenience
export const {
  checkDbConnection,
  getConnectionStatus,
  getProjects,
  searchProjects,
  createProject,
  getCompanies,
  createCompany,
  getContacts,
  createContact,
  getRfps,
  createRfp,
  getStats,
  healthCheck,
  getDbInfo,
  getTableSchema,
  positionWindow4K,
  selectFolder,
  openFolderInExplorer
} = ApiClient;

// Export default
export default ApiClient;