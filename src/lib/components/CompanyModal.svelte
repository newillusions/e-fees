<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { companiesActions } from '$lib/stores';
  import type { Company } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let company: Company | null = null; // null for create, company object for edit
  export let mode: 'create' | 'edit' = 'create';
  
  // Form data
  let formData = {
    name: '',
    name_short: '',
    abbreviation: '',
    city: '',
    country: '',
    reg_no: '',
    tax_no: ''
  };
  
  // Loading and error states
  let isSaving = false;
  let isDeleting = false;
  let saveMessage = '';
  let formErrors: Record<string, string> = {};
  let showDeleteConfirm = false;
  
  // Update form when company prop changes
  $: if (company && mode === 'edit') {
    formData = {
      name: company.name || '',
      name_short: company.name_short || '',
      abbreviation: company.abbreviation || '',
      city: company.city || '',
      country: company.country || '',
      reg_no: company.reg_no || '',
      tax_no: company.tax_no || ''
    };
  } else if (mode === 'create') {
    // Reset form for create mode
    formData = {
      name: '',
      name_short: '',
      abbreviation: '',
      city: '',
      country: '',
      reg_no: '',
      tax_no: ''
    };
  }
  
  // Reset state when modal closes
  $: if (!isOpen) {
    resetForm();
  }
  
  function resetForm() {
    formData = {
      name: '',
      name_short: '',
      abbreviation: '',
      city: '',
      country: '',
      reg_no: '',
      tax_no: ''
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
    if (!formData.name.trim()) {
      formErrors.name = 'Company name is required';
    }
    
    if (!formData.name_short.trim()) {
      formErrors.name_short = 'Short name is required';
    }
    
    if (!formData.abbreviation.trim()) {
      formErrors.abbreviation = 'Abbreviation is required';
    } else if (formData.abbreviation.length > 10) {
      formErrors.abbreviation = 'Abbreviation must be 10 characters or less';
    }
    
    if (!formData.city.trim()) {
      formErrors.city = 'City is required';
    }
    
    if (!formData.country.trim()) {
      formErrors.country = 'Country is required';
    }
    
    // Optional fields validation
    if (formData.name.length > 100) {
      formErrors.name = 'Company name must be 100 characters or less';
    }
    
    if (formData.name_short.length > 50) {
      formErrors.name_short = 'Short name must be 50 characters or less';
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
        // Add required time field for new companies
        const companyData = {
          ...formData,
          time: {
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString()
          }
        };
        await companiesActions.create(companyData);
        saveMessage = 'Company created successfully!';
      } else {
        const companyId = getCompanyId(company);
        if (companyId) {
          console.log('Updating company with ID:', companyId);
          console.log('Update data:', formData);
          await companiesActions.update(companyId, formData);
          saveMessage = 'Company updated successfully!';
        } else {
          throw new Error('No valid company ID found for update');
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
    const companyId = getCompanyId(company);
    if (!companyId) return;
    
    isDeleting = true;
    saveMessage = '';
    
    try {
      console.log('Deleting company with ID:', companyId);
      await companiesActions.delete(companyId);
      saveMessage = 'Company deleted successfully!';
      
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
  
  // Auto-generate abbreviation from company name
  function generateAbbreviation() {
    if (formData.name && !formData.abbreviation) {
      // Take first letter of each word, max 6 characters
      const words = formData.name.trim().split(/\s+/);
      const abbrev = words
        .map(word => word.charAt(0).toUpperCase())
        .join('')
        .substring(0, 6);
      formData.abbreviation = abbrev;
    }
  }
  
  // Auto-generate short name from company name
  function generateShortName() {
    if (formData.name && !formData.name_short) {
      // Take first 30 characters or first two words
      const words = formData.name.trim().split(/\s+/);
      if (words.length >= 2) {
        formData.name_short = words.slice(0, 2).join(' ');
      } else {
        formData.name_short = formData.name.substring(0, 30);
      }
    }
  }
  
  // Helper function to extract ID from SurrealDB Thing object
  function getCompanyId(company: Company | null): string | null {
    if (!company?.id) return null;
    
    if (typeof company.id === 'string') {
      return company.id;
    }
    
    // Handle SurrealDB Thing object format
    if (company.id && typeof company.id === 'object') {
      const thingObj = company.id as any;
      if (thingObj.tb && thingObj.id) {
        if (typeof thingObj.id === 'string') {
          return thingObj.id; // Return just the ID part, not "company:ID"
        } else if (thingObj.id.String) {
          return thingObj.id.String;
        }
      }
    }
    
    return null;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Modal Backdrop -->
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4"
    on:click={closeModal}
    role="dialog"
    aria-modal="true"
    aria-labelledby="company-modal-title"
    tabindex="-1"
  >
    <!-- Modal Content -->
    <div 
      class="bg-emittiv-darker border border-emittiv-dark rounded w-full max-h-[90vh] overflow-y-auto"
      style="padding: 24px; max-width: 700px;"
      on:click|stopPropagation
    >
      <!-- Header -->
      <div class="flex items-center justify-between" style="margin-bottom: 20px;">
        <h2 id="company-modal-title" class="font-semibold text-emittiv-white" style="font-size: 16px;">
          {mode === 'create' ? 'Add New Company' : 'Edit Company'}
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
        
        <!-- Company Basic Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Company Information</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Company Name -->
            <div>
              <label for="company_name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Company Name *
              </label>
              <input
                id="company_name"
                type="text"
                bind:value={formData.name}
                on:blur={generateShortName}
                on:blur={generateAbbreviation}
                placeholder="Meraas Holding LLC"
                required
                class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.name ? 'border-red-500' : ''}"
                style="padding: 8px 12px; font-size: 12px; height: 32px;"
              />
              {#if formErrors.name}
                <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.name}</p>
              {/if}
            </div>
            
            <!-- Short Name and Abbreviation Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="company_short" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Short Name *
                </label>
                <input
                  id="company_short"
                  type="text"
                  bind:value={formData.name_short}
                  placeholder="Meraas Holding"
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.name_short ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.name_short}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.name_short}</p>
                {/if}
              </div>
              
              <div>
                <label for="company_abbrev" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Abbreviation *
                </label>
                <input
                  id="company_abbrev"
                  type="text"
                  bind:value={formData.abbreviation}
                  placeholder="MERAAS"
                  required
                  maxlength="10"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.abbreviation ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.abbreviation}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.abbreviation}</p>
                {/if}
              </div>
            </div>
          </div>
        </div>

        <!-- Location Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Location</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- City and Country Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="company_city" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  City *
                </label>
                <input
                  id="company_city"
                  type="text"
                  bind:value={formData.city}
                  placeholder="Dubai"
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.city ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.city}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.city}</p>
                {/if}
              </div>
              
              <div>
                <label for="company_country" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Country *
                </label>
                <input
                  id="company_country"
                  type="text"
                  bind:value={formData.country}
                  placeholder="U.A.E."
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.country ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.country}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.country}</p>
                {/if}
              </div>
            </div>
          </div>
        </div>

        <!-- Optional Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 6px;">Registration Details</h3>
          <p class="text-emittiv-light" style="font-size: 11px; margin-bottom: 12px;">Optional registration and tax information</p>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Registration and Tax Numbers Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="company_reg" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Registration Number
                </label>
                <input
                  id="company_reg"
                  type="text"
                  bind:value={formData.reg_no}
                  placeholder="123456789"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
              
              <div>
                <label for="company_tax" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Tax Number
                </label>
                <input
                  id="company_tax"
                  type="text"
                  bind:value={formData.tax_no}
                  placeholder="100123456789003"
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
            <p style="margin-bottom: 8px;">⚠️ Are you sure you want to delete this company?</p>
            <p class="text-red-400" style="font-size: 10px; margin-bottom: 12px;">This action cannot be undone. All related contacts and proposals will be affected.</p>
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
                  <span>Delete Company</span>
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
                <span>{mode === 'create' ? 'Create Company' : 'Update Company'}</span>
              {/if}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}