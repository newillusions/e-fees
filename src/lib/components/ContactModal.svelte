<!--
  Contact Modal using Generic CrudModal Component
  
  Replaces the original ContactModal with the new generic system.
  Maintains all existing functionality with significantly reduced code.
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { contactsActions, companiesStore } from '$lib/stores';
  import { getEntityId } from '$lib/utils/surrealdb';
  import { CommonValidationRules } from '$lib/utils/validation';
  import { get } from 'svelte/store';
  import CrudModal from './base/CrudModal.svelte';
  import type { Contact, ContactCreate } from '../../types';
  import type { FormFieldConfig } from './base/types';

  const dispatch = createEventDispatcher();

  let { isOpen = $bindable(false), contact = null, mode = 'create', zIndex = 100 }: {
    isOpen?: boolean;
    contact?: Contact | null;
    mode?: 'create' | 'edit';
    zIndex?: number;
  } = $props();

  // Company typeahead field (defined first so it can be included in fields array)
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
            const companyId = getEntityId(company);
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
          computeFn: (formData: Record<string, unknown>) => {
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
        companyField
      ]
    }
  ];

  // Validation rules
  const validationRules = [
    CommonValidationRules.contact.firstName,
    CommonValidationRules.contact.lastName,
    CommonValidationRules.contact.email,
    { field: 'company', required: true, minLength: 1 }
  ];

  // Save handler
  async function handleSave(formData: Record<string, unknown>) {
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
      await contactsActions.create(contactData as unknown as ContactCreate);
    } else if (contact) {
      const contactId = getEntityId(contact);
      if (!contactId) {
        throw new Error('Invalid contact ID');
      }

      const contactData = {
        first_name: formData.first_name as string,
        last_name: formData.last_name as string,
        full_name: fullName,
        email: formData.email as string,
        phone: formData.phone as string,
        position: formData.position as string,
        company: formData.company as string
      };
      await contactsActions.update(String(contactId), contactData);
    }
  }

  // Delete handler
  async function handleDelete(entity: Contact) {
    const contactId = getEntityId(entity);
    if (!contactId) {
      throw new Error('Invalid contact ID');
    }
    await contactsActions.delete(contactId);
  }

  // Close handler
  function handleClose() {
    dispatch('close');
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
  {zIndex}
  on:close={handleClose}
/>
