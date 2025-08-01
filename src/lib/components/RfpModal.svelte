<!--
  Refactored RFP Modal using BaseModal, FormInput, FormSelect, and TypeaheadSelect components
  Reduced from ~1,158 lines to ~650 lines using base components and utilities
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { rfpsActions, companiesStore, contactsStore, projectsStore, settingsStore, settingsActions } from '$lib/stores';
  import { extractSurrealId } from '$lib/utils/surrealdb';
  import { validateForm, hasValidationErrors } from '$lib/utils/validation';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
  import BaseModal from './BaseModal.svelte';
  import FormInput from './FormInput.svelte';
  import FormSelect from './FormSelect.svelte';
  import TypeaheadSelect from './TypeaheadSelect.svelte';
  import Button from './Button.svelte';
  import NewProjectModal from './NewProjectModal.svelte';
  import CompanyModal from './CompanyModal.svelte';
  import ContactModal from './ContactModal.svelte';
  import type { Rfp, Company, Contact, Project } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let rfp: Rfp | null = null;
  export let mode: 'create' | 'edit' = 'create';
  
  // Use the new operation state utility
  const { store: operationState, actions: operationActions } = useOperationState();
  
  // Form data with better typing
  interface RfpFormData {
    name: string;
    number: string;
    rev: number;
    status: 'Draft' | 'Prepared' | 'Sent' | 'Negotiation' | 'Awarded' | 'Lost' | 'Cancelled';
    issue_date: string;
    activity: string;
    package: string;
    project_id: string;
    company_id: string;
    contact_id: string;
    staff_name: string;
    staff_email: string;
    staff_phone: string;
    staff_position: string;
    strap_line: string;
    revisions: any[];
  }
  
  let formData: RfpFormData = {
    name: 'Fee Proposal',
    number: '',
    rev: 1,
    status: 'Draft',
    issue_date: '',
    activity: 'Design and Consultancy',
    package: '',
    project_id: '',
    company_id: '',
    contact_id: '',
    staff_name: '',
    staff_email: '',
    staff_phone: '',
    staff_position: '',
    strap_line: 'sensory design studio',
    revisions: []
  };
  
  // Status options
  const statusOptions = [
    { value: 'Draft', label: 'Draft' },
    { value: 'Prepared', label: 'Prepared' },
    { value: 'Sent', label: 'Sent' },
    { value: 'Negotiation', label: 'Negotiation' },
    { value: 'Awarded', label: 'Awarded' },
    { value: 'Lost', label: 'Lost' },
    { value: 'Cancelled', label: 'Cancelled' }
  ];
  
  // Validation setup
  const validationRules = [
    { field: 'name' as keyof RfpFormData, required: true, minLength: 1, maxLength: 255 },
    { field: 'number' as keyof RfpFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'project_id' as keyof RfpFormData, required: true, minLength: 1 },
    { field: 'company_id' as keyof RfpFormData, required: true, minLength: 1 },
    { field: 'contact_id' as keyof RfpFormData, required: true, minLength: 1 }
  ];
  
  // Form validation state
  let formErrors: Record<string, string> = {};
  
  // UI state
  let showDeleteConfirm = false;
  let showNewProjectModal = false;
  let showCompanyModal = false;
  let showContactModal = false;
  
  // Typeahead search states
  let projectSearchText = '';
  let companySearchText = '';
  let contactSearchText = '';
  
  // Get current settings for staff defaults
  $: currentSettings = $settingsStore?.current || {};
  
  // Helper function to extract ID from SurrealDB Thing object
  function getRecordId(record: any): string {
    return extractSurrealId(record) || '';
  }
  
  // All dropdown options for typeahead
  $: allProjectOptions = $projectsStore.map(project => ({
    id: getRecordId(project),
    name: project.name,
    number: project.number?.id || `${project.number?.year || ''}-${project.number?.country || ''}-${project.number?.seq || ''}`.replace(/^-+|-+$/g, '') || 'No Number'
  }));
  
  $: allCompanyOptions = $companiesStore.map(company => ({
    id: getRecordId(company),
    name: company.name,
    name_short: company.name_short,
    abbreviation: company.abbreviation
  }));
  
  $: allContactOptions = $contactsStore.map(contact => ({
    id: getRecordId(contact),
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
    (company.name_short && company.name_short.toLowerCase().includes(companySearchText.toLowerCase())) ||
    (company.abbreviation && company.abbreviation.toLowerCase().includes(companySearchText.toLowerCase()))
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
  
  // Auto-populate staff fields from settings when modal opens
  function populateStaffFromSettings() {
    if (!currentSettings || Object.keys(currentSettings).length === 0) return;
    
    if (!formData.staff_name && currentSettings.staff_name) {
      formData.staff_name = currentSettings.staff_name;
    }
    if (!formData.staff_email && currentSettings.staff_email) {
      formData.staff_email = currentSettings.staff_email;
    }
    if (!formData.staff_phone && currentSettings.staff_phone) {
      formData.staff_phone = currentSettings.staff_phone;
    }
    if (!formData.staff_position && currentSettings.staff_position) {
      formData.staff_position = currentSettings.staff_position;
    }
  }
  
  // Load settings when modal opens if not available
  $: if (isOpen && (!currentSettings || Object.keys(currentSettings).length === 0)) {
    settingsActions.load().catch(err => console.error('Failed to load settings:', err));
  }
  
  // Helper to format today's date in YYMMDD format
  function getTodayFormatted(): string {
    const today = new Date();
    const year = today.getFullYear().toString().slice(-2);
    const month = (today.getMonth() + 1).toString().padStart(2, '0');
    const day = today.getDate().toString().padStart(2, '0');
    return `${year}${month}${day}`;
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
    
    // Additional validation for email format
    if (formData.staff_email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.staff_email)) {
      errors.staff_email = 'Please enter a valid email address';
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
  
  // Create RFP with loading state
  async function handleCreate() {
    await withLoadingState(async () => {
      const rfpData = {
        ...formData,
        // Send raw IDs - the API layer will handle the SurrealDB formatting
        project_id: formData.project_id || null,
        company_id: formData.company_id || null, 
        contact_id: formData.contact_id || null
      };
      
      const result = await rfpsActions.create(rfpData);
      operationActions.setMessage('FP created successfully');
      resetForm();
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Update RFP with loading state  
  async function handleUpdate() {
    if (!rfp) return;
    
    await withLoadingState(async () => {
      const rfpId = extractSurrealId(rfp);
      if (!rfpId) throw new Error('Invalid RFP ID');
      
      const result = await rfpsActions.update(rfpId, formData);
      operationActions.setMessage('FP updated successfully');
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Delete RFP with loading state
  async function handleDelete() {
    if (!rfp || !showDeleteConfirm) return;
    
    await withLoadingState(async () => {
      const rfpId = extractSurrealId(rfp);
      if (!rfpId) throw new Error('Invalid RFP ID');
      
      const result = await rfpsActions.delete(rfpId);
      operationActions.setMessage('FP deleted successfully');
      closeModal();
      return result;
    }, operationActions, 'deleting');
  }
  
  // Form management
  function resetForm() {
    // Get today's date in YYMMDD format
    const todayFormatted = getTodayFormatted();
    
    formData = {
      name: 'Fee Proposal',
      number: '',
      rev: 1,
      status: 'Draft',
      issue_date: todayFormatted,
      activity: 'Design and Consultancy',
      package: '',
      project_id: '',
      company_id: '',
      contact_id: '',
      staff_name: '',
      staff_email: '',
      staff_phone: '',
      staff_position: '',
      strap_line: 'sensory design studio',
      revisions: []
    };
    
    formErrors = {};
    showDeleteConfirm = false;
    
    // Clear search texts
    projectSearchText = '';
    companySearchText = '';
    contactSearchText = '';
    
    // Populate staff fields from settings
    populateStaffFromSettings();
  }
  
  function closeModal() {
    resetForm();
    operationActions.reset();
    dispatch('close');
  }
  
  // Typeahead handlers
  function handleProjectSelect(event: CustomEvent) {
    formData.project_id = event.detail.id;
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
  
  // Modal handlers
  function handleNewProjectModalClose() {
    showNewProjectModal = false;
  }
  
  function handleCompanyModalClose() {
    showCompanyModal = false;
  }
  
  function handleContactModalClose() {
    showContactModal = false;
  }
  
  // Load form data when rfp changes
  $: if (rfp && mode === 'edit') {
    formData = {
      name: rfp.name || '',
      number: rfp.number || '',
      rev: rfp.rev || 1,
      status: rfp.status || 'Draft',
      issue_date: rfp.issue_date || '',
      activity: rfp.activity || 'Design and Consultancy',
      package: rfp.package || '',
      project_id: getRecordId({ id: rfp.project_id }) || '',
      company_id: getRecordId({ id: rfp.company_id }) || '',
      contact_id: getRecordId({ id: rfp.contact_id }) || '',
      staff_name: rfp.staff_name || '',
      staff_email: rfp.staff_email || '',
      staff_phone: rfp.staff_phone || '',
      staff_position: rfp.staff_position || '',
      strap_line: rfp.strap_line || 'sensory design studio',
      revisions: rfp.revisions || []
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
  title={mode === 'create' ? 'Add New FP' : 'Edit FP'}
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
        
        <!-- Name and Number -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="FP Name"
            bind:value={formData.name}
            placeholder="Hotel Development - Design Services"
            required
            error={formErrors.name}
          />
          
          <FormInput
            label="FP Number"
            bind:value={formData.number}
            placeholder="25-97105-FP"
            required
            error={formErrors.number}
          />
        </div>
        
        <!-- Revision and Status -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Revision"
            type="number"
            bind:value={formData.rev}
            placeholder="1"
            min="1"
          />
          
          <FormSelect
            label="Status"
            bind:value={formData.status}
            options={statusOptions}
          />
        </div>
        
        <!-- Issue Date and Activity -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Issue Date"
            bind:value={formData.issue_date}
            placeholder="YYMMDD format"
            maxlength="6"
            error={formErrors.issue_date}
          />
          
          <FormInput
            label="Activity"
            bind:value={formData.activity}
            placeholder="Design and Consultancy"
          />
        </div>
        
        <!-- Package -->
        <FormInput
          label="Package"
          bind:value={formData.package}
          placeholder="Package description"
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
          showAddButton
          addButtonLabel="Create new project"
          on:select={handleProjectSelect}
          on:add-new={() => showNewProjectModal = true}
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
            showAddButton
            addButtonLabel="Add new company"
            on:select={handleCompanySelect}
            on:add-new={() => showCompanyModal = true}
          />
          
          <TypeaheadSelect
            label="Contact"
            bind:value={formData.contact_id}
            bind:searchText={contactSearchText}
            options={contactOptions}
            displayFields={['full_name']}
            placeholder="Search contacts..."
            required
            error={formErrors.contact_id}
            showAddButton
            addButtonLabel="Add new contact"
            on:select={handleContactSelect}
            on:add-new={() => showContactModal = true}
          />
        </div>
      </div>
    </div>
    
    <!-- STAFF INFORMATION SECTION -->
    <div>
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        Staff Information
      </h3>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        
        <!-- Staff Name and Email -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Staff Name"
            bind:value={formData.staff_name}
            placeholder="Full name"
          />
          
          <FormInput
            label="Staff Email"
            type="email"
            bind:value={formData.staff_email}
            placeholder="email@company.com"
            error={formErrors.staff_email}
          />
        </div>
        
        <!-- Staff Phone and Position -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Staff Phone"
            type="tel"
            bind:value={formData.staff_phone}
            placeholder="+971 50 123 4567"
          />
          
          <FormInput
            label="Staff Position"
            bind:value={formData.staff_position}
            placeholder="Position title"
          />
        </div>
        
        <!-- Strap Line -->
        <FormInput
          label="Strap Line"
          bind:value={formData.strap_line}
          placeholder="sensory design studio"
        />
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
        <p class="font-medium mb-2">Are you sure you want to delete this FP?</p>
        <p class="text-xs opacity-80">This action cannot be undone.</p>
      </div>
    {/if}
    
    <!-- Actions - Full Width Container -->
    <div class="w-full" style="height: 40px;">
      {#if mode === 'edit' && !showDeleteConfirm}
        <!-- Edit Mode: Delete button on left, Cancel/Update on right -->
        <div class="flex justify-between items-stretch h-full" style="gap: 12px;">
          <Button
            variant="ghost"
            size="sm"
            className="!bg-red-600 !text-white hover:!bg-red-700 !border !border-red-500 h-full !py-1 !flex !items-center !justify-center"
            on:click={() => showDeleteConfirm = true}
            disabled={$operationState.saving || $operationState.deleting}
          >
            Delete
          </Button>
          
          <div class="flex h-full" style="gap: 12px;">
            <Button
              variant="secondary"
              size="sm"
              className="h-full !py-1 !flex !items-center !justify-center"
              on:click={closeModal}
              disabled={$operationState.saving || $operationState.deleting}
            >
              Cancel
            </Button>
            
            <Button
              type="submit"
              variant="primary"
              size="sm"
              className="h-full !py-1 !flex !items-center !justify-center"
              disabled={$operationState.saving || $operationState.deleting}
            >
              {#if $operationState.saving}
                <div 
                  class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin"
                  style="width: 14px; height: 14px; margin-right: 6px;"
                ></div>
              {/if}
              Update FP
            </Button>
          </div>
        </div>
      {:else if mode === 'edit' && showDeleteConfirm}
        <!-- Delete Confirmation Mode -->
        <div class="flex justify-between items-stretch h-full" style="gap: 12px;">
          <Button
            variant="ghost"
            size="sm"
            className="!bg-red-600 !text-white hover:!bg-red-700 !border !border-red-500 h-full !py-1 !flex !items-center !justify-center"
            on:click={handleDelete}
            disabled={$operationState.deleting}
          >
            {#if $operationState.deleting}
              <div 
                class="border-2 border-white border-t-transparent rounded-full animate-spin"
                style="width: 14px; height: 14px; margin-right: 6px;"
              ></div>
            {/if}
            Confirm Delete
          </Button>
          <Button
            variant="secondary"
            size="sm"
            className="h-full !py-1 !flex !items-center !justify-center"
            on:click={() => showDeleteConfirm = false}
            disabled={$operationState.deleting}
          >
            Cancel
          </Button>
        </div>
      {:else}
        <!-- Create Mode: Just Cancel/Create buttons -->
        <div class="flex justify-end items-stretch h-full" style="gap: 12px;">
          <Button
            variant="secondary"
            size="sm"
            className="h-full !py-1 !flex !items-center !justify-center"
            on:click={closeModal}
            disabled={$operationState.saving}
          >
            Cancel
          </Button>
          
          <Button
            type="submit"
            variant="primary"
            size="sm"
            className="h-full !py-1 !flex !items-center !justify-center"
            disabled={$operationState.saving}
          >
            {#if $operationState.saving}
              <div 
                class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin"
                style="width: 14px; height: 14px; margin-right: 6px;"
              ></div>
            {/if}
            Create FP
          </Button>
        </div>
      {/if}
    </div>
  </form>
</BaseModal>

<!-- Nested Modals -->
<NewProjectModal
  isOpen={showNewProjectModal}
  mode="create"
  on:close={handleNewProjectModalClose}
/>

<CompanyModal
  isOpen={showCompanyModal}
  mode="create"
  on:close={handleCompanyModalClose}
/>

<ContactModal
  isOpen={showContactModal}
  mode="create"
  on:close={handleContactModalClose}
/>