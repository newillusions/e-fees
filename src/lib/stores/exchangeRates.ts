import { writable, get } from 'svelte/store';
import {
  fetchLatestRates,
  getCachedRates,
  getRate,
  type ExchangeRates,
  type CurrencyCode,
  SUPPORTED_CURRENCIES
} from '../services/exchangeRates';
import { logger } from '../services/logger';

const exchangeLogger = logger.child({ component: 'ExchangeRatesStore' });

// Store state
export const exchangeRatesStore = writable<ExchangeRates | null>(null);
export const exchangeRatesLoading = writable<boolean>(false);
export const exchangeRatesError = writable<string | null>(null);

// Actions
export const exchangeRatesActions = {
  /**
   * Fetch latest rates from Frankfurter.app and update the store.
   */
  async refresh(): Promise<void> {
    exchangeRatesLoading.set(true);
    exchangeRatesError.set(null);

    try {
      const rates = await fetchLatestRates();
      exchangeRatesStore.set(rates);
      exchangeLogger.info('Exchange rates refreshed', { date: rates.date });
    } catch (error) {
      const errorMessage = error?.toString() || 'Failed to fetch exchange rates';
      exchangeRatesError.set(errorMessage);
      exchangeLogger.warn('Exchange rates unavailable', { error });
    } finally {
      exchangeRatesLoading.set(false);
    }
  },

  /**
   * Get the live rate between two currencies.
   * Returns null if rates have not been fetched yet.
   */
  getLiveRate(from: CurrencyCode, to: CurrencyCode): number | null {
    return getRate(from, to);
  },

  /**
   * Get the date of the current cached rates.
   */
  getRateDate(): string | null {
    const rates = get(exchangeRatesStore);
    return rates?.date ?? null;
  }
};

// Convenience exports
export const refreshExchangeRates = () => exchangeRatesActions.refresh();
export { SUPPORTED_CURRENCIES, type CurrencyCode };
