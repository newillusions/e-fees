<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { contactsActions, companiesStore } from '$lib/stores';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import CompanyModal from './CompanyModal.svelte';
  import type { Contact, Company } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let contact: Contact | null = null; // null for create, contact object for edit
  export let mode: 'create' | 'edit' = 'create';
  
  // Form data
  let formData = {
    full_name: '',
    first_name: '',
    last_name: '',
    email: '',
    phone: '',
    position: '',
    company: ''
  };
  
  // Loading and error states
  let isSaving = false;
  let isDeleting = false;
  let saveMessage = '';
  let formErrors: Record<string, string> = {};
  let showDeleteConfirm = false;
  
  // Company modal state
  let isCompanyModalOpen = false;
  
  // Company lookup
  $: companyLookup = createCompanyLookup($companiesStore);
  $: companyOptions = $companiesStore.map(company => ({
    id: company.id,
    name: company.name,
    name_short: company.name_short
  }));
  
  // Update form when contact prop changes
  $: if (contact && mode === 'edit') {
    formData = {
      full_name: contact.full_name || '',
      first_name: contact.first_name || '',
      last_name: contact.last_name || '',
      email: contact.email || '',
      phone: contact.phone || '',
      position: contact.position || '',
      company: contact.company || ''
    };
  } else if (mode === 'create') {
    formData = {
      full_name: '',
      first_name: '',
      last_name: '',
      email: '',
      phone: '',
      position: '',
      company: ''
    };
  }
  
  // Auto-generate full name from first and last name
  function updateFullName() {
    if (formData.first_name || formData.last_name) {
      formData.full_name = `${formData.first_name} ${formData.last_name}`.trim();
    }
  }
  
  function validateForm(): boolean {
    formErrors = {};
    
    if (!formData.full_name.trim()) {
      formErrors.full_name = 'Full name is required';
    }
    
    if (!formData.email.trim()) {
      formErrors.email = 'Email is required';
    } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email)) {
      formErrors.email = 'Please enter a valid email address';
    }
    
    return Object.keys(formErrors).length === 0;
  }
  
  async function handleSave() {
    if (!validateForm()) return;
    
    isSaving = true;
    saveMessage = '';
    
    try {
      if (mode === 'create') {
        await contactsActions.create(formData);
        saveMessage = 'Contact created successfully!';
      } else if (contact) {
        await contactsActions.update(contact.id, formData);
        saveMessage = 'Contact updated successfully!';
      }
      
      setTimeout(() => {
        handleClose();
      }, 1000);
      
    } catch (error) {
      console.error('Failed to save contact:', error);
      saveMessage = `Failed to ${mode} contact. Please try again.`;
    } finally {
      isSaving = false;
    }
  }
  
  async function handleDelete() {
    if (!contact) return;
    
    isDeleting = true;
    
    try {
      await contactsActions.delete(contact.id);
      showDeleteConfirm = false;
      handleClose();
    } catch (error) {
      console.error('Failed to delete contact:', error);
      saveMessage = 'Failed to delete contact. Please try again.';
    } finally {
      isDeleting = false;
    }
  }
  
  function handleClose() {
    isOpen = false;
    setTimeout(() => {
      formData = {
        full_name: '',
        first_name: '',
        last_name: '',
        email: '',
        phone: '',
        position: '',
        company: ''
      };
      formErrors = {};
      saveMessage = '';
      showDeleteConfirm = false;
    }, 200);
    dispatch('close');
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
  }
  
  function handleAddCompany() {
    isCompanyModalOpen = true;
  }
  
  function handleCompanyModalClose() {
    isCompanyModalOpen = false;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div 
    class="fixed inset-0 z-50 flex items-center justify-center"
    style="background-color: rgba(0, 0, 0, 0.85);"
    on:click={handleClose}
    on:keydown={handleKeydown}
    role="dialog"
    aria-modal="true"
    aria-labelledby="contact-modal-title"
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
        <h2 id="contact-modal-title" class="font-semibold text-emittiv-white" style="font-size: 16px;">
          {mode === 'create' ? 'Add New Contact' : 'Edit Contact'}
        </h2>
        <button 
          on:click={handleClose}
          class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth"
          aria-label="Close modal"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <form on:submit|preventDefault={handleSave} style="display: flex; flex-direction: column; gap: 24px;">
        <!-- Contact Basic Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Contact Information</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Full Name -->
            <div>
              <label for="full_name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Full Name *
              </label>
              <input
                id="full_name"
                type="text"
                bind:value={formData.full_name}
                placeholder="Ryan Marginson"
                required
                class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.full_name ? 'border-red-500' : ''}"
                style="padding: 8px 12px; font-size: 12px; height: 32px;"
              />
              {#if formErrors.full_name}
                <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.full_name}</p>
              {/if}
            </div>
            
            <!-- First and Last Name Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="first_name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  First Name
                </label>
                <input
                  id="first_name"
                  type="text"
                  bind:value={formData.first_name}
                  on:blur={updateFullName}
                  placeholder="Ryan"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
              
              <div>
                <label for="last_name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Last Name
                </label>
                <input
                  id="last_name"
                  type="text"
                  bind:value={formData.last_name}
                  on:blur={updateFullName}
                  placeholder="Marginson"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
            </div>
            
            <!-- Email -->
            <div>
              <label for="email" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Email Address *
              </label>
              <input
                id="email"
                type="email"
                bind:value={formData.email}
                placeholder="ryan.marginson@example.com"
                required
                class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.email ? 'border-red-500' : ''}"
                style="padding: 8px 12px; font-size: 12px; height: 32px;"
              />
              {#if formErrors.email}
                <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.email}</p>
              {/if}
            </div>
          </div>
        </div>

        <!-- Professional Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Professional Details</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Position and Phone Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="position" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Position
                </label>
                <input
                  id="position"
                  type="text"
                  bind:value={formData.position}
                  placeholder="Senior Manager"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
              
              <div>
                <label for="phone" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Phone Number
                </label>
                <input
                  id="phone"
                  type="tel"
                  bind:value={formData.phone}
                  placeholder="+971 50 123 4567"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
            </div>
            
            <!-- Company -->
            <div>
              <label for="company" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Company
              </label>
              <div class="flex" style="gap: 8px;">
                <select
                  id="company"
                  bind:value={formData.company}
                  class="flex-1 bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all appearance-none"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                >
                  <option value="">Select a company</option>
                  {#each companyOptions as company}
                    <option value={company.id}>{company.name}</option>
                  {/each}
                </select>
                <button
                  type="button"
                  on:click={handleAddCompany}
                  class="bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded transition-colors flex items-center justify-center"
                  style="padding: 6px; width: 32px; height: 32px;"
                  title="Add new company"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>
        
        <!-- Save Message -->
        {#if saveMessage}
          <div class="text-center">
            <p class="text-emittiv-white" style="font-size: 12px;" class:text-green-400={saveMessage.includes('successfully')} class:text-red-400={saveMessage.includes('Failed')}>
              {saveMessage}
            </p>
          </div>
        {/if}
        
        <!-- Action Buttons -->
        <div class="flex justify-between" style="gap: 12px;">
          {#if mode === 'edit'}
            <button
              type="button"
              on:click={() => showDeleteConfirm = true}
              disabled={isSaving || isDeleting}
              class="bg-red-600 hover:bg-red-700 disabled:bg-red-800 text-white rounded transition-colors disabled:cursor-not-allowed flex items-center justify-center"
              style="padding: 8px 16px; font-size: 12px; font-weight: 500; height: 32px;"
            >
              Delete Contact
            </button>
          {/if}
          
          <div class="flex" style="gap: 12px; margin-left: auto;">
            <button
              type="button"
              on:click={handleClose}
              disabled={isSaving || isDeleting}
              class="border border-emittiv-dark text-emittiv-light hover:text-emittiv-white hover:border-emittiv-light rounded transition-colors disabled:cursor-not-allowed"
              style="padding: 8px 16px; font-size: 12px; height: 32px;"
            >
              Cancel
            </button>
            
            <button
              type="submit"
              disabled={isSaving || isDeleting}
              class="bg-emittiv-splash hover:bg-orange-600 disabled:bg-orange-800 text-emittiv-black rounded transition-colors disabled:cursor-not-allowed font-medium flex items-center justify-center"
              style="padding: 8px 16px; font-size: 12px; height: 32px; gap: 4px;"
            >
              {#if isSaving}
                <svg class="w-3 h-3 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Saving...
              {:else}
                {mode === 'create' ? 'Add Contact' : 'Save Changes'}
              {/if}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Delete Confirmation Modal -->
{#if showDeleteConfirm}
  <div class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-60 p-4">
    <div class="bg-emittiv-black rounded-2xl p-6 w-full max-w-sm border border-emittiv-dark">
      <h3 class="text-lg font-semibold text-emittiv-white mb-3">Delete Contact</h3>
      <p class="text-emittiv-light mb-6">
        Are you sure you want to delete <strong class="text-emittiv-white">{contact?.full_name}</strong>? This action cannot be undone.
      </p>
      
      <div class="flex gap-3">
        <button
          on:click={() => showDeleteConfirm = false}
          disabled={isDeleting}
          class="flex-1 px-4 py-2 border border-emittiv-dark text-emittiv-light hover:text-emittiv-white hover:border-emittiv-light rounded-lg transition-colors disabled:cursor-not-allowed"
        >
          Cancel
        </button>
        <button
          on:click={handleDelete}
          disabled={isDeleting}
          class="flex-1 px-4 py-2 bg-red-600 hover:bg-red-700 disabled:bg-red-800 text-white rounded-lg transition-colors disabled:cursor-not-allowed font-medium flex items-center justify-center gap-2"
        >
          {#if isDeleting}
            <svg class="w-4 h-4 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            Deleting...
          {:else}
            Delete Contact
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Company Modal -->
<CompanyModal 
  bind:isOpen={isCompanyModalOpen}
  mode="create"
  on:close={handleCompanyModalClose}
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