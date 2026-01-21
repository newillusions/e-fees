/**
 * API Client Module
 *
 * This module provides a comprehensive TypeScript wrapper around Tauri commands
 * for seamless frontend-backend communication. It serves as the primary interface
 * between the Svelte frontend and the Rust backend database operations.
 *
 * Note: Methods are now organized in domain-specific modules under ./api/
 * This file re-exports everything for backward compatibility.
 *
 * @fileoverview Main API interface for database and system operations
 */

// Import all functions from domain modules
import * as connection from './api/connection';
import * as projects from './api/projects';
import * as companies from './api/companies';
import * as contacts from './api/contacts';
import * as fees from './api/fees';
import * as filesystem from './api/filesystem';
import * as settings from './api/settings';
import * as system from './api/system';
import * as reference from './api/reference';
import * as activity from './api/activity';

// Re-export types from types file
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
} from '../types';

/**
 * Primary API client class providing static methods for all backend operations.
 *
 * This class serves as the central hub for all database operations, system
 * commands, and file management functions. Each method corresponds directly
 * to a Tauri command in the Rust backend.
 */
export class ApiClient {
  // Connection methods
  static checkDbConnection = connection.checkDbConnection;
  static getConnectionStatus = connection.getConnectionStatus;

  // Project methods
  static getProjects = projects.getProjects;
  static getProjectsPage = projects.getProjectsPage;
  static getProjectById = projects.getProjectById;
  static searchProjects = projects.searchProjects;
  static createProject = projects.createProject;
  static updateProject = projects.updateProject;
  static deleteProject = projects.deleteProject;
  static generateNextProjectNumber = projects.generateNextProjectNumber;
  static validateProjectNumber = projects.validateProjectNumber;
  static createProjectWithTemplate = projects.createProjectWithTemplate;
  static copyProjectTemplate = projects.copyProjectTemplate;
  static populateProjectData = projects.populateProjectData;

  // Company methods
  static getCompanies = companies.getCompanies;
  static getCompaniesPage = companies.getCompaniesPage;
  static getCompanyById = companies.getCompanyById;
  static createCompany = companies.createCompany;
  static updateCompany = companies.updateCompany;
  static deleteCompany = companies.deleteCompany;

  // Contact methods
  static getContacts = contacts.getContacts;
  static getContactsPage = contacts.getContactsPage;
  static getContactById = contacts.getContactById;
  static createContact = contacts.createContact;
  static updateContact = contacts.updateContact;
  static deleteContact = contacts.deleteContact;

  // Fee methods
  static getFees = fees.getFees;
  static getFeesPage = fees.getFeesPage;
  static createFee = fees.createFee;
  static updateFee = fees.updateFee;
  static deleteFee = fees.deleteFee;
  static writeFeeToJson = fees.writeFeeToJson;
  static writeFeeToJsonSafe = fees.writeFeeToJsonSafe;

  // Filesystem methods
  static selectFolder = filesystem.selectFolder;
  static openFolderInExplorer = filesystem.openFolderInExplorer;
  static checkProjectFolderExists = filesystem.checkProjectFolderExists;
  static checkVarJsonExists = filesystem.checkVarJsonExists;
  static checkVarJsonTemplateExists = filesystem.checkVarJsonTemplateExists;
  static renameFolderWithOldSuffix = filesystem.renameFolderWithOldSuffix;
  static renameVarJsonWithOldSuffix = filesystem.renameVarJsonWithOldSuffix;

  // Settings methods
  static getSettings = settings.getSettings;
  static saveSettings = settings.saveSettings;
  static reloadDatabaseConfig = settings.reloadDatabaseConfig;

  // System methods
  static getStats = system.getStats;
  static healthCheck = system.healthCheck;
  static getDbInfo = system.getDbInfo;
  static getTableSchema = system.getTableSchema;
  static positionWindow4K = system.positionWindow4K;
  static investigateRecord = system.investigateRecord;

  // Reference methods
  static searchCountries = reference.searchCountries;
  static getAreaSuggestions = reference.getAreaSuggestions;
  static getCitySuggestions = reference.getCitySuggestions;
  static getAllCities = reference.getAllCities;

  // Activity log methods
  static createActivityLog = activity.createActivityLog;
  static getActivityLogs = activity.getActivityLogs;
}

// Individual function exports for convenience
export const {
  // Connection management
  checkDbConnection,
  getConnectionStatus,

  // Project operations
  getProjects,
  searchProjects,
  createProject,
  updateProject,
  deleteProject,

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

  // Fee operations
  getFees,
  createFee,
  updateFee,
  deleteFee,
  writeFeeToJson,
  writeFeeToJsonSafe,

  // Pagination operations
  getProjectsPage,
  getCompaniesPage,
  getContactsPage,
  getFeesPage,

  // Single entity fetch
  getProjectById,
  getCompanyById,
  getContactById,

  // Statistics and monitoring
  getStats,
  healthCheck,
  getDbInfo,
  getTableSchema,

  // System operations
  positionWindow4K,
  getSettings,
  saveSettings,
  reloadDatabaseConfig,
  selectFolder,
  openFolderInExplorer,

  // Project workflow
  searchCountries,
  generateNextProjectNumber,
  validateProjectNumber,
  createProjectWithTemplate,
  copyProjectTemplate,
  checkProjectFolderExists,
  checkVarJsonExists,
  checkVarJsonTemplateExists,
  renameFolderWithOldSuffix,
  renameVarJsonWithOldSuffix,

  // Debugging and utilities
  investigateRecord,
  getAreaSuggestions,
  getCitySuggestions,
  getAllCities,

  // Activity log operations
  createActivityLog,
  getActivityLogs
} = ApiClient;

export default ApiClient;
