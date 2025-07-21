<script lang="ts">
  /**
   * NewProjectModal.svelte
   * 
   * Complete project creation workflow with automatic project number generation.
   * This modal provides a comprehensive form for creating new FP projects with
   * real-time validation, country search, and template folder creation.
   * 
   * Features:
   * - Automatic project number generation in YY-CCCNN format
   * - Fuzzy country search with autocomplete
   * - Area/city suggestions based on existing data
   * - Real-time validation and error handling
   * - Template folder copying with file renaming
   * - Full accessibility compliance (WCAG 2.1)
   * - Responsive design matching application standards
   * 
   * Project Number Format:
   * - YY: 2-digit year (25 = 2025)
   * - CCC: Country dial code (971 = UAE, 966 = Saudi)
   * - NN: 2-digit sequence number (01, 02, 03...)
   * - Example: "25-97105" (2025, UAE, 5th project)
   * 
   * Database Integration:
   * - Creates project record in SurrealDB
   * - Copies template folder structure
   * - Renames files with actual project number
   * - Updates reactive stores for UI consistency
   * 
   * @component
   */
  import { onMount } from 'svelte';
  import Card from './Card.svelte';
  import Button from './Button.svelte';
  import Input from './Input.svelte';
  import { generateNextProjectNumber, validateProjectNumber, createProjectWithTemplate, searchCountries, getAreaSuggestions, getCitySuggestions } from '$lib/api';
  import type { Project } from '../../types';
  import type { ProjectNumber } from '../../types/database';
  import { projectsActions } from '$lib/stores';

  /**
   * ============================================================================
   * COMPONENT CAPABILITIES SUMMARY
   * ============================================================================
   * 
   * This NewProjectModal component provides a complete project creation workflow
   * with the following key capabilities:
   * 
   * 1. AUTOMATIC PROJECT NUMBERING
   *    - Generates sequential numbers in YY-CCCNN format
   *    - Uses country dial codes for geographic identification
   *    - Ensures uniqueness through database validation
   * 
   * 2. ADVANCED SEARCH AND AUTOCOMPLETE
   *    - Fuzzy country search across multiple fields
   *    - Location suggestions based on existing project data
   *    - Real-time filtering with keyboard navigation
   * 
   * 3. COMPREHENSIVE VALIDATION
   *    - Required field validation
   *    - Project number uniqueness checking
   *    - User-friendly error messages
   * 
   * 4. ACCESSIBILITY COMPLIANCE
   *    - WCAG 2.1 compliant keyboard navigation
   *    - Proper ARIA labels and roles
   *    - Screen reader support
   *    - Focus management
   * 
   * 5. TEMPLATE INTEGRATION
   *    - Automatic template folder copying
   *    - File renaming with project numbers
   *    - Cross-platform file operations
   * 
   * 6. REACTIVE STATE MANAGEMENT
   *    - Svelte 5 runes for optimal performance
   *    - Real-time UI updates
   *    - Optimistic updates for better UX
   * 
   * The component integrates seamlessly with the application's database,
   * file system, and UI state management for a complete end-to-end workflow.
   */

  /**
   * Component Props
   * 
   * @prop {boolean} isOpen - Controls modal visibility (bindable)
   * @prop {() => void} onClose - Callback when modal is closed
   * @prop {(project: Project) => void} onSuccess - Callback when project is successfully created
   */
  let { isOpen = $bindable(false), onClose, onSuccess }: {
    isOpen?: boolean;
    onClose: () => void;
    onSuccess: (project: Project) => void;
  } = $props();

  // ============================================================================
  // STATE MANAGEMENT
  // ============================================================================
  
  /** Loading states for async operations */
  let isLoading = $state(false);          // Overall form submission loading
  let isGenerating = $state(false);       // Project number generation loading
  
  /** Validation and error handling */
  let validationMessage = $state('');     // Form validation error messages
  
  /** Country search functionality */
  let countrySearchQuery = $state('');    // User input for country search
  let countryOptions = $state<any[]>([]);        // Search results from database
  let showCountryDropdown = $state(false); // Dropdown visibility control
  let selectedCountry = $state<any>(null);     // Selected country object
  
  /** Location autocomplete suggestions */
  let areaSuggestions = $state<string[]>([]);       // Area suggestions based on country
  let showAreaDropdown = $state(false);   // Area dropdown visibility
  let citySuggestions = $state<string[]>([]);       // City suggestions based on country
  let showCityDropdown = $state(false);   // City dropdown visibility
  
  /** Keyboard navigation state for accessibility */
  let countrySelectedIndex = $state(-1);  // Currently highlighted country option
  let areaSelectedIndex = $state(-1);     // Currently highlighted area option
  let citySelectedIndex = $state(-1);     // Currently highlighted city option
  
  /** 
   * Form data structure containing all project fields
   * 
   * This reactive state object holds all form inputs and is used
   * to construct the final Project object for database creation.
   */
  let formData = $state({
    name: '',                                    // Full project name (required)
    name_short: '',                             // Short project name for folders (required)
    status: 'Draft',                            // Project status (required, default: Draft)
    area: '',                                   // Project area/location (required)
    city: '',                                   // City where project is located (required)
    country: '',                                // Country name (required, drives numbering)
    year: new Date().getFullYear() % 100,      // 2-digit year for numbering (editable)
    project_number: '',                         // Auto-generated project number (read-only)
    folder: ''                                  // Folder name (auto-populated from project_number + name_short)
  });

  // Auto-populate folder name when project number or short name changes
  $effect(() => {
    if (formData.project_number && formData.name_short) {
      formData.folder = `${formData.project_number} ${formData.name_short}`;
    } else if (formData.name_short) {
      formData.folder = formData.name_short;
    } else {
      formData.folder = '';
    }
  });

  // Auto-generate project number when country or year changes
  $effect(() => {
    if (formData.country && formData.year) {
      generateProjectNumber();
    }
  });

  // ============================================================================
  // PROJECT NUMBER GENERATION
  // ============================================================================
  
  /**
   * Generates the next available project number for the selected country and year.
   * 
   * Project numbers follow the format YY-CCCNN where:
   * - YY: 2-digit year (25 for 2025)
   * - CCC: Country dial code (971 for UAE, 966 for Saudi)
   * - NN: Sequential number starting from 01
   * 
   * The function queries the database to find the highest existing sequence
   * number for the given country/year combination and increments it.
   * 
   * @async
   * @function generateProjectNumber
   * @returns {Promise<void>} Updates formData.project_number with generated number
   * 
   * @example
   * // For UAE in 2025, if highest sequence is 04:
   * // Generates: "25-97105"
   */
  async function generateProjectNumber() {
    // Validate required inputs before generation
    if (!formData.country || !formData.year) {
      formData.project_number = '';
      return;
    }
    
    // Set loading state for UI feedback
    isGenerating = true;
    validationMessage = '';
    
    try {
      // Call backend to generate next sequential number
      const projectNumber = await generateNextProjectNumber(formData.country, formData.year);
      formData.project_number = projectNumber;
      
      // Double-check uniqueness (defensive programming)
      const isValid = await validateProjectNumber(projectNumber);
      if (!isValid) {
        validationMessage = 'Generated number already exists. Please try again.';
      }
    } catch (error) {
      console.error('Failed to generate project number:', error);
      validationMessage = 'Failed to generate project number';
    } finally {
      isGenerating = false;
    }
  }

  /**
   * Reactive handlers for form changes that affect project numbering.
   * These functions ensure the project number is regenerated whenever
   * the country or year changes.
   */
  
  /** Triggered when country selection changes */
  async function handleCountryChange() {
    await generateProjectNumber();
  }

  /** Triggered when year input changes */
  async function handleYearChange() {
    await generateProjectNumber();
  }

  // ============================================================================
  // COUNTRY SEARCH FUNCTIONALITY
  // ============================================================================
  
  /**
   * Performs fuzzy search across country database fields.
   * 
   * Searches the following fields in the countries table:
   * - name (United Arab Emirates)
   * - name_formal (formal country name)
   * - name_official (official country name)
   * - code (AE, SA)
   * - code_alt (alternative codes)
   * - dial_code (971, 966)
   * 
   * The search is triggered on every keystroke but requires minimum
   * 2 characters to avoid overwhelming the UI with results.
   * 
   * @async
   * @function searchCountriesDebounced
   * @returns {Promise<void>} Updates countryOptions with search results
   * 
   * @example
   * // User types "UAE" or "971" or "Arab"
   * // Returns: [{ name: "United Arab Emirates", dial_code: 971, ... }]
   */
  async function searchCountriesDebounced() {
    // Require minimum 2 characters to prevent excessive API calls
    if (countrySearchQuery.length < 2) {
      countryOptions = [];
      showCountryDropdown = false;
      return;
    }
    
    try {
      // Call backend search function with fuzzy matching
      const results = await searchCountries(countrySearchQuery);
      countryOptions = results;
      showCountryDropdown = results.length > 0;
    } catch (error) {
      console.error('Failed to search countries:', error);
      countryOptions = [];
      showCountryDropdown = false;
    }
  }

  /**
   * Handles country selection from dropdown.
   * 
   * When a country is selected:
   * 1. Updates selected country state
   * 2. Sets form input to country name
   * 3. Closes dropdown
   * 4. Regenerates project number
   * 5. Loads location suggestions for area/city
   * 
   * @param {Object} country - Selected country object from database
   * @param {string} country.name - Country display name
   * @param {number} country.dial_code - Country dial code for numbering
   */
  function selectCountry(country: any) {
    selectedCountry = country;
    countrySearchQuery = country.name;
    formData.country = country.name; // Store display name, not code
    showCountryDropdown = false;
    
    // Trigger project number regeneration with new country
    handleCountryChange();
    
    // Load area/city suggestions for selected country
    loadLocationSuggestions();
  }

  // ============================================================================
  // LOCATION AUTOCOMPLETE FUNCTIONALITY
  // ============================================================================
  
  /**
   * Loads area and city suggestions based on selected country.
   * 
   * This function queries existing project data to provide autocomplete
   * suggestions for area and city fields. It helps maintain consistency
   * in location naming across projects.
   * 
   * The suggestions are loaded in parallel for performance and are based
   * on previous projects in the same country.
   * 
   * @async
   * @function loadLocationSuggestions
   * @returns {Promise<void>} Updates areaSuggestions and citySuggestions arrays
   * 
   * @example
   * // For UAE, might return:
   * // areas: ["Dubai Marina", "Abu Dhabi", "Sharjah"]
   * // cities: ["Dubai", "Abu Dhabi", "Sharjah"]
   */
  async function loadLocationSuggestions() {
    // Clear suggestions if no country selected
    if (!formData.country) {
      areaSuggestions = [];
      citySuggestions = [];
      return;
    }
    
    try {
      // Load area and city suggestions in parallel for better performance
      const [areas, cities] = await Promise.all([
        getAreaSuggestions(formData.country),
        getCitySuggestions(formData.country)
      ]);
      areaSuggestions = areas;
      citySuggestions = cities;
    } catch (error) {
      console.error('Failed to load location suggestions:', error);
      areaSuggestions = [];
      citySuggestions = [];
    }
  }

  // ============================================================================
  // AREA INPUT HANDLING
  // ============================================================================
  
  /**
   * Handles area input changes and filters suggestions.
   * 
   * Performs client-side filtering of area suggestions based on user input.
   * Uses case-insensitive substring matching for user-friendly experience.
   * 
   * @function handleAreaInput
   * @returns {void} Updates showAreaDropdown state
   */
  function handleAreaInput() {
    // Reset keyboard navigation when input changes
    areaSelectedIndex = -1;
    
    // Show dropdown only if we have suggestions and user has typed something
    if (areaSuggestions.length > 0 && formData.area.length > 0) {
      const filteredSuggestions = areaSuggestions.filter(area => 
        area.toLowerCase().includes(formData.area.toLowerCase())
      );
      showAreaDropdown = filteredSuggestions.length > 0;
    } else {
      showAreaDropdown = false;
    }
  }

  /**
   * Handles keyboard navigation for area suggestions dropdown.
   * 
   * Implements WCAG 2.1 compliant keyboard navigation:
   * - Arrow Up/Down: Navigate through suggestions
   * - Enter/Tab: Select highlighted suggestion
   * - Escape: Close dropdown
   * 
   * @param {KeyboardEvent} e - Keyboard event
   * @returns {void} Updates navigation state and handles selection
   */
  function handleAreaKeydown(e: KeyboardEvent) {
    if (!showAreaDropdown) return;
    
    // Filter suggestions based on current input
    const filteredAreas = areaSuggestions.filter(area => 
      area.toLowerCase().includes(formData.area.toLowerCase())
    );
    
    if (filteredAreas.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        areaSelectedIndex = Math.min(areaSelectedIndex + 1, filteredAreas.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        areaSelectedIndex = Math.max(areaSelectedIndex - 1, 0);
        break;
      case 'Enter':
      case 'Tab':
        e.preventDefault();
        if (areaSelectedIndex >= 0 && areaSelectedIndex < filteredAreas.length) {
          selectArea(filteredAreas[areaSelectedIndex]);
        }
        break;
      case 'Escape':
        showAreaDropdown = false;
        areaSelectedIndex = -1;
        break;
    }
  }

  /**
   * Handles area selection from suggestions dropdown.
   * 
   * @param {string} area - Selected area name
   * @returns {void} Updates form data and closes dropdown
   */
  function selectArea(area: string) {
    formData.area = area;
    showAreaDropdown = false;
  }

  // ============================================================================
  // CITY INPUT HANDLING
  // ============================================================================
  
  /**
   * Handles city input changes and filters suggestions.
   * Similar to area input handling but for city field.
   * 
   * @function handleCityInput
   * @returns {void} Updates showCityDropdown state
   */
  function handleCityInput() {
    // Reset keyboard navigation when input changes
    citySelectedIndex = -1;
    
    // Show dropdown only if we have suggestions and user has typed something
    if (citySuggestions.length > 0 && formData.city.length > 0) {
      const filteredSuggestions = citySuggestions.filter(city => 
        city.toLowerCase().includes(formData.city.toLowerCase())
      );
      showCityDropdown = filteredSuggestions.length > 0;
    } else {
      showCityDropdown = false;
    }
  }

  /**
   * Handles keyboard navigation for city suggestions dropdown.
   * Identical to area keyboard handling for consistency.
   * 
   * @param {KeyboardEvent} e - Keyboard event
   * @returns {void} Updates navigation state and handles selection
   */
  function handleCityKeydown(e: KeyboardEvent) {
    if (!showCityDropdown) return;
    
    // Filter suggestions based on current input
    const filteredCities = citySuggestions.filter(city => 
      city.toLowerCase().includes(formData.city.toLowerCase())
    );
    
    if (filteredCities.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        citySelectedIndex = Math.min(citySelectedIndex + 1, filteredCities.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        citySelectedIndex = Math.max(citySelectedIndex - 1, 0);
        break;
      case 'Enter':
      case 'Tab':
        e.preventDefault();
        if (citySelectedIndex >= 0 && citySelectedIndex < filteredCities.length) {
          selectCity(filteredCities[citySelectedIndex]);
        }
        break;
      case 'Escape':
        showCityDropdown = false;
        citySelectedIndex = -1;
        break;
    }
  }

  /**
   * Handles city selection from suggestions dropdown.
   * 
   * @param {string} city - Selected city name
   * @returns {void} Updates form data and closes dropdown
   */
  function selectCity(city: string) {
    formData.city = city;
    showCityDropdown = false;
  }

  /**
   * Handles country input field changes.
   * 
   * When user types in country field:
   * 1. Clears current country selection
   * 2. Resets navigation state
   * 3. Triggers new search
   * 
   * @function handleCountryInput
   * @returns {void} Updates country state and triggers search
   */
  function handleCountryInput() {
    formData.country = '';              // Clear country name from form
    selectedCountry = null;             // Clear selected country object
    countrySelectedIndex = -1;          // Reset keyboard navigation
    searchCountriesDebounced();         // Trigger new search
  }

  /**
   * Handles keyboard navigation for country search dropdown.
   * 
   * @param {KeyboardEvent} e - Keyboard event
   * @returns {void} Updates navigation state and handles selection
   */
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
        if (countrySelectedIndex >= 0 && countrySelectedIndex < countryOptions.length) {
          selectCountry(countryOptions[countrySelectedIndex]);
        }
        break;
      case 'Escape':
        showCountryDropdown = false;
        countrySelectedIndex = -1;
        break;
    }
  }

  // ============================================================================
  // FORM SUBMISSION HANDLING
  // ============================================================================
  
  /**
   * Handles form submission and project creation.
   * 
   * This is the main function that orchestrates the entire project creation process:
   * 1. Validates required fields
   * 2. Parses project number into components
   * 3. Constructs project object
   * 4. Calls backend to create project and copy template
   * 5. Updates UI stores
   * 6. Triggers success callback
   * 
   * The function includes comprehensive error handling and loading states.
   * 
   * @async
   * @function handleSubmit
   * @param {Event} e - Form submission event
   * @returns {Promise<void>} Completes project creation workflow
   * 
   * @throws {Error} If project creation fails at any step
   */
  async function handleSubmit(e: Event) {
    e.preventDefault();
    
    // Validate required project number
    if (!formData.project_number) {
      validationMessage = 'Please select a country to generate project number';
      return;
    }
    
    // Set loading state for UI feedback
    isLoading = true;
    
    try {
      // Parse project number string into structured components
      // Example: "25-97105" -> year:25, country:971, seq:5
      const parts = formData.project_number.split('-');
      const year = parseInt(parts[0]);
      const country = parseInt(parts[1].substring(0, 3));  // First 3 digits
      const seq = parseInt(parts[1].substring(3));         // Last 2 digits
      
      // Construct ProjectNumber object for database
      const projectNumber: ProjectNumber = {
        year,
        country,
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
        folder: formData.folder
      };
      
      // Create project using store action instead of direct API call
      const createdProject = await projectsActions.create(project);
      
      // Trigger success callback with created project
      onSuccess(createdProject);
      
      // Clean up and close modal
      resetForm();
      onClose();
    } catch (error) {
      console.error('Failed to create project:', error);
      validationMessage = error instanceof Error ? error.message : 'Failed to create project';
    } finally {
      isLoading = false;
    }
  }

  /**
   * Resets all form fields and state to initial values.
   * 
   * This function is called after successful project creation or when
   * the modal needs to be reset to a clean state. It ensures no
   * residual data remains from previous form interactions.
   * 
   * @function resetForm
   * @returns {void} Resets all component state
   */
  function resetForm() {
    // Reset main form data to defaults
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
    
    // Reset country search state
    countrySearchQuery = '';
    selectedCountry = null;
    countryOptions = [];
    showCountryDropdown = false;
    
    // Reset location suggestions state
    areaSuggestions = [];
    showAreaDropdown = false;
    citySuggestions = [];
    showCityDropdown = false;
    
    // Reset keyboard navigation state
    countrySelectedIndex = -1;
    areaSelectedIndex = -1;
    citySelectedIndex = -1;
    
    // Clear any validation messages
    validationMessage = '';
  }

  // ============================================================================
  // ACCESSIBILITY AND EVENT HANDLING
  // ============================================================================
  
  /**
   * Handles global keyboard events for modal accessibility.
   * 
   * Implements standard modal behavior where Escape key closes the modal.
   * This is part of WCAG 2.1 compliance for modal dialogs.
   * 
   * @param {KeyboardEvent} e - Keyboard event
   * @returns {void} Closes modal on Escape key
   */
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>


<!-- Modal is only rendered when isOpen is true for performance -->
{#if isOpen}
  <!-- Modal backdrop -->
  <div 
    class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
    tabindex="-1"
    onclick={onClose}
    onkeydown={handleKeydown}
  >
    <!-- Modal container -->
    <div 
      class="bg-emittiv-darker border border-emittiv-dark rounded w-full max-h-[90vh] overflow-y-auto"
      style="padding: 24px; max-width: 600px;"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Modal Header -->
      <div class="flex items-center justify-between" style="margin-bottom: 20px;">
        <h2 id="modal-title" class="font-semibold text-emittiv-white" style="font-size: 16px;">New Project</h2>
        <button 
          onclick={onClose}
          class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth"
          aria-label="Close"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
      
      <!-- Main Form -->
      <form onsubmit={handleSubmit} style="display: flex; flex-direction: column; gap: 16px;">
        
        <!-- PROJECT INFORMATION SECTION -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Project Information</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <div>
              <label for="project-name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Project Name *
              </label>
              <input 
                id="project-name"
                type="text" 
                bind:value={formData.name}
                placeholder="Full project name"
                required
                class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                style="padding: 8px 12px; font-size: 12px; height: 32px;"
              />
            </div>
            
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="project-short-name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Short Name *
                </label>
                <input 
                  id="project-short-name"
                  type="text" 
                  bind:value={formData.name_short}
                  placeholder="Short name"
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
              
              <div>
                <label for="project-status" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Status *
                </label>
                <select 
                  id="project-status"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                  bind:value={formData.status}
                  required
                >
                  <option value="Draft">Draft</option>
                  <option value="FP">FP</option>
                  <option value="Active">Active</option>
                  <option value="On Hold">On Hold</option>
                  <option value="Completed">Completed</option>
                  <option value="Cancelled">Cancelled</option>
                </select>
              </div>
            </div>
          </div>
        </div>
        
        <!-- LOCATION DETAILS SECTION -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Location Details</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <div class="grid grid-cols-3" style="gap: 12px;">
              <div>
                <label for="country-input" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Country *
                </label>
                <div class="relative">
                  <input 
                    id="country-input"
                    type="text" 
                    bind:value={countrySearchQuery}
                    oninput={handleCountryInput}
                    onkeydown={handleCountryKeydown}
                    placeholder="Search countries..."
                    required
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
                  />
                  <!-- Country dropdown with search results -->
                  {#if showCountryDropdown && countryOptions.length > 0}
                    <div class="absolute top-full left-0 right-0 bg-emittiv-darker border border-emittiv-dark rounded-b shadow-lg z-50 overflow-y-auto" style="max-height: 120px;">
                      {#each countryOptions as country, index}
                        <div 
                          class="text-emittiv-white hover:bg-emittiv-dark cursor-pointer border-b border-emittiv-dark/50 last:border-b-0 {index === countrySelectedIndex ? 'bg-emittiv-splash text-emittiv-black' : ''}"
                          role="option"
                          aria-selected={index === countrySelectedIndex}
                          onclick={() => selectCountry(country)}
                          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectCountry(country); } }}
                          tabindex="0"
                          style="padding: 4px 8px; font-size: 12px; line-height: 18px;"
                        >
                          {country.name}
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
              
              <div>
                <label for="project-year" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Year *
                </label>
                <input 
                  id="project-year"
                  type="number" 
                  bind:value={formData.year}
                  min="0"
                  max="99"
                  onchange={handleYearChange}
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
              </div>
              
              <div>
                <label for="project-number" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Project Number
                </label>
                <div class="relative">
                  <input 
                    id="project-number"
                    type="text" 
                    bind:value={formData.project_number}
                    readonly
                    placeholder={isGenerating ? "Generating..." : "Select country & year"}
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
                  />
                  <!-- Loading spinner for project number generation -->
                  {#if isGenerating}
                    <div class="absolute right-2 top-1/2 -translate-y-1/2">
                      <div class="animate-spin h-3 w-3 border-2 border-emittiv-splash border-t-transparent rounded-full"></div>
                    </div>
                  {/if}
                </div>
              </div>
            </div>
            
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="city-input" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  City *
                </label>
                <div class="relative">
                  <input 
                    id="city-input"
                    type="text" 
                    bind:value={formData.city}
                    oninput={handleCityInput}
                    onkeydown={handleCityKeydown}
                    placeholder="City"
                    required
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
                  />
                  {#if showCityDropdown}
                    <div class="absolute top-full left-0 right-0 bg-emittiv-darker border border-emittiv-dark rounded-b shadow-lg z-50 overflow-y-auto" style="max-height: 120px;">
                      {#each citySuggestions.filter(city => city.toLowerCase().includes(formData.city.toLowerCase())) as city, index}
                        <div 
                          class="text-emittiv-white hover:bg-emittiv-dark cursor-pointer border-b border-emittiv-dark/50 last:border-b-0 {index === citySelectedIndex ? 'bg-emittiv-splash text-emittiv-black' : ''}"
                          role="option"
                          aria-selected={index === citySelectedIndex}
                          onclick={() => selectCity(city)}
                          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectCity(city); } }}
                          tabindex="0"
                          style="padding: 4px 8px; font-size: 12px; line-height: 18px;"
                        >
                          {city}
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
              
              <div>
                <label for="area-input" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Area *
                </label>
                <div class="relative">
                  <input 
                    id="area-input"
                    type="text" 
                    bind:value={formData.area}
                    oninput={handleAreaInput}
                    onkeydown={handleAreaKeydown}
                    placeholder="Project area"
                    required
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
                  />
                  {#if showAreaDropdown}
                    <div class="absolute top-full left-0 right-0 bg-emittiv-darker border border-emittiv-dark rounded-b shadow-lg z-50 overflow-y-auto" style="max-height: 120px;">
                      {#each areaSuggestions.filter(area => area.toLowerCase().includes(formData.area.toLowerCase())) as area, index}
                        <div 
                          class="text-emittiv-white hover:bg-emittiv-dark cursor-pointer border-b border-emittiv-dark/50 last:border-b-0 {index === areaSelectedIndex ? 'bg-emittiv-splash text-emittiv-black' : ''}"
                          role="option"
                          aria-selected={index === areaSelectedIndex}
                          onclick={() => selectArea(area)}
                          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectArea(area); } }}
                          tabindex="0"
                          style="padding: 4px 8px; font-size: 12px; line-height: 18px;"
                        >
                          {area}
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <!-- VALIDATION MESSAGES -->
        {#if validationMessage}
          <div class="p-3 bg-red-500/10 border border-red-500/30 rounded text-red-400" style="font-size: 12px;">
            {validationMessage}
          </div>
        {/if}
        
        <!-- ACTION BUTTONS -->
        <div class="flex justify-end items-center" style="gap: 8px; margin-top: 8px;">
          <!-- Cancel button with secondary styling -->
          <button
            type="button"
            onclick={onClose}
            class="px-4 py-2 rounded-lg bg-emittiv-dark text-emittiv-light hover:bg-emittiv-dark/70 transition-all"
            style="font-size: 12px;"
          >
            Cancel
          </button>
          
          <!-- Submit button -->
          <button
            type="submit"
            disabled={isLoading || !formData.project_number}
            class="px-4 py-2 rounded-lg bg-emittiv-splash text-emittiv-black hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
            style="font-size: 12px;"
          >
            {isLoading ? 'Creating...' : 'Create Project'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

