/**
 * Form field configuration for Company entities.
 */

import type { FormFieldConfig } from '$lib/components/base/types';
import { searchCountries, getCitySuggestions } from '$lib/api';

export const companyFormFields: FormFieldConfig[] = [
  {
    type: 'group',
    name: 'company_info',
    groupTitle: 'Company Information',
    fields: [
      {
        type: 'text',
        name: 'name',
        label: 'Company Name',
        placeholder: 'Full company name',
        required: true,
        colSpan: 2
      },
      {
        type: 'text',
        name: 'name_short',
        label: 'Short Name',
        placeholder: 'Short name',
        required: true,
        colSpan: 1
      },
      {
        type: 'text',
        name: 'abbreviation',
        label: 'Abbreviation',
        placeholder: 'ABC',
        required: true,
        maxlength: 10,
        colSpan: 1
      },
      {
        type: 'typeahead',
        name: 'country',
        label: 'Country',
        placeholder: 'Search countries...',
        required: true,
        colSpan: 1,
        displayFields: ['name'],
        onSearch: async (searchText: string) => {
          if (!searchText || searchText.length < 2) return [];
          try {
            const countries = await searchCountries(searchText);
            return countries.map(country => ({
              id: country.name,
              name: country.name,
              dial_code: country.dial_code
            }));
          } catch (error) {
            console.warn('Failed to search countries:', error);
            return [];
          }
        }
      },
      {
        type: 'typeahead',
        name: 'city',
        label: 'City',
        placeholder: 'Search cities...',
        required: true,
        colSpan: 1,
        displayFields: ['name'],
        onSearch: async (searchText: string) => {
          // This will need access to the country field value
          // For now, return empty array - will be handled by parent component
          return [];
        }
      },
      {
        type: 'text',
        name: 'reg_no',
        label: 'Registration No.',
        placeholder: 'Company registration number (optional)',
        required: false,
        colSpan: 1
      },
      {
        type: 'text',
        name: 'tax_no',
        label: 'Tax/VAT No.',
        placeholder: 'Tax identification number (optional)',
        required: false,
        colSpan: 1
      }
    ]
  }
];

export default companyFormFields;