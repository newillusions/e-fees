<!--
  Refactored New Project Modal using BaseModal, FormInput, FormSelect, and TypeaheadSelect components
  Reduced from ~990 lines to ~350 lines using base components and utilities
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { projectsActions } from '$lib/stores';
  import { generateNextProjectNumber, searchCountries, getAreaSuggestions, getCitySuggestions, checkProjectFolderExists, copyProjectTemplate } from '$lib/api';
  import { validateForm, hasValidationErrors } from '$lib/utils/validation';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
  import BaseModal from './BaseModal.svelte';
  import FormInput from './FormInput.svelte';
  import FormSelect from './FormSelect.svelte';
  import TypeaheadSelect from './TypeaheadSelect.svelte';
  import Button from './Button.svelte';
  import type { Project, ProjectCreationResult } from '$lib/../types';
  import type { ProjectNumber } from '$lib/../types/database';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let mode: 'create' = 'create';
  
  // Use the new operation state utility
  const { store: operationState, actions: operationActions } = useOperationState();
  
  // Form data with better typing
  interface ProjectFormData {
    name: string;
    name_short: string;
    status: 'Draft' | 'RFP' | 'Active' | 'On Hold' | 'Completed' | 'Cancelled';
    area: string;
    city: string;
    country: string;
    year: number;
    project_number: string;
    folder: string;
  }
  
  let formData: ProjectFormData = {
    name: '',
    name_short: '',
    status: 'Draft',
    area: '',
    city: '',
    country: '',
    year: new Date().getFullYear() % 100,
    project_number: '',
    folder: ''
  };
  
  // Status options
  const statusOptions = [
    { value: 'Draft', label: 'Draft' },
    { value: 'RFP', label: 'RFP' },
    { value: 'Active', label: 'Active' },
    { value: 'On Hold', label: 'On Hold' },
    { value: 'Completed', label: 'Completed' },
    { value: 'Cancelled', label: 'Cancelled' }
  ];
  
  // Validation setup
  const validationRules = [
    { field: 'name' as keyof ProjectFormData, required: true, minLength: 1, maxLength: 255 },
    { field: 'name_short' as keyof ProjectFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'area' as keyof ProjectFormData, required: true, minLength: 1, maxLength: 100 },
    { field: 'city' as keyof ProjectFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'country' as keyof ProjectFormData, required: true, minLength: 1, maxLength: 50 }
  ];
  
  // Form validation state
  let formErrors: Record<string, string> = {};
  
  // UI state
  let isGenerating = false;
  let isModalReady = false;
  let showFolderConfirm = false;
  let pendingProjectData: ProjectCreationResult | null = null;
  
  // Typeahead search states
  let countrySearchText = '';
  let areaSearchText = '';
  let citySearchText = '';
  
  // Options for typeahead components
  let countryOptions: Array<{ id: string; name: string; dial_code: string }> = [];
  let areaOptions: Array<{ id: string; name: string }> = [];
  let cityOptions: Array<{ id: string; name: string }> = [];
  
  // Auto-populate folder name when project number or short name changes
  $: if (formData.project_number && formData.name_short) {
    formData.folder = `${formData.project_number} ${formData.name_short}`;
  } else if (formData.name_short) {
    formData.folder = formData.name_short;
  } else {
    formData.folder = '';
  }
  
  // Track previous country and year to detect changes
  let previousCountry = '';
  let previousYear = 0;
  
  // Auto-generate project number when country or year changes
  $: if (isModalReady && formData.country && formData.year && !isGenerating) {
    // Check if country or year actually changed
    if (formData.country !== previousCountry || formData.year !== previousYear) {
      previousCountry = formData.country;
      previousYear = formData.year;
      generateProjectNumber();
    }
  }
  
  // Generate the next available project number
  async function generateProjectNumber() {
    if (!formData.country || !formData.year) {
      formData.project_number = '';
      return;
    }
    
    isGenerating = true;
    
    try {
      const projectNumber = await generateNextProjectNumber(formData.country, formData.year);
      formData.project_number = projectNumber;
    } catch (error) {
      console.error('Failed to generate project number:', error);
      operationActions.setError('Failed to generate project number');
    } finally {
      isGenerating = false;
    }
  }

  // Force regenerate project number (for manual refresh)
  async function regenerateProjectNumber() {
    formData.project_number = ''; // Clear current number
    await generateProjectNumber(); // Generate new one
  }
  
  // Country search handler
  async function handleCountrySearch(searchText: string) {
    if (!searchText || searchText.length < 2) {
      countryOptions = [];
      return;
    }
    
    try {
      const results = await searchCountries(searchText);
      countryOptions = results.map(country => ({
        id: country.name,
        name: country.name,
        dial_code: country.dial_code
      }));
    } catch (error) {
      console.error('Failed to search countries:', error);
      countryOptions = [];
    }
  }
  
  // Area search handler - works independently
  async function handleAreaSearch(searchText: string) {
    if (!searchText || searchText.length < 2) {
      areaOptions = [];
      return;
    }
    
    try {
      // Get suggestions for the selected country, or all if no country
      const country = formData.country || null;
      const results = await getAreaSuggestions(country);
      areaOptions = results
        .filter(area => area.toLowerCase().includes(searchText.toLowerCase()))
        .map(area => ({ id: area, name: area }))
        .slice(0, 10);
    } catch (error) {
      console.error('Failed to get area suggestions:', error);
      areaOptions = [];
    }
  }
  
  // City search handler - works independently
  async function handleCitySearch(searchText: string) {
    if (!searchText || searchText.length < 2) {
      cityOptions = [];
      return;
    }
    
    try {
      // Get suggestions for the selected country, or all if no country
      const country = formData.country || null;
      const results = await getCitySuggestions(country);
      cityOptions = results
        .filter(city => city.toLowerCase().includes(searchText.toLowerCase()))
        .map(city => ({ id: city, name: city }))
        .slice(0, 10);
    } catch (error) {
      console.error('Failed to get city suggestions:', error);
      cityOptions = [];
    }
  }
  
  // Form submission handler
  function handleSubmit(event: Event) {
    event.preventDefault();
    
    // If user typed in the fields but didn't select from dropdown, use the typed values
    if (countrySearchText && !formData.country) {
      formData.country = countrySearchText;
    }
    if (citySearchText && !formData.city) {
      formData.city = citySearchText;
    }
    if (areaSearchText && !formData.area) {
      formData.area = areaSearchText;
    }
    
    // Validate using the validation system
    const errors = validateForm(formData, validationRules);
    
    // Custom validation for project number
    if (!formData.project_number) {
      errors.project_number = 'Project number is required';
    }
    
    formErrors = errors;
    
    if (hasValidationErrors(errors)) {
      operationActions.setError('Please fix the validation errors above.');
      return;
    }
    
    handleCreate();
  }
  
  // Create project with loading state
  async function handleCreate() {
    await withLoadingState(async () => {
      // Parse project number to extract components
      const parts = formData.project_number.split('-');
      if (parts.length !== 2) {
        throw new Error('Invalid project number format');
      }
      
      const year = parseInt(parts[0]);
      const countryCode = parts[1].substring(0, 3);
      const seq = parseInt(parts[1].substring(3));
      
      // Construct ProjectNumber object for database
      const projectNumber: ProjectNumber = {
        year,
        country: parseInt(countryCode),
        seq,
        id: formData.project_number
      };
      
      // Construct Project object with required fields only
      const project = {
        name: formData.name,
        name_short: formData.name_short,
        status: formData.status,
        area: formData.area,
        city: formData.city,
        country: formData.country,
        number: projectNumber,
        folder: formData.folder,
        time: {
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        }
      };
      
      // Create project in database first
      const result = await projectsActions.create(project);
      
      // Check if project folder already exists
      const folderExists = await checkProjectFolderExists(formData.project_number, formData.name_short);
      
      if (folderExists) {
        // Store project data and show confirmation dialog
        pendingProjectData = { projectNumber: formData.project_number, projectShortName: formData.name_short };
        showFolderConfirm = true;
        operationActions.setMessage('Project created successfully! Folder already exists - confirm overwrite.');
        return result;
      } else {
        // Create folder directly
        await createProjectFolder(formData.project_number, formData.name_short);
        operationActions.setMessage('Project and folder created successfully');
        resetForm();
        closeModal();
        return result;
      }
    }, operationActions, 'saving');
  }
  
  // Create project folder
  async function createProjectFolder(projectNumber: string, projectShortName: string) {
    try {
      const result = await copyProjectTemplate(projectNumber, projectShortName);
      console.log('Project folder created:', result);
    } catch (error) {
      console.error('Failed to create project folder:', error);
      operationActions.setError(`Project created but folder creation failed: ${error}`);
    }
  }
  
  // Handle folder confirmation dialog
  async function handleFolderConfirm(overwrite: boolean) {
    showFolderConfirm = false;
    
    if (overwrite && pendingProjectData) {
      await withLoadingState(async () => {
        await createProjectFolder(pendingProjectData.projectNumber, pendingProjectData.projectShortName);
        operationActions.setMessage('Project and folder created successfully (existing folder overwritten)');
        resetForm();
        closeModal();
        return true;
      }, operationActions, 'saving');
    } else {
      operationActions.setMessage('Project created successfully (folder creation skipped)');
      resetForm();
      closeModal();
    }
    
    pendingProjectData = null;
  }
  
  // Form management
  function resetForm() {
    formData = {
      name: '',
      name_short: '',
      status: 'Draft',
      area: '',
      city: '',
      country: '',
      year: new Date().getFullYear() % 100,
      project_number: '',
      folder: ''
    };
    
    formErrors = {};
    isModalReady = false; // Reset modal ready state
    showFolderConfirm = false;
    pendingProjectData = null;
    
    // Reset tracking variables
    previousCountry = '';
    previousYear = 0;
    
    // Clear search texts and options
    countrySearchText = '';
    areaSearchText = '';
    citySearchText = '';
    countryOptions = [];
    areaOptions = [];
    cityOptions = [];
  }
  
  function closeModal() {
    resetForm();
    operationActions.reset();
    dispatch('close');
  }

  // Initialize modal when it opens/closes
  $: if (isOpen && !isModalReady) {
    // Small delay to ensure modal is fully rendered before enabling auto-generation
    setTimeout(() => {
      isModalReady = true;
    }, 100);
  } else if (!isOpen) {
    isModalReady = false;
  }
  
  // Sync search text with form values (in case they get out of sync)
  $: if (formData.country && !countrySearchText) {
    countrySearchText = formData.country;
  }
  $: if (formData.city && !citySearchText) {
    citySearchText = formData.city;
  }
  $: if (formData.area && !areaSearchText) {
    areaSearchText = formData.area;
  }
  
  // Typeahead handlers
  function handleCountrySelect(event: CustomEvent) {
    formData.country = event.detail.option.name;
    countrySearchText = event.detail.option.name; // Keep search text in sync
    // Don't clear area and city automatically - let user decide
    // This was causing the area field to be cleared unexpectedly
  }
  
  function handleAreaSelect(event: CustomEvent) {
    formData.area = event.detail.option.name;
    areaSearchText = event.detail.option.name; // Keep search text in sync
  }
  
  function handleCitySelect(event: CustomEvent) {
    formData.city = event.detail.option.name;
    citySearchText = event.detail.option.name; // Keep search text in sync
  }
  
  // Clear handlers for typeahead fields
  function handleCountryClear() {
    formData.country = '';
    countrySearchText = '';
    // Also clear dependent fields
    formData.area = '';
    formData.city = '';
    areaSearchText = '';
    citySearchText = '';
  }
  
  function handleCityClear() {
    formData.city = '';
    citySearchText = '';
  }
  
  function handleAreaClear() {
    formData.area = '';
    areaSearchText = '';
  }
</script>

<BaseModal 
  {isOpen} 
  title="New Project"
  maxWidth="500px"
  on:close={closeModal}
>
  <!-- Form -->
  <form on:submit={handleSubmit} style="display: flex; flex-direction: column; gap: 16px;">
    
    <!-- PROJECT INFORMATION SECTION -->
    <div>
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        Project Information
      </h3>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        
        <!-- Project Name -->
        <FormInput
          label="Project Name"
          bind:value={formData.name}
          placeholder="Full project name"
          required
          error={formErrors.name}
        />
        
        <!-- Short Name and Status -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Short Name"
            bind:value={formData.name_short}
            placeholder="Short name"
            required
            error={formErrors.name_short}
          />
          
          <FormSelect
            label="Status"
            bind:value={formData.status}
            options={statusOptions}
            required
          />
        </div>
        
        <!-- Year and Project Number -->
        <div style="display: grid; grid-template-columns: 80px 1fr; gap: 12px;">
          <FormInput
            label="Year"
            type="number"
            bind:value={formData.year}
            placeholder="25"
            min="20"
            max="99"
            required
          />
          
          <div>
            <label class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
              Project Number *
            </label>
            <div class="relative flex" style="max-width: 100%;">
              <input
                type="text"
                bind:value={formData.project_number}
                placeholder="Auto-generated"
                readonly
                class="flex-1 bg-emittiv-darker border border-emittiv-dark rounded-l text-emittiv-light placeholder-emittiv-light opacity-60 cursor-not-allowed"
                style="padding: 8px 12px; font-size: 12px; height: 32px; min-width: 0;"
              />
              <button
                type="button"
                on:click={regenerateProjectNumber}
                disabled={!formData.country || !formData.year || isGenerating}
                class="bg-emittiv-dark border border-emittiv-dark border-l-0 rounded-r text-emittiv-light hover:bg-emittiv-light hover:text-emittiv-darker disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center justify-center"
                style="height: 32px; font-size: 12px; padding: 0 8px; flex-shrink: 0;"
                title="Regenerate project number"
              >
                {#if isGenerating}
                  <div 
                    class="border-2 border-current border-t-transparent rounded-full animate-spin"
                    style="width: 12px; height: 12px;"
                  ></div>
                {:else}
                  ↻
                {/if}
              </button>
            </div>
            {#if formErrors.project_number}
              <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.project_number}</p>
            {/if}
          </div>
        </div>
        
        <!-- Folder Name -->
        <FormInput
          label="Folder Name"
          bind:value={formData.folder}
          placeholder="Auto-generated from project number and short name"
          readonly
        />
      </div>
    </div>
    
    <!-- LOCATION DETAILS SECTION -->
    <div>
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        Location Details
      </h3>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        
        <!-- Country Selection -->
        <TypeaheadSelect
          label="Country"
          value=""
          bind:searchText={countrySearchText}
          options={countryOptions}
          displayFields={['name']}
          placeholder="Search countries..."
          required
          error={formErrors.country}
          on:input={(e) => handleCountrySearch(e.detail)}
          on:select={handleCountrySelect}
          on:clear={handleCountryClear}
        >
          <svelte:fragment slot="option" let:option>
            <span>{option.name}</span>
            <span class="text-emittiv-light text-xs ml-auto">+{option.dial_code}</span>
          </svelte:fragment>
        </TypeaheadSelect>
        
        <!-- City and Area -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <TypeaheadSelect
            label="City"
            value=""
            bind:searchText={citySearchText}
            options={cityOptions}
            displayFields={['name']}
            placeholder="Search cities..."
            required
            error={formErrors.city}
            on:input={(e) => handleCitySearch(e.detail)}
            on:select={handleCitySelect}
            on:clear={handleCityClear}
          />
          
          <TypeaheadSelect
            label="Area"
            value=""
            bind:searchText={areaSearchText}
            options={areaOptions}
            displayFields={['name']}
            placeholder="Search areas..."
            required
            error={formErrors.area}
            on:input={(e) => handleAreaSearch(e.detail)}
            on:select={handleAreaSelect}
            on:clear={handleAreaClear}
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
    
    <!-- Folder Overwrite Confirmation -->
    {#if showFolderConfirm}
      <div class="text-yellow-400 text-sm bg-yellow-900/20 border border-yellow-500/30 rounded p-3">
        <p class="font-medium mb-2">Project folder already exists</p>
        <p class="text-xs opacity-80 mb-3">
          A folder for project "{pendingProjectData?.projectNumber} {pendingProjectData?.projectShortName}" already exists. 
          Do you want to overwrite it with a fresh template?
        </p>
        <div class="flex gap-2">
          <button
            type="button"
            on:click={() => handleFolderConfirm(true)}
            class="bg-yellow-600 hover:bg-yellow-700 text-white rounded font-medium transition-all flex items-center justify-center disabled:opacity-50"
            style="height: 24px; padding: 4px 8px; font-size: 11px;"
            disabled={$operationState.saving}
          >
            Yes, overwrite
          </button>
          <button
            type="button"
            on:click={() => handleFolderConfirm(false)}
            class="border border-yellow-500/30 rounded text-yellow-300 hover:text-yellow-200 transition-all"
            style="height: 24px; padding: 4px 8px; font-size: 11px;"
            disabled={$operationState.saving}
          >
            No, skip folder creation
          </button>
        </div>
      </div>
    {/if}
    
    <!-- Actions - Full Width Container -->
    <div class="w-full" style="height: 40px;">
      <div class="flex justify-end items-stretch h-full" style="gap: 12px;">
        <Button
          variant="secondary"
          size="sm"
          className="h-full !py-1 !flex !items-center !justify-center"
          on:click={closeModal}
          disabled={$operationState.saving || showFolderConfirm}
        >
          Cancel
        </Button>
        
        <Button
          type="submit"
          variant="primary"
          size="sm"
          className="h-full !py-1 !flex !items-center !justify-center"
          disabled={$operationState.saving || isGenerating || showFolderConfirm}
        >
          {#if $operationState.saving}
            <div 
              class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin"
              style="width: 14px; height: 14px; margin-right: 6px;"
            ></div>
          {/if}
          Create Project
        </Button>
      </div>
    </div>
  </form>
</BaseModal>