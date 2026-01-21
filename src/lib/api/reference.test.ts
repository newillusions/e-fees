/**
 * Reference Data API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  searchCountries,
  getAreaSuggestions,
  getCitySuggestions,
  getAllCities
} from './reference';
import type { CountrySearchResult } from '../../types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('Reference Data API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('searchCountries', () => {
    it('should call invoke with search query', async () => {
      const mockCountries: CountrySearchResult[] = [
        { name: 'United Arab Emirates', dial_code: 971, code: 'AE' }
      ];
      mockInvoke.mockResolvedValueOnce(mockCountries);

      const result = await searchCountries('emirates');

      expect(mockInvoke).toHaveBeenCalledWith('search_countries', { query: 'emirates' });
      expect(result).toEqual(mockCountries);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Search failed'));

      await expect(searchCountries('test')).rejects.toThrow('Search failed');
    });
  });

  describe('getAreaSuggestions', () => {
    it('should call invoke with country name', async () => {
      const mockAreas = ['Business Bay', 'Downtown Dubai', 'DIFC'];
      mockInvoke.mockResolvedValueOnce(mockAreas);

      const result = await getAreaSuggestions('United Arab Emirates');

      expect(mockInvoke).toHaveBeenCalledWith('get_area_suggestions', { country: 'United Arab Emirates' });
      expect(result).toEqual(mockAreas);
    });

    it('should pass empty string when country is null', async () => {
      mockInvoke.mockResolvedValueOnce([]);

      await getAreaSuggestions(null);

      expect(mockInvoke).toHaveBeenCalledWith('get_area_suggestions', { country: '' });
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Suggestions failed'));

      await expect(getAreaSuggestions('test')).rejects.toThrow('Suggestions failed');
    });
  });

  describe('getCitySuggestions', () => {
    it('should call invoke with country name', async () => {
      const mockCities = ['Dubai', 'Abu Dhabi', 'Sharjah'];
      mockInvoke.mockResolvedValueOnce(mockCities);

      const result = await getCitySuggestions('United Arab Emirates');

      expect(mockInvoke).toHaveBeenCalledWith('get_city_suggestions', { country: 'United Arab Emirates' });
      expect(result).toEqual(mockCities);
    });

    it('should pass empty string when country is null', async () => {
      mockInvoke.mockResolvedValueOnce([]);

      await getCitySuggestions(null);

      expect(mockInvoke).toHaveBeenCalledWith('get_city_suggestions', { country: '' });
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Suggestions failed'));

      await expect(getCitySuggestions('test')).rejects.toThrow('Suggestions failed');
    });
  });

  describe('getAllCities', () => {
    it('should call invoke and return all cities', async () => {
      const mockCities = ['Abu Dhabi', 'Dubai', 'Riyadh', 'Sharjah'];
      mockInvoke.mockResolvedValueOnce(mockCities);

      const result = await getAllCities();

      expect(mockInvoke).toHaveBeenCalledWith('get_all_cities');
      expect(result).toEqual(mockCities);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cities fetch failed'));

      await expect(getAllCities()).rejects.toThrow('Cities fetch failed');
    });
  });
});
