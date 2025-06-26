<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { rfpsActions, projectsStore, companiesStore, contactsStore } from '$lib/stores';
  import type { FeeProposal, Project, Company, Contact } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let proposal: FeeProposal | null = null; // null for create, proposal object for edit
  export let mode: 'create' | 'edit' = 'create';
  
  // Form data
  let formData = {
    number: '',
    name: '',
    issue_date: '',
    rev: '0',
    stage: 'ITT',
    status: 'Draft',
    package: '',
    staff_name: '',
    project_id: '',
    company_id: '',
    contact_id: ''
  };
  
  // Loading and error states
  let isSaving = false;
  let isDeleting = false;
  let saveMessage = '';
  let formErrors: Record<string, string> = {};
  let showDeleteConfirm = false;
  
  // Update form when proposal prop changes
  $: if (proposal && mode === 'edit') {
    formData = {
      number: proposal.number || '',
      name: proposal.name || '',
      issue_date: proposal.issue_date || '',
      rev: proposal.rev || '0',
      stage: proposal.stage || 'ITT',
      status: proposal.status || 'Draft',
      package: proposal.package || '',
      staff_name: proposal.staff_name || '',
      project_id: extractId(proposal.project_id) || '',
      company_id: extractId(proposal.company_id) || '',
      contact_id: extractId(proposal.contact_id) || ''
    };
  } else if (mode === 'create') {
    // Reset form for create mode
    formData = {
      number: '',
      name: '',
      issue_date: new Date().toISOString().split('T')[0].replace(/-/g, '').substring(2), // YYMMDD format
      rev: '0',
      stage: 'ITT',
      status: 'Draft',
      package: '',
      staff_name: '',
      project_id: '',
      company_id: '',
      contact_id: ''
    };
  }
  
  // Reset state when modal closes
  $: if (!isOpen) {
    resetForm();
  }
  
  function resetForm() {
    formData = {
      number: '',
      name: '',
      issue_date: new Date().toISOString().split('T')[0].replace(/-/g, '').substring(2),
      rev: '0',
      stage: 'ITT',
      status: 'Draft',
      package: '',
      staff_name: '',
      project_id: '',
      company_id: '',
      contact_id: ''
    };
    formErrors = {};
    saveMessage = '';
    isSaving = false;
    isDeleting = false;
    showDeleteConfirm = false;
  }
  
  function validateForm(): boolean {
    formErrors = {};
    
    // Required fields
    if (!formData.number.trim()) {
      formErrors.number = 'Proposal number is required';
    }
    
    if (!formData.name.trim()) {
      formErrors.name = 'Proposal name is required';
    }
    
    if (!formData.issue_date.trim()) {
      formErrors.issue_date = 'Issue date is required';
    } else if (formData.issue_date.length !== 6) {
      formErrors.issue_date = 'Issue date must be in YYMMDD format';
    }
    
    if (!formData.project_id) {
      formErrors.project_id = 'Project is required';
    }
    
    if (!formData.company_id) {
      formErrors.company_id = 'Company is required';
    }
    
    return Object.keys(formErrors).length === 0;
  }
  
  async function handleSubmit() {
    if (!validateForm()) {
      return;
    }
    
    isSaving = true;
    saveMessage = '';
    
    try {
      if (mode === 'create') {
        const proposalData = {
          ...formData,
          project_id: formData.project_id ? `projects:${formData.project_id}` : null,
          company_id: formData.company_id ? `company:${formData.company_id}` : null,
          contact_id: formData.contact_id ? `contacts:${formData.contact_id}` : null,
          time: {
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString()
          }
        };
        await rfpsActions.create(proposalData);
        saveMessage = 'Proposal created successfully!';
      } else {
        const proposalId = getProposalId(proposal);
        if (proposalId) {
          const updateData = {
            ...formData,
            project_id: formData.project_id ? `projects:${formData.project_id}` : null,
            company_id: formData.company_id ? `company:${formData.company_id}` : null,
            contact_id: formData.contact_id ? `contacts:${formData.contact_id}` : null
          };
          await rfpsActions.update(proposalId, updateData);
          saveMessage = 'Proposal updated successfully!';
        } else {
          throw new Error('No valid proposal ID found for update');
        }
      }
      
      // Auto-close after 1.5 seconds
      setTimeout(() => {
        closeModal();
      }, 1500);
      
    } catch (error: any) {
      saveMessage = `Error: ${error?.message || error}`;
    } finally {
      isSaving = false;
    }
  }
  
  async function handleDelete() {
    const proposalId = getProposalId(proposal);
    if (!proposalId) return;
    
    isDeleting = true;
    saveMessage = '';
    
    try {
      await rfpsActions.delete(proposalId);
      saveMessage = 'Proposal deleted successfully!';
      
      // Auto-close after 1.5 seconds
      setTimeout(() => {
        closeModal();
      }, 1500);
      
    } catch (error: any) {
      saveMessage = `Error: ${error?.message || error}`;
    } finally {
      isDeleting = false;
      showDeleteConfirm = false;
    }
  }
  
  function confirmDelete() {
    showDeleteConfirm = true;
  }
  
  function cancelDelete() {
    showDeleteConfirm = false;
  }
  
  function closeModal() {
    isOpen = false;
    resetForm();
    dispatch('close');
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeModal();
    }
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
  
  // Helper function to extract ID from various formats
  function extractId(value: any): string {
    if (!value) return '';
    if (typeof value === 'string') return value.replace(/^[^:]+:/, '');
    if (value.tb && value.id) {
      if (typeof value.id === 'string') return value.id;
      if (value.id.String) return value.id.String;
    }
    return '';
  }
  
  // Helper function to extract ID from SurrealDB Thing object
  function getProposalId(proposal: FeeProposal | null): string | null {
    if (!proposal?.id) return null;
    
    if (typeof proposal.id === 'string') {
      return proposal.id;
    }
    
    // Handle SurrealDB Thing object format
    if (proposal.id && typeof proposal.id === 'object') {
      const thingObj = proposal.id as any;
      if (thingObj.tb && thingObj.id) {
        if (typeof thingObj.id === 'string') {
          return thingObj.id;
        } else if (thingObj.id.String) {
          return thingObj.id.String;
        }
      }
    }
    
    return null;
  }
  
  // Format date for display
  function formatDateForInput(dateStr: string): string {
    if (!dateStr) return '';
    if (dateStr.length === 6) {
      // YYMMDD format
      return `20${dateStr.substring(0,2)}-${dateStr.substring(2,4)}-${dateStr.substring(4,6)}`;
    }
    return dateStr.split('T')[0];
  }
  
  // Format date from input
  function formatDateFromInput(event: Event) {
    const input = event.target as HTMLInputElement;
    const value = input.value;
    if (value) {
      // Convert YYYY-MM-DD to YYMMDD
      const parts = value.split('-');
      if (parts.length === 3) {
        formData.issue_date = parts[0].substring(2) + parts[1] + parts[2];
      }
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Modal Backdrop -->
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4"
    on:click={closeModal}
    on:keydown={(e) => e.key === 'Escape' && closeModal()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="proposal-modal-title"
    tabindex="-1"
  >
    <!-- Modal Content -->
    <div 
      class="bg-emittiv-darker border border-emittiv-dark rounded w-full max-h-[90vh] overflow-y-auto"
      style="padding: 24px; max-width: 700px;"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="presentation"
    >
      <!-- Header -->
      <div class="flex items-center justify-between" style="margin-bottom: 20px;">
        <h2 id="proposal-modal-title" class="font-semibold text-emittiv-white" style="font-size: 16px;">
          {mode === 'create' ? 'Create New Fee Proposal' : 'Edit Fee Proposal'}
        </h2>
        <button 
          on:click={closeModal}
          class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth"
          aria-label="Close modal"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <form on:submit|preventDefault={handleSubmit} style="display: flex; flex-direction: column; gap: 24px;">
        
        <!-- Proposal Basic Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Proposal Information</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Proposal Number and Name -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="proposal_number" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Proposal Number *
                </label>
                <input
                  id="proposal_number"
                  type="text"
                  bind:value={formData.number}
                  placeholder="FP24-97101-001"
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.number ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.number}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.number}</p>
                {/if}
              </div>
              
              <div>
                <label for="proposal_issue_date" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Issue Date *
                </label>
                <input
                  id="proposal_issue_date"
                  type="date"
                  value={formatDateForInput(formData.issue_date)}
                  on:change={formatDateFromInput}
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.issue_date ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.issue_date}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.issue_date}</p>
                {/if}
              </div>
            </div>
            
            <!-- Proposal Name -->
            <div>
              <label for="proposal_name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Proposal Name *
              </label>
              <input
                id="proposal_name"
                type="text"
                bind:value={formData.name}
                placeholder="Fee Proposal for MEP Design Services"
                required
                class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.name ? 'border-red-500' : ''}"
                style="padding: 8px 12px; font-size: 12px; height: 32px;"
              />
              {#if formErrors.name}
                <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.name}</p>
              {/if}
            </div>
            
            <!-- Package and Staff -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="proposal_package" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Package
                </label>
                <input
                  id="proposal_package"
                  type="text"
                  bind:value={formData.package}
                  placeholder="Package A"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
              
              <div>
                <label for="proposal_staff" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Staff Name
                </label>
                <input
                  id="proposal_staff"
                  type="text"
                  bind:value={formData.staff_name}
                  placeholder="John Doe"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
            </div>
            
            <!-- Status and Stage -->
            <div class="grid grid-cols-3" style="gap: 12px;">
              <div>
                <label for="proposal_rev" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Revision
                </label>
                <input
                  id="proposal_rev"
                  type="text"
                  bind:value={formData.rev}
                  placeholder="0"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
              
              <div>
                <label for="proposal_stage" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Stage
                </label>
                <select
                  id="proposal_stage"
                  bind:value={formData.stage}
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all appearance-none"
                  style="padding: 8px 12px; font-size: 12px; height: 32px; padding-right: 32px; background-image: url('data:image/svg+xml,%3csvg xmlns=\'http://www.w3.org/2000/svg\' fill=\'none\' viewBox=\'0 0 20 20\'%3e%3cpath stroke=\'%23999\' stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-width=\'1.5\' d=\'M6 8l4 4 4-4\'/%3e%3c/svg%3e'); background-position: right 0.5rem center; background-repeat: no-repeat; background-size: 16px 12px;"
                >
                  <option value="ITT">ITT</option>
                  <option value="Direct">Direct</option>
                  <option value="Client">Client</option>
                </select>
              </div>
              
              <div>
                <label for="proposal_status" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Status
                </label>
                <select
                  id="proposal_status"
                  bind:value={formData.status}
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all appearance-none"
                  style="padding: 8px 12px; font-size: 12px; height: 32px; padding-right: 32px; background-image: url('data:image/svg+xml,%3csvg xmlns=\'http://www.w3.org/2000/svg\' fill=\'none\' viewBox=\'0 0 20 20\'%3e%3cpath stroke=\'%23999\' stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-width=\'1.5\' d=\'M6 8l4 4 4-4\'/%3e%3c/svg%3e'); background-position: right 0.5rem center; background-repeat: no-repeat; background-size: 16px 12px;"
                >
                  <option value="Draft">Draft</option>
                  <option value="Sent">Sent</option>
                  <option value="Awarded">Awarded</option>
                  <option value="Lost">Lost</option>
                  <option value="Cancelled">Cancelled</option>
                </select>
              </div>
            </div>
          </div>
        </div>

        <!-- Related Entities Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Related Information</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Project Selection -->
            <div>
              <label for="proposal_project" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Project *
              </label>
              <select
                id="proposal_project"
                bind:value={formData.project_id}
                on:change={generateProposalNumber}
                required
                class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all appearance-none {formErrors.project_id ? 'border-red-500' : ''}"
                style="padding: 8px 12px; font-size: 12px; height: 32px; padding-right: 32px; background-image: url('data:image/svg+xml,%3csvg xmlns=\'http://www.w3.org/2000/svg\' fill=\'none\' viewBox=\'0 0 20 20\'%3e%3cpath stroke=\'%23999\' stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-width=\'1.5\' d=\'M6 8l4 4 4-4\'/%3e%3c/svg%3e'); background-position: right 0.5rem center; background-repeat: no-repeat; background-size: 16px 12px;"
              >
                <option value="">Select a project</option>
                {#each $projectsStore as project}
                  <option value={extractId(project.id)}>
                    {project.number?.id} - {project.name}
                  </option>
                {/each}
              </select>
              {#if formErrors.project_id}
                <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.project_id}</p>
              {/if}
            </div>
            
            <!-- Company and Contact -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="proposal_company" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Company *
                </label>
                <select
                  id="proposal_company"
                  bind:value={formData.company_id}
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all appearance-none {formErrors.company_id ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px; padding-right: 32px; background-image: url('data:image/svg+xml,%3csvg xmlns=\'http://www.w3.org/2000/svg\' fill=\'none\' viewBox=\'0 0 20 20\'%3e%3cpath stroke=\'%23999\' stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-width=\'1.5\' d=\'M6 8l4 4 4-4\'/%3e%3c/svg%3e'); background-position: right 0.5rem center; background-repeat: no-repeat; background-size: 16px 12px;"
                >
                  <option value="">Select a company</option>
                  {#each $companiesStore as company}
                    <option value={extractId(company.id)}>
                      {company.name}
                    </option>
                  {/each}
                </select>
                {#if formErrors.company_id}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.company_id}</p>
                {/if}
              </div>
              
              <div>
                <label for="proposal_contact" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Contact
                </label>
                <select
                  id="proposal_contact"
                  bind:value={formData.contact_id}
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all appearance-none"
                  style="padding: 8px 12px; font-size: 12px; height: 32px; padding-right: 32px; background-image: url('data:image/svg+xml,%3csvg xmlns=\'http://www.w3.org/2000/svg\' fill=\'none\' viewBox=\'0 0 20 20\'%3e%3cpath stroke=\'%23999\' stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-width=\'1.5\' d=\'M6 8l4 4 4-4\'/%3e%3c/svg%3e'); background-position: right 0.5rem center; background-repeat: no-repeat; background-size: 16px 12px;"
                >
                  <option value="">Select a contact</option>
                  {#each $contactsStore.filter(c => !formData.company_id || extractId(c.company) === formData.company_id) as contact}
                    <option value={extractId(contact.id)}>
                      {contact.full_name}
                    </option>
                  {/each}
                </select>
              </div>
            </div>
          </div>
        </div>

        <!-- Save Message -->
        {#if saveMessage}
          <div class="rounded-lg {saveMessage.startsWith('Error') ? 'bg-red-900/20 border border-red-500/30 text-red-300' : 'bg-green-900/20 border border-green-500/30 text-green-300'}" style="padding: 8px; font-size: 11px;">
            {saveMessage}
          </div>
        {/if}

        <!-- Delete Confirmation -->
        {#if showDeleteConfirm}
          <div class="rounded-lg bg-red-900/20 border border-red-500/30 text-red-300" style="padding: 12px; font-size: 12px;">
            <p style="margin-bottom: 8px;">⚠️ Are you sure you want to delete this proposal?</p>
            <p class="text-red-400" style="font-size: 10px; margin-bottom: 12px;">This action cannot be undone.</p>
            <div class="flex" style="gap: 8px;">
              <button
                type="button"
                on:click={cancelDelete}
                class="border border-red-500/30 rounded text-red-300 hover:text-red-200 hover:border-red-400 transition-all"
                style="padding: 4px 8px; font-size: 11px; height: 24px;"
                disabled={isDeleting}
              >
                Cancel
              </button>
              <button
                type="button"
                on:click={handleDelete}
                class="bg-red-600 hover:bg-red-700 text-white rounded transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
                style="padding: 4px 8px; font-size: 11px; height: 24px; gap: 4px;"
                disabled={isDeleting}
              >
                {#if isDeleting}
                  <div class="border-2 border-white border-t-transparent rounded-full animate-spin" style="width: 10px; height: 10px;"></div>
                  <span>Deleting...</span>
                {:else}
                  <span>Delete Proposal</span>
                {/if}
              </button>
            </div>
          </div>
        {/if}

        <!-- Action Buttons -->
        <div class="flex {mode === 'edit' ? 'justify-between' : 'justify-end'} border-t border-emittiv-dark" style="gap: 12px; padding-top: 16px; margin-top: 8px;">
          {#if mode === 'edit'}
            <button
              type="button"
              on:click={confirmDelete}
              class="border border-red-500/50 rounded text-red-400 hover:text-red-300 hover:border-red-400 transition-all"
              style="padding: 6px 12px; font-size: 12px; height: 28px;"
              disabled={isSaving || isDeleting || showDeleteConfirm}
            >
              Delete
            </button>
          {/if}
          
          <div class="flex" style="gap: 12px;">
            <button
              type="button"
              on:click={closeModal}
              class="border border-emittiv-dark rounded text-emittiv-light hover:text-emittiv-white hover:border-emittiv-light transition-all"
              style="padding: 6px 12px; font-size: 12px; height: 28px;"
              disabled={isSaving || isDeleting}
            >
              Cancel
            </button>
            <button
              type="submit"
              class="bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
              style="padding: 6px 12px; font-size: 12px; height: 28px; gap: 4px;"
              disabled={isSaving || isDeleting || showDeleteConfirm}
            >
              {#if isSaving}
                <div class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin" style="width: 12px; height: 12px;"></div>
                <span>Saving...</span>
              {:else}
                <span>{mode === 'create' ? 'Create Proposal' : 'Update Proposal'}</span>
              {/if}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}