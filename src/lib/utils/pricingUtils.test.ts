import { describe, it, expect } from 'vitest';
import { getRoundingConfig, roundWithConfig, calcWhtAmounts, whtTooltip } from './pricingUtils';

describe('getRoundingConfig', () => {
  it('extracts increment and mode from config', () => {
    const result = getRoundingConfig({ rounding_increment: 100, rounding_mode: 'nearest' });
    expect(result).toEqual({ increment: 100, mode: 'nearest' });
  });

  it('returns defaults when config is undefined', () => {
    const result = getRoundingConfig(undefined);
    expect(result).toEqual({ increment: 50, mode: 'ceiling' });
  });

  it('returns defaults when fields are missing', () => {
    const result = getRoundingConfig({});
    expect(result).toEqual({ increment: 50, mode: 'ceiling' });
  });
});

describe('roundWithConfig', () => {
  it('rounds up to increment with ceiling mode', () => {
    expect(roundWithConfig(12847, { rounding_increment: 50, rounding_mode: 'ceiling' })).toBe(12850);
  });

  it('rounds to nearest increment', () => {
    expect(roundWithConfig(12825, { rounding_increment: 50, rounding_mode: 'nearest' })).toBe(12850);
  });

  it('uses default config when undefined', () => {
    expect(roundWithConfig(12801, undefined)).toBe(12850);
  });

  it('returns 0 for 0 input', () => {
    expect(roundWithConfig(0, { rounding_increment: 50, rounding_mode: 'ceiling' })).toBe(0);
  });
});

describe('calcWhtAmounts', () => {
  it('calculates WHT gross-up with 5% rate', () => {
    const result = calcWhtAmounts(1000, 0.05);
    expect(result.invoiced).toBe(1053);
    expect(result.wht).toBe(53);
  });

  it('returns amount unchanged when rate is 0', () => {
    const result = calcWhtAmounts(1000, 0);
    expect(result.invoiced).toBe(1000);
    expect(result.wht).toBe(0);
  });

  it('handles non-trivial rounding', () => {
    const result = calcWhtAmounts(5000, 0.05);
    expect(result.invoiced).toBe(5263);
    expect(result.wht).toBe(263);
  });

  it('returns amount unchanged when rate is 1.0 (division by zero guard)', () => {
    const result = calcWhtAmounts(1000, 1.0);
    expect(result.invoiced).toBe(1000);
    expect(result.wht).toBe(0);
  });

  it('returns amount unchanged when rate exceeds 1.0', () => {
    const result = calcWhtAmounts(1000, 1.5);
    expect(result.invoiced).toBe(1000);
    expect(result.wht).toBe(0);
  });
});

describe('whtTooltip', () => {
  const mockFormatNumber = (n: number) => n.toLocaleString();

  it('returns tooltip string when WHT is active', () => {
    const config = { tax_type: 'withholding', vat_percent: 5 };
    const result = whtTooltip(1000, config, mockFormatNumber);
    expect(result).toContain('Invoice:');
    expect(result).toContain('WHT');
    expect(result).toContain('5%');
  });

  it('returns empty string when tax_type is vat (not withholding)', () => {
    const config = { tax_type: 'vat', vat_percent: 5 };
    expect(whtTooltip(1000, config, mockFormatNumber)).toBe('');
  });

  it('returns empty string when vat_percent is 0', () => {
    const config = { tax_type: 'withholding', vat_percent: 0 };
    expect(whtTooltip(1000, config, mockFormatNumber)).toBe('');
  });

  it('returns empty string when config is undefined', () => {
    expect(whtTooltip(1000, undefined, mockFormatNumber)).toBe('');
  });

  it('returns empty string when tax_type is missing', () => {
    const config = { vat_percent: 5 };
    expect(whtTooltip(1000, config, mockFormatNumber)).toBe('');
  });
});
