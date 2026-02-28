/**
 * DOM Validation Check Module - DOM Structure Verification for E-Fees
 *
 * This module exports DOM structure checks and helper functions that verify
 * the rendered UI is correct. Checks are executed via Tauri MCP get_dom tool
 * which returns the DOM tree as a string. Claude then searches for the expected
 * selectors and content.
 *
 * E-Fees uses svelte-spa-router with hash-based routing (#/projects, #/proposals, etc).
 * DOM checks verify:
 * - App shell elements present on every page (sidebar, connection indicator)
 * - Route-specific elements (tables, filters, search inputs)
 * - Text content visibility (app title, headers)
 *
 * Usage:
 * ```typescript
 * // Get all app-shell checks
 * const shellChecks = appShellChecks;
 *
 * // Get checks for specific route
 * const projectsChecks = routeChecks['/projects'];
 *
 * // Navigate to a route and get the JS code
 * const navCode = navigateToRoute('/proposals');
 * const result = await executeJs(navCode);  // {navigated: true, hash: '#/proposals'}
 *
 * // Check for console errors
 * const errorCode = checkConsoleErrors;
 * const result = await executeJs(errorCode);  // {check: 'console_errors', pass: true, ...}
 * ```
 */

/**
 * DOM Check definition interface.
 *
 * Properties:
 * - name: Unique identifier for the check (snake_case)
 * - description: Human-readable description of what is being verified
 * - selector?: CSS selector to look for in the DOM (e.g., 'nav', '[class*="sidebar"]')
 * - textContent?: Text string to search for in the DOM (exact match)
 * - route?: Hash router path to navigate to before running the check (e.g., '/projects')
 */
export interface DOMCheck {
  name: string;
  description: string;
  selector?: string;
  textContent?: string;
  route?: string;
}

/**
 * App shell checks - DOM elements that should be present on every route.
 * These verify the base layout and navigation are rendered correctly.
 */
export const appShellChecks: DOMCheck[] = [
  {
    name: 'sidebar_navigation',
    description: 'Verify sidebar navigation is present',
    selector: 'nav, [class*="sidebar"], [class*="navigation"]'
  },
  {
    name: 'connection_indicator',
    description: 'Verify connection status indicator is present',
    selector: '[class*="connection"], [class*="status-indicator"], [class*="connection-status"]'
  },
  {
    name: 'app_title',
    description: 'Verify E-Fees application title is visible',
    textContent: 'E-Fees'
  }
];

/**
 * Route-specific DOM checks.
 * Each route has checks for elements specific to that page.
 *
 * Routes:
 * - '/' or '#/' - Dashboard (stats cards, quick actions, activity feed)
 * - '#/projects' - Projects list (table/cards, filters, search)
 * - '#/proposals' - Proposals list (table/cards, filters)
 * - '#/companies' - Companies list (table/cards)
 * - '#/contacts' - Contacts list (table/cards)
 */
export const routeChecks: Record<string, DOMCheck[]> = {
  '/': [
    {
      name: 'dashboard_stats_cards',
      description: 'Dashboard statistics cards are rendered',
      selector: '[class*="stat"], [class*="card"], [class*="metric"]'
    },
    {
      name: 'dashboard_title',
      description: 'Dashboard title is visible',
      textContent: 'Dashboard'
    },
    {
      name: 'quick_actions',
      description: 'Quick actions section is present',
      selector: '[class*="quick"], [class*="action"]'
    }
  ],

  '/projects': [
    {
      name: 'projects_table_or_list',
      description: 'Projects list/table container exists',
      selector: '[class*="project"], table, [class*="list"]'
    },
    {
      name: 'projects_search_input',
      description: 'Search input for projects is visible',
      selector: 'input[type="text"], input[placeholder*="search" i], [class*="search"]'
    },
    {
      name: 'projects_filter_controls',
      description: 'Filter controls (status, country, city dropdowns) are present',
      selector: 'select, [class*="filter"], [class*="dropdown"]'
    },
    {
      name: 'projects_page_title',
      description: 'Projects page title is visible',
      textContent: 'Projects'
    }
  ],

  '/proposals': [
    {
      name: 'proposals_table_or_list',
      description: 'Proposals list/table container exists',
      selector: '[class*="proposal"], table, [class*="list"]'
    },
    {
      name: 'proposals_search_input',
      description: 'Search input for proposals is visible',
      selector: 'input[type="text"], input[placeholder*="search" i], [class*="search"]'
    },
    {
      name: 'proposals_filter_controls',
      description: 'Filter controls are present',
      selector: 'select, [class*="filter"], [class*="dropdown"]'
    },
    {
      name: 'proposals_page_title',
      description: 'Proposals page title is visible',
      textContent: 'Proposals'
    }
  ],

  '/companies': [
    {
      name: 'companies_table_or_list',
      description: 'Companies list/table container exists',
      selector: '[class*="compan"], table, [class*="list"]'
    },
    {
      name: 'companies_search_input',
      description: 'Search input for companies is visible',
      selector: 'input[type="text"], input[placeholder*="search" i], [class*="search"]'
    },
    {
      name: 'companies_page_title',
      description: 'Companies page title is visible',
      textContent: 'Companies'
    }
  ],

  '/contacts': [
    {
      name: 'contacts_table_or_list',
      description: 'Contacts list/table container exists',
      selector: '[class*="contact"], table, [class*="list"]'
    },
    {
      name: 'contacts_search_input',
      description: 'Search input for contacts is visible',
      selector: 'input[type="text"], input[placeholder*="search" i], [class*="search"]'
    },
    {
      name: 'contacts_page_title',
      description: 'Contacts page title is visible',
      textContent: 'Contacts'
    }
  ]
};

/**
 * Navigate to a route using svelte-spa-router hash-based routing.
 *
 * The E-Fees app uses svelte-spa-router which navigates via window.location.hash.
 * This function returns JavaScript code that:
 * 1. Sets the hash to the route (e.g., '#/projects')
 * 2. Waits 1 second for navigation to complete and DOM to render
 * 3. Returns {navigated: true, hash: actual hash value}
 *
 * @param route - The route path (e.g., '/projects', '/proposals')
 * @returns JavaScript code string that navigates and returns result
 *
 * @example
 * const code = navigateToRoute('/projects');
 * const result = await executeJs(code);  // {navigated: true, hash: '#/projects'}
 */
export function navigateToRoute(route: string): string {
  return `(async () => {
  try {
    // Navigate using hash router
    window.location.hash = '#${route}';

    // Wait for route change and DOM render
    await new Promise(resolve => setTimeout(resolve, 1000));

    return {
      navigated: true,
      hash: window.location.hash
    };
  } catch (error) {
    return {
      navigated: false,
      error: error instanceof Error ? error.message : String(error),
      hash: window.location.hash
    };
  }
})()`;
}

/**
 * Check for console errors and warnings.
 *
 * This template literal contains JavaScript code that:
 * 1. Accesses window.__capturedErrors (array of error objects)
 * 2. Counts total errors and warnings
 * 3. Returns pass: true only if no errors occurred
 *
 * The Tauri app should populate window.__capturedErrors with error details
 * before tests run (via window.addEventListener('error', ...)).
 *
 * Returns: { check: 'console_errors', pass: boolean, details: {...} }
 *
 * @example
 * const result = await executeJs(checkConsoleErrors);
 * // {
 * //   check: 'console_errors',
 * //   pass: true,
 * //   details: { totalErrors: 0, totalWarnings: 0, errors: [] }
 * // }
 */
export const checkConsoleErrors = `(async () => {
  try {
    // Get captured errors from window object
    const errors = window.__capturedErrors || [];

    // Count errors and warnings
    const errorCount = errors.filter((e: any) => e?.type === 'error').length;
    const warningCount = errors.filter((e: any) => e?.type === 'warning').length;

    // Pass only if no errors (warnings are OK)
    const pass = errorCount === 0;

    return {
      check: 'console_errors',
      pass,
      details: {
        totalErrors: errorCount,
        totalWarnings: warningCount,
        errors: errors.slice(0, 10)  // First 10 errors for debugging
      }
    };
  } catch (error) {
    return {
      check: 'console_errors',
      pass: false,
      error: error instanceof Error ? error.message : String(error)
    };
  }
})()`;
