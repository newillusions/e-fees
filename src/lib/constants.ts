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
  'Sent',
  'Negotiation',
  'Accepted',
  'Rejected',
  'No Response',
  'Superseded'
] as const;

export type ProposalStatus = (typeof PROPOSAL_STATUSES)[number];

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
export const ACTIVE_PROPOSAL_STATUSES: ProposalStatus[] = ['Draft', 'Sent', 'Negotiation'];

// =============================================================================
// PROJECT STATUSES
// =============================================================================

/**
 * All possible project statuses in display order.
 * Used in filters, dropdowns, and status badges.
 */
export const PROJECT_STATUSES = [
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
] as const;

export type ProjectStatus = (typeof PROJECT_STATUSES)[number];

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

/**
 * Active project statuses (for dashboard metrics, etc.)
 * These represent projects that are currently in progress.
 */
export const ACTIVE_PROJECT_STATUSES: ProjectStatus[] = [
  'RFP',
  'Submitted',
  'Awarded',
  'Design',
  'Construction'
];

// =============================================================================
// STATUS COLORS
// =============================================================================

/**
 * Status color mappings for badges and UI elements.
 * Returns semantic CSS class names referencing design tokens (defined in app.css).
 */
export const STATUS_COLORS: Record<string, string> = {
  Lead: 'status-color--lead',
  Draft: 'status-color--draft',
  RFP: 'status-color--rfp',
  Submitted: 'status-color--submitted',
  Sent: 'status-color--sent',
  Negotiation: 'status-color--negotiation',
  Awarded: 'status-color--awarded',
  Accepted: 'status-color--accepted',
  Design: 'status-color--design',
  Construction: 'status-color--construction',
  Completed: 'status-color--completed',
  Lost: 'status-color--lost',
  Rejected: 'status-color--rejected',
  'No Response': 'status-color--no-response',
  Cancelled: 'status-color--cancelled',
  'On Hold': 'status-color--on-hold',
  Superseded: 'status-color--superseded'
};

/**
 * Get status color classes for a given status.
 * Returns default styling if status not found.
 */
export function getStatusColor(status: string): string {
  return STATUS_COLORS[status] || 'text-emittiv-light bg-emittiv-dark';
}
