/**
 * Smoke Test Helper Module - JS Code Strings for Tauri MCP
 *
 * This module exports JavaScript code strings that are executed inside the E-Fees
 * Tauri application webview via the Tauri MCP execute_js tool. Each export is a
 * template literal containing an async IIFE that invokes Tauri commands and returns
 * serializable result objects.
 *
 * Usage:
 * ```typescript
 * const result = await executeJs(checkDbConnection);
 * // result: { check: 'db_connection', pass: true, details: {...} }
 * ```
 */

/**
 * Check database connection status.
 *
 * Invokes both `get_connection_status` and `get_db_info` Tauri commands
 * to verify the database connection and get configuration details.
 *
 * Returns: { check: 'db_connection', pass: boolean, details: {...} }
 */
export const checkDbConnection = `(async () => {
  try {
    const connectionStatus = await window.__TAURI__.invoke('get_connection_status');
    const dbInfo = await window.__TAURI__.invoke('get_db_info');

    const isConnected = connectionStatus?.is_connected === true;

    return {
      check: 'db_connection',
      pass: isConnected,
      details: {
        status: connectionStatus,
        config: {
          url: dbInfo?.url,
          namespace: dbInfo?.namespace,
          database: dbInfo?.database,
          username: dbInfo?.username
        },
        timestamp: dbInfo?.timestamp
      }
    };
  } catch (error) {
    return {
      check: 'db_connection',
      pass: false,
      error: error instanceof Error ? error.message : String(error)
    };
  }
})()`;

/**
 * Check that essential data is loaded.
 *
 * Invokes get_projects, get_companies, get_contacts, get_fees commands
 * and verifies counts are non-zero.
 *
 * Returns: { check: 'data_loaded', pass: boolean, details: { counts: {...} } }
 */
export const checkDataLoaded = `(async () => {
  try {
    const [projects, companies, contacts, fees] = await Promise.all([
      window.__TAURI__.invoke('get_projects'),
      window.__TAURI__.invoke('get_companies'),
      window.__TAURI__.invoke('get_contacts'),
      window.__TAURI__.invoke('get_fees')
    ]);

    const projectCount = Array.isArray(projects) ? projects.length : 0;
    const companyCount = Array.isArray(companies) ? companies.length : 0;
    const contactCount = Array.isArray(contacts) ? contacts.length : 0;
    const feeCount = Array.isArray(fees) ? fees.length : 0;

    const pass = projectCount > 0 && companyCount > 0;

    return {
      check: 'data_loaded',
      pass,
      details: {
        counts: {
          projects: projectCount,
          companies: companyCount,
          contacts: contactCount,
          fees: feeCount
        }
      }
    };
  } catch (error) {
    return {
      check: 'data_loaded',
      pass: false,
      error: error instanceof Error ? error.message : String(error)
    };
  }
})()`;

/**
 * Check that we are NOT connected to production database.
 *
 * Invokes get_db_info and verifies the URL does not include the production IP.
 * Production IP: 10.0.23.11
 *
 * Returns: { check: 'not_production', pass: boolean, error?: string }
 */
export const checkNotProduction = `(async () => {
  try {
    const dbInfo = await window.__TAURI__.invoke('get_db_info');
    const url = dbInfo?.url || '';

    // Production database IP
    const isProd = url.includes('10.0.23.11');

    if (isProd) {
      return {
        check: 'not_production',
        pass: false,
        error: 'ERROR: Connected to production database (10.0.23.11). Smoke tests must use development/test database only.',
        details: { url }
      };
    }

    return {
      check: 'not_production',
      pass: true,
      details: { url }
    };
  } catch (error) {
    return {
      check: 'not_production',
      pass: false,
      error: error instanceof Error ? error.message : String(error)
    };
  }
})()`;

/**
 * Check entity counts from stats command.
 *
 * Invokes get_stats Tauri command and returns the stats object.
 *
 * Returns: { check: 'entity_counts', pass: true, details: {...} }
 */
export const checkEntityCounts = `(async () => {
  try {
    const stats = await window.__TAURI__.invoke('get_stats');

    return {
      check: 'entity_counts',
      pass: true,
      details: stats
    };
  } catch (error) {
    return {
      check: 'entity_counts',
      pass: false,
      error: error instanceof Error ? error.message : String(error)
    };
  }
})()`;

/**
 * Check project statuses against valid list.
 *
 * Invokes get_projects command and collects unique statuses.
 * Validates against: Lead, RFP, Submitted, Awarded, Design, Construction,
 *                    Completed, Lost, No Response, Cancelled, On Hold, Superseded
 *
 * Returns: { check: 'project_statuses', pass: boolean, details: { statuses, valid, invalid } }
 */
export const checkProjectStatuses = `(async () => {
  try {
    const projects = await window.__TAURI__.invoke('get_projects');
    const validStatuses = [
      'Lead',
      'RFP',
      'Submitted',
      'Awarded',
      'Design',
      'Construction',
      'Completed',
      'Lost',
      'No Response',
      'Cancelled',
      'On Hold',
      'Superseded'
    ];

    const foundStatuses = new Set();
    const invalidStatuses = new Set();

    if (Array.isArray(projects)) {
      projects.forEach(project => {
        const status = project?.status;
        if (status) {
          foundStatuses.add(status);
          if (!validStatuses.includes(status)) {
            invalidStatuses.add(status);
          }
        }
      });
    }

    const pass = invalidStatuses.size === 0;

    return {
      check: 'project_statuses',
      pass,
      details: {
        foundStatuses: Array.from(foundStatuses),
        validStatuses,
        invalidStatuses: Array.from(invalidStatuses),
        projectCount: Array.isArray(projects) ? projects.length : 0
      }
    };
  } catch (error) {
    return {
      check: 'project_statuses',
      pass: false,
      error: error instanceof Error ? error.message : String(error)
    };
  }
})()`;

/**
 * Check fee statuses against valid list.
 *
 * Invokes get_fees command and collects unique statuses.
 * Validates against: Draft, Sent, Negotiation, Accepted, Rejected,
 *                    No Response, Superseded
 *
 * Returns: { check: 'fee_statuses', pass: boolean, details: { statuses, valid, invalid } }
 */
export const checkFeeStatuses = `(async () => {
  try {
    const fees = await window.__TAURI__.invoke('get_fees');
    const validStatuses = [
      'Draft',
      'Sent',
      'Negotiation',
      'Accepted',
      'Rejected',
      'No Response',
      'Superseded'
    ];

    const foundStatuses = new Set();
    const invalidStatuses = new Set();

    if (Array.isArray(fees)) {
      fees.forEach(fee => {
        const status = fee?.status;
        if (status) {
          foundStatuses.add(status);
          if (!validStatuses.includes(status)) {
            invalidStatuses.add(status);
          }
        }
      });
    }

    const pass = invalidStatuses.size === 0;

    return {
      check: 'fee_statuses',
      pass,
      details: {
        foundStatuses: Array.from(foundStatuses),
        validStatuses,
        invalidStatuses: Array.from(invalidStatuses),
        feeCount: Array.isArray(fees) ? fees.length : 0
      }
    };
  } catch (error) {
    return {
      check: 'fee_statuses',
      pass: false,
      error: error instanceof Error ? error.message : String(error)
    };
  }
})()`;
