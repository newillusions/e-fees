import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  fetchLatestRates,
  getCachedRates,
  clearCachedRates,
  convert,
  getRate,
  SUPPORTED_CURRENCIES,
  type CurrencyCode,
  type ExchangeRates
} from './exchangeRates';

// Mock global fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

beforeEach(() => {
  vi.clearAllMocks();
  clearCachedRates();
});

describe('exchangeRates service', () => {
  describe('SUPPORTED_CURRENCIES', () => {
    it('should contain AED, SAR, USD, EUR, GBP', () => {
      expect(SUPPORTED_CURRENCIES).toContain('AED');
      expect(SUPPORTED_CURRENCIES).toContain('SAR');
      expect(SUPPORTED_CURRENCIES).toContain('USD');
      expect(SUPPORTED_CURRENCIES).toContain('EUR');
      expect(SUPPORTED_CURRENCIES).toContain('GBP');
      expect(SUPPORTED_CURRENCIES.length).toBe(5);
    });
  });

  describe('fetchLatestRates', () => {
    it('should fetch rates from Frankfurter API and compute cross-rates', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          base: 'EUR',
          date: '2026-02-07',
          rates: { USD: 1.0388, GBP: 0.8345 }
        })
      });

      const rates = await fetchLatestRates();

      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.frankfurter.app/latest?from=EUR&to=USD,GBP'
      );
      expect(rates.base).toBe('EUR');
      expect(rates.date).toBe('2026-02-07');
      expect(rates.rates['EUR']).toBe(1);
      expect(rates.rates['USD']).toBe(1.0388);
      expect(rates.rates['GBP']).toBe(0.8345);
      // AED is pegged at 3.6725 * USD rate
      expect(rates.rates['AED']).toBeCloseTo(1.0388 * 3.6725, 4);
      // SAR is pegged at 3.75 * USD rate
      expect(rates.rates['SAR']).toBeCloseTo(1.0388 * 3.75, 4);
      expect(rates.fetchedAt).toBeInstanceOf(Date);
    });

    it('should throw on failed fetch', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500
      });

      await expect(fetchLatestRates()).rejects.toThrow('Exchange rate fetch failed: 500');
    });

    it('should cache rates after successful fetch', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          base: 'EUR',
          date: '2026-02-07',
          rates: { USD: 1.04, GBP: 0.83 }
        })
      });

      expect(getCachedRates()).toBeNull();
      await fetchLatestRates();
      expect(getCachedRates()).not.toBeNull();
      expect(getCachedRates()?.date).toBe('2026-02-07');
    });
  });

  describe('clearCachedRates', () => {
    it('should clear cached rates', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          base: 'EUR',
          date: '2026-02-07',
          rates: { USD: 1.04, GBP: 0.83 }
        })
      });

      await fetchLatestRates();
      expect(getCachedRates()).not.toBeNull();
      clearCachedRates();
      expect(getCachedRates()).toBeNull();
    });
  });

  describe('convert', () => {
    beforeEach(async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          base: 'EUR',
          date: '2026-02-07',
          rates: { USD: 1.04, GBP: 0.84 }
        })
      });
      await fetchLatestRates();
    });

    it('should return same amount when from === to', () => {
      expect(convert(100, 'AED', 'AED')).toBe(100);
    });

    it('should use custom rate when provided', () => {
      expect(convert(100, 'AED', 'SAR', 0.98)).toBeCloseTo(98, 2);
    });

    it('should convert between currencies via EUR cross-rate', () => {
      // AED rate = 1.04 * 3.6725 = 3.8194
      // SAR rate = 1.04 * 3.75 = 3.9
      // 100 AED in SAR = (100 / 3.8194) * 3.9
      const aedRate = 1.04 * 3.6725;
      const sarRate = 1.04 * 3.75;
      const expected = (100 / aedRate) * sarRate;
      expect(convert(100, 'AED', 'SAR')).toBeCloseTo(expected, 4);
    });

    it('should return null if no cached rates', () => {
      clearCachedRates();
      expect(convert(100, 'AED', 'SAR')).toBeNull();
    });

    it('should fall through to cached rates when custom rate is zero', () => {
      // customRate of 0 is not > 0, so it falls through to cached rates
      const result = convert(100, 'AED', 'SAR', 0);
      // Should use the cached cross-rate, not the custom rate of 0
      expect(result).not.toBeNull();
      expect(result).toBeGreaterThan(0);
    });
  });

  describe('getRate', () => {
    beforeEach(async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          base: 'EUR',
          date: '2026-02-07',
          rates: { USD: 1.04, GBP: 0.84 }
        })
      });
      await fetchLatestRates();
    });

    it('should return 1 for same currency', () => {
      expect(getRate('AED', 'AED')).toBe(1);
    });

    it('should return the cross rate', () => {
      // 1 AED = SAR/AED = (1.04*3.75) / (1.04*3.6725)
      const aedRate = 1.04 * 3.6725;
      const sarRate = 1.04 * 3.75;
      expect(getRate('AED', 'SAR')).toBeCloseTo(sarRate / aedRate, 6);
    });

    it('should return null if no cached rates', () => {
      clearCachedRates();
      expect(getRate('AED', 'SAR')).toBeNull();
    });

    it('should return EUR to USD rate directly', () => {
      expect(getRate('EUR', 'USD')).toBeCloseTo(1.04, 4);
    });
  });
});
