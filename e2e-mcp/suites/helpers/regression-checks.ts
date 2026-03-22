/**
 * Regression Check Module - Guards Against Past E-Fees Bugs
 *
 * Each export is an async IIFE (as a template literal string) executed inside
 * the E-Fees Tauri webview via Tauri MCP execute_js. Results are stored in
 * window.__SMOKE_RESULT as a JSON string — read back with a second execute_js call.
 *
 * All checks are READ-ONLY. No data is created or modified.
 *
 * Past bugs guarded against:
 * - SurrealDB v3 RecordId serialized as {table, key} object instead of string
 * - SurrealDB v3 math::max([]) returns -Infinity, breaking fee deserialization
 * - Contact full_name not computed by the backend (first_name + last_name)
 * - Fee/project status enums mixed up before domain model restructure (v0.13.0)
 * - Sidebar navigation order changed in a past update
 *
 * Usage:
 * ```typescript
 * // Execute check:
 * await executeJs(recordidV3);
 * // Read result:
 * const raw = await executeJs('window.__SMOKE_RESULT');
 * const result = JSON.parse(raw);  // { check: 'recordid_v3', pass: boolean, ... }
 * ```
 */

/**
 * Check 1: recordidV3
 *
 * SurrealDB v3 changed RecordId serialization from a plain string to
 * an object { tb: "projects", id: { String: "abc" } }. The frontend must
 * handle IDs as strings. This check verifies every project's `id` field
 * is a string, not an object.
 *
 * Bug introduced: SurrealDB v3 upgrade
 * Fix applied: Frontend normalization in multiple files
 *
 * Result: { check: 'recordid_v3', pass: boolean, details: { objectIds, total } }
 */
export const recordidV3 = `(async () => {
  try {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const projects = await invoke('get_projects');

    if (!Array.isArray(projects)) {
      window.__SMOKE_RESULT = JSON.stringify({
        check: 'recordid_v3',
        pass: false,
        error: 'get_projects did not return an array: ' + typeof projects
      });
      return;
    }

    const objectIds = [];
    for (const project of projects) {
      if (project.id !== null && project.id !== undefined && typeof project.id !== 'string') {
        objectIds.push({
          id: JSON.stringify(project.id),
          name: project.name || '(no name)'
        });
      }
    }

    const pass = objectIds.length === 0;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'recordid_v3',
      pass,
      details: {
        total: projects.length,
        objectIds,
        message: pass
          ? 'All project IDs are strings'
          : objectIds.length + ' project(s) have non-string IDs (SurrealDB v3 RecordId regression)'
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'recordid_v3',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 2: feeNoInfinity
 *
 * SurrealDB v3 math::max([]) returns -Infinity (float) instead of NONE/null.
 * When a fee has empty pricing arrays, deserialized numeric fields come back
 * as -Infinity, which breaks the frontend. This check scans all fees for the
 * presence of -Infinity or NaN in their serialized JSON.
 *
 * Bug: SurrealDB v3 math::max([]) = -Infinity
 * Fix: IF array::len(arr) > 0 THEN math::max(arr) ELSE 0 END in SurrealQL
 *
 * Result: { check: 'fee_no_infinity', pass: boolean, details: { badFees, total } }
 */
export const feeNoInfinity = `(async () => {
  try {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const fees = await invoke('get_fees');

    if (!Array.isArray(fees)) {
      window.__SMOKE_RESULT = JSON.stringify({
        check: 'fee_no_infinity',
        pass: false,
        error: 'get_fees did not return an array: ' + typeof fees
      });
      return;
    }

    const badFees = [];
    for (const fee of fees) {
      let serialized;
      try {
        serialized = JSON.stringify(fee);
      } catch(serErr) {
        // JSON.stringify throws on circular refs but not -Infinity
        // -Infinity serializes to "null" in JSON.stringify, but we catch it via isFinite
        serialized = '';
      }

      // JSON.stringify converts -Infinity to null, so check the object directly
      function hasInfinityOrNaN(obj, path) {
        if (obj === null || obj === undefined) return [];
        if (typeof obj === 'number') {
          if (!isFinite(obj) || isNaN(obj)) return [path + '=' + obj];
          return [];
        }
        if (typeof obj === 'object') {
          const found = [];
          for (const key of Object.keys(obj)) {
            found.push(...hasInfinityOrNaN(obj[key], path ? path + '.' + key : key));
          }
          return found;
        }
        return [];
      }

      const badPaths = hasInfinityOrNaN(fee, '');
      if (badPaths.length > 0) {
        const feeId = typeof fee.id === 'string' ? fee.id : JSON.stringify(fee.id);
        badFees.push({ id: feeId, badPaths });
      }
    }

    const pass = badFees.length === 0;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'fee_no_infinity',
      pass,
      details: {
        total: fees.length,
        badFees,
        message: pass
          ? 'No fees contain -Infinity or NaN'
          : badFees.length + ' fee(s) contain -Infinity or NaN (math::max([]) regression)'
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'fee_no_infinity',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 3: contactFullName
 *
 * The backend computes full_name from first_name + last_name when creating/updating
 * contacts. An earlier bug meant full_name was not being set, resulting in empty
 * or null full_name fields. This check verifies every contact has a non-empty full_name.
 *
 * Bug: Contact struct in Rust did not compute full_name manually
 * Fix: db/mod.rs:712 — manual full_name computation added
 *
 * Result: { check: 'contact_full_name', pass: boolean, details: { missing, total } }
 */
export const contactFullName = `(async () => {
  try {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const contacts = await invoke('get_contacts');

    if (!Array.isArray(contacts)) {
      window.__SMOKE_RESULT = JSON.stringify({
        check: 'contact_full_name',
        pass: false,
        error: 'get_contacts did not return an array: ' + typeof contacts
      });
      return;
    }

    const missing = [];
    for (const contact of contacts) {
      const fullName = contact.full_name || contact.fullName || '';
      if (!fullName || fullName.trim() === '') {
        const contactId = typeof contact.id === 'string' ? contact.id : JSON.stringify(contact.id);
        missing.push({
          id: contactId,
          first_name: contact.first_name || contact.firstName || '',
          last_name: contact.last_name || contact.lastName || '',
          full_name: contact.full_name || contact.fullName || null
        });
      }
    }

    const pass = missing.length === 0;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'contact_full_name',
      pass,
      details: {
        total: contacts.length,
        missing,
        message: pass
          ? 'All contacts have a non-empty full_name'
          : missing.length + ' contact(s) missing full_name (backend computation regression)'
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'contact_full_name',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 4: feeDeser
 *
 * Verifies all numeric pricing fields on fees are actual finite numbers.
 * Checks pricing.total and any top-level numeric fields. Catches both the
 * -Infinity regression and any future deserialization failures that produce
 * non-numeric values in numeric slots.
 *
 * Result: { check: 'fee_deser', pass: boolean, details: { badFees, total } }
 */
export const feeDeser = `(async () => {
  try {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const fees = await invoke('get_fees');

    if (!Array.isArray(fees)) {
      window.__SMOKE_RESULT = JSON.stringify({
        check: 'fee_deser',
        pass: false,
        error: 'get_fees did not return an array: ' + typeof fees
      });
      return;
    }

    // Fields we expect to be numeric when present
    const NUMERIC_TOP_FIELDS = ['rev'];
    const NUMERIC_PRICING_FIELDS = ['total', 'design_fee', 'post_contract_fee', 'reimbursable'];

    const badFees = [];
    for (const fee of fees) {
      const badFields = [];
      const feeId = typeof fee.id === 'string' ? fee.id : JSON.stringify(fee.id);

      // Check top-level numeric fields
      for (const field of NUMERIC_TOP_FIELDS) {
        if (fee[field] !== undefined && fee[field] !== null) {
          const val = fee[field];
          if (typeof val !== 'number' || !isFinite(val) || isNaN(val)) {
            badFields.push({ field, value: val, type: typeof val });
          }
        }
      }

      // Check pricing sub-object
      const pricing = fee.pricing;
      if (pricing && typeof pricing === 'object') {
        for (const field of NUMERIC_PRICING_FIELDS) {
          if (pricing[field] !== undefined && pricing[field] !== null) {
            const val = pricing[field];
            if (typeof val !== 'number' || !isFinite(val) || isNaN(val)) {
              badFields.push({ field: 'pricing.' + field, value: val, type: typeof val });
            }
          }
        }
      }

      if (badFields.length > 0) {
        badFees.push({ id: feeId, badFields });
      }
    }

    const pass = badFees.length === 0;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'fee_deser',
      pass,
      details: {
        total: fees.length,
        badFees,
        message: pass
          ? 'All fee numeric fields deserialize correctly'
          : badFees.length + ' fee(s) have non-finite numeric fields (deserialization regression)'
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'fee_deser',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 5: companyIdExtract
 *
 * Mirrors the recordidV3 check but for companies. SurrealDB v3 RecordId
 * objects (`{tb, id}`) must be normalized to strings by the frontend.
 * An earlier bug in company ID extraction caused company lookups to fail.
 *
 * Bug: SurrealDB Thing objects not normalized in frontend
 * Fix: Company ID extraction helper in ContactModal.svelte and stores.ts
 *
 * Result: { check: 'company_id_extract', pass: boolean, details: { objectIds, total } }
 */
export const companyIdExtract = `(async () => {
  try {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const companies = await invoke('get_companies');

    if (!Array.isArray(companies)) {
      window.__SMOKE_RESULT = JSON.stringify({
        check: 'company_id_extract',
        pass: false,
        error: 'get_companies did not return an array: ' + typeof companies
      });
      return;
    }

    const objectIds = [];
    for (const company of companies) {
      if (company.id !== null && company.id !== undefined && typeof company.id !== 'string') {
        objectIds.push({
          id: JSON.stringify(company.id),
          name: company.name || '(no name)'
        });
      }
    }

    const pass = objectIds.length === 0;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'company_id_extract',
      pass,
      details: {
        total: companies.length,
        objectIds,
        message: pass
          ? 'All company IDs are strings'
          : objectIds.length + ' company/companies have non-string IDs (SurrealDB v3 RecordId regression)'
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'company_id_extract',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 6: feeStatusLegacy
 *
 * Before the domain model restructure (v0.13.0), fee statuses "Awarded" and "Lost"
 * were incorrectly used (these belong to the project status enum). This check
 * verifies no fee has a legacy project status.
 *
 * Valid fee statuses: Draft, Sent, Negotiation, Accepted, Rejected, No Response, Superseded
 * Legacy (project) statuses incorrectly used on fees: Awarded, Lost
 *
 * Bug fixed: v0.13.0 domain model restructure (2026-02-27)
 *
 * Result: { check: 'fee_status_legacy', pass: boolean, details: { legacyFees, total } }
 */
export const feeStatusLegacy = `(async () => {
  try {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const fees = await invoke('get_fees');

    if (!Array.isArray(fees)) {
      window.__SMOKE_RESULT = JSON.stringify({
        check: 'fee_status_legacy',
        pass: false,
        error: 'get_fees did not return an array: ' + typeof fees
      });
      return;
    }

    // These are project statuses — INVALID on fee records
    const LEGACY_PROJECT_STATUSES = ['Awarded', 'Lost'];

    const legacyFees = [];
    for (const fee of fees) {
      if (LEGACY_PROJECT_STATUSES.includes(fee.status)) {
        const feeId = typeof fee.id === 'string' ? fee.id : JSON.stringify(fee.id);
        legacyFees.push({ id: feeId, status: fee.status });
      }
    }

    const pass = legacyFees.length === 0;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'fee_status_legacy',
      pass,
      details: {
        total: fees.length,
        legacyFees,
        message: pass
          ? 'No fees have legacy project statuses'
          : legacyFees.length + ' fee(s) have invalid legacy status (Awarded/Lost — v0.13.0 regression)'
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'fee_status_legacy',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 7: projectStatusLegacy
 *
 * Mirrors feeStatusLegacy but for projects. Before v0.13.0, fee statuses
 * "Accepted" and "Rejected" were incorrectly used on project records.
 * This check verifies no project has a legacy fee status.
 *
 * Valid project statuses: Lead, RFP, Submitted, Awarded, Design, Construction,
 *                         Completed, Lost, No Response, Cancelled, On Hold, Superseded
 * Legacy (fee) statuses incorrectly used on projects: Accepted, Rejected
 *
 * Bug fixed: v0.13.1 patch release (fee status alignment)
 *
 * Result: { check: 'project_status_legacy', pass: boolean, details: { legacyProjects, total } }
 */
export const projectStatusLegacy = `(async () => {
  try {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const projects = await invoke('get_projects');

    if (!Array.isArray(projects)) {
      window.__SMOKE_RESULT = JSON.stringify({
        check: 'project_status_legacy',
        pass: false,
        error: 'get_projects did not return an array: ' + typeof projects
      });
      return;
    }

    // These are fee statuses — INVALID on project records
    const LEGACY_FEE_STATUSES = ['Accepted', 'Rejected'];

    const legacyProjects = [];
    for (const project of projects) {
      if (LEGACY_FEE_STATUSES.includes(project.status)) {
        const projectId = typeof project.id === 'string' ? project.id : JSON.stringify(project.id);
        legacyProjects.push({ id: projectId, name: project.name || '(no name)', status: project.status });
      }
    }

    const pass = legacyProjects.length === 0;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'project_status_legacy',
      pass,
      details: {
        total: projects.length,
        legacyProjects,
        message: pass
          ? 'No projects have legacy fee statuses'
          : legacyProjects.length + ' project(s) have invalid legacy status (Accepted/Rejected — v0.13.1 regression)'
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'project_status_legacy',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 8: navOrder
 *
 * Verifies the sidebar navigation items appear in the correct order:
 * Dashboard → Projects → Companies → Contacts → Proposals
 *
 * The navigation was reordered in a past update (Proposals moved to last position,
 * Companies and Contacts inserted before it). This check guards against regression
 * to the old ordering.
 *
 * Expected order: Dashboard, Projects, Companies, Contacts, Proposals
 *
 * Result: { check: 'nav_order', pass: boolean, details: { found, expected, outOfOrder } }
 */
export const navOrder = `(async () => {
  try {
    // Find all nav links/items in the sidebar
    const navLinks = document.querySelectorAll(
      'nav a, nav [class*="nav-item"], nav [class*="link"], [class*="sidebar"] a, [class*="sidebar"] [class*="nav"]'
    );

    const EXPECTED_ORDER = ['Dashboard', 'Projects', 'Companies', 'Contacts', 'Proposals'];

    // Extract text content from nav items
    const foundTexts = [];
    for (const link of navLinks) {
      const text = (link.textContent || '').trim();
      // Filter to only known nav labels (ignore icons, tooltips, etc.)
      if (EXPECTED_ORDER.some(label => text === label || text.includes(label))) {
        // Use the matching expected label to normalize
        const matchedLabel = EXPECTED_ORDER.find(label => text === label || text.includes(label));
        if (matchedLabel && !foundTexts.includes(matchedLabel)) {
          foundTexts.push(matchedLabel);
        }
      }
    }

    // Check order: every label that appears must appear in the same relative order as EXPECTED_ORDER
    const foundInExpectedOrder = EXPECTED_ORDER.filter(label => foundTexts.includes(label));
    const outOfOrder = [];

    for (let i = 0; i < foundTexts.length - 1; i++) {
      const currentIdx = EXPECTED_ORDER.indexOf(foundTexts[i]);
      const nextIdx = EXPECTED_ORDER.indexOf(foundTexts[i + 1]);
      if (currentIdx > nextIdx) {
        outOfOrder.push({ before: foundTexts[i], after: foundTexts[i + 1] });
      }
    }

    // Pass if all expected labels found and none out of order
    const allFound = EXPECTED_ORDER.every(label => foundTexts.includes(label));
    const pass = allFound && outOfOrder.length === 0;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'nav_order',
      pass,
      details: {
        found: foundTexts,
        expected: EXPECTED_ORDER,
        allFound,
        outOfOrder,
        message: pass
          ? 'Navigation order is correct: ' + foundTexts.join(' → ')
          : !allFound
            ? 'Missing nav items: ' + EXPECTED_ORDER.filter(l => !foundTexts.includes(l)).join(', ')
            : 'Nav items out of order: ' + outOfOrder.map(o => o.before + ' before ' + o.after).join(', ')
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'nav_order',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Execution order for regression checks.
 * All checks are independent and read-only — order is for reporting consistency.
 */
export const REGRESSION_CHECK_ORDER = [
  'recordidV3',
  'feeNoInfinity',
  'contactFullName',
  'feeDeser',
  'companyIdExtract',
  'feeStatusLegacy',
  'projectStatusLegacy',
  'navOrder',
] as const;

export type RegressionCheckName = typeof REGRESSION_CHECK_ORDER[number];
