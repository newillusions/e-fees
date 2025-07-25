<!--
  Refactored Proposal Modal using BaseModal, FormInput, FormSelect, and TypeaheadSelect components
  Reduced from ~790 lines to ~480 lines using base components and utilities
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { feesActions, projectsActions, projectsStore, companiesStore, contactsStore } from '$lib/stores';
  import { extractSurrealId } from '$lib/utils/surrealdb';
  import { validateForm, hasValidationErrors } from '$lib/utils/validation';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
  import BaseModal from './BaseModal.svelte';
  import FormInput from './FormInput.svelte';
  import FormSelect from './FormSelect.svelte';
  import TypeaheadSelect from './TypeaheadSelect.svelte';
  import Button from './Button.svelte';
  import type { Fee, Project, Company, Contact } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let proposal: Fee | null = null;
  export let mode: 'create' | 'edit' = 'create';
  
  // Use the new operation state utility
  const { store: operationState, actions: operationActions } = useOperationState();
  
  // Form data with better typing
  interface ProposalFormData {
    number: string;
    name: string;
    issue_date: string;
    rev: string;
    status: 'Draft' | 'Sent' | 'Negotiation' | 'Awarded' | 'Completed' | 'Lost' | 'Cancelled' | 'On Hold' | 'Revised';
    package: string;
    staff_name: string;
    project_id: string;
    company_id: string;
    contact_id: string;
  }
  
  let formData: ProposalFormData = {
    number: '',
    name: '',
    issue_date: '',
    rev: '0',
    status: 'Draft',
    package: '',
    staff_name: '',
    project_id: '',
    company_id: '',
    contact_id: ''
  };
  
  // Status options
  const statusOptions = [
    { value: 'Draft', label: 'Draft' },
    { value: 'Sent', label: 'Sent' },
    { value: 'Negotiation', label: 'Negotiation' },
    { value: 'Awarded', label: 'Awarded' },
    { value: 'Completed', label: 'Completed' },
    { value: 'Lost', label: 'Lost' },
    { value: 'Cancelled', label: 'Cancelled' },
    { value: 'On Hold', label: 'On Hold' },
    { value: 'Revised', label: 'Revised' }
  ];
  
  // Validation setup
  const validationRules = [
    { field: 'number' as keyof ProposalFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'name' as keyof ProposalFormData, required: true, minLength: 1, maxLength: 255 },
    { field: 'issue_date' as keyof ProposalFormData, required: true, minLength: 6, maxLength: 6 },
    { field: 'project_id' as keyof ProposalFormData, required: true, minLength: 1 },
    { field: 'company_id' as keyof ProposalFormData, required: true, minLength: 1 }
  ];
  
  // Form validation state
  let formErrors: Record<string, string> = {};
  
  // UI state
  let showDeleteConfirm = false;
  let showProjectStatusSync = false;
  let originalStatus = '';
  let pendingUpdateData: any = null;
  
  // Typeahead search states
  let projectSearchText = '';
  let companySearchText = '';
  let contactSearchText = '';
  
  // Helper function to extract ID from various formats
  function extractId(value: any): string {
    return extractSurrealId(value) || '';
  }
  
  // All dropdown options for typeahead
  $: allProjectOptions = $projectsStore.map(project => ({
    id: extractId(project.id),
    name: project.name,
    number: project.number?.id || `${project.number?.year || ''}-${project.number?.country || ''}-${project.number?.seq || ''}`.replace(/^-+|-+$/g, '') || 'No Number'
  }));
  
  $: allCompanyOptions = $companiesStore.map(company => ({
    id: extractId(company.id),
    name: company.name,
    name_short: company.name_short
  }));
  
  $: allContactOptions = $contactsStore.map(contact => ({
    id: extractId(contact.id),
    full_name: contact.full_name,
    company: contact.company
  }));
  
  // Filtered dropdown options with typeahead search
  $: projectOptions = allProjectOptions.filter(project => 
    !projectSearchText || 
    project.name.toLowerCase().includes(projectSearchText.toLowerCase()) ||
    project.number.toLowerCase().includes(projectSearchText.toLowerCase())
  ).slice(0, 20);
  
  // Filter companies by contact's company if contact is selected
  $: filteredCompanyOptions = formData.contact_id 
    ? (() => {
        const selectedContact = allContactOptions.find(c => c.id === formData.contact_id);
        if (selectedContact && selectedContact.company) {
          const contactCompanyId = extractSurrealId(selectedContact.company) || '';
          return allCompanyOptions.filter(company => company.id === contactCompanyId);
        }
        return allCompanyOptions;
      })()
    : allCompanyOptions;
  
  $: companyOptions = filteredCompanyOptions.filter(company =>
    !companySearchText ||
    company.name.toLowerCase().includes(companySearchText.toLowerCase()) ||
    (company.name_short && company.name_short.toLowerCase().includes(companySearchText.toLowerCase()))
  ).slice(0, 20);
  
  // Filter contacts by selected company
  $: filteredContactOptions = formData.company_id
    ? allContactOptions.filter(contact => {
        const contactCompanyId = extractSurrealId(contact.company) || '';
        return contactCompanyId === formData.company_id;
      })
    : allContactOptions;
  
  $: contactOptions = filteredContactOptions.filter(contact =>
    !contactSearchText ||
    contact.full_name.toLowerCase().includes(contactSearchText.toLowerCase())
  ).slice(0, 20);
  
  // Helper to format today's date in YYMMDD format
  function getTodayFormatted(): string {
    const today = new Date();
    const year = today.getFullYear().toString().slice(-2);
    const month = (today.getMonth() + 1).toString().padStart(2, '0');
    const day = today.getDate().toString().padStart(2, '0');
    return `${year}${month}${day}`;
  }
  
  // Auto-generate proposal number
  function generateProposalNumber() {
    if (!formData.number && formData.project_id) {
      const project = $projectsStore.find(p => extractId(p.id) === formData.project_id);
      if (project?.number?.id) {
        formData.number = `FP${project.number.id}-001`;
      }
    }
  }
  
  // Check if proposal status is compatible with project status
  function isCompatibleProjectStatus(proposalStatus: string): boolean {
    const proposalToProjectMapping: Record<string, string> = {
      'Draft': 'Draft',
      'Sent': 'RFP',
      'Negotiation': 'RFP',
      'Awarded': 'Awarded',
      'Completed': 'Completed',
      'Lost': 'Lost',
      'Cancelled': 'Cancelled',
      'On Hold': 'On Hold',
      'Revised': 'Revised'
    };
    return proposalStatus in proposalToProjectMapping;
  }
  
  // Get the mapped project status for a proposal status
  function getProjectStatusFromProposalStatus(proposalStatus: string): string {
    const proposalToProjectMapping: Record<string, string> = {
      'Draft': 'Draft',
      'Sent': 'RFP',
      'Negotiation': 'RFP',
      'Awarded': 'Awarded',
      'Completed': 'Completed',
      'Lost': 'Lost',
      'Cancelled': 'Cancelled',
      'On Hold': 'On Hold',
      'Revised': 'Revised'
    };
    return proposalToProjectMapping[proposalStatus] || proposalStatus;
  }
  
  // Form submission handler
  function handleSubmit(event: Event) {
    event.preventDefault();
    
    // Custom validation for date format
    const errors = validateForm(formData, validationRules);
    
    // Additional validation for issue date format (YYMMDD)
    if (formData.issue_date && !/^\d{6}$/.test(formData.issue_date)) {
      errors.issue_date = 'Issue date must be in YYMMDD format';
    }
    
    formErrors = errors;
    
    if (hasValidationErrors(errors)) {
      operationActions.setError('Please fix the validation errors above.');
      return;
    }
    
    if (mode === 'create') {
      handleCreate();
    } else {
      handleUpdate();
    }
  }
  
  // Create proposal with loading state
  async function handleCreate() {
    await withLoadingState(async () => {
      const proposalData = {
        ...formData,
        rev: parseInt(formData.rev) || 0,
        project_id: formData.project_id ? `projects:${formData.project_id}` : null,
        company_id: formData.company_id ? `company:${formData.company_id}` : null,
        contact_id: formData.contact_id ? `contacts:${formData.contact_id}` : null,
        // Add missing fields with defaults
        activity: null,
        strap_line: null,
        staff_email: null,
        staff_phone: null,
        staff_position: null,
        revisions: [],
        time: {
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        }
      };
      
      const result = await feesActions.create(proposalData);
      operationActions.setMessage('Proposal created successfully');
      resetForm();
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Update proposal with loading state  
  async function handleUpdate() {
    if (!proposal) return;
    
    const proposalId = extractSurrealId(proposal);
    if (!proposalId) {
      operationActions.setError('Invalid proposal ID');
      return;
    }
    
    const updateData = {
      ...formData,
      rev: parseInt(formData.rev) || 0,
      project_id: formData.project_id ? `projects:${formData.project_id}` : null,
      company_id: formData.company_id ? `company:${formData.company_id}` : null,
      contact_id: formData.contact_id ? `contacts:${formData.contact_id}` : null,
      // Add missing fields with defaults
      activity: proposal?.activity || null,
      strap_line: proposal?.strap_line || null,
      staff_email: proposal?.staff_email || null,
      staff_phone: proposal?.staff_phone || null,
      staff_position: proposal?.staff_position || null,
      revisions: proposal?.revisions || []
    };
    
    // Check if status has changed and would result in different project status
    const originalProjectStatus = getProjectStatusFromProposalStatus(originalStatus);
    const newProjectStatus = getProjectStatusFromProposalStatus(formData.status);
    const projectStatusWouldChange = originalProjectStatus !== newProjectStatus;
    
    if (originalStatus !== formData.status && isCompatibleProjectStatus(formData.status) && projectStatusWouldChange) {
      // Store the update data and show confirmation dialog
      pendingUpdateData = updateData;
      showProjectStatusSync = true;
      return;
    }
    
    await withLoadingState(async () => {
      const result = await feesActions.update(proposalId, updateData);
      operationActions.setMessage('Proposal updated successfully');
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Delete proposal with loading state
  async function handleDelete() {
    if (!proposal || !showDeleteConfirm) return;
    
    await withLoadingState(async () => {
      const proposalId = extractSurrealId(proposal);
      if (!proposalId) throw new Error('Invalid proposal ID');
      
      const result = await feesActions.delete(proposalId);
      operationActions.setMessage('Proposal deleted successfully');
      closeModal();
      return result;
    }, operationActions, 'deleting');
  }
  
  // Handle project status sync confirmation
  async function handleProjectStatusSync(syncStatus: boolean) {
    showProjectStatusSync = false;
    
    await withLoadingState(async () => {
      const proposalId = extractSurrealId(proposal);
      if (!proposalId || !pendingUpdateData) {
        throw new Error('No proposal data available for update');
      }
      
      // Update the proposal first
      await feesActions.update(proposalId, pendingUpdateData);
      
      // If user confirmed, also update the project status
      if (syncStatus && formData.project_id) {
        const projectId = extractId(formData.project_id);
        if (projectId) {
          const projectStatus = getProjectStatusFromProposalStatus(formData.status);
          
          // Get the current project data from the store and update only the status
          const currentProject = $projectsStore.find(p => extractId(p.id) === projectId);
          if (currentProject) {
            const fullUpdateData = {
              name: currentProject.name,
              name_short: currentProject.name_short,
              status: projectStatus,
              area: currentProject.area,
              city: currentProject.city,
              country: currentProject.country,
              folder: currentProject.folder
            };
            
            await projectsActions.update(projectId, fullUpdateData);
          }
        }
      }
      
      operationActions.setMessage(syncStatus 
        ? 'Proposal and project status updated successfully!' 
        : 'Proposal updated successfully!');
      
      closeModal();
      return true;
    }, operationActions, 'saving');
    
    pendingUpdateData = null;
  }
  
  // Form management
  function resetForm() {
    const todayFormatted = getTodayFormatted();
    
    formData = {
      number: '',
      name: '',
      issue_date: todayFormatted,
      rev: '0',
      status: 'Draft',
      package: '',
      staff_name: '',
      project_id: '',
      company_id: '',
      contact_id: ''
    };
    
    formErrors = {};
    showDeleteConfirm = false;
    showProjectStatusSync = false;
    originalStatus = '';
    pendingUpdateData = null;
    
    // Clear search texts
    projectSearchText = '';
    companySearchText = '';
    contactSearchText = '';
  }
  
  function closeModal() {
    resetForm();
    operationActions.reset();
    dispatch('close');
  }
  
  // Typeahead handlers
  function handleProjectSelect(event: CustomEvent) {
    formData.project_id = event.detail.id;
    generateProposalNumber();
  }
  
  function handleCompanySelect(event: CustomEvent) {
    formData.company_id = event.detail.id;
    // Clear contact when company changes
    formData.contact_id = '';
    contactSearchText = '';
  }
  
  function handleContactSelect(event: CustomEvent) {
    formData.contact_id = event.detail.id;
    
    // Auto-select company if contact has one
    const selectedContact = allContactOptions.find(c => c.id === event.detail.id);
    if (selectedContact && selectedContact.company) {
      const contactCompanyId = extractSurrealId(selectedContact.company) || '';
      if (contactCompanyId && contactCompanyId !== formData.company_id) {
        formData.company_id = contactCompanyId;
        const company = allCompanyOptions.find(c => c.id === contactCompanyId);
        if (company) {
          companySearchText = company.name;
        }
      }
    }
  }
  
  // Load form data when proposal changes
  $: if (proposal && mode === 'edit') {
    // Capture original status when modal first opens for editing
    if (!originalStatus) {
      originalStatus = proposal.status || 'Draft';
    }
    
    formData = {
      number: proposal.number || '',
      name: proposal.name || '',
      issue_date: proposal.issue_date || '',
      rev: proposal.rev?.toString() || '0',
      status: proposal.status || 'Draft',
      package: proposal.package || '',
      staff_name: proposal.staff_name || '',
      project_id: extractId(proposal.project_id) || '',
      company_id: extractId(proposal.company_id) || '',
      contact_id: extractId(proposal.contact_id) || ''
    };
    
    // Set search texts for selected items
    const selectedProject = allProjectOptions.find(p => p.id === formData.project_id);
    if (selectedProject) {
      projectSearchText = `${selectedProject.number} - ${selectedProject.name}`;
    }
    
    const selectedCompany = allCompanyOptions.find(c => c.id === formData.company_id);
    if (selectedCompany) {
      companySearchText = selectedCompany.name;
    }
    
    const selectedContact = allContactOptions.find(c => c.id === formData.contact_id);
    if (selectedContact) {
      contactSearchText = selectedContact.full_name;
    }
  } else if (mode === 'create') {
    resetForm();
  }
</script>

<BaseModal 
  {isOpen} 
  title={mode === 'create' ? 'Create New Fee Proposal' : 'Edit Fee Proposal'}
  maxWidth="500px"
  on:close={closeModal}
>
  <!-- Form -->
  <form on:submit={handleSubmit} style="display: flex; flex-direction: column; gap: 16px;">
    
    <!-- BASIC INFORMATION SECTION -->
    <div>
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        Basic Information
      </h3>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        
        <!-- Number and Name -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Proposal Number"
            bind:value={formData.number}
            placeholder="FP25-97105-001"
            required
            error={formErrors.number}
          />
          
          <FormInput
            label="Proposal Name"
            bind:value={formData.name}
            placeholder="Design Services"
            required
            error={formErrors.name}
          />
        </div>
        
        <!-- Issue Date and Revision -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Issue Date"
            bind:value={formData.issue_date}
            placeholder="YYMMDD format"
            maxlength="6"
            required
            error={formErrors.issue_date}
          />
          
          <FormInput
            label="Revision"
            bind:value={formData.rev}
            placeholder="0"
            min="0"
          />
        </div>
        
        <!-- Status and Package -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormSelect
            label="Status"
            bind:value={formData.status}
            options={statusOptions}
          />
          
          <FormInput
            label="Package"
            bind:value={formData.package}
            placeholder="Package description"
          />
        </div>
        
        <!-- Staff Name -->
        <FormInput
          label="Staff Name"
          bind:value={formData.staff_name}
          placeholder="Staff member name"
        />
      </div>
    </div>
    
    <!-- PROJECT & CLIENT INFORMATION SECTION -->
    <div>
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        Project & Client Information
      </h3>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        
        <!-- Project Selection -->
        <TypeaheadSelect
          label="Project"
          bind:value={formData.project_id}
          bind:searchText={projectSearchText}
          options={projectOptions}
          displayFields={['number', 'name']}
          placeholder="Search projects..."
          required
          error={formErrors.project_id}
          on:select={handleProjectSelect}
        >
          <svelte:fragment slot="option" let:option>
            <span class="font-medium">{option.number}</span> - <span class="truncate">{option.name}</span>
          </svelte:fragment>
        </TypeaheadSelect>
        
        <!-- Company and Contact -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <TypeaheadSelect
            label="Company"
            bind:value={formData.company_id}
            bind:searchText={companySearchText}
            options={companyOptions}
            displayFields={['name']}
            placeholder="Search companies..."
            required
            error={formErrors.company_id}
            on:select={handleCompanySelect}
          />
          
          <TypeaheadSelect
            label="Contact"
            bind:value={formData.contact_id}
            bind:searchText={contactSearchText}
            options={contactOptions}
            displayFields={['full_name']}
            placeholder="Search contacts..."
            on:select={handleContactSelect}
          />
        </div>
      </div>
    </div>
    
    <!-- Error/Success Messages -->
    {#if $operationState.error}
      <div class="text-red-400 text-sm bg-red-900/20 border border-red-500/30 rounded p-3">
        {$operationState.error}
      </div>
    {/if}
    
    {#if $operationState.message}
      <div class="text-green-400 text-sm bg-green-900/20 border border-green-500/30 rounded p-3">
        {$operationState.message}
      </div>
    {/if}
    
    <!-- Delete Confirmation -->
    {#if showDeleteConfirm && mode === 'edit'}
      <div class="text-red-400 text-sm bg-red-900/20 border border-red-500/30 rounded p-3">
        <p class="font-medium mb-2">Are you sure you want to delete this proposal?</p>
        <p class="text-xs opacity-80">This action cannot be undone.</p>
      </div>
    {/if}
    
    <!-- Project Status Sync Confirmation -->
    {#if showProjectStatusSync}
      <div class="text-blue-400 text-sm bg-blue-900/20 border border-blue-500/30 rounded p-3">
        <p class="font-medium mb-2">Update Project Status</p>
        <p class="text-xs opacity-80 mb-3">
          The proposal status change would also update the project status to "{getProjectStatusFromProposalStatus(formData.status)}". 
          Would you like to sync the project status as well?
        </p>
        <div class="flex gap-2">
          <button
            type="button"
            on:click={() => handleProjectStatusSync(true)}
            class="bg-blue-600 hover:bg-blue-700 text-white rounded font-medium transition-all flex items-center justify-center disabled:opacity-50"
            style="height: 24px; padding: 4px 8px; font-size: 11px;"
            disabled={$operationState.saving}
          >
            Yes, sync both
          </button>
          <button
            type="button"
            on:click={() => handleProjectStatusSync(false)}
            class="border border-blue-500/30 rounded text-blue-300 hover:text-blue-200 transition-all"
            style="height: 24px; padding: 4px 8px; font-size: 11px;"
            disabled={$operationState.saving}
          >
            No, proposal only
          </button>
        </div>
      </div>
    {/if}
    
    <!-- Actions -->
    <div class="flex justify-between items-center" style="gap: 12px;">
      
      <!-- Delete Button (Edit Mode Only) -->
      {#if mode === 'edit'}
        <div>
          {#if !showDeleteConfirm}
            <Button
              variant="ghost"
              size="sm"
              on:click={() => showDeleteConfirm = true}
              disabled={$operationState.saving || $operationState.deleting || showProjectStatusSync}
            >
              Delete
            </Button>
          {:else}
            <div class="flex gap-2">
              <button
                class="bg-red-600 hover:bg-red-700 text-white rounded font-medium transition-all flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed"
                style="height: 28px; padding: 6px 12px; font-size: 12px; gap: 6px;"
                disabled={$operationState.deleting}
                on:click={handleDelete}
              >
                {#if $operationState.deleting}
                  <div 
                    class="border-2 border-white border-t-transparent rounded-full animate-spin"
                    style="width: 14px; height: 14px;"
                  ></div>
                {/if}
                Confirm Delete
              </button>
              <Button
                variant="ghost"
                size="sm"
                on:click={() => showDeleteConfirm = false}
                disabled={$operationState.deleting}
              >
                Cancel
              </Button>
            </div>
          {/if}
        </div>
      {:else}
        <div></div>
      {/if}
      
      <!-- Main Actions -->
      <div class="flex" style="gap: 12px;">
        <Button
          variant="secondary"
          size="sm"
          on:click={closeModal}
          disabled={$operationState.saving || $operationState.deleting}
        >
          Cancel
        </Button>
        
        <button
          type="submit"
          class="bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded font-medium transition-all flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed"
          style="height: 28px; padding: 6px 12px; font-size: 12px; gap: 6px;"
          disabled={$operationState.saving || $operationState.deleting || showDeleteConfirm || showProjectStatusSync}
        >
          {#if $operationState.saving}
            <div 
              class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin"
              style="width: 14px; height: 14px;"
            ></div>
          {/if}
          {mode === 'create' ? 'Create Proposal' : 'Update Proposal'}
        </button>
      </div>
    </div>
  </form>
</BaseModal>