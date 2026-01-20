/**
 * Reference Data API methods
 *
 * Handles reference data lookups including countries, areas, and cities.
 */

import { invoke } from '@tauri-apps/api/core';
import type { CountrySearchResult } from '../../types';

/**
 * Searches countries by name with fuzzy matching.
 * @param query - Search term to match against country names
 * @returns Array of matching country records
 */
export async function searchCountries(query: string): Promise<CountrySearchResult[]> {
  try {
    const countries = await invoke<CountrySearchResult[]>('search_countries', { query });
    return countries;
  } catch (error) {
    console.error('Failed to search countries:', error);
    throw error;
  }
}

/**
 * Retrieves area suggestions based on country selection.
 * @param country - Country name to get area suggestions for
 * @returns Array of area names
 */
export async function getAreaSuggestions(country: string | null): Promise<string[]> {
  try {
    const suggestions = await invoke('get_area_suggestions', { country: country || '' });
    return suggestions as string[];
  } catch (error) {
    console.error('Failed to get area suggestions:', error);
    throw error;
  }
}

/**
 * Retrieves city suggestions based on country selection.
 * @param country - Country name to get city suggestions for
 * @returns Array of city names
 */
export async function getCitySuggestions(country: string | null): Promise<string[]> {
  try {
    const suggestions = await invoke('get_city_suggestions', { country: country || '' });
    return suggestions as string[];
  } catch (error) {
    console.error('Failed to get city suggestions:', error);
    throw error;
  }
}

/**
 * Gets all unique cities from projects and companies tables.
 * @returns Array of all city names
 */
export async function getAllCities(): Promise<string[]> {
  try {
    const cities = await invoke('get_all_cities');
    return cities as string[];
  } catch (error) {
    console.error('Failed to get all cities:', error);
    throw error;
  }
}
