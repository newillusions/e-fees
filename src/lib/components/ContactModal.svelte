<!--
  Contact Modal using Generic CrudModal Component
  
  Replaces the original ContactModal with the new generic system.
  Maintains all existing functionality with significantly reduced code.
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { contactsActions, companiesStore } from '$lib/stores';
  import { extractSurrealId } from '$lib/utils/surrealdb';
  import { CommonValidationRules } from '$lib/utils/validation';
  import { get } from 'svelte/store';
  import CrudModal from './base/CrudModal.svelte';
  import CompanyModal from './CompanyModal.svelte';
  import type { Contact } from '$lib/../types';
  import type { FormFieldConfig } from './base/types';

  const dispatch = createEventDispatcher();

  export let isOpen = false;
  export let contact: Contact | null = null;
  export let mode: 'create' | 'edit' = 'create';

  // Company modal state
  let showCompanyModal = false;

  // Form field configuration
  const fields: FormFieldConfig[] = [
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
          name: 'full_name_display',
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
        }
      ]
    }
  ];

  // Enhanced company field with add button functionality
  const companyField: FormFieldConfig = {
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
            const companyId = extractSurrealId(company.id) || company.id || '';
            return {
              id: String(companyId),
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
  };

  // Add company field to fields array
  fields[0].fields?.push(companyField);

  // Validation rules
  const validationRules = [
    CommonValidationRules.contact.firstName,
    CommonValidationRules.contact.lastName,
    CommonValidationRules.contact.email,
    { field: 'company', required: true, minLength: 1 }
  ];

  // Save handler
  async function handleSave(formData: any) {
    const timestamp = new Date().toISOString();
    const fullName = `${formData.first_name} ${formData.last_name}`.trim();
    
    if (mode === 'create') {
      const contactData = {
        ...formData,
        full_name: fullName,
        time: {
          created_at: timestamp,
          updated_at: timestamp
        }
      };
      await contactsActions.create(contactData);
    } else if (contact) {
      const contactId = extractSurrealId(contact.id) || extractSurrealId(contact) || contact.id || '';
      if (!contactId) {
        throw new Error('Invalid contact ID');
      }
      
      const contactData = {
        first_name: formData.first_name,
        last_name: formData.last_name,
        full_name: fullName,
        email: formData.email,
        phone: formData.phone,
        position: formData.position,
        company: formData.company
      };
      await contactsActions.update(String(contactId), contactData);
    }
  }

  // Delete handler
  async function handleDelete(entity: Contact) {
    const contactId = extractSurrealId(entity.id) || extractSurrealId(entity) || entity.id || '';
    if (!contactId) {
      throw new Error('Invalid contact ID');
    }
    await contactsActions.delete(String(contactId));
  }

  // Close handler
  function handleClose() {
    dispatch('close');
  }

  // Company modal handlers
  function handleAddCompany() {
    showCompanyModal = true;
  }

  function handleCompanyModalClose() {
    showCompanyModal = false;
  }
</script>

<CrudModal
  {isOpen}
  entity={contact}
  {mode}
  title={mode === 'create' ? 'New Contact' : 'Edit Contact'}
  {fields}
  {validationRules}
  onSave={handleSave}
  onDelete={mode === 'edit' ? handleDelete : null}
  maxWidth="500px"
  customClass="contact-modal"
  on:close={handleClose}
/>

<!-- Company Modal -->
<CompanyModal
  isOpen={showCompanyModal}
  mode="create"
  on:close={handleCompanyModalClose}
/>

<style>
  /* Ensure ContactModal appears above DetailPanel (z-50) */
  :global(.contact-modal) {
    z-index: 65 !important;
  }
</style>