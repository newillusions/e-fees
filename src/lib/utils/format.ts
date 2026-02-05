/**
 * Formatting utilities for currency, percentages, and numbers.
 */

/**
 * Format a number as currency with the specified currency code.
 * Uses locale-aware formatting with proper thousand separators.
 *
 * @param amount - The amount to format
 * @param currency - Currency code (default: 'AED')
 * @param locale - Locale for formatting (default: 'en-AE')
 * @returns Formatted currency string
 *
 * @example
 * formatCurrency(105263, 'AED') // "105,263 AED"
 * formatCurrency(1500.50, 'USD') // "1,500.50 USD"
 */
export function formatCurrency(
  amount: number,
  currency: string = 'AED',
  locale: string = 'en-AE'
): string {
  // Format number with thousand separators
  const formatted = new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  }).format(amount);

  return `${formatted} ${currency}`;
}

/**
 * Format a number as currency without the currency code.
 * Useful for tables where the currency is shown in the header.
 *
 * @param amount - The amount to format
 * @param locale - Locale for formatting (default: 'en-AE')
 * @returns Formatted number string
 */
export function formatNumber(
  amount: number,
  locale: string = 'en-AE'
): string {
  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  }).format(amount);
}

/**
 * Format a number as a percentage.
 *
 * @param value - The percentage value (e.g., 5 for 5%)
 * @param decimals - Number of decimal places (default: 0)
 * @returns Formatted percentage string
 *
 * @example
 * formatPercent(5) // "5%"
 * formatPercent(12.5, 1) // "12.5%"
 */
export function formatPercent(value: number, decimals: number = 0): string {
  return `${value.toFixed(decimals)}%`;
}

/**
 * Parse a currency string back to a number.
 * Removes currency codes and thousand separators.
 *
 * @param value - The currency string to parse
 * @returns Parsed number or 0 if invalid
 */
export function parseCurrency(value: string): number {
  // Remove currency codes and whitespace
  const cleaned = value.replace(/[A-Z]{3}/g, '').replace(/\s/g, '');
  // Remove thousand separators (commas)
  const normalized = cleaned.replace(/,/g, '');
  const parsed = parseFloat(normalized);
  return isNaN(parsed) ? 0 : parsed;
}

/**
 * Format a date for display.
 *
 * @param dateStr - ISO date string or YYMMDD format
 * @param format - Output format ('short' | 'long' | 'iso')
 * @returns Formatted date string
 */
export function formatDate(
  dateStr: string,
  format: 'short' | 'long' | 'iso' = 'short'
): string {
  let date: Date;

  // Handle YYMMDD format
  if (/^\d{6}$/.test(dateStr)) {
    const yy = parseInt(dateStr.slice(0, 2));
    const mm = parseInt(dateStr.slice(2, 4)) - 1;
    const dd = parseInt(dateStr.slice(4, 6));
    const yyyy = yy <= 50 ? 2000 + yy : 1900 + yy;
    date = new Date(yyyy, mm, dd);
  } else {
    date = new Date(dateStr);
  }

  if (isNaN(date.getTime())) {
    return dateStr; // Return original if invalid
  }

  switch (format) {
    case 'long':
      return date.toLocaleDateString('en-GB', {
        day: 'numeric',
        month: 'long',
        year: 'numeric',
      });
    case 'iso':
      return date.toISOString().split('T')[0];
    case 'short':
    default:
      return date.toLocaleDateString('en-GB', {
        day: '2-digit',
        month: 'short',
        year: 'numeric',
      });
  }
}

/**
 * Round a value to the nearest increment.
 * Used for producing client-friendly "human readable" fee values.
 *
 * @param value - The value to round
 * @param increment - The rounding increment (e.g., 50, 100, 500)
 * @param mode - 'ceiling' always rounds up, 'nearest' rounds to closest
 * @returns Rounded value
 *
 * @example
 * roundToIncrement(12847, 50, 'ceiling') // 12850
 * roundToIncrement(12847, 100, 'ceiling') // 12900
 * roundToIncrement(12847, 50, 'nearest') // 12850
 * roundToIncrement(12825, 50, 'nearest') // 12850
 */
export function roundToIncrement(
  value: number,
  increment: number,
  mode: 'ceiling' | 'nearest' = 'ceiling'
): number {
  if (increment <= 0) return value;
  if (mode === 'ceiling') {
    return Math.ceil(value / increment) * increment;
  }
  return Math.round(value / increment) * increment;
}
