/**
 * API Client Module
 * 
 * This module provides a comprehensive TypeScript wrapper around Tauri commands
 * for seamless frontend-backend communication. It serves as the primary interface
 * between the Svelte frontend and the Rust backend database operations.
 * 
 * Features:
 * - Type-safe command invocation with TypeScript generics
 * - Comprehensive error handling with detailed logging
 * - Automatic data serialization/deserialization
 * - Connection status monitoring and health checks
 * - Project workflow automation (numbering, templates, etc.)
 * - File system integration for template management
 * 
 * Architecture:
 * The ApiClient follows a static class pattern where each method corresponds
 * to a Tauri command defined in the Rust backend. All methods are async and
 * return Promise types for proper error handling in the frontend.
 * 
 * Error Handling:
 * - Database errors are logged and re-thrown to calling components
 * - Connection failures return safe fallback values
 * - All operations include comprehensive console logging for debugging
 * 
 * @fileoverview Main API interface for database and system operations
 * @author Fee Proposal Management System
 * @version 2.0.0
 */

import { invoke } from '@tauri-apps/api/core';
import type { 
  Project, 
  Company, 
  Contact, 
  Rfp, 
  ConnectionStatus,
  DatabaseStats,
  DatabaseInfo,
  TableSchema,
  CountrySearchResult,
  ProjectCreationResult,
  FileOperationResult
} from '../types';

/**
 * Primary API client class providing static methods for all backend operations.
 * 
 * This class serves as the central hub for all database operations, system
 * commands, and file management functions. Each method corresponds directly
 * to a Tauri command in the Rust backend.
 * 
 * Usage Pattern:
 * ```typescript
 * // Import the client
 * import { ApiClient } from '$lib/api';
 * 
 * // Use static methods directly
 * const projects = await ApiClient.getProjects();
 * const status = await ApiClient.getConnectionStatus();
 * ```
 * 
 * Error Handling:
 * All methods implement comprehensive error handling with console logging.
 * Critical operations throw errors to be handled by calling components,
 * while status checks return safe fallback values.
 */
export class ApiClient {
  
  // ============================================================================
  // DATABASE CONNECTION METHODS
  // ============================================================================

  /**
   * Performs a simple database connection test.
   * 
   * This method attempts to establish a connection to the SurrealDB instance
   * and returns a boolean indicating success or failure. It's used for basic
   * connectivity testing without detailed status information.
   * 
   * @returns Promise<boolean> - True if connection successful, false otherwise
   * 
   * @example
   * ```typescript
   * const isConnected = await ApiClient.checkDbConnection();
   * if (isConnected) {
   *   console.log('Database is accessible');
   * } else {
   *   console.log('Cannot connect to database');
   * }
   * ```
   * 
   * @throws Never throws - always returns a boolean value for safety
   */
  static async checkDbConnection(): Promise<boolean> {
    try {
      return await invoke<boolean>('check_db_connection');
    } catch (error) {
      console.error('Failed to check database connection:', error);
      return false;
    }
  }

  /**
   * Retrieves comprehensive database connection status information.
   * 
   * This method returns detailed information about the current database
   * connection state, including connection status, last check timestamp,
   * and any error messages. Used by the ConnectionStatus component.
   * 
   * @returns Promise<ConnectionStatus> - Detailed connection information
   * 
   * @example
   * ```typescript
   * const status = await ApiClient.getConnectionStatus();
   * console.log(`Connected: ${status.is_connected}`);
   * console.log(`Last check: ${status.last_check}`);
   * if (!status.is_connected) {
   *   console.error(`Error: ${status.error_message}`);
   * }
   * ```
   * 
   * Connection Status Structure:
   * - `is_connected`: Boolean indicating current connection state
   * - `last_check`: Timestamp of last connection attempt
   * - `error_message`: Human-readable error description if disconnected
   * 
   * @throws Never throws - returns safe fallback on error
   */
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

  // ============================================================================
  // PROJECT MANAGEMENT METHODS
  // ============================================================================

  /**
   * Retrieves all projects from the database.
   * 
   * This method fetches the complete list of projects from the SurrealDB
   * projects table, including all metadata and relationships. Projects are
   * returned with their full project numbers, status information, and
   * location details.
   * 
   * @returns Promise<Project[]> - Array of all project records
   * 
   * @example
   * ```typescript
   * try {
   *   const projects = await ApiClient.getProjects();
   *   console.log(`Found ${projects.length} projects`);
   *   projects.forEach(project => {
   *     console.log(`${project.number.id}: ${project.name}`);
   *   });
   * } catch (error) {
   *   console.error('Failed to load projects:', error);
   * }
   * ```
   * 
   * Project Structure:
   * - `id`: SurrealDB record identifier
   * - `name`: Full project name
   * - `name_short`: Abbreviated name for folders
   * - `number`: Project number object (year, country, seq, id)
   * - `status`: Current project status (Draft, RFP, Active, etc.)
   * - `area`, `city`, `country`: Location information
   * - `folder`: File system path for project files
   * - `time`: Creation and modification timestamps
   * 
   * @throws Error - Re-throws database connection or query errors
   */
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

  /**
   * Searches projects by name, location, or project number.
   * 
   * This method performs a fuzzy search across project fields including
   * name, location information, and project numbers. It's used by the
   * search functionality in the Projects page to filter results in real-time.
   * 
   * @param query - Search term to match against project fields
   * @returns Promise<Project[]> - Array of matching project records
   * 
   * @example
   * ```typescript
   * // Search by project name
   * const museums = await ApiClient.searchProjects('museum');
   * 
   * // Search by location
   * const dubaiProjects = await ApiClient.searchProjects('dubai');
   * 
   * // Search by project number
   * const project25 = await ApiClient.searchProjects('25-971');
   * ```
   * 
   * Search Behavior:
   * - Case-insensitive partial matching
   * - Searches across name, area, city, country fields
   * - Supports project number pattern matching
   * - Returns empty array if no matches found
   * 
   * @throws Error - Re-throws database connection or query errors
   */
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

  /**
   * Creates a new project record in the database.
   * 
   * This method creates a new project with automatically generated timestamps
   * and ensures all required fields are properly validated before insertion.
   * Used by the basic project creation workflow.
   * 
   * @param project - Project data without ID (will be generated by database)
   * @returns Promise<Project | null> - Created project or null on failure
   * 
   * @example
   * ```typescript
   * const newProject = {
   *   name: 'Dubai Museum of Future',
   *   name_short: 'DMOF',
   *   area: 'Business Bay',
   *   city: 'Dubai',
   *   country: 'United Arab Emirates',
   *   folder: 'E:\\Projects\\25-97105',
   *   status: 'Draft',
   *   number: {
   *     year: 25,
   *     country: 971,
   *     seq: 5,
   *     id: '25-97105'
   *   }
   * };
   * 
   * try {
   *   const created = await ApiClient.createProject(newProject);
   *   if (created) {
   *     console.log('Project created:', created.id);
   *   }
   * } catch (error) {
   *   console.error('Creation failed:', error);
   * }
   * ```
   * 
   * Automatic Fields:
   * - `id`: Generated by SurrealDB
   * - `time.created_at`: Set to current timestamp
   * - `time.updated_at`: Set to current timestamp
   * 
   * @throws Never throws - returns null on error for safe handling
   */
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

  // ============================================================================
  // COMPANY MANAGEMENT METHODS
  // ============================================================================

  /**
   * Retrieves all companies from the database.
   * 
   * This method fetches the complete list of companies from the SurrealDB
   * company table, including all contact information and metadata. Companies
   * are used for project assignments and proposal management.
   * 
   * @returns Promise<Company[]> - Array of all company records
   * 
   * @example
   * ```typescript
   * try {
   *   const companies = await ApiClient.getCompanies();
   *   console.log(`Found ${companies.length} companies`);
   *   companies.forEach(company => {
   *     console.log(`${company.abbreviation}: ${company.name}`);
   *   });
   * } catch (error) {
   *   console.error('Failed to load companies:', error);
   * }
   * ```
   * 
   * Company Structure:
   * - `id`: SurrealDB record identifier
   * - `name`: Full company name
   * - `name_short`: Abbreviated company name
   * - `abbreviation`: Short code (e.g., "CHE", "DMCC")
   * - `city`, `country`: Company headquarters location
   * - `reg_no`, `tax_no`: Optional registration numbers
   * - `time`: Creation and modification timestamps
   * 
   * @throws Error - Re-throws database connection or query errors
   */
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

  /**
   * Creates a new company record in the database.
   * 
   * This method creates a new company with automatic timestamp generation
   * and validates all required fields before insertion. Used by the
   * company creation modal and import processes.
   * 
   * @param company - Company data without ID (will be generated by database)
   * @returns Promise<Company | null> - Created company or null on failure
   * 
   * @example
   * ```typescript
   * const newCompany = {
   *   name: 'Conrad Hilton Hotels',
   *   name_short: 'Conrad Etihad',
   *   abbreviation: 'CHE',
   *   city: 'Dubai',
   *   country: 'United Arab Emirates',
   *   reg_no: 'DM-1234567',
   *   tax_no: 'TRN-987654321'
   * };
   * 
   * try {
   *   const created = await ApiClient.createCompany(newCompany);
   *   if (created) {
   *     console.log('Company created:', created.id);
   *   }
   * } catch (error) {
   *   console.error('Creation failed:', error);
   * }
   * ```
   * 
   * Required Fields:
   * - `name`: Full legal company name
   * - `name_short`: Display name for UI
   * - `abbreviation`: Unique short code
   * - `city`, `country`: Headquarters location
   * 
   * @throws Never throws - returns null on error for safe handling
   */
  static async createCompany(company: Omit<Company, 'id' | 'time'>): Promise<Company | null> {
    try {
      // Only send the fields needed for creation (exclude auto-managed id and time)
      const newCompany = {
        name: company.name,
        name_short: company.name_short,
        abbreviation: company.abbreviation,
        city: company.city,
        country: company.country,
        reg_no: company.reg_no || null,
        tax_no: company.tax_no || null
      };
      
      const created = await invoke<Company>('create_company', { company: newCompany });
      return created;
    } catch (error) {
      console.error('Failed to create company:', error);
      return null;
    }
  }
  
  /**
   * Updates an existing company record with partial data.
   * 
   * This method performs a partial update of a company record, only modifying
   * the fields provided in the update object. Uses SurrealDB's MERGE operation
   * for efficient partial updates without overwriting existing data.
   * 
   * @param id - SurrealDB record ID of the company to update
   * @param company - Partial company data (only changed fields)
   * @returns Promise<Company | null> - Updated company record
   * 
   * @example
   * ```typescript
   * // Update only the city and tax number
   * try {
   *   const updated = await ApiClient.updateCompany('company:abc123', {
   *     city: 'Abu Dhabi',
   *     tax_no: 'TRN-111222333'
   *   });
   *   
   *   if (updated) {
   *     console.log('Company updated successfully');
   *   }
   * } catch (error) {
   *   console.error('Update failed:', error);
   * }
   * ```
   * 
   * Update Behavior:
   * - Only provided fields are updated
   * - Null values clear existing data
   * - Undefined/missing fields are ignored
   * - `updated_at` timestamp is automatically set
   * 
   * @throws Error - Re-throws database or validation errors
   */
  static async updateCompany(id: string, company: Partial<Company>): Promise<Company | null> {
    try {
      console.log('API updateCompany called with:');
      console.log('ID:', id);
      console.log('Company data:', company);
      
      // Convert to the structure expected by backend
      const companyUpdate = {
        name: company.name || null,
        name_short: company.name_short || null,
        abbreviation: company.abbreviation || null,
        city: company.city || null,
        country: company.country || null,
        reg_no: company.reg_no || null,
        tax_no: company.tax_no || null
      };
      
      console.log('Sending companyUpdate:', companyUpdate);
      const updated = await invoke<Company>('update_company', { id, companyUpdate });
      console.log('Update response:', updated);
      return updated;
    } catch (error) {
      console.error('Failed to update company:', error);
      throw error; // Throw instead of returning null so errors are properly caught
    }
  }
  
  /**
   * Deletes a company record from the database.
   * 
   * This method permanently removes a company record from the database.
   * Before deletion, the backend checks for any references from contacts
   * or projects to prevent orphaned data.
   * 
   * @param id - SurrealDB record ID of the company to delete
   * @returns Promise<Company | null> - Deleted company record for confirmation
   * 
   * @example
   * ```typescript
   * try {
   *   const deleted = await ApiClient.deleteCompany('company:abc123');
   *   if (deleted) {
   *     console.log(`Company "${deleted.name}" deleted successfully`);
   *   }
   * } catch (error) {
   *   if (error.message.includes('referenced')) {
   *     console.error('Cannot delete: Company has associated contacts/projects');
   *   } else {
   *     console.error('Delete failed:', error);
   *   }
   * }
   * ```
   * 
   * Safety Features:
   * - Checks for referential integrity before deletion
   * - Returns the deleted record for confirmation
   * - Provides detailed error messages for constraint violations
   * 
   * @throws Error - Throws on constraint violations or database errors
   */
  static async deleteCompany(id: string): Promise<Company | null> {
    try {
      console.log('API deleteCompany called with ID:', id);
      const deleted = await invoke<Company>('delete_company', { id });
      console.log('Delete response:', deleted);
      return deleted;
    } catch (error) {
      console.error('Failed to delete company:', error);
      throw error; // Throw instead of returning null so errors are properly caught
    }
  }

  // ============================================================================
  // CONTACT MANAGEMENT METHODS
  // ============================================================================

  /**
   * Retrieves all contacts from the database with company information.
   * 
   * This method fetches all contacts with their associated company details
   * through client-side joins. Contacts are returned with full company
   * information for display purposes.
   * 
   * @returns Promise<Contact[]> - Array of all contact records with company data
   * 
   * @example
   * ```typescript
   * try {
   *   const contacts = await ApiClient.getContacts();
   *   console.log(`Found ${contacts.length} contacts`);
   *   contacts.forEach(contact => {
   *     console.log(`${contact.full_name} at ${contact.company.name}`);
   *   });
   * } catch (error) {
   *   console.error('Failed to load contacts:', error);
   * }
   * ```
   * 
   * Contact Structure:
   * - `id`: SurrealDB record identifier
   * - `first_name`, `last_name`: Individual name components
   * - `full_name`: Auto-computed full name
   * - `email`, `phone`: Contact information
   * - `position`: Job title or role
   * - `company`: Full company record (joined)
   * - `time`: Creation and modification timestamps
   * 
   * @throws Error - Re-throws database connection or query errors
   */
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

  /**
   * Creates a new contact record in the database.
   * 
   * This method creates a new contact with automatic full_name computation
   * and timestamp generation. The contact must be associated with an
   * existing company record.
   * 
   * @param contact - Contact data without ID (will be generated by database)
   * @returns Promise<Contact | null> - Created contact or null on failure
   * 
   * @example
   * ```typescript
   * const newContact = {
   *   first_name: 'Ahmed',
   *   last_name: 'Al-Mansoori',
   *   email: 'ahmed.almansoori@company.ae',
   *   phone: '+971-50-123-4567',
   *   position: 'Project Manager',
   *   company: 'company:abc123' // SurrealDB record reference
   * };
   * 
   * try {
   *   const created = await ApiClient.createContact(newContact);
   *   if (created) {
   *     console.log('Contact created:', created.full_name);
   *   }
   * } catch (error) {
   *   console.error('Creation failed:', error);
   * }
   * ```
   * 
   * Required Fields:
   * - `first_name`, `last_name`: Individual name components
   * - `email`: Valid email address format
   * - `phone`: International phone number with '+' prefix
   * - `position`: Job title or role
   * - `company`: SurrealDB record reference to existing company
   * 
   * @throws Never throws - returns null on error for safe handling
   */
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

  /**
   * Updates an existing contact with partial data.
   * 
   * @param id Contact ID (string, extracted from SurrealDB Thing object)
   * @param contactUpdate Partial contact data - only fields to update
   * @returns Promise<Contact | null> Updated contact or null on failure
   */
  static async updateContact(id: string, contactUpdate: Partial<Contact>): Promise<Contact | null> {
    try {
      console.log('Invoking update_contact command with ID:', id, 'and data:', contactUpdate);
      
      const updated = await invoke<Contact>('update_contact', { 
        id, 
        contactUpdate: contactUpdate 
      });
      return updated;
    } catch (error) {
      console.error('Failed to update contact:', error);
      return null;
    }
  }

  /**
   * Deletes a contact from the database.
   * 
   * @param id Contact ID (string, extracted from SurrealDB Thing object) 
   * @returns Promise<Contact | null> Deleted contact or null on failure
   */
  static async deleteContact(id: string): Promise<Contact | null> {
    try {
      console.log('Invoking delete_contact command with ID:', id);
      
      const deleted = await invoke<Contact>('delete_contact', { id });
      return deleted;
    } catch (error) {
      console.error('Failed to delete contact:', error);
      return null;
    }
  }

  // ============================================================================
  // RFP (REQUEST FOR PROPOSAL) METHODS
  // ============================================================================

  /**
   * Retrieves all RFP records from the database.
   * 
   * This method fetches all Request for Proposal records with their
   * associated project, company, and contact information. RFPs represent
   * the core business documents for proposal management.
   * 
   * @returns Promise<Rfp[]> - Array of all RFP records
   * 
   * @example
   * ```typescript
   * try {
   *   const rfps = await ApiClient.getRfps();
   *   console.log(`Found ${rfps.length} RFPs`);
   *   rfps.forEach(rfp => {
   *     console.log(`${rfp.number}: ${rfp.name} (${rfp.status})`);
   *   });
   * } catch (error) {
   *   console.error('Failed to load RFPs:', error);
   * }
   * ```
   * 
   * RFP Structure:
   * - `id`: SurrealDB record identifier
   * - `name`, `number`: RFP identification
   * - `project_id`, `company_id`, `contact_id`: Related record references
   * - `status`: Current status (Draft, Active, Sent, Awarded, Lost, Cancelled)
   * - `stage`: Detailed workflow stage
   * - `issue_date`: Date in YYMMDD format
   * - `staff_*`: Assigned staff member information
   * - `revisions`: Revision history array
   * 
   * @throws Error - Re-throws database connection or query errors
   */
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

  /**
   * Creates a new RFP record in the database.
   * 
   * This method creates a new Request for Proposal with automatic timestamp
   * generation and revision tracking initialization. RFPs must be linked to
   * existing project, company, and contact records.
   * 
   * @param rfp - RFP data without ID (will be generated by database)
   * @returns Promise<Rfp | null> - Created RFP or null on failure
   * 
   * @example
   * ```typescript
   * const newRfp = {
   *   name: 'Dubai Museum Design Proposal',
   *   number: 'RFP-25-001',
   *   project_id: 'project:xyz789',
   *   company_id: 'company:abc123',
   *   contact_id: 'contact:def456',
   *   status: 'Draft',
   *   stage: 'Prepared',
   *   issue_date: '250315', // March 15, 2025
   *   activity: 'Design and Consultancy',
   *   staff_name: 'Martin Smith',
   *   staff_email: 'martin@emittiv.com'
   * };
   * 
   * try {
   *   const created = await ApiClient.createRfp(newRfp);
   *   if (created) {
   *     console.log('RFP created:', created.number);
   *   }
   * } catch (error) {
   *   console.error('Creation failed:', error);
   * }
   * ```
   * 
   * Required Fields:
   * - `name`, `number`: RFP identification
   * - `project_id`, `company_id`, `contact_id`: Valid record references
   * - `issue_date`: Date in YYMMDD format
   * 
   * @throws Never throws - returns null on error for safe handling
   */
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

  // ============================================================================
  // STATISTICS AND DASHBOARD METHODS
  // ============================================================================

  /**
   * Retrieves comprehensive application statistics for the dashboard.
   * 
   * This method calculates and returns key metrics about the current
   * state of the database, providing overview information for the
   * dashboard display and business intelligence.
   * 
   * @returns Promise<StatsObject> - Object containing all statistics
   * 
   * @example
   * ```typescript
   * try {
   *   const stats = await ApiClient.getStats();
   *   console.log(`Projects: ${stats.totalProjects}`);
   *   console.log(`Active RFPs: ${stats.activeRfps}`);
   *   console.log(`Companies: ${stats.totalCompanies}`);
   *   console.log(`Contacts: ${stats.totalContacts}`);
   * } catch (error) {
   *   console.error('Failed to load statistics:', error);
   * }
   * ```
   * 
   * Statistics Included:
   * - `totalProjects`: Count of all projects in database
   * - `activeRfps`: Count of RFPs with status 'Active' or 'Sent'
   * - `totalCompanies`: Count of all company records
   * - `totalContacts`: Count of all contact records
   * - `totalRfps`: Count of all RFP records regardless of status
   * 
   * Performance:
   * Statistics are calculated in real-time from the database using
   * efficient count queries without loading full record sets.
   * 
   * @throws Error - Re-throws database connection or query errors
   */
  static async getStats(): Promise<DatabaseStats> {
    try {
      console.log('Invoking get_stats command...');
      const stats = await invoke<DatabaseStats>('get_stats');
      console.log('get_stats response:', stats);
      return stats;
    } catch (error) {
      console.error('Failed to fetch stats from database:', error);
      throw error;
    }
  }

  // ============================================================================
  // SYSTEM HEALTH AND DEBUGGING METHODS
  // ============================================================================

  /**
   * Performs a comprehensive system health check.
   * 
   * This method tests all critical system components including database
   * connectivity, file system access, and backend service status.
   * Used for system monitoring and troubleshooting.
   * 
   * @returns Promise<string> - Health status message
   * 
   * @example
   * ```typescript
   * const health = await ApiClient.healthCheck();
   * console.log('System health:', health);
   * 
   * if (health.includes('OK')) {
   *   console.log('All systems operational');
   * } else {
   *   console.warn('System issues detected:', health);
   * }
   * ```
   * 
   * Health Check Components:
   * - Database connection and authentication
   * - File system read/write permissions
   * - Backend service responsiveness
   * - Memory and resource utilization
   * 
   * @throws Never throws - returns error message string on failure
   */
  static async healthCheck(): Promise<string> {
    try {
      return await invoke<string>('health_check');
    } catch (error) {
      console.error('Health check failed:', error);
      return 'Health check failed';
    }
  }

  /**
   * Retrieves detailed database connection information for debugging.
   * 
   * This method returns comprehensive information about the current
   * database connection state, configuration, and performance metrics.
   * Used for troubleshooting connection issues and monitoring.
   * 
   * @returns Promise<any> - Database information object
   * 
   * @example
   * ```typescript
   * const dbInfo = await ApiClient.getDbInfo();
   * console.log('DB URL:', dbInfo.url);
   * console.log('Namespace:', dbInfo.namespace);
   * console.log('Database:', dbInfo.database);
   * console.log('Connection time:', dbInfo.connected_at);
   * ```
   * 
   * Information Included:
   * - Connection URL and credentials (sanitized)
   * - Namespace and database names
   * - Connection timestamps and duration
   * - Query performance metrics
   * - Error history and diagnostics
   * 
   * @throws Never throws - returns error object on failure
   */
  static async getDbInfo(): Promise<DatabaseInfo> {
    try {
      return await invoke<DatabaseInfo>('get_db_info');
    } catch (error) {
      console.error('Failed to get database info:', error);
      return {
        error: error?.toString() || 'Unknown error',
        status: 'error'
      };
    }
  }

  /**
   * Retrieves schema information for a specific database table.
   * 
   * This method returns detailed schema information including field
   * definitions, data types, constraints, and relationships for the
   * specified table. Used for debugging and documentation generation.
   * 
   * @param tableName - Name of the table to inspect
   * @returns Promise<any> - Table schema information
   * 
   * @example
   * ```typescript
   * const projectSchema = await ApiClient.getTableSchema('projects');
   * console.log('Project fields:', projectSchema.fields);
   * 
   * const companySchema = await ApiClient.getTableSchema('company');
   * console.log('Company constraints:', companySchema.constraints);
   * ```
   * 
   * Schema Information:
   * - Field names and data types
   * - Required vs optional fields
   * - Foreign key relationships
   * - Validation constraints
   * - Index information
   * 
   * @throws Never throws - returns error object on failure
   */
  static async getTableSchema(tableName: string): Promise<TableSchema> {
    try {
      return await invoke<TableSchema>('get_table_schema', { tableName });
    } catch (error) {
      console.error(`Failed to get schema for table ${tableName}:`, error);
      return {
        table: tableName,
        fields: [],
        relationships: [],
        error: error?.toString() || 'Unknown error',
        retrieved_at: new Date().toISOString()
      };
    }
  }

  /**
   * Positions the application window optimally for 4K displays.
   * 
   * This method adjusts the application window size and position to
   * provide an optimal viewing experience on high-resolution 4K monitors.
   * Triggered by the Cmd+W keyboard shortcut.
   * 
   * @returns Promise<string> - Success or error message
   * 
   * @example
   * ```typescript
   * const result = await ApiClient.positionWindow4K();
   * if (result.includes('Success')) {
   *   console.log('Window positioned for 4K display');
   * } else {
   *   console.error('Window positioning failed:', result);
   * }
   * ```
   * 
   * Window Positioning:
   * - Calculates optimal size based on screen resolution
   * - Centers window on primary monitor
   * - Adjusts for high-DPI scaling factors
   * - Maintains aspect ratio and usability
   * 
   * @throws Never throws - returns error message string on failure
   */
  static async positionWindow4K(): Promise<string> {
    try {
      return await invoke<string>('position_window_4k');
    } catch (error) {
      console.error('Failed to position window:', error);
      return 'Failed to position window';
    }
  }

  // ============================================================================
  // FILE SYSTEM OPERATIONS
  // ============================================================================

  /**
   * Opens a native folder selection dialog.
   * 
   * This method displays the operating system's native folder picker
   * dialog, allowing users to select directories for project templates
   * or other file operations. Used in settings configuration.
   * 
   * @returns Promise<string | null> - Selected folder path or null if cancelled
   * 
   * @example
   * ```typescript
   * const folderPath = await ApiClient.selectFolder();
   * if (folderPath) {
   *   console.log('User selected:', folderPath);
   *   // Update settings with new path
   * } else {
   *   console.log('User cancelled folder selection');
   * }
   * ```
   * 
   * Dialog Behavior:
   * - Shows native OS folder picker (Windows Explorer, macOS Finder, etc.)
   * - Returns absolute path to selected folder
   * - Returns null if user cancels or dialog fails
   * - Respects OS-specific path formats (backslash on Windows, forward slash on Unix)
   * 
   * @throws Never throws - returns null on error for safe handling
   */
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

  /**
   * Opens a folder in the system's default file explorer.
   * 
   * This method launches the operating system's file manager application
   * and navigates to the specified folder. Used for opening project folders
   * directly from the application interface.
   * 
   * @param folderPath - Absolute path to the folder to open
   * @returns Promise<string> - Success message or error description
   * 
   * @example
   * ```typescript
   * // Open a project folder
   * const projectFolder = 'E:\\Projects\\25-97105 Dubai Museum';
   * const result = await ApiClient.openFolderInExplorer(projectFolder);
   * 
   * if (result.includes('Success')) {
   *   console.log('Folder opened in Explorer');
   * } else {
   *   console.error('Failed to open folder:', result);
   * }
   * ```
   * 
   * Cross-Platform Support:
   * - Windows: Opens in Windows Explorer
   * - macOS: Opens in Finder
   * - Linux: Opens in default file manager
   * 
   * Error Handling:
   * - Returns error message if folder doesn't exist
   * - Handles permission issues gracefully
   * - Provides descriptive error messages for debugging
   * 
   * @throws Never throws - returns error message string on failure
   */
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

  // ============================================================================
  // PROJECT NUMBER GENERATION METHODS
  // ============================================================================

  /**
   * Generates the next sequential project number for a given country and year.
   * 
   * This method implements the core project numbering algorithm, generating
   * the next available project number in the format YY-CCCNN where:
   * - YY: 2-digit year (25 = 2025)
   * - CCC: Country dial code (971 = UAE, 966 = Saudi)
   * - NN: 2-digit sequence number (01, 02, 03...)
   * 
   * @param countryName - Full country name to lookup dial code
   * @param year - Optional year override (defaults to current year)
   * @returns Promise<string> - Generated project number (e.g., "25-97105")
   * 
   * @example
   * ```typescript
   * // Generate for current year
   * const current = await ApiClient.generateNextProjectNumber('United Arab Emirates');
   * console.log(current); // "25-97105"
   * 
   * // Generate for specific year (backdating)
   * const backdated = await ApiClient.generateNextProjectNumber('Saudi Arabia', 24);
   * console.log(backdated); // "24-96601"
   * ```
   * 
   * Number Generation Logic:
   * 1. Look up country dial code from countries table
   * 2. Determine year (current or specified)
   * 3. Find highest existing sequence number for year/country combination
   * 4. Increment sequence and format as YY-CCCNN
   * 
   * @throws Error - Throws on invalid country name or database errors
   */
  static async generateNextProjectNumber(countryName: string, year?: number): Promise<string> {
    try {
      console.log('Generating next project number for country:', countryName, 'year:', year);
      const projectNumber = await invoke<string>('generate_next_project_number', { 
        countryName, 
        year: year || null 
      });
      console.log('Generated project number:', projectNumber);
      return projectNumber;
    } catch (error) {
      console.error('Failed to generate project number:', error);
      throw error;
    }
  }

  /**
   * Validates that a project number is unique and follows correct format.
   * 
   * This method checks that a proposed project number:
   * 1. Follows the YY-CCCNN format
   * 2. Uses a valid country dial code
   * 3. Doesn't already exist in the database
   * 
   * @param projectNumber - Project number to validate (e.g., "25-97105")
   * @returns Promise<boolean> - True if valid and unique, false otherwise
   * 
   * @example
   * ```typescript
   * // Validate a manually entered project number
   * const isValid = await ApiClient.validateProjectNumber('25-97105');
   * if (isValid) {
   *   console.log('Project number is available');
   * } else {
   *   console.log('Project number already exists or invalid format');
   * }
   * ```
   * 
   * Validation Rules:
   * - Format: Must match YY-CCCNN pattern
   * - Year: Must be 20-50 (2020-2050)
   * - Country: Dial code must exist in countries table
   * - Sequence: Must be 01-99
   * - Uniqueness: No existing project with same number
   * 
   * @throws Error - Throws on database connection errors
   */
  static async validateProjectNumber(projectNumber: string): Promise<boolean> {
    try {
      console.log('Validating project number:', projectNumber);
      const isValid = await invoke<boolean>('validate_project_number', { projectNumber });
      console.log('Project number is valid:', isValid);
      return isValid;
    } catch (error) {
      console.error('Failed to validate project number:', error);
      throw error;
    }
  }

  /**
   * Creates a new project with automatic template folder copying.
   * 
   * This method combines project creation with file system operations:
   * 1. Creates the project record in the database
   * 2. Copies template folder structure from base template
   * 3. Renames template files with actual project number
   * 4. Updates folder paths in the project record
   * 
   * @param project - Partial project data (number will be generated)
   * @returns Promise<Project> - Created project with template folder
   * 
   * @example
   * ```typescript
   * const projectData = {
   *   name: 'Dubai Museum of Future',
   *   name_short: 'DMOF',
   *   area: 'Business Bay',
   *   city: 'Dubai',
   *   country: 'United Arab Emirates',
   *   status: 'Draft'
   * };
   * 
   * try {
   *   const created = await ApiClient.createProjectWithTemplate(projectData);
   *   console.log(`Project created: ${created.number.id}`);
   *   console.log(`Folder: ${created.folder}`);
   * } catch (error) {
   *   console.error('Failed to create project with template:', error);
   * }
   * ```
   * 
   * Template Process:
   * 1. Generate unique project number
   * 2. Copy from template: "E:\Projects\_yy-cccnn Project Name"
   * 3. Rename to: "E:\Projects\25-97105 Dubai Museum of Future"
   * 4. Update all internal file references
   * 5. Set project.folder to new path
   * 
   * @throws Error - Throws on database, file system, or permission errors
   */
  static async createProjectWithTemplate(project: Partial<Project>): Promise<Project> {
    try {
      console.log('Creating project with template:', project);
      const created = await invoke<Project>('create_project_with_template', { project });
      console.log('Project created successfully:', created);
      return created;
    } catch (error) {
      console.error('Failed to create project with template:', error);
      throw error;
    }
  }

  /**
   * Searches countries by name with fuzzy matching.
   * 
   * This method searches across multiple country name fields using
   * case-insensitive partial matching. Used for country autocomplete
   * in the NewProjectModal component.
   * 
   * @param query - Search term to match against country names
   * @returns Promise<any[]> - Array of matching country records
   * 
   * @example
   * ```typescript
   * // Search for UAE variants
   * const uae = await ApiClient.searchCountries('emirates');
   * console.log(uae); // [{ name: 'United Arab Emirates', dial_code: 971, ... }]
   * 
   * // Search by partial name
   * const saudi = await ApiClient.searchCountries('saudi');
   * console.log(saudi); // [{ name: 'Saudi Arabia', dial_code: 966, ... }]
   * ```
   * 
   * Search Fields:
   * - `name`: Primary country name
   * - `name_formal`: Formal/official name
   * - `name_official`: Official government name
   * - `code`: ISO country codes (e.g., "AE", "SA")
   * - `code_alt`: Alternative codes
   * 
   * Returns:
   * Each country record includes name, dial_code, and all name variants
   * for display in autocomplete dropdowns.
   * 
   * @throws Error - Throws on database connection errors
   */
  static async searchCountries(query: string): Promise<CountrySearchResult[]> {
    try {
      console.log('Searching countries with query:', query);
      const countries = await invoke<CountrySearchResult[]>('search_countries', { query });
      console.log('Search returned countries:', countries);
      return countries;
    } catch (error) {
      console.error('Failed to search countries:', error);
      throw error;
    }
  }

  /**
   * Investigates a database record for debugging and analysis.
   * 
   * This method provides detailed information about a specific database
   * record including its structure, relationships, and metadata. Used
   * for debugging data issues and understanding record relationships.
   * 
   * @param recordId - SurrealDB record ID to investigate
   * @returns Promise<any> - Detailed record information
   * 
   * @example
   * ```typescript
   * // Investigate a project record
   * const info = await ApiClient.investigateRecord('project:abc123');
   * console.log('Record type:', info.table);
   * console.log('Field count:', info.fields.length);
   * console.log('Related records:', info.relationships);
   * ```
   * 
   * Investigation Results:
   * - Record structure and field values
   * - Related records and foreign keys
   * - Creation and modification history
   * - Data validation status
   * - Performance metrics
   * 
   * @throws Error - Throws on invalid record ID or database errors
   */
  static async investigateRecord(recordId: string): Promise<any> {
    try {
      console.log('Investigating database record:', recordId);
      const result = await invoke('investigate_record', { recordId });
      console.log('Investigation result:', result);
      return result;
    } catch (error) {
      console.error('Failed to investigate record:', error);
      throw error;
    }
  }

  /**
   * Retrieves area suggestions based on country selection.
   * 
   * This method returns a list of common areas/regions for the specified
   * country based on existing project data. Used for autocomplete in
   * the project creation forms.
   * 
   * @param country - Country name to get area suggestions for
   * @returns Promise<string[]> - Array of area names
   * 
   * @example
   * ```typescript
   * const areas = await ApiClient.getAreaSuggestions('United Arab Emirates');
   * console.log('UAE areas:', areas);
   * // ['Business Bay', 'Downtown Dubai', 'Dubai Marina', 'DIFC']
   * ```
   * 
   * Data Source:
   * - Extracts unique area values from existing projects
   * - Filters by country to provide relevant suggestions
   * - Returns most frequently used areas first
   * - Excludes empty or invalid area names
   * 
   * @throws Error - Throws on database connection errors
   */
  static async getAreaSuggestions(country: string): Promise<string[]> {
    try {
      const suggestions = await invoke('get_area_suggestions', { country });
      return suggestions as string[];
    } catch (error) {
      console.error('Failed to get area suggestions:', error);
      throw error;
    }
  }

  /**
   * Retrieves city suggestions based on country selection.
   * 
   * This method returns a list of common cities for the specified
   * country based on existing project data. Used for autocomplete in
   * the project creation forms.
   * 
   * @param country - Country name to get city suggestions for
   * @returns Promise<string[]> - Array of city names
   * 
   * @example
   * ```typescript
   * const cities = await ApiClient.getCitySuggestions('United Arab Emirates');
   * console.log('UAE cities:', cities);
   * // ['Dubai', 'Abu Dhabi', 'Sharjah', 'Ajman']
   * ```
   * 
   * Data Source:
   * - Extracts unique city values from existing projects
   * - Filters by country to provide relevant suggestions
   * - Returns most frequently used cities first
   * - Excludes empty or invalid city names
   * 
   * @throws Error - Throws on database connection errors
   */
  static async getCitySuggestions(country: string): Promise<string[]> {
    try {
      const suggestions = await invoke('get_city_suggestions', { country });
      return suggestions as string[];
    } catch (error) {
      console.error('Failed to get city suggestions:', error);
      throw error;
    }
  }

  /**
   * Gets all unique cities from projects and companies tables.
   * 
   * @returns Promise<string[]> - Array of all city names
   * 
   * @example
   * ```typescript
   * const allCities = await ApiClient.getAllCities();
   * console.log('All cities:', allCities);
   * // ['Abu Dhabi', 'Dubai', 'Riyadh', 'Sharjah', ...]
   * ```
   * 
   * Data Source:
   * - Combines cities from both projects and companies tables
   * - Returns alphabetically sorted list
   * - Limited to 50 most common cities
   * 
   * @throws Error - Throws on database connection errors
   */
  static async getAllCities(): Promise<string[]> {
    try {
      const cities = await invoke('get_all_cities');
      return cities as string[];
    } catch (error) {
      console.error('Failed to get all cities:', error);
      throw error;
    }
  }
}

// ============================================================================
// CONVENIENCE EXPORTS
// ============================================================================

/**
 * Individual function exports for convenience and flexible importing.
 * 
 * These destructured exports allow importing specific functions without
 * the ApiClient class prefix, providing a more functional programming
 * approach when preferred.
 * 
 * @example
 * ```typescript
 * // Class-based approach
 * import { ApiClient } from '$lib/api';
 * const projects = await ApiClient.getProjects();
 * 
 * // Function-based approach
 * import { getProjects, createProject } from '$lib/api';
 * const projects = await getProjects();
 * await createProject(newProject);
 * 
 * // Mixed approach
 * import ApiClient, { getProjects } from '$lib/api';
 * const projects = await getProjects();
 * const status = await ApiClient.getConnectionStatus();
 * ```
 */
export const {
  // Connection management
  checkDbConnection,
  getConnectionStatus,
  
  // Project operations
  getProjects,
  searchProjects,
  createProject,
  
  // Company operations
  getCompanies,
  createCompany,
  updateCompany,
  deleteCompany,
  
  // Contact operations
  getContacts,
  createContact,
  updateContact,
  deleteContact,
  
  // RFP operations
  getRfps,
  createRfp,
  
  // Statistics and monitoring
  getStats,
  healthCheck,
  getDbInfo,
  getTableSchema,
  
  // System operations
  positionWindow4K,
  selectFolder,
  openFolderInExplorer,
  
  // Project workflow
  searchCountries,
  generateNextProjectNumber,
  validateProjectNumber,
  createProjectWithTemplate,
  
  // Debugging and utilities
  investigateRecord,
  getAreaSuggestions,
  getCitySuggestions,
  getAllCities
} = ApiClient;

/**
 * Default export of the complete ApiClient class.
 * 
 * Provides access to all API methods through a single class interface.
 * Recommended for most use cases due to better IntelliSense support
 * and clearer API organization.
 * 
 * @example
 * ```typescript
 * import ApiClient from '$lib/api';
 * 
 * // All methods available through class
 * const projects = await ApiClient.getProjects();
 * const companies = await ApiClient.getCompanies();
 * const status = await ApiClient.getConnectionStatus();
 * ```
 */
export default ApiClient;