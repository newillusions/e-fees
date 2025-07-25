<!--
  Refactored Contact Modal using BaseModal, FormInput, and FormSelect components
  Reduced from ~538 lines to ~300 lines using base components
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { contactsActions, companiesStore } from '$lib/stores';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import { extractSurrealId } from '$lib/utils/surrealdb';
  import { validateForm, CommonValidationRules, hasValidationErrors } from '$lib/utils/validation';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
  import BaseModal from './BaseModal.svelte';
  import FormInput from './FormInput.svelte';
  import FormSelect from './FormSelect.svelte';
  import Button from './Button.svelte';
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
  
  // Form validation state
  let formErrors: Record<string, string> = {};
  
  // UI state
  let showDeleteConfirm = false;
  let showCompanyModal = false;
  
  // Company lookup functionality
  const { companyOptions } = createCompanyLookup(companiesStore);
  
  // Computed values
  $: fullName = `${formData.first_name} ${formData.last_name}`.trim();
  
  // Form submission handler
  function handleSubmit(event: Event) {
    event.preventDefault();
    
    // Validate using the new validation system
    const errors = validateForm(formData, validationRules);
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
  
  // Create contact with loading state
  async function handleCreate() {
    await withLoadingState(async () => {
      const timestamp = new Date().toISOString();
      const contactData = {
        ...formData,
        full_name: fullName,
        time: {
          created_at: timestamp,
          updated_at: timestamp
        }
      };
      
      const result = await contactsActions.create(contactData);
      operationActions.setMessage('Contact created successfully');
      resetForm();
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Update contact with loading state  
  async function handleUpdate() {
    if (!contact) return;
    
    await withLoadingState(async () => {
      const contactId = extractSurrealId(contact);
      if (!contactId) throw new Error('Invalid contact ID');
      
      const contactData = {
        ...formData,
        full_name: fullName,
        time: {
          created_at: contact.time.created_at,
          updated_at: new Date().toISOString()
        }
      };
      
      const result = await contactsActions.update(contactId, contactData);
      operationActions.setMessage('Contact updated successfully');
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Delete contact with loading state
  async function handleDelete() {
    if (!contact || !showDeleteConfirm) return;
    
    await withLoadingState(async () => {
      const contactId = extractSurrealId(contact);
      if (!contactId) throw new Error('Invalid contact ID');
      
      const result = await contactsActions.delete(contactId);
      operationActions.setMessage('Contact deleted successfully');
      closeModal();
      return result;
    }, operationActions, 'deleting');
  }
  
  // Form management
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
    showDeleteConfirm = false;
  }
  
  function closeModal() {
    resetForm();
    operationActions.reset();
    dispatch('close');
  }
  
  // Company modal handlers
  function handleAddCompany() {
    showCompanyModal = true;
  }
  
  function handleCompanyModalClose() {
    showCompanyModal = false;
  }
  
  // Load form data when contact changes
  $: if (contact && mode === 'edit') {
    formData = {
      first_name: contact.first_name || '',
      last_name: contact.last_name || '',
      email: contact.email || '',
      phone: contact.phone || '',
      position: contact.position || '',
      company: extractSurrealId(contact.company) || ''
    };
  }
</script>

<BaseModal 
  {isOpen} 
  title={mode === 'create' ? 'New Contact' : 'Edit Contact'}
  maxWidth="450px"
  on:close={closeModal}
>
  <!-- Form -->
  <form on:submit={handleSubmit} style="display: flex; flex-direction: column; gap: 16px;">
    
    <!-- CONTACT INFORMATION SECTION -->
    <div>
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        Contact Information
      </h3>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        
        <!-- Name Fields -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="First Name"
            bind:value={formData.first_name}
            placeholder="John"
            required
            error={formErrors.first_name}
          />
          
          <FormInput
            label="Last Name"
            bind:value={formData.last_name}
            placeholder="Doe"
            required
            error={formErrors.last_name}
          />
        </div>
        
        <!-- Full Name Display -->
        {#if fullName}
          <div>
            <label class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
              Full Name (Auto-generated)
            </label>
            <div class="w-full bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-light flex items-center" style="padding: 8px 12px; font-size: 12px; height: 32px; opacity: 0.6;">
              {fullName}
            </div>
          </div>
        {/if}
        
        <!-- Contact Details -->
        <FormInput
          label="Email"
          type="email"
          bind:value={formData.email}
          placeholder="john.doe@company.com"
          required
          error={formErrors.email}
        />
        
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Phone"
            type="tel"
            bind:value={formData.phone}
            placeholder="+971 50 123 4567"
            error={formErrors.phone}
          />
          
          <FormInput
            label="Position"
            bind:value={formData.position}
            placeholder="Manager"
            error={formErrors.position}
          />
        </div>
        
        <!-- Company Selection -->
        <div>
          <label class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
            Company *
          </label>
          <div class="flex" style="gap: 8px;">
            <FormSelect
              bind:value={formData.company}
              placeholder="Select company"
              required
              error={formErrors.company}
              options={$companyOptions}
            />
            
            <button
              type="button"
              on:click={handleAddCompany}
              class="bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded transition-colors flex items-center justify-center"
              style="padding: 6px; width: 32px; height: 32px;"
              title="Add new company"
              aria-label="Add new company"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
            </button>
          </div>
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
        <p class="font-medium mb-2">Are you sure you want to delete this contact?</p>
        <p class="text-xs opacity-80">This action cannot be undone.</p>
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
              disabled={$operationState.saving || $operationState.deleting}
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
          disabled={$operationState.saving || $operationState.deleting || showDeleteConfirm}
        >
          {#if $operationState.saving}
            <div 
              class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin"
              style="width: 14px; height: 14px;"
            ></div>
          {/if}
          {mode === 'create' ? 'Create Contact' : 'Update Contact'}
        </button>
      </div>
    </div>
  </form>
</BaseModal>

<!-- Company Modal -->
<CompanyModal
  isOpen={showCompanyModal}
  mode="create"
  on:close={handleCompanyModalClose}
/>