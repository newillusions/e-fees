/**
 * Application Constants
 *
 * Centralized configuration for status options, select field values,
 * and other application-wide constants.
 */

// =============================================================================
// PROPOSAL STATUSES
// =============================================================================

/**
 * All possible proposal (fee) statuses in display order.
 * Used in filters, dropdowns, and status badges.
 */
export const PROPOSAL_STATUSES = [
  'Draft',
  'Active',
  'Sent',
  'Negotiation',
  'Awarded',
  'Completed',
  'Lost',
  'Cancelled',
  'On Hold',
  'Revised'
] as const;

export type ProposalStatus = typeof PROPOSAL_STATUSES[number];

/**
 * Proposal status options formatted for select dropdowns.
 * { value, label } format for FormSelect compatibility.
 */
export const PROPOSAL_STATUS_OPTIONS = PROPOSAL_STATUSES.map(status => ({
  value: status,
  label: status
}));

/**
 * Proposal status options formatted for typeahead components.
 * { id, name } format for TypeaheadSelect compatibility.
 * @deprecated Use PROPOSAL_STATUS_OPTIONS for select dropdowns
 */
export const PROPOSAL_STATUS_OPTIONS_TYPEAHEAD = PROPOSAL_STATUSES.map(status => ({
  id: status,
  name: status
}));

/**
 * Active proposal statuses (for dashboard metrics, etc.)
 * These represent proposals that are still "in progress"
 */
export const ACTIVE_PROPOSAL_STATUSES: ProposalStatus[] = [
  'Draft',
  'Sent',
  'Negotiation'
];

// =============================================================================
// PROJECT STATUSES
// =============================================================================

/**
 * All possible project statuses in display order.
 * Used in filters, dropdowns, and status badges.
 */
export const PROJECT_STATUSES = [
  'Draft',
  'RFP',
  'Active',
  'Awarded',
  'Completed',
  'Lost',
  'Cancelled',
  'On Hold',
  'Revised'
] as const;

export type ProjectStatus = typeof PROJECT_STATUSES[number];

/**
 * Project status options formatted for select/dropdown components.
 * { value, label } format for FormSelect compatibility.
 */
export const PROJECT_STATUS_OPTIONS = PROJECT_STATUSES.map(status => ({
  value: status,
  label: status
}));

/**
 * Project status options formatted for typeahead components.
 * { id, name } format for TypeaheadSelect compatibility.
 */
export const PROJECT_STATUS_OPTIONS_TYPEAHEAD = PROJECT_STATUSES.map(status => ({
  id: status,
  name: status
}));

// =============================================================================
// STATUS COLORS
// =============================================================================

/**
 * Status color mappings for badges and UI elements.
 * Returns Tailwind classes for text and background colors.
 */
export const STATUS_COLORS: Record<string, string> = {
  // Proposal/Project statuses
  'Draft': 'text-yellow-400 bg-yellow-400/10',
  'Sent': 'text-purple-400 bg-purple-400/10',
  'Negotiation': 'text-blue-400 bg-blue-400/10',
  'Awarded': 'text-green-400 bg-green-400/10',
  'Completed': 'text-green-500 bg-green-500/10',
  'Lost': 'text-red-400 bg-red-400/10',
  'Cancelled': 'text-gray-400 bg-gray-400/10',
  'On Hold': 'text-orange-400 bg-orange-400/10',
  'Revised': 'text-cyan-400 bg-cyan-400/10',
  // Project-specific
  'RFP': 'text-blue-400 bg-blue-400/10',
  'Active': 'text-blue-400 bg-blue-400/10',
};

/**
 * Get status color classes for a given status.
 * Returns default styling if status not found.
 */
export function getStatusColor(status: string): string {
  return STATUS_COLORS[status] || 'text-emittiv-light bg-emittiv-dark';
}
