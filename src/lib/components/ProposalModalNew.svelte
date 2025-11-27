<!--
  Proposal Modal using Generic CrudModal Component
  
  Simplified version of ProposalModal using the new generic system.
  Focuses on core CRUD functionality.
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { feesActions, projectsStore, companiesStore, contactsStore } from '$lib/stores';
  import { extractSurrealId } from '$lib/utils/surrealdb';
  import { CommonValidationRules } from '$lib/utils/validation';
  import { get } from 'svelte/store';
  import CrudModal from './base/CrudModal.svelte';
  import type { Fee } from '$lib/../types';
  import type { FormFieldConfig } from './base/types';

  const dispatch = createEventDispatcher();

  export let isOpen = false;
  export let proposal: Fee | null = null;
  export let mode: 'create' | 'edit' = 'create';

  // Status options
  const statusOptions = [
    { id: 'Draft', name: 'Draft' },
    { id: 'Sent', name: 'Sent' },
    { id: 'Negotiation', name: 'Negotiation' },
    { id: 'Awarded', name: 'Awarded' },
    { id: 'Completed', name: 'Completed' },
    { id: 'Lost', name: 'Lost' },
    { id: 'Cancelled', name: 'Cancelled' },
    { id: 'On Hold', name: 'On Hold' },
    { id: 'Revised', name: 'Revised' }
  ];

  // Form field configuration
  const fields: FormFieldConfig[] = [
    {
      type: 'group',
      name: 'proposal_info',
      groupTitle: 'Proposal Information',
      fields: [
        {
          type: 'text',
          name: 'number',
          label: 'Proposal Number',
          placeholder: 'e.g., P-001',
          required: true,
          colSpan: 1
        },
        {
          type: 'text',
          name: 'name',
          label: 'Proposal Name',
          placeholder: 'Brief description',
          required: true,
          colSpan: 1
        },
        {
          type: 'text',
          name: 'issue_date',
          label: 'Issue Date (YYMMDD)',
          placeholder: '250101',
          required: true,
          maxlength: 6,
          colSpan: 1
        },
        {
          type: 'select',
          name: 'status',
          label: 'Status',
          options: statusOptions,
          required: true,
          colSpan: 1
        },
        {
          type: 'typeahead',
          name: 'project_id',
          label: 'Project',
          placeholder: 'Search projects...',
          required: true,
          colSpan: 2,
          displayFields: ['name'],
          onSearch: async (searchText: string) => {
            if (!searchText || searchText.length < 1) return [];
            
            try {
              const searchLower = searchText.toLowerCase();
              const projects = get(projectsStore);
              
              return projects
                .filter(project => {
                  const nameMatch = project.name?.toLowerCase().includes(searchLower);
                  const shortNameMatch = project.name_short?.toLowerCase().includes(searchLower);
                  const numberMatch = project.number?.id?.toLowerCase().includes(searchLower);
                  return nameMatch || shortNameMatch || numberMatch;
                })
                .map(project => {
                  const projectId = extractSurrealId(project) || extractSurrealId(project.id) || project.id || '';
                  return {
                    id: String(projectId),
                    name: project.name || '',
                    name_short: project.name_short || '',
                    number: project.number?.id || 'No Number'
                  };
                })
                .slice(0, 10);
            } catch (error) {
              console.warn('Failed to search projects:', error);
              return [];
            }
          }
        },
        {
          type: 'typeahead',
          name: 'company_id',
          label: 'Company',
          placeholder: 'Search companies...',
          required: true,
          colSpan: 1,
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
        },
        {
          type: 'typeahead',
          name: 'contact_id',
          label: 'Contact',
          placeholder: 'Search contacts...',
          required: true,
          colSpan: 1,
          displayFields: ['full_name'],
          onSearch: async (searchText: string) => {
            if (!searchText || searchText.length < 1) return [];
            
            try {
              const searchLower = searchText.toLowerCase();
              const contacts = get(contactsStore);
              
              return contacts
                .filter(contact => {
                  const nameMatch = contact.full_name?.toLowerCase().includes(searchLower);
                  const emailMatch = contact.email?.toLowerCase().includes(searchLower);
                  return nameMatch || emailMatch;
                })
                .map(contact => {
                  const contactId = extractSurrealId(contact) || extractSurrealId(contact.id) || contact.id || '';
                  return {
                    id: String(contactId),
                    name: contact.full_name || '',
                    full_name: contact.full_name || '',
                    email: contact.email || ''
                  };
                })
                .slice(0, 10);
            } catch (error) {
              console.warn('Failed to search contacts:', error);
              return [];
            }
          }
        }
      ]
    }
  ];

  // Validation rules
  const validationRules = [
    { field: 'number', required: true, minLength: 1, maxLength: 50 },
    { field: 'name', required: true, minLength: 1, maxLength: 255 },
    { field: 'issue_date', required: true, minLength: 6, maxLength: 6 },
    { field: 'project_id', required: true, minLength: 1 },
    { field: 'company_id', required: true, minLength: 1 },
    { field: 'contact_id', required: true, minLength: 1 }
  ];

  // Save handler
  async function handleSave(formData: any) {
    const timestamp = new Date().toISOString();
    
    if (mode === 'create') {
      const proposalData = {
        ...formData,
        // Set default values for fields not in form
        currency: 'AED',
        amount: 0,
        sub_total: 0,
        vat_amount: 0,
        total_amount: 0,
        vat_rate: 5,
        time: {
          created_at: timestamp,
          updated_at: timestamp
        }
      };
      await feesActions.create(proposalData);
    } else if (proposal) {
      const proposalId = extractSurrealId(proposal.id) || extractSurrealId(proposal) || proposal.id || '';
      if (!proposalId) {
        throw new Error('Invalid proposal ID');
      }
      
      const proposalData = {
        ...formData,
        time: {
          created_at: proposal.time?.created_at || timestamp,
          updated_at: timestamp
        }
      };
      await feesActions.update(String(proposalId), proposalData);
    }
  }

  // Delete handler
  async function handleDelete(entity: Fee) {
    const proposalId = extractSurrealId(entity.id) || extractSurrealId(entity) || entity.id || '';
    if (!proposalId) {
      throw new Error('Invalid proposal ID');
    }
    await feesActions.delete(String(proposalId));
  }

  // Close handler
  function handleClose() {
    dispatch('close');
  }
</script>

<CrudModal
  {isOpen}
  entity={proposal}
  {mode}
  title={mode === 'create' ? 'New Proposal' : 'Edit Proposal'}
  {fields}
  {validationRules}
  onSave={handleSave}
  onDelete={mode === 'edit' ? handleDelete : null}
  maxWidth="600px"
  customClass="proposal-modal"
  on:close={handleClose}
/>

<style>
  /* Ensure ProposalModal appears above other modals */
  :global(.proposal-modal) {
    z-index: 65 !important;
  }
</style>