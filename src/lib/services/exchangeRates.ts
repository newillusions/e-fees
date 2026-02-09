/**
 * Exchange rate service using Frankfurter.app (free, no API key, ECB rates).
 * Fetches latest rates on app startup, caches in memory.
 */

const API_BASE = 'https://api.frankfurter.app';

export const SUPPORTED_CURRENCIES = ['AED', 'SAR', 'USD', 'EUR', 'GBP'] as const;
export type CurrencyCode = typeof SUPPORTED_CURRENCIES[number];

export interface ExchangeRates {
  base: string;
  date: string;          // ISO date of the rates (from ECB)
  rates: Record<string, number>;
  fetchedAt: Date;       // When we fetched them
}

let cachedRates: ExchangeRates | null = null;

/**
 * Fetch latest exchange rates from Frankfurter.app.
 * AED is not directly supported by ECB, so we fetch EUR-based rates
 * and compute cross-rates via EUR.
 *
 * @param base - The base currency (default: 'EUR' since ECB uses EUR)
 * @returns Exchange rates object with all supported currencies
 */
export async function fetchLatestRates(): Promise<ExchangeRates> {
  // Frankfurter.app uses ECB data which is EUR-based.
  // AED and SAR are not in ECB rates, so we fetch EUR→USD,GBP
  // then compute AED and SAR from USD (pegged currencies).
  const targets = 'USD,GBP';
  const response = await fetch(`${API_BASE}/latest?from=EUR&to=${targets}`);
  if (!response.ok) throw new Error(`Exchange rate fetch failed: ${response.status}`);
  const data = await response.json();

  // AED is pegged to USD at 3.6725, SAR is pegged at 3.75
  const eurToUsd = data.rates['USD'];
  const eurToGbp = data.rates['GBP'];

  // Build full rate table with EUR as base
  const eurRates: Record<string, number> = {
    'USD': eurToUsd,
    'GBP': eurToGbp,
    'AED': eurToUsd * 3.6725,
    'SAR': eurToUsd * 3.75,
    'EUR': 1,
  };

  cachedRates = {
    base: 'EUR',
    date: data.date,
    rates: eurRates,
    fetchedAt: new Date(),
  };

  return cachedRates;
}

/**
 * Get the cached rates (null if not yet fetched).
 */
export function getCachedRates(): ExchangeRates | null {
  return cachedRates;
}

/**
 * Clear the cached rates (useful for testing).
 */
export function clearCachedRates(): void {
  cachedRates = null;
}

/**
 * Convert an amount between two supported currencies.
 * Uses cached rates (cross-rate through EUR) or an optional custom rate.
 *
 * @param amount - The amount to convert
 * @param from - Source currency code
 * @param to - Target currency code
 * @param customRate - Optional custom rate (1 from = customRate to)
 * @returns Converted amount, or null if rates unavailable
 */
export function convert(
  amount: number,
  from: CurrencyCode,
  to: CurrencyCode,
  customRate?: number
): number | null {
  if (from === to) return amount;
  if (customRate !== undefined && customRate > 0) return amount * customRate;
  if (!cachedRates) return null;

  const fromRate = cachedRates.rates[from];
  const toRate = cachedRates.rates[to];

  if (!fromRate || !toRate) return null;

  // Cross-rate: amount in FROM → EUR → TO
  // EUR value = amount / fromRate
  // TO value = eurValue * toRate
  return (amount / fromRate) * toRate;
}

/**
 * Get the exchange rate between two currencies (1 from = X to).
 * Uses cached rates.
 *
 * @param from - Source currency code
 * @param to - Target currency code
 * @returns The exchange rate, or null if rates unavailable
 */
export function getRate(from: CurrencyCode, to: CurrencyCode): number | null {
  if (from === to) return 1;
  if (!cachedRates) return null;

  const fromRate = cachedRates.rates[from];
  const toRate = cachedRates.rates[to];

  if (!fromRate || !toRate) return null;

  return toRate / fromRate;
}
