<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import CompanyModal from '$lib/components/CompanyModal.svelte';
  import CompanyDetail from '$lib/components/CompanyDetail.svelte';
  import { companiesStore, companiesActions } from '$lib/stores';
  import { createFilterFunction, getUniqueFieldValues, hasActiveFilters, clearAllFilters, type FilterConfig } from '$lib/utils/filters';
  import { onMount } from 'svelte';
  import type { Company } from '../types';
  
  // Filter states
  let searchQuery = '';
  let filters = {
    country: '',
    city: ''
  };
  
  // Filter configuration for companies
  const filterConfig: FilterConfig<Company> = {
    searchFields: ['name', 'name_short', 'abbreviation', 'city', 'country'],
    filterFields: {
      country: (company) => company.country,
      city: (company) => company.city
    },
    sortFunction: (a, b) => new Date(b.time.updated_at).getTime() - new Date(a.time.updated_at).getTime()
  };

  // Reactive filtered companies using optimized filter function
  $: filteredCompanies = createFilterFunction($companiesStore, searchQuery, filters, filterConfig);
  
  // Get unique values for filters using optimized functions
  $: uniqueCountries = getUniqueFieldValues($companiesStore, (company) => company.country).filter(Boolean);
  $: uniqueCities = getUniqueFieldValues($companiesStore, (company) => company.city).filter(Boolean);
  
  // Modal states
  let isCompanyModalOpen = false;
  let isCompanyDetailOpen = false;
  let selectedCompany: Company | null = null;
  let modalMode: 'create' | 'edit' = 'create';
  
  function handleAddCompany() {
    selectedCompany = null;
    modalMode = 'create';
    isCompanyModalOpen = true;
  }
  
  function handleEditCompany(company: Company) {
    selectedCompany = company;
    modalMode = 'edit';
    isCompanyModalOpen = true;
  }
  
  function handleViewCompany(company: Company) {
    selectedCompany = company;
    isCompanyDetailOpen = true;
  }
  
  function handleCloseModal() {
    isCompanyModalOpen = false;
    selectedCompany = null;
    // Refresh companies list after modal closes
    companiesActions.load();
  }
  
  function handleCloseDetail() {
    isCompanyDetailOpen = false;
    selectedCompany = null;
  }
  
  function handleEditFromDetail(event: CustomEvent) {
    // Close detail view and open edit modal
    isCompanyDetailOpen = false;
    selectedCompany = event.detail;
    modalMode = 'edit';
    isCompanyModalOpen = true;
  }
  
  function clearFilters() {
    searchQuery = clearAllFilters(filters);
  }
  
  // Load companies on mount
  onMount(() => {
    companiesActions.load();
  });
  
  // Check if any filters are active
  $: hasFiltersActive = hasActiveFilters(filters, searchQuery);
</script>

<div class="p-8">
  <!-- Search and Add Button Row -->
  <div class="flex justify-between items-center mb-6">
    <div class="flex-1 max-w-2xl">
      <div class="flex items-center gap-2">
        <div class="relative flex-1">
          <input
            type="text"
            placeholder="Search companies..."
            bind:value={searchQuery}
            class="w-full px-2 py-2.5 bg-emittiv-darker border border-emittiv-dark rounded-lg text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
          />
        </div>
        <button 
          class="p-2.5 bg-emittiv-darker border border-emittiv-dark rounded-lg text-emittiv-light hover:text-emittiv-white hover:border-emittiv-splash transition-all"
          aria-label="Search"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </button>
      </div>
    </div>
    <button
      class="ml-4 w-12 h-12 rounded-full bg-emittiv-splash hover:bg-orange-600 text-emittiv-black flex items-center justify-center transition-smooth hover:scale-105 active:scale-95 shadow-lg"
      on:click={handleAddCompany}
      aria-label="Add new company"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
  </div>
  
  <!-- Filter Options -->
  <div class="flex flex-wrap items-center gap-2 mb-2">
    <!-- Country Filter -->
    <select 
      bind:value={filters.country} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Countries</option>
      {#each uniqueCountries as country}
        <option value={country}>{country}</option>
      {/each}
    </select>
    
    <!-- City Filter -->
    <select 
      bind:value={filters.city} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Cities</option>
      {#each uniqueCities as city}
        <option value={city}>{city}</option>
      {/each}
    </select>
  </div>

<style>
  /* Custom styles for native select dropdowns */
  select {
    background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3e%3cpath stroke='%23999' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3e%3c/svg%3e");
    background-position: right 0.25rem center;
    background-repeat: no-repeat;
    background-size: 16px 12px;
    appearance: none;
  }
</style>
  
  <!-- Results count and Clear button -->
  <div class="flex justify-between items-center mb-2">
    <div class="px-2 py-0.5 text-xs text-emittiv-light border border-emittiv-dark rounded bg-emittiv-darker">
      {#if hasFiltersActive}
        Showing {filteredCompanies.length} of {$companiesStore.length} companies
      {:else if $companiesStore.length > 0}
        {$companiesStore.length} compan{$companiesStore.length === 1 ? 'y' : 'ies'}
      {/if}
    </div>
    {#if hasFiltersActive}
      <button 
        on:click={clearFilters}
        class="px-2 py-0.5 text-xs text-emittiv-light hover:text-emittiv-white border border-emittiv-dark hover:border-emittiv-light rounded bg-emittiv-darker transition-smooth flex items-center gap-1"
      >
        <svg style="width: 18px; height: 18px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
        Clear
      </button>
    {/if}
  </div>
  
  {#if $companiesStore.length === 0}
    <EmptyState 
      icon="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"
      title="No Companies Yet"
      description="Add your first company to start organizing your clients and managing relationships."
      actionText="Add Company"
      onAction={handleAddCompany}
    />
  {:else if filteredCompanies.length === 0}
    <div class="text-center py-12">
      <svg class="w-16 h-16 mx-auto mb-4 text-emittiv-light opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <h3 class="text-lg font-medium text-emittiv-light mb-2">No companies found</h3>
      <p class="text-emittiv-light opacity-60 mb-4">Try adjusting your search or filters</p>
      <button 
        on:click={clearFilters}
        class="text-sm text-emittiv-splash hover:text-orange-400 transition-smooth"
      >
        Clear all filters
      </button>
    </div>
  {:else}
    <div class="grid gap-6">
      {#each filteredCompanies as company}
        <Card>
          <div>
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-lg font-semibold text-emittiv-white truncate">{company.name}</h3>
              <div class="flex items-center space-x-1 ml-4 flex-shrink-0">
                <span class="px-2 py-1 rounded-full text-xs font-medium text-blue-400 bg-blue-400/10">
                  {company.abbreviation}
                </span>
                <button on:click={() => handleEditCompany(company)} class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth" aria-label="Edit company">
                  <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                  </svg>
                </button>
                <button on:click={() => handleViewCompany(company)} class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth" aria-label="View company details">
                  <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                </button>
              </div>
            </div>
            <p class="text-emittiv-lighter mb-2 truncate">
              {company.city}, {company.country}
            </p>
            <div class="flex items-center justify-between text-sm text-emittiv-light">
              <div class="flex items-center space-x-4">
                <span>Short Name: {company.name_short}</span>
                {#if company.reg_no}
                  <span>Reg: {company.reg_no}</span>
                {/if}
                {#if company.tax_no}
                  <span>VAT: {company.tax_no}</span>
                {/if}
              </div>
              <span>Created: {new Date(company.time.created_at).toLocaleDateString()}</span>
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<!-- Company Modal -->
<CompanyModal 
  bind:isOpen={isCompanyModalOpen}
  company={selectedCompany}
  mode={modalMode}
  on:close={handleCloseModal}
/>

<!-- Company Detail View -->
<CompanyDetail 
  bind:isOpen={isCompanyDetailOpen}
  company={selectedCompany}
  on:close={handleCloseDetail}
  on:edit={handleEditFromDetail}
/>