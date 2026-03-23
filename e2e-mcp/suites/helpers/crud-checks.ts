/**
 * CRUD Check Module - Sequential Pipeline for E-Fees Smoke Tests
 *
 * This module exports JavaScript code strings executed inside the E-Fees Tauri
 * application webview via the Tauri MCP execute_js tool. Each check is an async
 * IIFE that stores its result in window.__SMOKE_RESULT as a JSON string.
 *
 * NOTE: Unlike smoke-checks.ts (which returns directly), these checks use
 * window.__SMOKE_RESULT because execute_js cannot resolve async promises.
 * After executing a check, read the result with a second execute_js call:
 *   `window.__SMOKE_RESULT`
 *
 * SEQUENTIAL PIPELINE — run in order:
 * 1. crudCompany   — creates a test company, stores ID in window.__CRUD_IDS.company
 * 2. crudContact   — creates a test contact linked to company
 * 3. crudProject   — creates a test project
 * 4. crudFee       — creates a test fee linking all three entities
 * 5. crudCleanup   — deletes all created entities + any orphaned DELETE ME records
 *
 * IDs are persisted in window.__CRUD_IDS across execute_js calls (SPA global state).
 *
 * All test data names MUST start with "DELETE ME" — enforced in every check.
 */

/**
 * Helper: extract a string ID from a SurrealDB record response.
 *
 * SurrealDB v3 may return IDs as:
 *   - string: "company:abc123"
 *   - object: { tb: "company", id: { String: "abc123" } }
 *   - object: { tb: "company", id: "abc123" }
 *
 * Returns the full "table:id" string suitable for invoke() calls.
 */
const extractIdHelper = `
  function extractId(record, tableName) {
    if (!record) return null;
    if (typeof record.id === 'string') return record.id;
    if (record.id && typeof record.id === 'object') {
      const tb = record.id.tb || record.id.table || tableName;
      const key = record.id.id || record.id.key;
      if (key && typeof key === 'object') {
        // { String: "abc" } or { Number: 1 } etc.
        const inner = Object.values(key)[0];
        return tb + ':' + inner;
      }
      if (key !== undefined && key !== null) return tb + ':' + key;
    }
    // Already a plain string at top level
    if (typeof record === 'string') return record;
    return null;
  }
  // Strip table prefix for foreign key fields (e.g. "company:ABC" → "ABC")
  function stripTable(fullId) {
    if (!fullId) return fullId;
    const idx = fullId.indexOf(':');
    return idx >= 0 ? fullId.substring(idx + 1) : fullId;
  }
`;

/**
 * Check 1: Company CRUD
 *
 * - Cleans up any pre-existing "DELETE ME" companies
 * - Creates a company, reads it back, updates it, verifies the update
 * - Stores the created ID in window.__CRUD_IDS.company
 *
 * Result: { check: 'crud_company', pass: boolean, details: {...}, error?: string }
 */
export const crudCompany = `(async () => {
  try {
    window.__CRUD_IDS = window.__CRUD_IDS || {};
    ${extractIdHelper}

    const invoke = window.__TAURI_INTERNALS__.invoke;

    // Clean up any orphaned DELETE ME companies from previous runs
    const existing = await invoke('get_companies');
    const orphans = Array.isArray(existing)
      ? existing.filter(c => c.name && c.name.startsWith('DELETE ME'))
      : [];
    for (const orphan of orphans) {
      const orphanId = extractId(orphan, 'company');
      if (orphanId) {
        try { await invoke('delete_company', { id: orphanId }); } catch(_) {}
      }
    }

    // Create
    const ts = Date.now();
    const abbr = 'DM' + String(ts).slice(-4);
    const created = await invoke('create_company', {
      company: { name: 'DELETE ME - Test Company ' + ts, name_short: abbr, abbreviation: abbr, city: '', country: 'Test', reg_no: null, tax_no: null }
    });
    const createdId = extractId(created, 'company');
    if (!createdId) throw new Error('create_company returned no ID: ' + JSON.stringify(created));

    // Read back
    const companies = await invoke('get_companies');
    const found = Array.isArray(companies)
      ? companies.find(c => extractId(c, 'company') === createdId)
      : null;
    if (!found) throw new Error('Could not find created company with id: ' + createdId);

    // Update — must include all required fields to avoid SurrealDB v3 NONE rejection
    const updatedName = 'DELETE ME - Updated Company ' + Date.now();
    await invoke('update_company', {
      id: stripTable(createdId),
      companyUpdate: { name: updatedName, name_short: abbr, abbreviation: abbr, city: '', country: 'Test', reg_no: '', tax_no: '' }
    });

    // Verify update
    const afterUpdate = await invoke('get_companies');
    const updated = Array.isArray(afterUpdate)
      ? afterUpdate.find(c => extractId(c, 'company') === createdId)
      : null;
    if (!updated) throw new Error('Could not find company after update: ' + createdId);
    if (!updated.name.startsWith('DELETE ME - Updated Company')) {
      throw new Error('Update did not take effect. name=' + updated.name);
    }

    window.__CRUD_IDS.company = createdId;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'crud_company',
      pass: true,
      details: {
        id: createdId,
        name: updated.name,
        orphansCleaned: orphans.length
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'crud_company',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 2: Contact CRUD
 *
 * Requires window.__CRUD_IDS.company from crudCompany.
 *
 * - Creates a contact linked to the company
 * - Reads it back, updates last_name, verifies the update
 * - Stores the created ID in window.__CRUD_IDS.contact
 *
 * Result: { check: 'crud_contact', pass: boolean, details: {...}, error?: string }
 */
export const crudContact = `(async () => {
  try {
    window.__CRUD_IDS = window.__CRUD_IDS || {};
    ${extractIdHelper}

    const invoke = window.__TAURI_INTERNALS__.invoke;

    const companyId = window.__CRUD_IDS.company;
    if (!companyId) throw new Error('Missing window.__CRUD_IDS.company — run crudCompany first');

    // Create
    const ts = Date.now();
    const created = await invoke('create_contact', {
      contact: {
        first_name: 'DELETE ME',
        last_name: 'Test Contact ' + ts,
        email: 'delete-me-' + ts + '@example.com',
        phone: '+971500000000',
        position: 'Test',
        company: stripTable(companyId)
      }
    });
    const createdId = extractId(created, 'contacts');
    if (!createdId) throw new Error('create_contact returned no ID: ' + JSON.stringify(created));

    // Read back
    const contacts = await invoke('get_contacts');
    const found = Array.isArray(contacts)
      ? contacts.find(c => extractId(c, 'contacts') === createdId)
      : null;
    if (!found) throw new Error('Could not find created contact with id: ' + createdId);

    // Update (uses ContactUpdate partial struct, param name is contact_update)
    const updatedLastName = 'Updated Contact ' + Date.now();
    await invoke('update_contact', {
      id: stripTable(createdId),
      contactUpdate: { last_name: updatedLastName }
    });

    // Verify update
    const afterUpdate = await invoke('get_contacts');
    const updated = Array.isArray(afterUpdate)
      ? afterUpdate.find(c => extractId(c, 'contacts') === createdId)
      : null;
    if (!updated) throw new Error('Could not find contact after update: ' + createdId);
    if (updated.last_name !== updatedLastName && updated.lastName !== updatedLastName) {
      throw new Error('Update did not take effect. last_name=' + (updated.last_name || updated.lastName));
    }

    window.__CRUD_IDS.contact = createdId;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'crud_contact',
      pass: true,
      details: {
        id: createdId,
        first_name: updated.first_name || updated.firstName,
        last_name: updated.last_name || updated.lastName
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'crud_contact',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 3: Project CRUD
 *
 * - Creates a test project with country code 971 (UAE), year 26, sequence 99
 * - Reads it back, updates the name, verifies the update
 * - Stores the created ID in window.__CRUD_IDS.project
 *
 * Result: { check: 'crud_project', pass: boolean, details: {...}, error?: string }
 */
export const crudProject = `(async () => {
  try {
    window.__CRUD_IDS = window.__CRUD_IDS || {};
    ${extractIdHelper}

    const invoke = window.__TAURI_INTERNALS__.invoke;

    // Create
    const ts = Date.now();
    const now = new Date().toISOString();
    const seqNum = Number(String(ts).slice(-2));
    const projId = '26_971' + String(seqNum).padStart(2, '0');
    const created = await invoke('create_project', {
      project: {
        name: 'DELETE ME - Test Project ' + ts,
        name_short: 'DELME',
        status: 'Lead',
        number: { country: 971, year: 26, seq: seqNum, id: projId },
        city: 'Test',
        country: 'Test',
        area: 'Test',
        folder: '',
        time: { created_at: now, updated_at: now }
      }
    });
    const createdId = extractId(created, 'projects');
    if (!createdId) throw new Error('create_project returned no ID: ' + JSON.stringify(created));

    // Read back
    const projects = await invoke('get_projects');
    const found = Array.isArray(projects)
      ? projects.find(p => extractId(p, 'projects') === createdId)
      : null;
    if (!found) throw new Error('Could not find created project with id: ' + createdId);

    // Update
    const updatedName = 'DELETE ME - Updated Project ' + Date.now();
    await invoke('update_project', {
      id: createdId,
      project: { name: updatedName }
    });

    // Verify update
    const afterUpdate = await invoke('get_projects');
    const updated = Array.isArray(afterUpdate)
      ? afterUpdate.find(p => extractId(p, 'projects') === createdId)
      : null;
    if (!updated) throw new Error('Could not find project after update: ' + createdId);
    if (!updated.name.startsWith('DELETE ME - Updated Project')) {
      throw new Error('Update did not take effect. name=' + updated.name);
    }

    window.__CRUD_IDS.project = createdId;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'crud_project',
      pass: true,
      details: {
        id: createdId,
        name: updated.name
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'crud_project',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 4: Fee CRUD
 *
 * Requires window.__CRUD_IDS.project, .company, and .contact from previous checks.
 *
 * - Creates a fee linking all three entities
 * - Reads it back and verifies the fee fields
 * - Stores the created ID in window.__CRUD_IDS.fee
 *
 * Result: { check: 'crud_fee', pass: boolean, details: {...}, error?: string }
 */
export const crudFee = `(async () => {
  try {
    window.__CRUD_IDS = window.__CRUD_IDS || {};
    ${extractIdHelper}

    const invoke = window.__TAURI_INTERNALS__.invoke;

    const projectId = window.__CRUD_IDS.project;
    const companyId = window.__CRUD_IDS.company;
    const contactId = window.__CRUD_IDS.contact;

    if (!projectId) throw new Error('Missing window.__CRUD_IDS.project — run crudProject first');
    if (!companyId) throw new Error('Missing window.__CRUD_IDS.company — run crudCompany first');
    if (!contactId) throw new Error('Missing window.__CRUD_IDS.contact — run crudContact first');

    // Create — FeeCreate requires all staff/display fields
    const ts = Date.now();
    const created = await invoke('create_fee', {
      fee: {
        name: 'DELETE ME - Test Fee ' + ts,
        number: 'DM-' + ts,
        rev: 1,
        status: 'Draft',
        issue_date: '202603',
        activity: 'Test',
        package: 'Test',
        project_id: stripTable(projectId),
        company_id: stripTable(companyId),
        contact_id: stripTable(contactId),
        staff_name: 'Test',
        staff_email: 'test@example.com',
        staff_phone: '+971500000000',
        staff_position: 'Test',
        strap_line: 'Test',
        revisions: []
      }
    });
    const createdId = extractId(created, 'fee');
    if (!createdId) throw new Error('create_fee returned no ID: ' + JSON.stringify(created));

    // Read back
    const fees = await invoke('get_fees');
    const found = Array.isArray(fees)
      ? fees.find(f => extractId(f, 'fee') === createdId)
      : null;
    if (!found) throw new Error('Could not find created fee with id: ' + createdId);

    // Verify key fields
    if (found.status !== 'Draft') {
      throw new Error('Fee status mismatch. expected=Draft actual=' + found.status);
    }
    if (String(found.issue_date) !== '202603' && String(found.issueDate) !== '202603') {
      throw new Error('Fee issue_date mismatch. got=' + (found.issue_date || found.issueDate));
    }

    window.__CRUD_IDS.fee = createdId;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'crud_fee',
      pass: true,
      details: {
        id: createdId,
        status: found.status,
        issue_date: found.issue_date || found.issueDate,
        rev: found.rev
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'crud_fee',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 5: Cleanup
 *
 * Deletes all entities created by the CRUD pipeline in reverse order:
 *   fee → project → contact → company
 *
 * Then scans all four entity types for any remaining "DELETE ME" records
 * and deletes them (catches orphans from failed runs).
 *
 * Reports total number of entities cleaned up.
 *
 * Result: { check: 'crud_cleanup', pass: boolean, details: { deleted, orphans }, error?: string }
 */
export const crudCleanup = `(async () => {
  try {
    window.__CRUD_IDS = window.__CRUD_IDS || {};
    ${extractIdHelper}

    const invoke = window.__TAURI_INTERNALS__.invoke;
    const ids = window.__CRUD_IDS;
    const deleted = [];
    const errors = [];

    // Delete in reverse dependency order
    if (ids.fee) {
      try {
        await invoke('delete_fee', { id: ids.fee });
        deleted.push(ids.fee);
      } catch(e) {
        errors.push('delete_fee(' + ids.fee + '): ' + (e instanceof Error ? e.message : String(e)));
      }
    }

    if (ids.project) {
      try {
        await invoke('delete_project', { id: ids.project });
        deleted.push(ids.project);
      } catch(e) {
        errors.push('delete_project(' + ids.project + '): ' + (e instanceof Error ? e.message : String(e)));
      }
    }

    if (ids.contact) {
      try {
        await invoke('delete_contact', { id: ids.contact });
        deleted.push(ids.contact);
      } catch(e) {
        errors.push('delete_contact(' + ids.contact + '): ' + (e instanceof Error ? e.message : String(e)));
      }
    }

    if (ids.company) {
      try {
        await invoke('delete_company', { id: ids.company });
        deleted.push(ids.company);
      } catch(e) {
        errors.push('delete_company(' + ids.company + '): ' + (e instanceof Error ? e.message : String(e)));
      }
    }

    // Orphan scan — catch any DELETE ME records from previous failed runs
    const orphanDeleted = [];

    const [fees, projects, contacts, companies] = await Promise.all([
      invoke('get_fees').catch(() => []),
      invoke('get_projects').catch(() => []),
      invoke('get_contacts').catch(() => []),
      invoke('get_companies').catch(() => [])
    ]);

    for (const fee of (Array.isArray(fees) ? fees : [])) {
      if (fee.name && fee.name.startsWith('DELETE ME')) {
        const fId = extractId(fee, 'fee');
        if (fId && fId !== ids.fee) {
          try {
            await invoke('delete_fee', { id: fId });
            orphanDeleted.push(fId);
          } catch(_) {}
        }
      }
    }

    for (const project of (Array.isArray(projects) ? projects : [])) {
      if (project.name && project.name.startsWith('DELETE ME')) {
        const pId = extractId(project, 'projects');
        if (pId && pId !== ids.project) {
          try {
            await invoke('delete_project', { id: pId });
            orphanDeleted.push(pId);
          } catch(_) {}
        }
      }
    }

    for (const contact of (Array.isArray(contacts) ? contacts : [])) {
      const firstName = contact.first_name || contact.firstName || '';
      if (firstName.startsWith('DELETE ME')) {
        const cId = extractId(contact, 'contacts');
        if (cId && cId !== ids.contact) {
          try {
            await invoke('delete_contact', { id: cId });
            orphanDeleted.push(cId);
          } catch(_) {}
        }
      }
    }

    for (const company of (Array.isArray(companies) ? companies : [])) {
      if (company.name && company.name.startsWith('DELETE ME')) {
        const coId = extractId(company, 'company');
        if (coId && coId !== ids.company) {
          try {
            await invoke('delete_company', { id: coId });
            orphanDeleted.push(coId);
          } catch(_) {}
        }
      }
    }

    // Clear the IDs store
    window.__CRUD_IDS = {};

    const pass = errors.length === 0;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'crud_cleanup',
      pass,
      details: {
        deleted,
        orphansCleaned: orphanDeleted.length,
        orphanIds: orphanDeleted,
        errors: errors.length > 0 ? errors : undefined
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'crud_cleanup',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Execution order for CRUD checks.
 * Must run sequentially — each step depends on IDs from the previous step.
 */
export const CRUD_CHECK_ORDER = [
  'crudCompany',
  'crudContact',
  'crudProject',
  'crudFee',
  'crudCleanup',
] as const;

export type CrudCheckName = typeof CRUD_CHECK_ORDER[number];
