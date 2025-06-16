<script lang="ts">
  import { onMount } from 'svelte';
  import Card from './Card.svelte';
  import Button from './Button.svelte';
  import Input from './Input.svelte';
  import { generateNextProjectNumber, validateProjectNumber, createProjectWithTemplate, searchCountries, getAreaSuggestions, getCitySuggestions } from '$lib/api';
  import type { Project, ProjectNumber } from '../../types';
  import { projectStore } from '$lib/stores/data';

  let { isOpen = $bindable(false), onClose, onSuccess }: {
    isOpen?: boolean;
    onClose: () => void;
    onSuccess: (project: Project) => void;
  } = $props();

  let isLoading = $state(false);
  let isGenerating = $state(false);
  let validationMessage = $state('');
  let countrySearchQuery = $state('');
  let countryOptions = $state([]);
  let showCountryDropdown = $state(false);
  let selectedCountry = $state(null);
  
  let areaSuggestions = $state([]);
  let showAreaDropdown = $state(false);
  let citySuggestions = $state([]);
  let showCityDropdown = $state(false);
  
  // Keyboard navigation state
  let countrySelectedIndex = $state(-1);
  let areaSelectedIndex = $state(-1);
  let citySelectedIndex = $state(-1);
  
  let formData = $state({
    name: '',
    name_short: '',
    area: '',
    city: '',
    country: '',
    status: 'Draft',
    year: new Date().getFullYear() % 100,
    project_number: ''
  });

  async function generateProjectNumber() {
    if (!formData.country || !formData.year) {
      formData.project_number = '';
      return;
    }
    
    isGenerating = true;
    validationMessage = '';
    
    try {
      const projectNumber = await generateNextProjectNumber(formData.country, formData.year);
      formData.project_number = projectNumber;
      
      // Validate the generated number
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

  async function handleCountryChange() {
    await generateProjectNumber();
  }

  async function handleYearChange() {
    await generateProjectNumber();
  }

  async function searchCountriesDebounced() {
    if (countrySearchQuery.length < 2) {
      countryOptions = [];
      showCountryDropdown = false;
      return;
    }
    
    try {
      const results = await searchCountries(countrySearchQuery);
      countryOptions = results;
      showCountryDropdown = results.length > 0;
    } catch (error) {
      console.error('Failed to search countries:', error);
      countryOptions = [];
      showCountryDropdown = false;
    }
  }

  function selectCountry(country: any) {
    selectedCountry = country;
    countrySearchQuery = country.name;
    formData.country = country.name; // Use name instead of code
    showCountryDropdown = false;
    handleCountryChange();
    loadLocationSuggestions();
  }

  async function loadLocationSuggestions() {
    if (!formData.country) {
      areaSuggestions = [];
      citySuggestions = [];
      return;
    }
    
    try {
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

  function handleAreaInput() {
    areaSelectedIndex = -1;
    if (areaSuggestions.length > 0 && formData.area.length > 0) {
      const filteredSuggestions = areaSuggestions.filter(area => 
        area.toLowerCase().includes(formData.area.toLowerCase())
      );
      showAreaDropdown = filteredSuggestions.length > 0;
    } else {
      showAreaDropdown = false;
    }
  }

  function handleAreaKeydown(e: KeyboardEvent) {
    if (!showAreaDropdown) return;
    
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

  function selectArea(area: string) {
    formData.area = area;
    showAreaDropdown = false;
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

  function handleCityKeydown(e: KeyboardEvent) {
    if (!showCityDropdown) return;
    
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

  function selectCity(city: string) {
    formData.city = city;
    showCityDropdown = false;
  }

  function handleCountryInput() {
    formData.country = '';
    selectedCountry = null;
    countrySelectedIndex = -1;
    searchCountriesDebounced();
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

  async function handleSubmit(e: Event) {
    e.preventDefault();
    
    if (!formData.project_number) {
      validationMessage = 'Please select a country to generate project number';
      return;
    }
    
    isLoading = true;
    
    try {
      // Parse project number to create number object
      const parts = formData.project_number.split('-');
      const year = parseInt(parts[0]);
      const country = parseInt(parts[1].substring(0, 3));
      const seq = parseInt(parts[1].substring(3));
      
      const projectNumber: ProjectNumber = {
        year,
        country,
        seq,
        id: formData.project_number
      };
      
      const project: Partial<Project> = {
        name: formData.name,
        name_short: formData.name_short,
        area: formData.area,
        city: formData.city,
        country: formData.country,
        status: formData.status,
        number: projectNumber,
        folder: `${formData.project_number} ${formData.name_short}`
        // time field omitted - database will auto-manage it
      };
      
      // Create project with template
      const createdProject = await createProjectWithTemplate(project);
      
      // Update store
      projectStore.update(projects => [...projects, createdProject]);
      
      onSuccess(createdProject);
      resetForm();
      onClose();
    } catch (error) {
      console.error('Failed to create project:', error);
      validationMessage = error instanceof Error ? error.message : 'Failed to create project';
    } finally {
      isLoading = false;
    }
  }

  function resetForm() {
    formData = {
      name: '',
      name_short: '',
      area: '',
      city: '',
      country: '',
      status: 'Draft',
      year: new Date().getFullYear() % 100,
      project_number: ''
    };
    countrySearchQuery = '';
    selectedCountry = null;
    countryOptions = [];
    showCountryDropdown = false;
    areaSuggestions = [];
    showAreaDropdown = false;
    citySuggestions = [];
    showCityDropdown = false;
    countrySelectedIndex = -1;
    areaSelectedIndex = -1;
    citySelectedIndex = -1;
    validationMessage = '';
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

{#if isOpen}
  <div 
    class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
    tabindex="-1"
    onclick={onClose}
    onkeydown={handleKeydown}
  >
    <div 
      class="bg-emittiv-darker border border-emittiv-dark rounded w-full max-h-[90vh] overflow-y-auto"
      style="padding: 24px; max-width: 600px;"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
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
      
      <form onsubmit={handleSubmit} style="display: flex; flex-direction: column; gap: 16px;">
        <!-- Project Information Section -->
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
                  Status
                </label>
                <select 
                  id="project-status"
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                  bind:value={formData.status}
                >
                  <option value="Draft">Draft</option>
                  <option value="RFP">RFP</option>
                  <option value="Active">Active</option>
                  <option value="On Hold">On Hold</option>
                  <option value="Completed">Completed</option>
                  <option value="Cancelled">Cancelled</option>
                </select>
              </div>
            </div>
          </div>
        </div>
        
        <!-- Location Section -->
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
        
        {#if validationMessage}
          <div class="p-3 bg-red-500/10 border border-red-500/30 rounded text-red-400" style="font-size: 12px;">
            {validationMessage}
          </div>
        {/if}
        
        <!-- Action Buttons -->
        <div class="flex justify-end items-center" style="gap: 8px; margin-top: 8px;">
          <button
            type="button"
            onclick={onClose}
            class="px-4 py-2 rounded-lg bg-emittiv-dark text-emittiv-light hover:bg-emittiv-dark/70 transition-all"
            style="font-size: 12px;"
          >
            Cancel
          </button>
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

