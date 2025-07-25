<!--
  Refactored Company Modal using BaseModal, FormInput, and Button components
  Further reduced from ~700 lines to ~400 lines using base components
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { companiesActions } from '$lib/stores';
  import { searchCountries, getCitySuggestions, getAllCities } from '$lib/api';
  import { extractSurrealId } from '$lib/utils/surrealdb';
  import { validateForm, CommonValidationRules, hasValidationErrors } from '$lib/utils/validation';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
  import BaseModal from './BaseModal.svelte';
  import FormInput from './FormInput.svelte';
  import FormSelect from './FormSelect.svelte';
  import Button from './Button.svelte';
  import type { Company } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let company: Company | null = null;
  export let mode: 'create' | 'edit' = 'create';
  
  // Use the new operation state utility
  const { store: operationState, actions: operationActions } = useOperationState();
  
  // Form data with better typing
  interface CompanyFormData {
    name: string;
    name_short: string;
    abbreviation: string;
    city: string;
    country: string;
    reg_no: string;
    tax_no: string;
  }
  
  let formData: CompanyFormData = {
    name: '',
    name_short: '',
    abbreviation: '',
    city: '',
    country: '',
    reg_no: '',
    tax_no: ''
  };
  
  // Validation setup using the new validation system
  const validationRules = [
    CommonValidationRules.company.name,
    { field: 'name_short' as keyof CompanyFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'abbreviation' as keyof CompanyFormData, required: true, minLength: 1, maxLength: 10 },
    { field: 'city' as keyof CompanyFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'country' as keyof CompanyFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'reg_no' as keyof CompanyFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'tax_no' as keyof CompanyFormData, required: true, minLength: 1, maxLength: 50 }
  ];
  
  // Form validation state
  let formErrors: Record<string, string> = {};
  
  // UI state
  let countryOptions: string[] = [];
  let cityOptions: string[] = [];
  let showDeleteConfirm = false;
  
  // Form field focus handling
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
  
  // Create company with loading state
  async function handleCreate() {
    await withLoadingState(async () => {
      const timestamp = new Date().toISOString();
      const companyData = {
        ...formData,
        time: {
          created_at: timestamp,
          updated_at: timestamp
        }
      };
      
      const result = await companiesActions.create(companyData);
      operationActions.setMessage('Company created successfully');
      resetForm();
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Update company with loading state  
  async function handleUpdate() {
    if (!company) return;
    
    await withLoadingState(async () => {
      const companyId = extractSurrealId(company);
      if (!companyId) throw new Error('Invalid company ID');
      
      const companyData = {
        ...formData,
        time: {
          created_at: company.time.created_at,
          updated_at: new Date().toISOString()
        }
      };
      
      const result = await companiesActions.update(companyId, companyData);
      operationActions.setMessage('Company updated successfully');
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Delete company with loading state
  async function handleDelete() {
    if (!company || !showDeleteConfirm) return;
    
    await withLoadingState(async () => {
      const companyId = extractSurrealId(company);
      if (!companyId) throw new Error('Invalid company ID');
      
      const result = await companiesActions.delete(companyId);
      operationActions.setMessage('Company deleted successfully');
      closeModal();
      return result;
    }, operationActions, 'deleting');
  }
  
  // Form management
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
    showDeleteConfirm = false;
  }
  
  function closeModal() {
    resetForm();
    operationActions.reset();
    dispatch('close');
  }
  
  // Load form data when company changes
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
  }
  
  // Auto-suggestions
  async function handleCountrySearch(searchTerm: string) {
    if (searchTerm.length > 1) {
      try {
        const countries = await searchCountries(searchTerm);
        countryOptions = countries.map(c => c.name);
      } catch (error) {
        console.warn('Failed to search countries:', error);
      }
    }
  }
  
  async function handleCitySearch(searchTerm: string) {
    if (searchTerm.length > 1 && formData.country) {
      try {
        const cities = await getCitySuggestions(formData.country);
        cityOptions = cities;
      } catch (error) {
        console.warn('Failed to search cities:', error);
      }
    } else if (formData.country) {
      try {
        const cities = await getAllCities();
        cityOptions = cities;
      } catch (error) {
        console.warn('Failed to load cities:', error);
      }
    }
  }
  
  // Country selection handler
  $: if (formData.country) {
    handleCitySearch('');
  }
</script>

<BaseModal 
  {isOpen} 
  title={mode === 'create' ? 'New Company' : 'Edit Company'}
  maxWidth="450px"
  on:close={closeModal}
>
  <!-- Form -->
  <form on:submit={handleSubmit} style="display: flex; flex-direction: column; gap: 16px;">
    
    <!-- COMPANY INFORMATION SECTION -->
    <div>
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        Company Information
      </h3>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        
        <!-- Company Name -->
        <FormInput
          label="Company Name"
          bind:value={formData.name}
          placeholder="Full company name"
          required
          error={formErrors.name}
        />
        
        <!-- Short Name and Abbreviation -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Short Name"
            bind:value={formData.name_short}
            placeholder="Short name"
            required
            error={formErrors.name_short}
          />
          
          <FormInput
            label="Abbreviation"
            bind:value={formData.abbreviation}
            placeholder="ABC"
            maxlength={10}
            required
            error={formErrors.abbreviation}
          />
        </div>
        
        <!-- Location -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Country"
            bind:value={formData.country}
            placeholder="Country"
            required
            error={formErrors.country}
            on:input={(e) => handleCountrySearch((e.target as HTMLInputElement).value)}
          />
          
          <FormInput
            label="City"
            bind:value={formData.city}
            placeholder="City"
            required
            error={formErrors.city}
            on:input={(e) => handleCitySearch((e.target as HTMLInputElement).value)}
          />
        </div>
        
        <!-- Registration Details -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Registration No."
            bind:value={formData.reg_no}
            placeholder="Company registration number"
            required
            error={formErrors.reg_no}
          />
          
          <FormInput
            label="Tax/VAT No."
            bind:value={formData.tax_no}
            placeholder="Tax identification number"
            required
            error={formErrors.tax_no}
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
        <p class="font-medium mb-2">Are you sure you want to delete this company?</p>
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
          {mode === 'create' ? 'Create Company' : 'Update Company'}
        </button>
      </div>
    </div>
  </form>
</BaseModal>