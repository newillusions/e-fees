/**
 * Smoke Test Suite: Executable Check Definitions
 *
 * This module exports all smoke test checks as executable JavaScript/TypeScript strings.
 * Each check is a self-executing async IIFE that returns structured test results.
 *
 * NOTE: Tauri MCP's execute_js does not resolve async IIFE promises (returns {}).
 * When executing via Tauri MCP, store the result in window.__SMOKE_RESULT and
 * read it back with a second execute_js call.
 *
 * EXECUTION ORDER:
 * Phase 1: Safety — abort if production DB detected
 * Phase 2: Infrastructure (db_connection, data_loaded)
 * Phase 3: Data validation (project_statuses, fee_statuses, entity_counts)
 * Phase 4: Navigation (dashboard, projects, proposals, companies, contacts)
 * Phase 5: UI State (modals, forms, filters, keyboard shortcuts, bulk select)
 * Phase 6: CRUD (sequential pipeline: company → contact → project → fee → cleanup)
 * Phase 7: Integration (status consistency, connection, settings, detail panel)
 * Phase 8: Regression (guards against known past bugs)
 *
 * Result format: { check: string; pass: boolean; ABORT?: boolean; error?: string; details?: any }
 *
 * If any check returns { ABORT: true }, stop execution immediately and report error.
 */

import {
  crudCompany,
  crudContact,
  crudProject,
  crudFee,
  crudCleanup,
} from './helpers/crud-checks';

import {
  modalOpenClose,
  formValidation,
  searchFilter,
  dropdownFilter,
  keyboardNav1,
  keyboardNav2,
  keyboardNav3,
  keyboardNav4,
  keyboardNav5,
  bulkSelect,
} from './helpers/ui-state-checks';

import {
  statusTransition,
  feeProjectMapping,
  connectionIndicator,
  entityCountConsistency,
  settingsModal,
  detailPanel,
} from './helpers/integration-checks';

import {
  recordidV3,
  feeNoInfinity,
  contactFullName,
  feeDeser,
  companyIdExtract,
  feeStatusLegacy,
  projectStatusLegacy,
  navOrder,
} from './helpers/regression-checks';

export const CHECKS = {
  // Phase 1: Safety
  safety: `(async () => {
    const dbInfo = await window.__TAURI_INTERNALS__.invoke('get_db_info');
    const url = dbInfo?.url || dbInfo?.endpoint || '';
    const isProd = url.includes('10.0.23.11');
    if (isProd) return { check: 'safety', pass: false, ABORT: true, error: 'PRODUCTION DATABASE - STOP ALL TESTING' };
    return { check: 'safety', pass: true, db: url };
  })()`,

  // Phase 2: Infrastructure
  db_connection: `(async () => {
    const status = await window.__TAURI_INTERNALS__.invoke('get_connection_status');
    const info = await window.__TAURI_INTERNALS__.invoke('get_db_info');
    return { check: 'db_connection', pass: !!status, details: { status, info } };
  })()`,

  data_loaded: `(async () => {
    const p = await window.__TAURI_INTERNALS__.invoke('get_projects');
    const c = await window.__TAURI_INTERNALS__.invoke('get_companies');
    const f = await window.__TAURI_INTERNALS__.invoke('get_fees');
    const co = await window.__TAURI_INTERNALS__.invoke('get_contacts');
    const counts = { projects: p.length, companies: c.length, fees: f.length, contacts: co.length };
    return { check: 'data_loaded', pass: p.length > 0 && c.length > 0, details: counts };
  })()`,

  // Phase 3: Data validation
  project_statuses: `(async () => {
    const p = await window.__TAURI_INTERNALS__.invoke('get_projects');
    const statuses = [...new Set(p.map(x => x.status))].sort();
    const valid = ['Lead','RFP','Submitted','Awarded','Design','Construction','Completed','Lost','No Response','Cancelled','On Hold','Superseded'];
    const invalid = statuses.filter(s => !valid.includes(s));
    return { check: 'project_statuses', pass: invalid.length === 0, details: { found: statuses, invalid } };
  })()`,

  fee_statuses: `(async () => {
    const f = await window.__TAURI_INTERNALS__.invoke('get_fees');
    const statuses = [...new Set(f.map(x => x.status))].sort();
    const valid = ['Draft','Sent','Negotiation','Accepted','Rejected','No Response','Superseded'];
    const invalid = statuses.filter(s => !valid.includes(s));
    return { check: 'fee_statuses', pass: invalid.length === 0, details: { found: statuses, invalid } };
  })()`,

  entity_counts: `(async () => {
    const stats = await window.__TAURI_INTERNALS__.invoke('get_stats');
    return { check: 'entity_counts', pass: stats != null, details: stats };
  })()`,

  // Phase 4: Navigation
  navigate_dashboard: `(async () => {
    window.location.hash = '#/';
    await new Promise(r => setTimeout(r, 1000));
    const title = document.title;
    const bodyLen = document.body.innerHTML.length;
    return { check: 'navigate_dashboard', pass: bodyLen > 500, details: { title, domSize: bodyLen, hash: window.location.hash } };
  })()`,

  navigate_projects: `(async () => {
    window.location.hash = '#/projects';
    await new Promise(r => setTimeout(r, 1500));
    const items = document.querySelectorAll('.list-card');
    const filters = document.querySelectorAll('.emittiv-filter-select, .emittiv-search-input');
    return { check: 'navigate_projects', pass: items.length > 1, details: { items: items.length, filters: filters.length } };
  })()`,

  navigate_proposals: `(async () => {
    window.location.hash = '#/proposals';
    await new Promise(r => setTimeout(r, 1500));
    const items = document.querySelectorAll('.list-card');
    return { check: 'navigate_proposals', pass: items.length > 1, details: { items: items.length } };
  })()`,

  navigate_companies: `(async () => {
    window.location.hash = '#/companies';
    await new Promise(r => setTimeout(r, 1500));
    const items = document.querySelectorAll('.list-card');
    return { check: 'navigate_companies', pass: items.length > 1, details: { items: items.length } };
  })()`,

  navigate_contacts: `(async () => {
    window.location.hash = '#/contacts';
    await new Promise(r => setTimeout(r, 1500));
    const items = document.querySelectorAll('.list-card');
    return { check: 'navigate_contacts', pass: items.length > 1, details: { items: items.length } };
  })()`,

  // Phase 5: UI State
  modal_open_close: modalOpenClose,
  form_validation: formValidation,
  search_filter: searchFilter,
  dropdown_filter: dropdownFilter,
  keyboard_nav_1: keyboardNav1,
  keyboard_nav_2: keyboardNav2,
  keyboard_nav_3: keyboardNav3,
  keyboard_nav_4: keyboardNav4,
  keyboard_nav_5: keyboardNav5,
  bulk_select: bulkSelect,

  // Phase 6: CRUD
  crud_company: crudCompany,
  crud_contact: crudContact,
  crud_project: crudProject,
  crud_fee: crudFee,
  crud_cleanup: crudCleanup,

  // Phase 7: Integration
  status_transition: statusTransition,
  fee_project_mapping: feeProjectMapping,
  connection_indicator: connectionIndicator,
  entity_count_consistency: entityCountConsistency,
  settings_modal: settingsModal,
  detail_panel: detailPanel,

  // Phase 8: Regression
  recordid_v3: recordidV3,
  fee_no_infinity: feeNoInfinity,
  contact_full_name: contactFullName,
  fee_deser: feeDeser,
  company_id_extract: companyIdExtract,
  fee_status_legacy: feeStatusLegacy,
  project_status_legacy: projectStatusLegacy,
  nav_order: navOrder,
};

/**
 * Execution order for smoke tests (52 checks total).
 *
 * Safety check must always be first to prevent testing against production.
 * CRUD checks run as a sequential pipeline — crud_cleanup always runs last in the group.
 * If any check returns { ABORT: true }, stop and report immediately.
 */
export const CHECK_ORDER = [
  // Phase 1: Safety
  'safety',
  // Phase 2: Infrastructure
  'db_connection', 'data_loaded',
  // Phase 3: Data validation
  'project_statuses', 'fee_statuses', 'entity_counts',
  // Phase 4: Navigation
  'navigate_dashboard', 'navigate_projects', 'navigate_proposals',
  'navigate_companies', 'navigate_contacts',
  // Phase 5: UI State
  'modal_open_close', 'form_validation', 'search_filter', 'dropdown_filter',
  'keyboard_nav_1', 'keyboard_nav_2', 'keyboard_nav_3',
  'keyboard_nav_4', 'keyboard_nav_5', 'bulk_select',
  // Phase 6: CRUD (sequential pipeline)
  'crud_company', 'crud_contact', 'crud_project', 'crud_fee', 'crud_cleanup',
  // Phase 7: Integration
  'status_transition', 'fee_project_mapping', 'connection_indicator',
  'entity_count_consistency', 'settings_modal', 'detail_panel',
  // Phase 8: Regression
  'recordid_v3', 'fee_no_infinity', 'contact_full_name', 'fee_deser',
  'company_id_extract', 'fee_status_legacy', 'project_status_legacy', 'nav_order',
] as const;
