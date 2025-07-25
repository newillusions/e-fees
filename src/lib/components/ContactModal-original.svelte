<!--
  Contact Modal using new utility functions for reduced code duplication
  Refactored to use standardized validation, CRUD operations, and SurrealDB utilities
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { contactsActions, companiesStore } from '$lib/stores';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import { extractSurrealId } from '$lib/utils/surrealdb';
  import { validateForm, CommonValidationRules, hasValidationErrors } from '$lib/utils/validation';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
  import CompanyModal from './CompanyModal.svelte';
  import type { Contact, Company } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let contact: Contact | null = null;
  export let mode: 'create' | 'edit' = 'create';
  
  // Use the new operation state utility
  const { store: operationState, actions: operationActions } = useOperationState();
  
  // Form data with better typing
  interface ContactFormData {
    first_name: string;
    last_name: string;
    email: string;
    phone: string;
    position: string;
    company: string;
  }
  
  let formData: ContactFormData = {
    first_name: '',
    last_name: '',
    email: '',
    phone: '',
    position: '',
    company: ''
  };
  
  // Validation setup using the new validation system
  const validationRules = [
    CommonValidationRules.contact.firstName,
    CommonValidationRules.contact.lastName,
    CommonValidationRules.contact.email,
    { field: 'company' as keyof ContactFormData, required: true, minLength: 1 },
  ];
  
  let formErrors: Record<string, string> = {};
  let showDeleteConfirm = false;
  
  // Company modal state
  let isCompanyModalOpen = false;
  
  // Company lookup using extracted utility
  $: companyLookup = createCompanyLookup($companiesStore);
  $: companyOptions = $companiesStore.map(company => ({
    id: extractSurrealId(company.id) || '',
    name: company.name,
    name_short: company.name_short
  }));
  
  // Update form when contact prop changes
  $: if (contact && mode === 'edit') {
    formData = {
      first_name: contact.first_name || '',
      last_name: contact.last_name || '',
      email: contact.email || '',
      phone: contact.phone || '',
      position: contact.position || '',
      company: extractSurrealId(contact.company) || ''
    };
  } else if (mode === 'create') {
    resetForm();
  }
  
  // Reset state when modal closes
  $: if (!isOpen) {
    resetForm();
  }
  
  
  function resetForm() {
    formData = {
      first_name: '',
      last_name: '',
      email: '',
      phone: '',
      position: '',
      company: ''
    };
    formErrors = {};
    operationActions.reset();
    showDeleteConfirm = false;
  }

  // Computed full name for display
  $: fullName = `${formData.first_name} ${formData.last_name}`.trim();
  
  // Validation using the new system
  function validateFormData(): boolean {
    formErrors = validateForm(formData, validationRules);
    
    // Additional phone validation - must contain '+' as per database schema
    if (formData.phone && formData.phone.trim() && !formData.phone.includes('+')) {
      formErrors.phone = 'Phone number must include country code with + (e.g., +971 50 123 4567)';
    }
    
    return !hasValidationErrors(formErrors);
  }
  
  async function handleSubmit(event: Event) {
    event.preventDefault();
    
    if (!validateFormData()) {
      return;
    }
    
    try {
      if (mode === 'create') {
        // Add required full_name and time for database
        const contactData = {
          ...formData,
          full_name: fullName,
          time: {
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString()
          }
        };
        const newContact = await withLoadingState(
          () => contactsActions.create(contactData),
          operationActions,
          'saving'
        );
        operationActions.setMessage('Contact created successfully!');
        dispatch('contactCreated', newContact);
      } else {
        const contactId = extractSurrealId(contact?.id);
        if (contactId) {
          const updateData = {
            ...formData,
            full_name: fullName
          };
          await withLoadingState(
            () => contactsActions.update(contactId, updateData),
            operationActions,
            'saving'
          );
          operationActions.setMessage('Contact updated successfully!');
        } else {
          throw new Error('No valid contact ID found for update');
        }
      }
      
      // Auto-close after success
      setTimeout(() => {
        closeModal();
      }, 1500);
      
    } catch (error: any) {
      operationActions.setError(error?.message || 'Failed to save contact');
    }
  }
  
  async function handleDelete() {
    const contactId = extractSurrealId(contact?.id);
    if (!contactId) {
      operationActions.setError('No valid contact ID found for delete');
      return;
    }
    
    try {
      await withLoadingState(
        () => contactsActions.delete(contactId),
        operationActions,
        'deleting'
      );
      operationActions.setMessage('Contact deleted successfully!');
      
      setTimeout(() => {
        closeModal();
      }, 1500);
      
    } catch (error: any) {
      operationActions.setError(error?.message || 'Failed to delete contact');
    }
  }
  
  function confirmDelete() {
    showDeleteConfirm = true;
  }
  
  function cancelDelete() {
    showDeleteConfirm = false;
  }
  
  function closeModal() {
    dispatch('close');
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeModal();
    }
  }
  
  function handleAddCompany() {
    isCompanyModalOpen = true;
  }
  
  function handleCompanyModalClose() {
    isCompanyModalOpen = false;
  }
  
  // Listen for company creation events
  function handleCompanyCreated(event: CustomEvent) {
    const newCompany = event.detail;
    if (newCompany && newCompany.id) {
      // Extract company ID using the same logic as getCompanyId helper
      let companyId = null;
      
      if (typeof newCompany.id === 'string') {
        companyId = newCompany.id;
      } else if (newCompany.id && typeof newCompany.id === 'object') {
        const thingObj = newCompany.id as any;
        if (thingObj.tb && thingObj.id) {
          if (typeof thingObj.id === 'string') {
            companyId = thingObj.id;
          } else if (thingObj.id.String) {
            companyId = thingObj.id.String;
          }
        }
      }
      
      if (companyId) {
        formData.company = companyId;
      }
    }
    handleCompanyModalClose();
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
    aria-labelledby="contact-modal-title"
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
        <h2 id="contact-modal-title" class="font-semibold text-emittiv-white" style="font-size: 16px;">
          {mode === 'create' ? 'Add New Contact' : 'Edit Contact'}
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

      <form on:submit={handleSubmit} style="display: flex; flex-direction: column; gap: 24px;">
        <!-- Contact Basic Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Contact Information</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Full Name (Auto-generated) -->
            <div>
              <label class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Full Name (Auto-generated)
              </label>
              <div class="w-full bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-light flex items-center" style="padding: 8px 12px; font-size: 12px; height: 32px; opacity: 0.6;">
                <span class="{fullName ? '' : 'text-emittiv-light'}">
                  {fullName || 'Will be generated from first + last name'}
                </span>
              </div>
              <p class="text-emittiv-light" style="font-size: 10px; margin-top: 2px;">This field is automatically generated from first and last name</p>
            </div>
            
            <!-- First and Last Name Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="first_name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  First Name *
                </label>
                <input
                  id="first_name"
                  type="text"
                  bind:value={formData.first_name}
                  placeholder="Ryan"
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.first_name ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.first_name}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.first_name}</p>
                {/if}
              </div>
              
              <div>
                <label for="last_name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Last Name *
                </label>
                <input
                  id="last_name"
                  type="text"
                  bind:value={formData.last_name}
                  placeholder="Marginson"
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.last_name ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.last_name}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.last_name}</p>
                {/if}
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
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.phone ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.phone}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.phone}</p>
                {/if}
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
        <!-- Success/Error Messages -->
        {#if $operationState.message}
          <div class="rounded-lg bg-green-900/20 border border-green-500/30 text-green-300" style="padding: 12px; font-size: 12px;">
            ✓ {$operationState.message}
          </div>
        {/if}

        {#if $operationState.error}
          <div class="rounded-lg bg-red-900/20 border border-red-500/30 text-red-300" style="padding: 12px; font-size: 12px;">
            ✗ {$operationState.error}
          </div>
        {/if}

        <!-- Delete Confirmation -->
        {#if showDeleteConfirm}
          <div class="rounded-lg bg-red-900/20 border border-red-500/30 text-red-300" style="padding: 12px; font-size: 12px;">
            <p style="margin-bottom: 8px;">⚠️ Are you sure you want to delete this contact?</p>
            <p class="text-red-400" style="font-size: 10px; margin-bottom: 12px;">This action cannot be undone. All related contacts and proposals will be affected.</p>
            <div class="flex" style="gap: 8px;">
              <button
                type="button"
                on:click={cancelDelete}
                class="border border-red-500/30 rounded text-red-300 hover:text-red-200 hover:border-red-400 transition-all"
                style="padding: 4px 8px; font-size: 11px; height: 24px;"
                disabled={$operationState.deleting}
              >
                Cancel
              </button>
              <button
                type="button"
                on:click={handleDelete}
                class="bg-red-600 hover:bg-red-700 text-white rounded transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
                style="padding: 4px 8px; font-size: 11px; height: 24px; gap: 4px;"
                disabled={$operationState.deleting}
              >
                {#if $operationState.deleting}
                  <div class="border-2 border-white border-t-transparent rounded-full animate-spin" style="width: 10px; height: 10px;"></div>
                  <span>Deleting...</span>
                {:else}
                  <span>Delete Contact</span>
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
              disabled={$operationState.saving || $operationState.deleting || showDeleteConfirm}
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
              disabled={$operationState.saving || $operationState.deleting}
            >
              Cancel
            </button>
            <button
              type="submit"
              class="bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
              style="padding: 6px 12px; font-size: 12px; height: 28px; gap: 4px;"
              disabled={$operationState.saving || $operationState.deleting || showDeleteConfirm}
            >
              {#if $operationState.saving}
                <div class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin" style="width: 12px; height: 12px;"></div>
                <span>Saving...</span>
              {:else}
                <span>{mode === 'create' ? 'Create Contact' : 'Update Contact'}</span>
              {/if}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}


<!-- Company Modal -->
<CompanyModal 
  bind:isOpen={isCompanyModalOpen}
  mode="create"
  on:close={handleCompanyModalClose}
  on:companyCreated={handleCompanyCreated}
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