<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { feesActions, projectsActions, projectsStore, companiesStore, contactsStore } from '$lib/stores';
  import type { Fee, Project, Company, Contact } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let proposal: Fee | null = null; // null for create, proposal object for edit
  export let mode: 'create' | 'edit' = 'create';
  
  // Form data
  let formData = {
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
  
  // Loading and error states
  let isSaving = false;
  let isDeleting = false;
  let saveMessage = '';
  let formErrors: Record<string, string> = {};
  let showDeleteConfirm = false;
  let showProjectStatusSync = false;
  let originalStatus = '';
  let pendingUpdateData: any = null;
  
  // Update form when proposal prop changes
  $: if (proposal && mode === 'edit') {
    formData = {
      number: proposal.number || '',
      name: proposal.name || '',
      issue_date: proposal.issue_date || '',
      rev: proposal.rev || '0',
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
      status: 'Draft',
      package: '',
      staff_name: '',
      project_id: '',
      company_id: '',
      contact_id: ''
    };
  }
  
  // Handle modal open/close state
  $: if (isOpen && mode === 'edit' && proposal && !originalStatus) {
    // Capture original status when modal first opens for editing
    originalStatus = proposal.status || 'Draft';
  } else if (!isOpen) {
    resetForm();
  }
  
  function resetForm() {
    formData = {
      number: '',
      name: '',
      issue_date: new Date().toISOString().split('T')[0].replace(/-/g, '').substring(2),
      rev: '0',
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
    showProjectStatusSync = false;
    originalStatus = '';
    pendingUpdateData = null;
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
          rev: parseInt(formData.rev) || 0, // Convert to number
          project_id: formData.project_id ? `projects:${formData.project_id}` : null,
          company_id: formData.company_id ? `company:${formData.company_id}` : null,
          contact_id: formData.contact_id ? `contacts:${formData.contact_id}` : null,
          // Add missing fields with defaults
          activity: null,
          strap_line: null,
          staff_email: null,
          staff_phone: null,
          staff_position: null,
          revisions: [], // Initialize with empty revisions array
          time: {
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString()
          }
        };
        await feesActions.create(proposalData);
        saveMessage = 'Proposal created successfully!';
      } else {
        const proposalId = getProposalId(proposal);
        if (proposalId) {
          const updateData = {
            ...formData,
            rev: parseInt(formData.rev) || 0, // Convert to number
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
            isSaving = false; // Stop saving spinner until user decides
            return;
          }
          
          await feesActions.update(proposalId, updateData);
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
      await feesActions.delete(proposalId);
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
  function getProposalId(proposal: Fee | null): string | null {
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
  
  // Check if proposal status is compatible with project status
  function isCompatibleProjectStatus(proposalStatus: string): boolean {
    // Map fee proposal statuses to compatible project statuses
    const proposalToProjectMapping: Record<string, string> = {
      'Draft': 'Draft',
      'Sent': 'RFP',        // When proposal is sent, project is typically in RFP stage
      'Negotiation': 'RFP', // During negotiation, project is still in RFP stage
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
      'Sent': 'RFP',        // When proposal is sent, project is typically in RFP stage
      'Negotiation': 'RFP', // During negotiation, project is still in RFP stage
      'Awarded': 'Awarded',
      'Completed': 'Completed',
      'Lost': 'Lost',
      'Cancelled': 'Cancelled',
      'On Hold': 'On Hold',
      'Revised': 'Revised'
    };
    
    return proposalToProjectMapping[proposalStatus] || proposalStatus;
  }
  
  // Handle confirmation to sync project status
  async function handleProjectStatusSync(syncStatus: boolean) {
    showProjectStatusSync = false;
    isSaving = true;
    
    try {
      const proposalId = getProposalId(proposal);
      if (!proposalId || !pendingUpdateData) {
        throw new Error('No proposal data available for update');
      }
      
      // Update the proposal first
      await feesActions.update(proposalId, pendingUpdateData);
      
      // If user confirmed, also update the project status
      if (syncStatus && formData.project_id) {
        const projectId = extractId(formData.project_id);
        if (projectId) {
          // Map fee proposal status to appropriate project status
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
          
          const projectStatus = proposalToProjectMapping[formData.status] || formData.status;
          
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
          } else {
            console.error('Could not find project in store for status update:', projectId);
          }
        }
      }
      
      saveMessage = syncStatus 
        ? 'Proposal and project status updated successfully!' 
        : 'Proposal updated successfully!';
      
      // Auto-close after 1.5 seconds
      setTimeout(() => {
        closeModal();
      }, 1500);
      
    } catch (error: any) {
      saveMessage = `Error: ${error?.message || error}`;
    } finally {
      isSaving = false;
      pendingUpdateData = null;
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
      style="padding: 16px; max-width: 450px;"
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
            
            <!-- Status and Revision -->
            <div class="grid grid-cols-2" style="gap: 12px;">
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
                  <option value="Negotiation">Negotiation</option>
                  <option value="Awarded">Awarded</option>
                  <option value="Completed">Completed</option>
                  <option value="Lost">Lost</option>
                  <option value="Cancelled">Cancelled</option>
                  <option value="On Hold">On Hold</option>
                  <option value="Revised">Revised</option>
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

        <!-- Project Status Sync Confirmation -->
        {#if showProjectStatusSync}
          <div class="rounded-lg bg-blue-900/20 border border-blue-500/30 text-blue-300" style="padding: 12px; font-size: 12px;">
            <p style="margin-bottom: 8px;">🔄 Proposal status changed to "{formData.status}"</p>
            <p class="text-blue-400" style="font-size: 10px; margin-bottom: 12px;">
              Would you like to also update the project status to "{
                ({'Draft': 'Draft', 'Sent': 'RFP', 'Negotiation': 'RFP', 'Awarded': 'Awarded', 'Completed': 'Completed', 'Lost': 'Lost', 'Cancelled': 'Cancelled', 'On Hold': 'On Hold', 'Revised': 'Revised'})[formData.status] || formData.status
              }"?
            </p>
            <div class="flex" style="gap: 8px;">
              <button
                type="button"
                on:click={() => handleProjectStatusSync(false)}
                class="border border-blue-500/30 rounded text-blue-300 hover:text-blue-200 hover:border-blue-400 transition-all"
                style="padding: 4px 8px; font-size: 11px; height: 24px;"
                disabled={isSaving}
              >
                No, Just Proposal
              </button>
              <button
                type="button"
                on:click={() => handleProjectStatusSync(true)}
                class="bg-blue-600 hover:bg-blue-700 text-white rounded transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
                style="padding: 4px 8px; font-size: 11px; height: 24px; gap: 4px;"
                disabled={isSaving}
              >
                {#if isSaving}
                  <div class="border-2 border-white border-t-transparent rounded-full animate-spin" style="width: 10px; height: 10px;"></div>
                  <span>Updating...</span>
                {:else}
                  <span>Yes, Update Both</span>
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
              disabled={isSaving || isDeleting || showDeleteConfirm || showProjectStatusSync}
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
              disabled={isSaving || isDeleting || showProjectStatusSync}
            >
              Cancel
            </button>
            <button
              type="submit"
              class="bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
              style="padding: 6px 12px; font-size: 12px; height: 28px; gap: 4px;"
              disabled={isSaving || isDeleting || showDeleteConfirm || showProjectStatusSync}
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