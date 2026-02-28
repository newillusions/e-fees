/**
 * Smoke Test Suite: Executable Check Definitions
 *
 * This module exports all smoke test checks as executable JavaScript/TypeScript strings.
 * Each check is a self-executing async IIFE that returns structured test results.
 *
 * EXECUTION ORDER:
 * 1. Safety check MUST run first — aborts all testing if production database detected
 * 2. Infrastructure checks (db_connection, data_loaded, counts)
 * 3. Data validation checks (project_statuses, fee_statuses)
 * 4. Navigation checks (dashboard, projects, proposals, companies, contacts)
 *
 * Result format: { check: string; pass: boolean; ABORT?: boolean; error?: string; details?: any }
 *
 * If any check returns { ABORT: true }, stop execution immediately and report error.
 */

export const CHECKS = {
  safety: `(async () => {
    const dbInfo = await window.__TAURI__.invoke('get_db_info');
    const url = dbInfo?.url || dbInfo?.endpoint || '';
    const isProd = url.includes('10.0.23.11');
    if (isProd) return { check: 'safety', pass: false, ABORT: true, error: 'PRODUCTION DATABASE - STOP ALL TESTING' };
    return { check: 'safety', pass: true, db: url };
  })()`,

  db_connection: `(async () => {
    const status = await window.__TAURI__.invoke('get_connection_status');
    const info = await window.__TAURI__.invoke('get_db_info');
    return { check: 'db_connection', pass: !!status, details: { status, info } };
  })()`,

  data_loaded: `(async () => {
    const p = await window.__TAURI__.invoke('get_projects');
    const c = await window.__TAURI__.invoke('get_companies');
    const f = await window.__TAURI__.invoke('get_fees');
    const co = await window.__TAURI__.invoke('get_contacts');
    const counts = { projects: p.length, companies: c.length, fees: f.length, contacts: co.length };
    return { check: 'data_loaded', pass: p.length > 0 && c.length > 0, details: counts };
  })()`,

  project_statuses: `(async () => {
    const p = await window.__TAURI__.invoke('get_projects');
    const statuses = [...new Set(p.map(x => x.status))].sort();
    const valid = ['Lead','RFP','Submitted','Awarded','Design','Construction','Completed','Lost','No Response','Cancelled','On Hold','Superseded'];
    const invalid = statuses.filter(s => !valid.includes(s));
    return { check: 'project_statuses', pass: invalid.length === 0, details: { found: statuses, invalid } };
  })()`,

  fee_statuses: `(async () => {
    const f = await window.__TAURI__.invoke('get_fees');
    const statuses = [...new Set(f.map(x => x.status))].sort();
    const valid = ['Draft','Sent','Negotiation','Accepted','Rejected','No Response','Superseded'];
    const invalid = statuses.filter(s => !valid.includes(s));
    return { check: 'fee_statuses', pass: invalid.length === 0, details: { found: statuses, invalid } };
  })()`,

  entity_counts: `(async () => {
    const stats = await window.__TAURI__.invoke('get_stats');
    return { check: 'entity_counts', pass: stats != null, details: stats };
  })()`,

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
    const rows = document.querySelectorAll('tr, [class*="row"], [class*="project-item"]');
    const filters = document.querySelectorAll('select, [class*="filter"]');
    return { check: 'navigate_projects', pass: rows.length > 1, details: { rows: rows.length, filters: filters.length } };
  })()`,

  navigate_proposals: `(async () => {
    window.location.hash = '#/proposals';
    await new Promise(r => setTimeout(r, 1500));
    const rows = document.querySelectorAll('tr, [class*="row"], [class*="fee-item"]');
    return { check: 'navigate_proposals', pass: rows.length > 1, details: { rows: rows.length } };
  })()`,

  navigate_companies: `(async () => {
    window.location.hash = '#/companies';
    await new Promise(r => setTimeout(r, 1500));
    const rows = document.querySelectorAll('tr, [class*="row"], [class*="company-item"]');
    return { check: 'navigate_companies', pass: rows.length > 1, details: { rows: rows.length } };
  })()`,

  navigate_contacts: `(async () => {
    window.location.hash = '#/contacts';
    await new Promise(r => setTimeout(r, 1500));
    const rows = document.querySelectorAll('tr, [class*="row"], [class*="contact-item"]');
    return { check: 'navigate_contacts', pass: rows.length > 1, details: { rows: rows.length } };
  })()`,
};

/**
 * Execution order for smoke tests.
 *
 * Safety check must always be first to prevent testing against production.
 * If any check returns { ABORT: true }, stop and report immediately.
 */
export const CHECK_ORDER = [
  'safety',
  'db_connection',
  'data_loaded',
  'project_statuses',
  'fee_statuses',
  'entity_counts',
  'navigate_dashboard',
  'navigate_projects',
  'navigate_proposals',
  'navigate_companies',
  'navigate_contacts',
] as const;
