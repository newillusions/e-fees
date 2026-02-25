export interface NavItem {
  id: string;
  label: string;
  icon: string;
  shortcut?: string;
  /** If true, only show this nav item when dev_mode is enabled in settings */
  devOnly?: boolean;
}

export interface AppRoute {
  id: string;
  component: unknown; // Svelte component constructor
  title: string;
}

export interface ConnectionStatus {
  is_connected: boolean;
  last_check?: string;
  error_message?: string;
}

// SurrealDB Thing object type (v2 format: {tb, id})
export interface SurrealThing {
  tb: string;
  id: string | { String: string };
}

// SurrealDB v3 RecordId format: {table, key}
export interface SurrealV3RecordId {
  table: string;
  key: string | { String: string } | { Number: number };
}

// Union type for SurrealDB ID values (can be string, v2 Thing, or v3 RecordId)
export type SurrealId = string | SurrealThing | SurrealV3RecordId;

// Type for unknown SurrealDB Thing objects (before parsing)
export type UnknownSurrealThing = {
  tb?: string;
  id?: unknown;
  table?: string;
  key?: unknown;
  String?: string;
} | string | null | undefined;

// Import types needed within this file
import type { Project } from './database';

// Re-export all entity types from database.ts (single source of truth)
export type {
  Project,
  Company,
  Contact,
  Fee,
  Venue,
  VenueLocation,
  VenueCreate,
  Revision,
  ProjectStatus,
  ProjectActivity,
  ProjectStage,
  FeeStatus,
  FeeStage,
  TimeInfo,
  ProjectNumber,
  ProjectCreate,
  ProjectUpdate,
  CompanyCreate,
  CompanyUpdate,
  ContactCreate,
  ContactUpdate,
  FeeCreate,
  FeeUpdate,
} from './database';


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
  /** Number of Fees in-progress (Draft, Sent, or Negotiation status) */
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
    default?: string | number | boolean | null;
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

/**
 * Paginated Response Structure
 *
 * Returned by paginated query endpoints for lazy loading and infinite scroll.
 * Contains both the data items and pagination metadata.
 */
export interface PaginatedResponse<T> {
  /** The items for the current page */
  items: T[];
  /** Total number of records across all pages */
  total: number;
  /** Current page number (1-indexed) */
  page: number;
  /** Number of items per page */
  page_size: number;
  /** Whether there are more pages to load */
  has_more: boolean;
}

/**
 * Pagination State for UI stores
 *
 * Tracks pagination state for lazy loading patterns.
 */
export interface PaginationState {
  /** Current page number (1-indexed) */
  currentPage: number;
  /** Number of items per page */
  pageSize: number;
  /** Total number of records (from server) */
  totalRecords: number;
  /** Whether more records are available */
  hasMore: boolean;
  /** Set of loaded record IDs (for deduplication) */
  loadedIds: Set<string>;
  /** Whether a page load is in progress */
  isLoading: boolean;
}

// ============================================================================
// ACTIVITY LOG TYPES
// ============================================================================

/**
 * Activity Log Entry
 *
 * Represents a logged user action for display in the activity feed.
 * Logs are stored in the database and synced across all machines.
 */
export interface ActivityLog {
  /** Database record ID (e.g., 'activity_log:abc123') */
  id?: string | SurrealThing;
  /** Action type: create, update, delete, status_change */
  action: 'create' | 'update' | 'delete' | 'status_change';
  /** Entity type: project, fee, company, contact */
  entity_type: 'project' | 'fee' | 'company' | 'contact';
  /** Entity ID (e.g., 'projects:25-97105') */
  entity_id: string;
  /** Human-readable entity name for display */
  entity_name: string;
  /** Description of the action */
  description: string;
  /** Previous value (for updates/status changes) */
  old_value?: string;
  /** New value (for updates/status changes) */
  new_value?: string;
  /** User who performed the action */
  user: string;
  /** ISO timestamp of when the action occurred */
  timestamp: string;
  /** Additional metadata (JSON object) */
  metadata?: Record<string, unknown>;
}

/**
 * Activity Log Create
 *
 * Data structure for creating new activity log entries.
 * The timestamp is automatically set by the database.
 */
export interface ActivityLogCreate {
  action: 'create' | 'update' | 'delete' | 'status_change';
  entity_type: 'project' | 'fee' | 'company' | 'contact';
  entity_id: string;
  entity_name: string;
  description: string;
  old_value?: string;
  new_value?: string;
  user?: string;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// FOLDER SYNC TYPES
// ============================================================================
export * from './folderSync';