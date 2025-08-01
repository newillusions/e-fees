<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { rfpsActions, companiesStore, contactsStore, projectsStore, settingsStore, settingsActions } from '$lib/stores';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import type { Rfp, Company, Contact, Project } from '$lib/../types';
  import NewProjectModal from './NewProjectModal.svelte';
  import CompanyModal from './CompanyModal.svelte';
  import ContactModal from './ContactModal.svelte';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let rfp: Rfp | null = null; // null for create, rfp object for edit
  export let mode: 'create' | 'edit' = 'create';
  
  // Form data
  let formData = {
    name: 'Fee Proposal',
    number: '',
    rev: 1,
    status: 'Draft' as const,
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
    revisions: [] as any[]
  };
  
  // Loading and error states
  let isSaving = false;
  let isDeleting = false;
  let saveMessage = '';
  let formErrors: Record<string, string> = {};
  let showDeleteConfirm = false;

  // Nested modal states
  let showNewProjectModal = false;
  let showCompanyModal = false;
  let showContactModal = false;
  
  // Search/filter states for typeahead functionality
  let projectSearchText = '';
  let companySearchText = '';
  let contactSearchText = '';
  let projectDropdownOpen = false;
  let companyDropdownOpen = false;
  let contactDropdownOpen = false;
  
  // Status and stage options
  const statusOptions = ['Draft', 'Prepared', 'Sent', 'Negotiation', 'Awarded', 'Lost', 'Cancelled'];
  
  // Helper function to extract ID from SurrealDB Thing object
  // Helper function to extract ID from SurrealDB Thing object - COPIED FROM WORKING CONTACTS MODAL
  function getRfpId(rfp: Rfp | null): string | null {
    if (!rfp?.id) return null;
    
    if (typeof rfp.id === 'string') {
      return rfp.id;
    }
    
    // Handle SurrealDB Thing object format
    if (rfp.id && typeof rfp.id === 'object') {
      const thingObj = rfp.id as any;
      if (thingObj.tb && thingObj.id) {
        if (typeof thingObj.id === 'string') {
          return thingObj.id; // Return just the ID part, not "rfp:ID"
        } else if (thingObj.id.String) {
          return thingObj.id.String;
        }
      }
    }
    
    return null;
  }

  function getRecordId(record: any): string {
    if (!record?.id) return '';
    
    if (typeof record.id === 'string') {
      return record.id;
    }
    
    // Handle SurrealDB Thing object format
    const thingObj = record.id as any;
    if (thingObj.tb && thingObj.id) {
      if (typeof thingObj.id === 'string') {
        return thingObj.id;
      } else if (thingObj.id.String) {
        return thingObj.id.String;
      }
    }
    
    return '';
  }
  
  // All dropdown options
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
  ).slice(0, 20); // Limit to 20 results for performance
  
  // Filter companies by contact's company if contact is selected
  $: filteredCompanyOptions = formData.contact_id 
    ? (() => {
        const selectedContact = allContactOptions.find(c => c.id === formData.contact_id);
        if (selectedContact && selectedContact.company) {
          let contactCompanyId = '';
          if (typeof selectedContact.company === 'string') {
            contactCompanyId = selectedContact.company;
          } else if (selectedContact.company && typeof selectedContact.company === 'object') {
            const thingObj = selectedContact.company as any;
            if (thingObj.tb && thingObj.id) {
              if (typeof thingObj.id === 'string') {
                contactCompanyId = thingObj.id;
              } else if (thingObj.id.String) {
                contactCompanyId = thingObj.id.String;
              }
            }
          }
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
  
  $: contactOptions = allContactOptions.filter(contact => 
    !contactSearchText || 
    contact.full_name.toLowerCase().includes(contactSearchText.toLowerCase())
  ).slice(0, 20);
  
  // Filter contacts by selected company
  $: filteredContactOptions = formData.company_id 
    ? contactOptions.filter(contact => {
        // Handle different ID formats for contact.company
        let contactCompanyId = '';
        if (typeof contact.company === 'string') {
          contactCompanyId = contact.company;
        } else if (contact.company && typeof contact.company === 'object') {
          const thingObj = contact.company as any;
          if (thingObj.tb && thingObj.id) {
            if (typeof thingObj.id === 'string') {
              contactCompanyId = thingObj.id;
            } else if (thingObj.id.String) {
              contactCompanyId = thingObj.id.String;
            }
          }
        }
        return contactCompanyId === formData.company_id;
      })
    : contactOptions;
  
  // Get current settings for auto-population
  $: currentSettings = $settingsStore;
  
  // Auto-populate staff fields from settings when modal opens in create mode
  $: if (isOpen && mode === 'create' && currentSettings) {
    // Only populate if fields are empty and settings are available
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
  
  // Update form when rfp prop changes
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
  } else if (mode === 'create') {
    // Reset form for create mode
    resetFormData();
  }
  
  // Reset state when modal closes
  $: if (!isOpen) {
    resetForm();
  }
  
  function resetFormData() {
    // Get today's date in YYMMDD format
    const today = new Date();
    const year = today.getFullYear().toString().slice(-2);
    const month = (today.getMonth() + 1).toString().padStart(2, '0');
    const day = today.getDate().toString().padStart(2, '0');
    const todayFormatted = `${year}${month}${day}`;
    
    formData = {
      name: 'Fee Proposal',
      number: '',
      rev: 1, // Default revision for new FPs
      status: 'Draft',
      issue_date: todayFormatted, // Auto-populate with today's date
      activity: 'Design and Consultancy', // Default activity
      package: '',
      project_id: '',
      company_id: '',
      contact_id: '',
      staff_name: '', // Will be populated by reactive statement
      staff_email: '', // Will be populated by reactive statement
      staff_phone: '', // Will be populated by reactive statement
      staff_position: '', // Will be populated by reactive statement
      strap_line: 'sensory design studio', // Default strap line
      revisions: []
    };
  }
  
  function resetForm() {
    resetFormData();
    formErrors = {};
    saveMessage = '';
    isSaving = false;
    isDeleting = false;
    showDeleteConfirm = false;
    // Reset search states
    projectSearchText = '';
    companySearchText = '';
    contactSearchText = '';
    projectDropdownOpen = false;
    companyDropdownOpen = false;
    contactDropdownOpen = false;
  }
  
  // Auto-generate FP number from project when project is selected
  function updateRfpNumber() {
    if (formData.project_id && mode === 'create') {
      const selectedProject = allProjectOptions.find(p => p.id === formData.project_id);
      if (selectedProject && selectedProject.number) {
        // Simple FP number generation: PROJECT_NUMBER-FP
        formData.number = `${selectedProject.number}-FP`;
      }
    }
  }
  
  // Format issue date to YYMMDD
  function formatIssueDate(dateString: string): string {
    if (!dateString) return '';
    const date = new Date(dateString);
    const year = date.getFullYear().toString().slice(-2);
    const month = (date.getMonth() + 1).toString().padStart(2, '0');
    const day = date.getDate().toString().padStart(2, '0');
    return `${year}${month}${day}`;
  }
  
  function validateForm(): boolean {
    formErrors = {};
    
    // Validate required fields
    if (!formData.name.trim()) {
      formErrors.name = 'FP name is required';
    }
    
    if (!formData.number.trim()) {
      formErrors.number = 'FP number is required';
    }
    
    if (!formData.project_id) {
      formErrors.project_id = 'Project is required';
    }
    
    if (!formData.company_id) {
      formErrors.company_id = 'Company is required';
    }
    
    if (!formData.contact_id) {
      formErrors.contact_id = 'Contact is required';
    }
    
    // Validate issue date format (YYMMDD)
    if (formData.issue_date && !/^\d{6}$/.test(formData.issue_date)) {
      formErrors.issue_date = 'Issue date must be in YYMMDD format';
    }
    
    // Validate email format if provided
    if (formData.staff_email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.staff_email)) {
      formErrors.staff_email = 'Please enter a valid email address';
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
        const rfpData = {
          name: formData.name,
          number: formData.number,
          rev: formData.rev || 1, // Use form value or default to 1
          status: formData.status,
          issue_date: formData.issue_date,
          activity: formData.activity,
          package: formData.package,
          staff_name: formData.staff_name,
          staff_email: formData.staff_email,
          staff_phone: formData.staff_phone,
          staff_position: formData.staff_position,
          strap_line: formData.strap_line,
          revisions: formData.revisions,
          // Send raw IDs - the API layer will handle the SurrealDB formatting
          project_id: formData.project_id || null,
          company_id: formData.company_id || null, 
          contact_id: formData.contact_id || null
        };
        
        await rfpsActions.create(rfpData);
        saveMessage = 'FP created successfully!';
      } else {
        const rfpId = getRfpId(rfp);
        if (rfpId) {
          await rfpsActions.update(rfpId, formData);
          saveMessage = 'FP updated successfully!';
        } else {
          throw new Error('No valid FP ID found for update');
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
    const rfpId = getRecordId(rfp);
    if (!rfpId) return;
    
    isDeleting = true;
    saveMessage = '';
    
    try {
      await rfpsActions.delete(rfpId);
      saveMessage = 'FP deleted successfully!';
      
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

  // Nested modal handlers
  function openNewProjectModal() {
    showNewProjectModal = true;
  }

  function openCompanyModal() {
    showCompanyModal = true;
  }

  function openContactModal() {
    showContactModal = true;
  }

  function handleProjectCreated(newProject: Project) {
    showNewProjectModal = false;
    // Auto-select the newly created project
    formData.project_id = getRecordId(newProject);
    updateRfpNumber(); // Update FP number based on new project
  }

  function handleCompanyCreated(event: CustomEvent) {
    const newCompany = event.detail;
    showCompanyModal = false;
    // Auto-select the newly created company
    formData.company_id = getRecordId(newCompany);
    // Clear contact selection since company changed
    formData.contact_id = '';
  }

  function handleContactCreated(event: CustomEvent) {
    const newContact = event.detail;
    showContactModal = false;
    // Auto-select the newly created contact
    formData.contact_id = getRecordId(newContact);
    // Auto-select the contact's company if not already selected
    if (!formData.company_id && newContact.company) {
      formData.company_id = newContact.company;
    }
  }
  
  // Typeahead helper functions
  function selectProject(projectId: string) {
    formData.project_id = projectId;
    const selectedProject = allProjectOptions.find(p => p.id === projectId);
    projectSearchText = selectedProject ? `${selectedProject.number} - ${selectedProject.name}` : '';
    projectDropdownOpen = false;
    updateRfpNumber(); // Auto-generate FP number
  }
  
  function selectCompany(companyId: string) {
    formData.company_id = companyId;
    const selectedCompany = allCompanyOptions.find(c => c.id === companyId);
    companySearchText = selectedCompany ? selectedCompany.name : '';
    companyDropdownOpen = false;
    // Clear contact selection when company changes
    formData.contact_id = '';
    contactSearchText = '';
  }
  
  function clearCompany() {
    formData.company_id = '';
    companySearchText = '';
    companyDropdownOpen = false;
    // Clear contact selection when company is cleared
    formData.contact_id = '';
    contactSearchText = '';
  }
  
  function clearProject() {
    formData.project_id = '';
    projectSearchText = '';
    projectDropdownOpen = false;
    // Clear FP number when project is cleared
    formData.number = '';
  }
  
  function clearContact() {
    formData.contact_id = '';
    contactSearchText = '';
    contactDropdownOpen = false;
    // Note: We don't clear company when clearing contact, as user might want to keep company selection
    // The company dropdown will expand to show all companies again
  }
  
  
  function selectContact(contactId: string) {
    formData.contact_id = contactId;
    const selectedContact = allContactOptions.find(c => c.id === contactId);
    contactSearchText = selectedContact ? selectedContact.full_name : '';
    contactDropdownOpen = false;
    
    // Auto-select the contact's company if contact has a company
    if (selectedContact && selectedContact.company) {
      let contactCompanyId = '';
      if (typeof selectedContact.company === 'string') {
        contactCompanyId = selectedContact.company;
      } else if (selectedContact.company && typeof selectedContact.company === 'object') {
        const thingObj = selectedContact.company as any;
        if (thingObj.tb && thingObj.id) {
          if (typeof thingObj.id === 'string') {
            contactCompanyId = thingObj.id;
          } else if (thingObj.id.String) {
            contactCompanyId = thingObj.id.String;
          }
        }
      }
      
      if (contactCompanyId && contactCompanyId !== formData.company_id) {
        formData.company_id = contactCompanyId;
        const selectedCompany = allCompanyOptions.find(c => c.id === contactCompanyId);
        companySearchText = selectedCompany ? selectedCompany.name : '';
      }
    }
  }
  
  // Initialize search text when form data is populated (edit mode)
  $: if (formData.project_id && !projectSearchText) {
    const project = allProjectOptions.find(p => p.id === formData.project_id);
    if (project) {
      projectSearchText = `${project.number} - ${project.name}`;
    }
  }
  
  $: if (formData.company_id && !companySearchText) {
    const company = allCompanyOptions.find(c => c.id === formData.company_id);
    if (company) {
      companySearchText = company.name;
    }
  }
  
  $: if (formData.contact_id && !contactSearchText) {
    const contact = allContactOptions.find(c => c.id === formData.contact_id);
    if (contact) {
      contactSearchText = contact.full_name;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4"
    on:click={closeModal}
    on:keydown={(e) => e.key === 'Escape' && closeModal()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="rfp-modal-title"
    tabindex="-1"
  >
    <!-- Modal Content -->
    <div 
      class="bg-emittiv-darker border border-emittiv-dark rounded w-full max-h-[90vh] overflow-y-auto"
      style="padding: 16px; max-width: 500px;"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="presentation"
    >
      <!-- Header -->
      <div class="flex items-center justify-between" style="margin-bottom: 20px;">
        <h2 id="rfp-modal-title" class="font-semibold text-emittiv-white" style="font-size: 16px;">
          {mode === 'create' ? 'Add New FP' : 'Edit FP'}
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
        <!-- Basic Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Basic Information</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Name and Number Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="rfp_name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  FP Name *
                </label>
                <input
                  id="rfp_name"
                  type="text"
                  bind:value={formData.name}
                  placeholder="Hotel Development - Design Services"
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.name ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.name}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.name}</p>
                {/if}
              </div>
              
              <div>
                <label for="rfp_number" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  FP Number *
                </label>
                <input
                  id="rfp_number"
                  type="text"
                  bind:value={formData.number}
                  placeholder="25-97105-FP"
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.number ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.number}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.number}</p>
                {/if}
              </div>
            </div>
            
            <!-- Rev and Status Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="rfp_rev" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Revision
                </label>
                <input
                  id="rfp_rev"
                  type="number"
                  bind:value={formData.rev}
                  placeholder="1"
                  min="1"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
              
              <div>
                <label for="rfp_status" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Status
                </label>
                <select
                  id="rfp_status"
                  bind:value={formData.status}
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all appearance-none"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                >
                  {#each statusOptions as status}
                    <option value={status}>{status}</option>
                  {/each}
                </select>
              </div>
            </div>
          </div>
        </div>

        <!-- Project & Client Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Project & Client Information</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Project Selection -->
            <div>
              <label for="project_id" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Project *
              </label>
              <div class="flex relative" style="gap: 8px;">
                <div class="flex-1 relative">
                  <input
                    id="project_id"
                    type="text"
                    bind:value={projectSearchText}
                    on:input={() => { projectDropdownOpen = true; }}
                    on:focus={() => { projectDropdownOpen = true; }}
                    on:blur={() => { setTimeout(() => projectDropdownOpen = false, 200); }}
                    placeholder="Search projects..."
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.project_id ? 'border-red-500' : ''}"
                    style="padding: 8px 30px 8px 12px; font-size: 12px; height: 32px;"
                  />
                  {#if projectSearchText}
                    <button
                      type="button"
                      on:click={clearProject}
                      class="absolute right-1 top-1/2 transform -translate-y-1/2 text-emittiv-light hover:text-emittiv-white flex items-center justify-center"
                      style="width: 16px; height: 16px;"
                      aria-label="Clear project"
                    >
                      <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  {/if}
                  {#if projectDropdownOpen && projectOptions.length > 0}
                    <div class="absolute top-full left-0 right-0 bg-emittiv-darker border border-emittiv-dark rounded-b max-h-48 overflow-y-auto z-50" style="margin-top: 1px;">
                      {#each projectOptions as project}
                        <button
                          type="button"
                          on:click={() => selectProject(project.id)}
                          class="w-full text-left text-emittiv-white hover:bg-emittiv-dark text-xs border-b border-emittiv-dark last:border-b-0 truncate"
                          style="height: 24px; line-height: 22px; padding: 1px 8px;"
                        >
                          <span class="font-medium">{project.number}</span> - <span class="truncate">{project.name}</span>
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
                <button
                  type="button"
                  on:click={openNewProjectModal}
                  class="w-8 h-8 bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded flex items-center justify-center transition-all hover:scale-105 active:scale-95"
                  aria-label="Add new project"
                  title="Create new project"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                  </svg>
                </button>
              </div>
              {#if formErrors.project_id}
                <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.project_id}</p>
              {/if}
            </div>
            
            <!-- Company and Contact Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="company_id" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Company *
                </label>
                <div class="flex relative" style="gap: 8px;">
                  <div class="flex-1 relative">
                    <input
                      id="company_id"
                      type="text"
                      bind:value={companySearchText}
                      on:input={() => { companyDropdownOpen = true; }}
                      on:focus={() => { companyDropdownOpen = true; }}
                      on:blur={() => { setTimeout(() => companyDropdownOpen = false, 200); }}
                      placeholder="Search companies..."
                      class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.company_id ? 'border-red-500' : ''}"
                      style="padding: 8px 30px 8px 12px; font-size: 12px; height: 32px;"
                    />
                    {#if companySearchText}
                      <button
                        type="button"
                        on:click={clearCompany}
                        class="absolute right-1 top-1/2 transform -translate-y-1/2 text-emittiv-light hover:text-emittiv-white flex items-center justify-center"
                        style="width: 16px; height: 16px;"
                        aria-label="Clear company"
                      >
                        <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    {/if}
                    {#if companyDropdownOpen && companyOptions.length > 0}
                      <div class="absolute top-full left-0 right-0 bg-emittiv-darker border border-emittiv-dark rounded-b max-h-48 overflow-y-auto z-50" style="margin-top: 1px;">
                        {#each companyOptions as company}
                          <button
                            type="button"
                            on:click={() => selectCompany(company.id)}
                            class="w-full text-left text-emittiv-white hover:bg-emittiv-dark text-xs border-b border-emittiv-dark last:border-b-0 truncate"
                            style="height: 24px; line-height: 22px; padding: 1px 8px;"
                          >
                            <span class="font-medium truncate">{company.name}</span>
                            {#if company.name_short && company.name_short !== company.name}
                              <span class="text-emittiv-light"> ({company.name_short})</span>
                            {/if}
                          </button>
                        {/each}
                      </div>
                    {/if}
                  </div>
                  <button
                    type="button"
                    on:click={openCompanyModal}
                    class="w-8 h-8 bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded flex items-center justify-center transition-all hover:scale-105 active:scale-95"
                    aria-label="Add new company"
                    title="Create new company"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                    </svg>
                  </button>
                </div>
                {#if formErrors.company_id}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.company_id}</p>
                {/if}
              </div>
              
              <div>
                <label for="contact_id" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Contact *
                </label>
                <div class="flex relative" style="gap: 8px;">
                  <div class="flex-1 relative">
                    <input
                      id="contact_id"
                      type="text"
                      bind:value={contactSearchText}
                      on:input={() => { contactDropdownOpen = true; }}
                      on:focus={() => { contactDropdownOpen = true; }}
                      on:blur={() => { setTimeout(() => contactDropdownOpen = false, 200); }}
                      placeholder="Search contacts..."
                      class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.contact_id ? 'border-red-500' : ''}"
                      style="padding: 8px 30px 8px 12px; font-size: 12px; height: 32px;"
                    />
                    {#if contactSearchText}
                      <button
                        type="button"
                        on:click={clearContact}
                        class="absolute right-1 top-1/2 transform -translate-y-1/2 text-emittiv-light hover:text-emittiv-white flex items-center justify-center"
                        style="width: 16px; height: 16px;"
                        aria-label="Clear contact"
                      >
                        <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    {/if}
                    {#if contactDropdownOpen && filteredContactOptions.length > 0}
                      <div class="absolute top-full left-0 right-0 bg-emittiv-darker border border-emittiv-dark rounded-b max-h-48 overflow-y-auto z-50" style="margin-top: 1px;">
                        {#each filteredContactOptions as contact}
                          <button
                            type="button"
                            on:click={() => selectContact(contact.id)}
                            class="w-full text-left text-emittiv-white hover:bg-emittiv-dark text-xs border-b border-emittiv-dark last:border-b-0 truncate"
                            style="height: 24px; line-height: 22px; padding: 1px 8px;"
                          >
                            <span class="font-medium truncate">{contact.full_name}</span>
                            {#if contact.company}
                              {@const contactCompanyId = typeof contact.company === 'string' ? contact.company : (contact.company?.id?.String || contact.company?.id || '')}
                              {@const companyName = allCompanyOptions.find(c => c.id === contactCompanyId)?.name}
                              {#if companyName}
                                <span class="text-emittiv-light"> - {companyName}</span>
                              {/if}
                            {/if}
                          </button>
                        {/each}
                      </div>
                    {/if}
                  </div>
                  <button
                    type="button"
                    on:click={openContactModal}
                    class="w-8 h-8 bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded flex items-center justify-center transition-all hover:scale-105 active:scale-95"
                    aria-label="Add new contact"
                    title="Create new contact"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                    </svg>
                  </button>
                </div>
                {#if formErrors.contact_id}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.contact_id}</p>
                {/if}
              </div>
            </div>
          </div>
        </div>
        
        <!-- FP Details Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">FP Details</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Issue Date and Activity Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="issue_date" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Issue Date (YYMMDD)
                </label>
                <input
                  id="issue_date"
                  type="text"
                  bind:value={formData.issue_date}
                  placeholder="250715"
                  maxlength="6"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.issue_date ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.issue_date}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.issue_date}</p>
                {/if}
              </div>
              
              <div>
                <label for="activity" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Activity
                </label>
                <input
                  id="activity"
                  type="text"
                  bind:value={formData.activity}
                  placeholder="Architecture, Interior Design"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
            </div>
            
            <!-- Package and Strap Line -->
            <div>
              <label for="package" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Package
              </label>
              <input
                id="package"
                type="text"
                bind:value={formData.package}
                placeholder="Design Development Package"
                class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                style="padding: 8px 12px; font-size: 12px; height: 32px;"
              />
            </div>
            
            <div>
              <label for="strap_line" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Strap Line
              </label>
              <input
                id="strap_line"
                type="text"
                bind:value={formData.strap_line}
                placeholder="Luxury Hotel Design & Development"
                class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                style="padding: 8px 12px; font-size: 12px; height: 32px;"
              />
            </div>
          </div>
        </div>

        <!-- Staff Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Staff Information</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Staff Name and Position Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="staff_name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Staff Name
                </label>
                <input
                  id="staff_name"
                  type="text"
                  bind:value={formData.staff_name}
                  placeholder="Martin Smith"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
              
              <div>
                <label for="staff_position" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Staff Position
                </label>
                <input
                  id="staff_position"
                  type="text"
                  bind:value={formData.staff_position}
                  placeholder="Senior Architect"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
            </div>
            
            <!-- Staff Email and Phone Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="staff_email" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Staff Email
                </label>
                <input
                  id="staff_email"
                  type="email"
                  bind:value={formData.staff_email}
                  placeholder="martin@emittiv.com"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.staff_email ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.staff_email}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.staff_email}</p>
                {/if}
              </div>
              
              <div>
                <label for="staff_phone" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Staff Phone
                </label>
                <input
                  id="staff_phone"
                  type="tel"
                  bind:value={formData.staff_phone}
                  placeholder="+971 50 123 4567"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
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
            <p style="margin-bottom: 8px;">⚠️ Are you sure you want to delete this FP?</p>
            <p class="text-red-400" style="font-size: 10px; margin-bottom: 12px;">This action cannot be undone. All related data and documents will be affected.</p>
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
                  <span>Delete FP</span>
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
                <span>{mode === 'create' ? 'Create FP' : 'Update FP'}</span>
              {/if}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Nested Modals -->
<NewProjectModal 
  bind:isOpen={showNewProjectModal}
  onClose={() => showNewProjectModal = false}
  onSuccess={handleProjectCreated}
/>

<CompanyModal 
  bind:isOpen={showCompanyModal}
  mode="create"
  company={null}
  on:close={() => showCompanyModal = false}
  on:companyCreated={handleCompanyCreated}
/>

<ContactModal 
  bind:isOpen={showContactModal}
  mode="create"
  contact={null}
  on:close={() => showContactModal = false}
  on:contactCreated={handleContactCreated}
/>

<style>
  /* Custom select styling */
  select {
    background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3e%3cpath stroke='%23999' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3e%3c/svg%3e");
    background-position: right 0.5rem center;
    background-repeat: no-repeat;
    background-size: 16px 12px;
    padding-right: 2.5rem;
  }
</style>