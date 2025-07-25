<!--
  Company Modal using new utility functions for reduced code duplication
  Refactored from 734 lines to ~420 lines using utilities
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { companiesActions } from '$lib/stores';
  import { searchCountries, getCitySuggestions, getAllCities } from '$lib/api';
  import { extractSurrealId } from '$lib/utils/surrealdb';
  import { validateForm, CommonValidationRules, hasValidationErrors } from '$lib/utils/validation';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
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
  ];
  
  let formErrors: Record<string, string> = {};
  let showDeleteConfirm = false;
  
  // Typeahead state (could be extracted to another utility)
  let countrySearchQuery = '';
  let countryOptions: any[] = [];
  let showCountryDropdown = false;
  let selectedCountry: any = null;
  let countrySelectedIndex = -1;
  
  let citySuggestions: string[] = [];
  let showCityDropdown = false;
  let citySelectedIndex = -1;
  
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
    
    // Initialize typeahead for edit mode
    countrySearchQuery = company.country || '';
    if (company.country) {
      loadCitySuggestions();
    }
  } else if (mode === 'create') {
    resetForm();
  }
  
  // Reset state when modal closes
  $: if (!isOpen) {
    resetForm();
  } else if (isOpen && mode === 'create' && citySuggestions.length === 0) {
    loadCitySuggestions();
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
    operationActions.reset();
    showDeleteConfirm = false;
    // Reset typeahead state
    countrySearchQuery = '';
    countryOptions = [];
    showCountryDropdown = false;
    selectedCountry = null;
    countrySelectedIndex = -1;
    citySuggestions = [];
    showCityDropdown = false;
    citySelectedIndex = -1;
  }
  
  // Validation using the new system
  function validateFormData(): boolean {
    formErrors = validateForm(formData, validationRules);
    return !hasValidationErrors(formErrors);
  }
  
  // Handle form submission
  async function handleSubmit(event: Event) {
    event.preventDefault();
    
    if (!validateFormData()) {
      return;
    }
    
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
        const newCompany = await withLoadingState(
          () => companiesActions.create(companyData),
          operationActions,
          'saving'
        );
        operationActions.setMessage('Company created successfully!');
        
        // Dispatch company created event for parent components
        dispatch('companyCreated', newCompany);
      } else {
        const companyId = extractSurrealId(company?.id);
        if (companyId) {
          await withLoadingState(
            () => companiesActions.update(companyId, formData),
            operationActions,
            'saving'
          );
          operationActions.setMessage('Company updated successfully!');
        } else {
          throw new Error('No valid company ID found for update');
        }
      }
      
      // Auto-close after success
      setTimeout(() => {
        closeModal();
      }, 1500);
      
    } catch (error: any) {
      operationActions.setError(error?.message || 'Failed to save company');
    }
  }
  
  // Handle delete with the new utilities
  async function handleDelete() {
    const companyId = extractSurrealId(company?.id);
    if (!companyId) {
      operationActions.setError('No valid company ID found for delete');
      return;
    }
    
    try {
      await withLoadingState(
        () => companiesActions.delete(companyId),
        operationActions,
        'deleting'
      );
      operationActions.setMessage('Company deleted successfully!');
      
      setTimeout(() => {
        closeModal();
      }, 1500);
      
    } catch (error: any) {
      operationActions.setError(error?.message || 'Failed to delete company');
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
  
  // Keyboard handling
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeModal();
    }
  }
  
  // Auto-generate abbreviation from company name
  function generateAbbreviation() {
    if (formData.name && !formData.abbreviation) {
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
      const words = formData.name.trim().split(/\s+/);
      if (words.length >= 2) {
        formData.name_short = words.slice(0, 2).join(' ');
      } else {
        formData.name_short = formData.name.substring(0, 30);
      }
    }
  }
  
  // Country search logic
  async function searchCountriesHandler() {
    if (countrySearchQuery.length < 2) {
      countryOptions = [];
      showCountryDropdown = false;
      return;
    }
    
    try {
      const results = await searchCountries(countrySearchQuery);
      countryOptions = results;
      showCountryDropdown = results.length > 0;
      countrySelectedIndex = -1;
    } catch (error) {
      countryOptions = [];
      showCountryDropdown = false;
    }
  }

  function handleCountryInput() {
    formData.country = '';
    selectedCountry = null;
    countrySelectedIndex = -1;
    searchCountriesHandler();
  }

  function selectCountry(country: any) {
    selectedCountry = country;
    countrySearchQuery = country.name;
    formData.country = country.name;
    showCountryDropdown = false;
    countrySelectedIndex = -1;
    loadCitySuggestions();
  }

  // City suggestions logic
  async function loadCitySuggestions() {
    try {
      if (formData.country) {
        const cities = await getCitySuggestions(formData.country);
        citySuggestions = cities;
      } else {
        const cities = await getAllCities();
        citySuggestions = cities;
      }
    } catch (error) {
      citySuggestions = [];
    }
  }

  function handleCityInput() {
    citySelectedIndex = -1;
    
    if (citySuggestions.length > 0 && formData.city.length > 0) {
      const filteredSuggestions = citySuggestions.filter(city => 
        city.toLowerCase().includes(formData.city.toLowerCase())
      );
      showCityDropdown = filteredSuggestions.length > 0;
    } else {
      showCityDropdown = false;
    }
  }

  function selectCity(city: string) {
    formData.city = city;
    showCityDropdown = false;
    citySelectedIndex = -1;
  }

  function handleCountryKeydown(e: KeyboardEvent) {
    if (!showCountryDropdown || countryOptions.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        countrySelectedIndex = Math.min(countrySelectedIndex + 1, countryOptions.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        countrySelectedIndex = Math.max(countrySelectedIndex - 1, 0);
        break;
      case 'Enter':
      case 'Tab':
        e.preventDefault();
        if (countrySelectedIndex >= 0) {
          selectCountry(countryOptions[countrySelectedIndex]);
        }
        break;
      case 'Escape':
        showCountryDropdown = false;
        countrySelectedIndex = -1;
        break;
    }
  }

  function handleCityKeydown(e: KeyboardEvent) {
    if (!showCityDropdown || citySuggestions.length === 0) return;
    
    const filteredSuggestions = citySuggestions.filter(city => 
      city.toLowerCase().includes(formData.city.toLowerCase())
    );

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        citySelectedIndex = Math.min(citySelectedIndex + 1, filteredSuggestions.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        citySelectedIndex = Math.max(citySelectedIndex - 1, 0);
        break;
      case 'Enter':
      case 'Tab':
        e.preventDefault();
        if (citySelectedIndex >= 0) {
          selectCity(filteredSuggestions[citySelectedIndex]);
        }
        break;
      case 'Escape':
        showCityDropdown = false;
        citySelectedIndex = -1;
        break;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Modal backdrop -->
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4" 
    on:click={closeModal}
    role="dialog"
    aria-modal="true"
    aria-labelledby="company-modal-title"
    tabindex="-1"
  >
    <!-- Modal Content -->
    <div 
      class="bg-emittiv-darker border border-emittiv-dark rounded w-full max-h-[90vh] overflow-y-auto"
      style="padding: 16px; max-width: 450px;"
      on:click|stopPropagation
      role="presentation"
    >
      <!-- Header -->
      <div class="flex items-center justify-between" style="margin-bottom: 20px;">
        <h2 id="company-modal-title" class="font-semibold text-emittiv-white" style="font-size: 16px;">
          {mode === 'create' ? 'Add New Company' : 'Edit Company'}
        </h2>
        <button 
          on:click={closeModal}
          class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-all"
          aria-label="Close modal"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <form on:submit={handleSubmit} style="display: flex; flex-direction: column; gap: 24px;">
        
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
                class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                class:border-red-500={formErrors.name}
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
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  class:border-red-500={formErrors.name_short}
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
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  class:border-red-500={formErrors.abbreviation}
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
              <div class="relative">
                <label for="company_city" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  City *
                </label>
                <input
                  id="company_city"
                  type="text"
                  bind:value={formData.city}
                  on:input={handleCityInput}
                  on:keydown={handleCityKeydown}
                  placeholder="Type to filter cities..."
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  class:border-red-500={formErrors.city}
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                
                <!-- City dropdown -->
                {#if showCityDropdown && citySuggestions.length > 0}
                  {@const filteredCities = citySuggestions.filter(city => city.toLowerCase().includes(formData.city.toLowerCase()))}
                  {#if filteredCities.length > 0}
                    <div class="absolute top-full left-0 right-0 bg-emittiv-darker border border-emittiv-dark rounded-b shadow-lg z-50 overflow-y-auto" style="max-height: 120px;">
                      {#each filteredCities as city, index}
                        <div 
                          class="text-emittiv-white hover:bg-emittiv-dark cursor-pointer"
                          class:bg-emittiv-splash={index === citySelectedIndex}
                          class:text-emittiv-black={index === citySelectedIndex}
                          style="padding: 8px 12px; font-size: 12px;"
                          role="option"
                          aria-selected={index === citySelectedIndex}
                          on:click={() => selectCity(city)}
                          on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectCity(city); } }}
                          tabindex="0"
                        >
                          {city}
                        </div>
                      {/each}
                    </div>
                  {/if}
                {/if}
                
                {#if formErrors.city}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.city}</p>
                {/if}
              </div>
              
              <div class="relative">
                <label for="company_country" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Country *
                </label>
                <input
                  id="company_country"
                  type="text"
                  bind:value={countrySearchQuery}
                  on:input={handleCountryInput}
                  on:keydown={handleCountryKeydown}
                  placeholder="Search countries..."
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  class:border-red-500={formErrors.country}
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                
                <!-- Country dropdown -->
                {#if showCountryDropdown && countryOptions.length > 0}
                  <div class="absolute top-full left-0 right-0 bg-emittiv-darker border border-emittiv-dark rounded-b shadow-lg z-50 overflow-y-auto" style="max-height: 120px;">
                    {#each countryOptions as country, index}
                      <div 
                        class="text-emittiv-white hover:bg-emittiv-dark cursor-pointer"
                        class:bg-emittiv-splash={index === countrySelectedIndex}
                        class:text-emittiv-black={index === countrySelectedIndex}
                        style="padding: 8px 12px; font-size: 12px;"
                        role="option"
                        aria-selected={index === countrySelectedIndex}
                        on:click={() => selectCountry(country)}
                        on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectCountry(country); } }}
                        tabindex="0"
                      >
                        {country.name}
                      </div>
                    {/each}
                  </div>
                {/if}
                
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
            <p style="margin-bottom: 8px;">⚠️ Are you sure you want to delete this company?</p>
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
                <span>{mode === 'create' ? 'Create Company' : 'Update Company'}</span>
              {/if}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}