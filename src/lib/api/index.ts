/**
 * API Module Index
 *
 * Re-exports all API methods from domain-specific modules.
 * Import from this index for a unified API interface.
 *
 * @example
 * ```typescript
 * import { getProjects, createCompany, getConnectionStatus } from '$lib/api';
 * ```
 */

// Connection methods
export { checkDbConnection, getConnectionStatus } from './connection';

// Project operations
export {
  getProjects,
  getProjectsPage,
  getProjectById,
  searchProjects,
  createProject,
  updateProject,
  deleteProject,
  generateNextProjectNumber,
  validateProjectNumber,
  createProjectWithTemplate,
  copyProjectTemplate,
  populateProjectData
} from './projects';

// Company operations
export {
  getCompanies,
  getCompaniesPage,
  getCompanyById,
  createCompany,
  updateCompany,
  deleteCompany
} from './companies';

// Contact operations
export {
  getContacts,
  getContactsPage,
  getContactById,
  createContact,
  updateContact,
  deleteContact
} from './contacts';

// Fee operations
export {
  getFees,
  getFeesPage,
  createFee,
  updateFee,
  deleteFee,
  writeFeeToJson,
  writeFeeToJsonSafe
} from './fees';

// Filesystem operations
export {
  selectFolder,
  openFolderInExplorer,
  checkProjectFolderExists,
  checkVarJsonExists,
  checkVarJsonTemplateExists,
  renameFolderWithOldSuffix,
  renameVarJsonWithOldSuffix
} from './filesystem';

// Revision and template export operations
export { cloneFeeRevision, getFeesForProject, exportFeeTemplate } from './revisions';

// Settings operations
export { getSettings, saveSettings, reloadDatabaseConfig, reconnectDatabase } from './settings';

// System operations
export {
  getStats,
  healthCheck,
  getDbInfo,
  getTableSchema,
  positionWindow4K,
  investigateRecord,
  getDevMode,
  logMessage,
  type LogLevel
} from './system';

// Reference data operations
export { searchCountries, getAreaSuggestions, getCitySuggestions, getAllCities } from './reference';

// Batch operations
export { batchDeleteEntities, batchUpdateStatus } from './batch';

// Activity log operations
export { createActivityLog, getActivityLogs } from './activity';

// Re-export types for convenience
export type {
  Project,
  Company,
  Contact,
  Fee,
  ProjectCreate,
  ProjectUpdate,
  CompanyCreate,
  CompanyUpdate,
  ContactCreate,
  ContactUpdate,
  FeeCreate,
  FeeUpdate,
  ConnectionStatus,
  DatabaseStats,
  DatabaseInfo,
  TableSchema,
  CountrySearchResult,
  ProjectCreationResult,
  FileOperationResult,
  PaginatedResponse,
  ActivityLog,
  ActivityLogCreate
} from '../../types';
