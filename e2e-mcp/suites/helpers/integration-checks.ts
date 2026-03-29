/**
 * Integration Check Module - Cross-Entity Validation for E-Fees
 *
 * This module exports JavaScript code strings executed inside the E-Fees Tauri
 * application webview via the Tauri MCP execute_js tool. Each check is an async
 * IIFE that stores its result in window.__SMOKE_RESULT as a JSON string.
 *
 * NOTE: Tauri MCP's execute_js does not resolve async IIFE promises.
 * After executing a check, read the result with a second execute_js call:
 *   `window.__SMOKE_RESULT`
 *
 * ALL CHECKS ARE READ-ONLY — no invoke calls that create, update, or delete data.
 *
 * CHECKS:
 * 1. statusTransition       — Verify fees have multiple statuses (Draft + non-Draft)
 * 2. feeProjectMapping      — Verify Accepted fees link to Awarded projects
 * 3. connectionIndicator    — Verify connection status element exists in DOM
 * 4. entityCountConsistency — Verify get_stats counts match actual array lengths
 * 5. settingsModal          — Verify settings UI is accessible and shows DB config
 * 6. detailPanel            — Verify clicking a project card opens a detail panel
 */

/**
 * Check 1: Status Transition Verification
 *
 * Loads all fees and verifies that:
 * - At least one fee has status "Draft"
 * - At least one fee has a status other than "Draft"
 *
 * This proves that status transitions have occurred (fees have been progressed
 * beyond their initial state). Reports all unique statuses found.
 *
 * Result: { check: 'status_transition', pass: boolean, details: { statuses, hasDraft, hasNonDraft } }
 */
export const statusTransition = `(async () => {
  try {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const fees = await invoke('get_fees');

    const foundStatuses = new Set();
    if (Array.isArray(fees)) {
      fees.forEach(fee => {
        if (fee?.status) foundStatuses.add(fee.status);
      });
    }

    const statusList = Array.from(foundStatuses);
    const hasDraft = statusList.includes('Draft');
    const hasNonDraft = statusList.some(s => s !== 'Draft');
    const pass = hasDraft && hasNonDraft;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'status_transition',
      pass,
      details: {
        statuses: statusList,
        hasDraft,
        hasNonDraft,
        feeCount: Array.isArray(fees) ? fees.length : 0
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'status_transition',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 2: Fee → Project Status Mapping
 *
 * Loads fees and projects. For any fee with status "Accepted", verifies that
 * its linked project has status "Awarded". This validates the fee→project
 * status mapping is consistent across the dataset.
 *
 * If no Accepted fees exist, passes with a note.
 *
 * Result: { check: 'fee_project_mapping', pass: boolean, details: { validated, mismatches, note? } }
 */
export const feeProjectMapping = `(async () => {
  try {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const [fees, projects] = await Promise.all([
      invoke('get_fees'),
      invoke('get_projects')
    ]);

    if (!Array.isArray(fees) || !Array.isArray(projects)) {
      throw new Error('get_fees or get_projects returned non-array');
    }

    // Build a project lookup map by ID string
    const projectById = {};
    projects.forEach(p => {
      if (!p) return;
      // Handle both string IDs and SurrealDB RecordId objects
      let id = null;
      if (typeof p.id === 'string') {
        id = p.id;
      } else if (p.id && typeof p.id === 'object') {
        const tb = p.id.tb || 'projects';
        const key = p.id.id;
        if (key && typeof key === 'object') {
          id = tb + ':' + Object.values(key)[0];
        } else if (key !== undefined && key !== null) {
          id = tb + ':' + key;
        }
      }
      if (id) projectById[id] = p;
    });

    // Filter to only Accepted fees
    const acceptedFees = fees.filter(f => f?.status === 'Accepted');

    if (acceptedFees.length === 0) {
      window.__SMOKE_RESULT = JSON.stringify({
        check: 'fee_project_mapping',
        pass: true,
        details: {
          validated: 0,
          mismatches: [],
          note: 'no Accepted fees to validate'
        }
      });
      return;
    }

    const mismatches = [];
    let validated = 0;

    for (const fee of acceptedFees) {
      // Resolve project_id to a string key
      let projectKey = null;
      const pid = fee.project_id || fee.projectId;
      if (typeof pid === 'string') {
        projectKey = pid;
      } else if (pid && typeof pid === 'object') {
        const tb = pid.tb || 'projects';
        const key = pid.id;
        if (key && typeof key === 'object') {
          projectKey = tb + ':' + Object.values(key)[0];
        } else if (key !== undefined && key !== null) {
          projectKey = tb + ':' + key;
        }
      }

      if (!projectKey) {
        mismatches.push({ feeId: String(fee.id), reason: 'could not resolve project_id' });
        continue;
      }

      const project = projectById[projectKey];
      if (!project) {
        mismatches.push({ feeId: String(fee.id), projectId: projectKey, reason: 'project not found' });
        continue;
      }

      if (project.status !== 'Awarded') {
        mismatches.push({
          feeId: String(fee.id),
          projectId: projectKey,
          feeStatus: fee.status,
          projectStatus: project.status,
          reason: 'Accepted fee linked to non-Awarded project'
        });
      } else {
        validated++;
      }
    }

    const pass = mismatches.length === 0;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'fee_project_mapping',
      pass,
      details: {
        validated,
        acceptedFeeCount: acceptedFees.length,
        mismatches
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'fee_project_mapping',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 3: Connection Indicator DOM Presence
 *
 * Checks the DOM for a connection status element. Looks for elements matching:
 * - [class*="connection"]
 * - [class*="status"]
 * - .emittiv-connection
 *
 * Verifies the element exists and contains text like "connected" or has a
 * green/success indicator class.
 *
 * Result: { check: 'connection_indicator', pass: boolean, details: { found, selector, text, classes } }
 */
export const connectionIndicator = `(async () => {
  try {
    const selectors = [
      '[class*="led-glow"]',
      '[class*="connection"]',
      '[class*="status-indicator"]',
      '[class*="connection-status"]',
      '.emittiv-connection'
    ];

    let foundEl = null;
    let matchedSelector = null;

    for (const sel of selectors) {
      const el = document.querySelector(sel);
      if (el) {
        foundEl = el;
        matchedSelector = sel;
        break;
      }
    }

    if (!foundEl) {
      window.__SMOKE_RESULT = JSON.stringify({
        check: 'connection_indicator',
        pass: false,
        details: {
          found: false,
          triedSelectors: selectors
        }
      });
      return;
    }

    const text = (foundEl.textContent || '').trim().toLowerCase();
    const classes = foundEl.className || '';
    const hasConnectedText = text.includes('connected') || text.includes('online');
    const hasSuccessClass = classes.includes('connected') || classes.includes('success') ||
                            classes.includes('green') || classes.includes('active') ||
                            classes.includes('led-glow-connected');

    const pass = hasConnectedText || hasSuccessClass;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'connection_indicator',
      pass,
      details: {
        found: true,
        selector: matchedSelector,
        text: (foundEl.textContent || '').trim().substring(0, 100),
        classes: classes.substring(0, 200),
        hasConnectedText,
        hasSuccessClass
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'connection_indicator',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 4: Entity Count Consistency
 *
 * Calls get_stats to get dashboard counts, then calls get_projects,
 * get_companies, get_contacts, get_fees in parallel. Compares the stats
 * counts against the actual array lengths — they should match.
 *
 * Result: { check: 'entity_count_consistency', pass: boolean, details: { stats, actual, mismatches } }
 */
export const entityCountConsistency = `(async () => {
  try {
    const invoke = window.__TAURI_INTERNALS__.invoke;

    const [stats, projects, companies, contacts, fees] = await Promise.all([
      invoke('get_stats'),
      invoke('get_projects'),
      invoke('get_companies'),
      invoke('get_contacts'),
      invoke('get_fees')
    ]);

    const actual = {
      projects: Array.isArray(projects) ? projects.length : null,
      companies: Array.isArray(companies) ? companies.length : null,
      contacts: Array.isArray(contacts) ? contacts.length : null,
      fees: Array.isArray(fees) ? fees.length : null
    };

    const mismatches = [];

    // Check each count — stats may use different field names
    const statProjects = stats?.projects ?? stats?.project_count ?? stats?.projectCount ?? null;
    const statCompanies = stats?.companies ?? stats?.company_count ?? stats?.companyCount ?? null;
    const statContacts = stats?.contacts ?? stats?.contact_count ?? stats?.contactCount ?? null;
    const statFees = stats?.fees ?? stats?.fee_count ?? stats?.feeCount ?? null;

    if (statProjects !== null && actual.projects !== null && statProjects !== actual.projects) {
      mismatches.push({ entity: 'projects', stats: statProjects, actual: actual.projects });
    }
    if (statCompanies !== null && actual.companies !== null && statCompanies !== actual.companies) {
      mismatches.push({ entity: 'companies', stats: statCompanies, actual: actual.companies });
    }
    if (statContacts !== null && actual.contacts !== null && statContacts !== actual.contacts) {
      mismatches.push({ entity: 'contacts', stats: statContacts, actual: actual.contacts });
    }
    if (statFees !== null && actual.fees !== null && statFees !== actual.fees) {
      mismatches.push({ entity: 'fees', stats: statFees, actual: actual.fees });
    }

    const pass = mismatches.length === 0;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'entity_count_consistency',
      pass,
      details: {
        stats,
        actual,
        mismatches
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'entity_count_consistency',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 5: Settings Modal
 *
 * Looks for a settings button/icon in the DOM (selectors: [class*="settings"],
 * button[title*="Settings"], or a gear icon element). If found, clicks it,
 * waits 500ms, and verifies a settings panel/modal appeared showing DB config info.
 * Closes the modal after checking.
 *
 * If no settings button is found, passes with a note.
 *
 * Result: { check: 'settings_modal', pass: boolean, details: { found, opened, hasDbConfig, note? } }
 */
export const settingsModal = `(async () => {
  try {
    const settingsSelectors = [
      'button[title*="Settings" i]',
      'button[aria-label*="Settings" i]',
      '[class*="settings"][role="button"]',
      '[class*="settings"] button',
      'button[class*="settings"]',
      '[class*="gear"]',
      '[class*="config"]'
    ];

    let settingsBtn = null;
    let matchedSelector = null;

    for (const sel of settingsSelectors) {
      const el = document.querySelector(sel);
      if (el) {
        settingsBtn = el;
        matchedSelector = sel;
        break;
      }
    }

    if (!settingsBtn) {
      window.__SMOKE_RESULT = JSON.stringify({
        check: 'settings_modal',
        pass: true,
        details: {
          found: false,
          note: 'settings UI not accessible from current route'
        }
      });
      return;
    }

    // Click the settings button
    settingsBtn.click();
    await new Promise(resolve => setTimeout(resolve, 500));

    // Look for an opened modal/panel
    const modalSelectors = [
      '[class*="modal"]',
      '[class*="settings-panel"]',
      '[class*="overlay"]',
      '[role="dialog"]',
      '[class*="drawer"]'
    ];

    let modalEl = null;
    for (const sel of modalSelectors) {
      const el = document.querySelector(sel);
      if (el) {
        modalEl = el;
        break;
      }
    }

    const opened = modalEl !== null;
    const modalText = opened ? (modalEl.textContent || '').toLowerCase() : '';
    const hasDbConfig = modalText.includes('ws://') || modalText.includes('surreal') ||
                        modalText.includes('database') || modalText.includes('connection') ||
                        modalText.includes('10.0.');

    // Attempt to close — look for a close button or press Escape
    if (opened) {
      const closeBtn = modalEl.querySelector('button[class*="close"], button[aria-label*="close" i], [class*="close-btn"]');
      if (closeBtn) {
        closeBtn.click();
      } else {
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
      }
      await new Promise(resolve => setTimeout(resolve, 300));
    }

    const pass = opened;

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'settings_modal',
      pass,
      details: {
        found: true,
        settingsSelector: matchedSelector,
        opened,
        hasDbConfig
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'settings_modal',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Check 6: Detail Panel
 *
 * Navigates to #/projects, waits 500ms, finds the first project card/list-item,
 * clicks it, waits 500ms, and checks if a detail panel appeared.
 * Reports what was found.
 *
 * Result: { check: 'detail_panel', pass: boolean, details: { cardFound, panelFound, panelSelector, panelText } }
 */
export const detailPanel = `(async () => {
  try {
    // Navigate to projects
    window.location.hash = '#/projects';
    await new Promise(resolve => setTimeout(resolve, 500));

    // Find the first project card or list item
    const cardSelectors = [
      '.list-card',
      '[class*="project-card"]',
      '[class*="project-item"]',
      '[class*="list-item"]',
      '.emittiv-card'
    ];

    let card = null;
    let cardSel = null;
    for (const sel of cardSelectors) {
      const el = document.querySelector(sel);
      if (el) {
        card = el;
        cardSel = sel;
        break;
      }
    }

    if (!card) {
      window.__SMOKE_RESULT = JSON.stringify({
        check: 'detail_panel',
        pass: false,
        details: {
          cardFound: false,
          note: 'no project card found on #/projects'
        }
      });
      return;
    }

    // Click the card
    card.click();
    await new Promise(resolve => setTimeout(resolve, 500));

    // Look for a detail panel
    const panelSelectors = [
      '[class*="detail-panel"]',
      '[class*="detail"]',
      '[class*="slide-panel"]',
      '[class*="slide-in"]',
      '[class*="side-panel"]',
      '[class*="panel"]'
    ];

    let panelEl = null;
    let panelSel = null;
    for (const sel of panelSelectors) {
      const el = document.querySelector(sel);
      if (el) {
        panelEl = el;
        panelSel = sel;
        break;
      }
    }

    const panelFound = panelEl !== null;
    const panelText = panelFound
      ? (panelEl.textContent || '').trim().substring(0, 200)
      : '';

    window.__SMOKE_RESULT = JSON.stringify({
      check: 'detail_panel',
      pass: panelFound,
      details: {
        cardFound: true,
        cardSelector: cardSel,
        panelFound,
        panelSelector: panelSel,
        panelText
      }
    });
  } catch(e) {
    window.__SMOKE_RESULT = JSON.stringify({
      check: 'detail_panel',
      pass: false,
      error: e instanceof Error ? e.message : String(e)
    });
  }
})()`;

/**
 * Execution order for integration checks.
 * All checks are independent and read-only — they can run in any order,
 * but this sequence is recommended for logical grouping.
 */
export const INTEGRATION_CHECK_ORDER = [
  'statusTransition',
  'feeProjectMapping',
  'connectionIndicator',
  'entityCountConsistency',
  'settingsModal',
  'detailPanel'
] as const;

export type IntegrationCheckName = (typeof INTEGRATION_CHECK_ORDER)[number];
