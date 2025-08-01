export interface NavItem {
  id: string;
  label: string;
  icon: string;
  shortcut?: string;
}

export interface AppRoute {
  id: string;
  component: any;
  title: string;
}

export interface ConnectionStatus {
  is_connected: boolean;
  last_check?: string;
  error_message?: string;
}

// SurrealDB Thing object type
export interface SurrealThing {
  tb: string;
  id: string | { String: string };
}

export interface Project {
  id?: string | SurrealThing;
  name: string;
  name_short: string;
  status: 'Draft' | 'RFP' | 'Active' | 'Awarded' | 'Completed' | 'Lost' | 'Cancelled' | 'On Hold' | 'Revised';
  area: string;
  city: string;
  country: string;
  folder: string;
  number: {
    year: number;
    country: number;
    seq: number;
    id: string;
  };
  time: {
    created_at: string;
    updated_at: string;
  };
}

export interface Company {
  id?: string | SurrealThing;
  name: string;
  name_short: string;
  abbreviation: string;
  city: string;
  country: string;
  reg_no?: string;
  tax_no?: string;
  time: {
    created_at: string;
    updated_at: string;
  };
}

export interface Contact {
  id?: string | SurrealThing;
  first_name: string;
  last_name: string;
  full_name: string;
  email: string;
  phone: string;
  position: string;
  company: string | SurrealThing;
  time: {
    created_at: string;
    updated_at: string;
  };
}

export interface Fee {
  id?: string | SurrealThing;
  name: string;
  number: string;
  rev: number;
  status: 'Draft' | 'Sent' | 'Negotiation' | 'Awarded' | 'Completed' | 'Lost' | 'Cancelled' | 'On Hold' | 'Revised';
  issue_date: string;
  activity: string;
  package: string;
  project_id: string | SurrealThing;
  company_id: string | SurrealThing;
  contact_id: string | SurrealThing;
  staff_name: string;
  staff_email: string;
  staff_phone: string;
  staff_position: string;
  strap_line: string;
  revisions: Revision[];
  time: {
    created_at: string;
    updated_at: string;
  };
}

export interface Revision {
  revision_number: number;
  revision_date: string;
  author_email: string;
  author_name: string;
  notes: string;
}


// ============================================================================
// API RESPONSE TYPES
// ============================================================================

/**
 * Database statistics response structure.
 * 
 * Returned by the getStats API endpoint to provide dashboard metrics.
 */
export interface DatabaseStats {
  /** Total number of projects in the database */
  totalProjects: number;
  /** Number of Fees with status 'Active' or 'Sent' */
  activeFees: number;
  /** Total number of company records */
  totalCompanies: number;
  /** Total number of contact records */
  totalContacts: number;
  /** Total number of Fee records regardless of status */
  totalFees: number;
}

/**
 * Database connection information structure.
 * 
 * Provides detailed information about the current database connection
 * for debugging and monitoring purposes.
 */
export interface DatabaseInfo {
  /** Database connection URL (sanitized for security) */
  url?: string;
  /** SurrealDB namespace name */
  namespace?: string;
  /** SurrealDB database name */
  database?: string;
  /** Timestamp when connection was established */
  connected_at?: string;
  /** Number of queries executed in this session */
  query_count?: number;
  /** Connection duration in milliseconds */
  connection_duration?: number;
  /** Error message if connection failed */
  error?: string;
  /** Connection status details */
  status?: 'connected' | 'disconnected' | 'error';
}

/**
 * Database table schema information structure.
 * 
 * Provides detailed schema information for a specific table
 * including fields, relationships, and constraints.
 */
export interface TableSchema {
  /** Name of the table */
  table: string;
  /** Array of field definitions */
  fields: Array<{
    /** Field name */
    name: string;
    /** Field data type */
    type: string;
    /** Whether the field is required */
    required: boolean;
    /** Default value if any */
    default?: any;
    /** Additional constraints */
    constraints?: string[];
  }>;
  /** Foreign key relationships */
  relationships: Array<{
    /** Field name that contains the foreign key */
    field: string;
    /** Table and field being referenced */
    references: string;
    /** Type of relationship (one-to-one, one-to-many, etc.) */
    type?: 'one-to-one' | 'one-to-many' | 'many-to-one' | 'many-to-many';
  }>;
  /** Indexes defined on the table */
  indexes?: Array<{
    /** Index name */
    name: string;
    /** Fields included in the index */
    fields: string[];
    /** Whether the index is unique */
    unique: boolean;
  }>;
  /** Error message if schema retrieval failed */
  error?: string;
  /** Timestamp when schema was retrieved */
  retrieved_at?: string;
}

/**
 * Country search result structure.
 * 
 * Returned by country search API for autocomplete functionality.
 */
export interface CountrySearchResult {
  /** Primary country name */
  name: string;
  /** Formal country name */
  name_formal?: string;
  /** Official country name */
  name_official?: string;
  /** ISO country code */
  code: string;
  /** Alternative country codes */
  code_alt?: string;
  /** International dialing code */
  dial_code: number;
}

/**
 * Project creation result structure.
 * 
 * Returned when creating a project with template folder operations.
 */
export interface ProjectCreationResult {
  /** The created project object */
  project: Project;
  /** Path to the created project folder */
  folder_path: string;
  /** List of files that were copied */
  copied_files: string[];
  /** Any warnings during the creation process */
  warnings?: string[];
}

/**
 * File operation result structure.
 * 
 * Returned by file system operations like folder creation and file copying.
 */
export interface FileOperationResult {
  /** Whether the operation was successful */
  success: boolean;
  /** Result message */
  message: string;
  /** Path that was operated on */
  path?: string;
  /** List of files affected */
  files?: string[];
  /** Error details if operation failed */
  error?: string;
}