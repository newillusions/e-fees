/**
 * Shared pricing utilities for rounding and WHT calculations.
 *
 * Extracted from PricingCalculatorPanel, PaymentSchedulePanel, and
 * PricingSummaryBar to eliminate duplication and ensure consistent
 * rounding/WHT behavior across all pricing views.
 */

import { roundToIncrement } from './format';

/**
 * Extract rounding config from a pricing config object, with defaults.
 */
export function getRoundingConfig(config: any): { increment: number; mode: string } {
  return {
    increment: config?.rounding_increment ?? 50,
    mode: config?.rounding_mode ?? 'ceiling'
  };
}

/**
 * Round a value using the rounding config from a pricing breakdown.
 */
export function roundWithConfig(rawTotal: number, config: any): number {
  const { increment, mode } = getRoundingConfig(config);
  return roundToIncrement(rawTotal, increment, mode as 'ceiling' | 'nearest');
}

/**
 * Calculate WHT (withholding tax) gross-up amounts.
 * Formula: invoiced = amount / (1 - whtRate), wht = invoiced - amount.
 */
export function calcWhtAmounts(amount: number, whtRate: number): { invoiced: number; wht: number } {
  if (whtRate === 0) return { invoiced: Math.round(amount), wht: 0 };
  if (whtRate >= 1) return { invoiced: Math.round(amount), wht: 0 };
  const invoiced = Math.round(amount / (1 - whtRate));
  const wht = Math.round(invoiced - amount);
  return { invoiced, wht };
}

/**
 * Generate a WHT tooltip string, or empty string if WHT is not active.
 */
export function whtTooltip(
  amount: number,
  config: any,
  formatNumber: (n: number) => string
): string {
  if (config?.tax_type !== 'withholding') return '';
  const vat = config?.vat_percent ?? 0;
  if (vat === 0) return '';
  const whtRate = vat / 100;
  const { invoiced, wht } = calcWhtAmounts(amount, whtRate);
  return `Invoice: ${formatNumber(invoiced)} (incl. ${formatNumber(wht)} WHT ${vat}%)`;
}
