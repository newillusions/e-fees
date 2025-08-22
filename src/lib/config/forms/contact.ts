/**
 * Form field configuration for Contact entities.
 */

import type { FormFieldConfig } from '$lib/components/base/types';
import { companiesStore } from '$lib/stores';
import { extractSurrealId } from '$lib/utils/surrealdb';
import { get } from 'svelte/store';

export const contactFormFields: FormFieldConfig[] = [
  {
    type: 'group',
    name: 'contact_info',
    groupTitle: 'Contact Information',
    fields: [
      {
        type: 'text',
        name: 'first_name',
        label: 'First Name',
        placeholder: 'John',
        required: true,
        colSpan: 1
      },
      {
        type: 'text',
        name: 'last_name',
        label: 'Last Name',
        placeholder: 'Doe',
        required: true,
        colSpan: 1
      },
      {
        type: 'computed',
        name: 'full_name',
        label: 'Full Name (Auto-generated)',
        computeFn: (formData: any) => {
          const firstName = formData.first_name || '';
          const lastName = formData.last_name || '';
          return `${firstName} ${lastName}`.trim();
        },
        colSpan: 2
      },
      {
        type: 'email',
        name: 'email',
        label: 'Email',
        placeholder: 'john.doe@company.com',
        required: true,
        colSpan: 2
      },
      {
        type: 'tel',
        name: 'phone',
        label: 'Phone',
        placeholder: '+971 50 123 4567',
        required: false,
        colSpan: 1
      },
      {
        type: 'text',
        name: 'position',
        label: 'Position',
        placeholder: 'Manager',
        required: false,
        colSpan: 1
      },
      {
        type: 'typeahead',
        name: 'company',
        label: 'Company',
        placeholder: 'Search companies...',
        required: true,
        colSpan: 2,
        displayFields: ['name'],
        onSearch: async (searchText: string) => {
          if (!searchText || searchText.length < 1) return [];
          
          try {
            const searchLower = searchText.toLowerCase();
            const companies = get(companiesStore);
            
            return companies
              .filter(company => {
                const nameMatch = company.name?.toLowerCase().includes(searchLower);
                const shortNameMatch = company.name_short?.toLowerCase().includes(searchLower);
                const abbreviationMatch = company.abbreviation?.toLowerCase().includes(searchLower);
                return nameMatch || shortNameMatch || abbreviationMatch;
              })
              .map(company => {
                const companyId = extractSurrealId(company) || extractSurrealId(company.id) || company.id || '';
                return {
                  id: companyId,
                  name: company.name || '',
                  name_short: company.name_short || '',
                  abbreviation: company.abbreviation || ''
                };
              })
              .slice(0, 10);
          } catch (error) {
            console.warn('Failed to search companies:', error);
            return [];
          }
        }
      }
    ]
  }
];

export default contactFormFields;