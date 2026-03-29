import { describe, it, expect } from 'vitest';
import { parseProjectNumber, buildProjectPayload } from './projectNumber';

describe('parseProjectNumber', () => {
  it('parses UAE project number', () => {
    const result = parseProjectNumber('25-97105');
    expect(result).toEqual({
      year: 25,
      country: 971,
      seq: 5,
      id: '25-97105'
    });
  });

  it('parses Saudi project number', () => {
    const result = parseProjectNumber('26-96606');
    expect(result).toEqual({
      year: 26,
      country: 966,
      seq: 6,
      id: '26-96606'
    });
  });

  it('parses project number with leading zeros in seq', () => {
    const result = parseProjectNumber('25-97101');
    expect(result).toEqual({
      year: 25,
      country: 971,
      seq: 1,
      id: '25-97101'
    });
  });

  it('throws on invalid format (no dash)', () => {
    expect(() => parseProjectNumber('2597105')).toThrow('Invalid project number format');
  });

  it('throws on empty string', () => {
    expect(() => parseProjectNumber('')).toThrow('Invalid project number format');
  });

  it('throws on wrong number of parts', () => {
    expect(() => parseProjectNumber('25-971-05')).toThrow('Invalid project number format');
  });

  it('throws on non-numeric country code', () => {
    expect(() => parseProjectNumber('25-abc05')).toThrow('non-numeric components');
  });

  it('throws on non-numeric year', () => {
    expect(() => parseProjectNumber('YY-97105')).toThrow('non-numeric components');
  });
});

describe('buildProjectPayload', () => {
  it('builds a complete project payload', () => {
    const formData = {
      name: 'Dubai Islands Hotel',
      name_short: 'Dubai Islands',
      status: 'Lead' as const,
      area: 'Dubai Islands',
      city: 'Dubai',
      country: 'U.A.E.',
      folder: '25-97105 Dubai Islands'
    };

    const projectNumber = parseProjectNumber('25-97105');
    const result = buildProjectPayload(formData, projectNumber);

    expect(result.name).toBe('Dubai Islands Hotel');
    expect(result.name_short).toBe('Dubai Islands');
    expect(result.status).toBe('Lead');
    expect(result.area).toBe('Dubai Islands');
    expect(result.city).toBe('Dubai');
    expect(result.country).toBe('U.A.E.');
    expect(result.folder).toBe('25-97105 Dubai Islands');
    expect(result.number).toEqual(projectNumber);
    expect(result.time).toBeDefined();
    expect(result.time.created_at).toBe(result.time.updated_at);
  });

  it('uses consistent timestamps', () => {
    const formData = {
      name: 'Test',
      name_short: 'T',
      status: 'Lead' as const,
      area: 'A',
      city: 'C',
      country: 'X',
      folder: '25-97101 T'
    };

    const pn = parseProjectNumber('25-97101');
    const result = buildProjectPayload(formData, pn);

    // Both timestamps should be identical (single timestamp for consistency)
    expect(result.time.created_at).toBe(result.time.updated_at);
    // Should be a valid ISO date string
    expect(new Date(result.time.created_at).toISOString()).toBe(result.time.created_at);
  });
});
