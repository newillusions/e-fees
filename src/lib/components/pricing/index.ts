/**
 * Pricing components for fee proposal management.
 *
 * These components provide a complete pricing workflow:
 * 1. DisciplinesPanel - Define team allocation percentages
 * 2. StagesPanel - Define design phases and post-contract services
 * 3. PricingCalculatorPanel - Target fee, buffer, and pricing matrix
 * 4. CostsPanel - Reimbursable expenses with markup
 * 5. PaymentSchedulePanel - Billing milestones and payment tracking
 * 6. PricingSummaryPanel - Full breakdown view
 * 7. PricingSummaryBar - Compact totals bar for all panels
 */

export { default as DisciplinesPanel } from './DisciplinesPanel.svelte';
export { default as StagesPanel } from './StagesPanel.svelte';
export { default as PricingCalculatorPanel } from './PricingCalculatorPanel.svelte';
export { default as CostsPanel } from './CostsPanel.svelte';
export { default as PaymentSchedulePanel } from './PaymentSchedulePanel.svelte';
export { default as PricingSummaryPanel } from './PricingSummaryPanel.svelte';
export { default as PricingSummaryBar } from './PricingSummaryBar.svelte';
export { default as FeePricingModal } from './FeePricingModal.svelte';
