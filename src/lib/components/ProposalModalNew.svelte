<!--
  Proposal Modal using Generic CrudModal Component
  
  Simplified version of ProposalModal using the new generic system.
  Focuses on core CRUD functionality.
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { feesActions, projectsStore, companiesStore, contactsStore } from '$lib/stores';
  import { extractSurrealId, getEntityId } from '$lib/utils/surrealdb';
  import { CommonValidationRules } from '$lib/utils/validation';
  import { createProjectTypeaheadSearch, createCompanyTypeaheadSearch, createContactTypeaheadSearch } from '$lib/utils/search';
  import { get } from 'svelte/store';
  import { PROPOSAL_STATUS_OPTIONS } from '$lib/constants';
  import CrudModal from './base/CrudModal.svelte';
  import type { Fee } from '$lib/../types';
  import type { FormFieldConfig } from './base/types';

  // Create optimized typeahead search functions (PERF-H5 + QUAL-H4 fix)
  const searchProjects = createProjectTypeaheadSearch(extractSurrealId);
  const searchCompanies = createCompanyTypeaheadSearch(extractSurrealId);
  const searchContacts = createContactTypeaheadSearch(extractSurrealId);

  const dispatch = createEventDispatcher();

  let { isOpen = $bindable(false), proposal = null, mode = 'create' }: {
    isOpen?: boolean;
    proposal?: Fee | null;
    mode?: 'create' | 'edit';
  } = $props();


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
          options: PROPOSAL_STATUS_OPTIONS,
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
            try {
              return searchProjects(get(projectsStore), searchText);
            } catch {
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
            try {
              return searchCompanies(get(companiesStore), searchText);
            } catch {
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
            try {
              return searchContacts(get(contactsStore), searchText);
            } catch {
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
      const proposalId = getEntityId(proposal);
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
    const proposalId = getEntityId(entity);
    if (!proposalId) {
      throw new Error('Invalid proposal ID');
    }
    await feesActions.delete(proposalId);
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
  zIndex={65}
  on:close={handleClose}
/>