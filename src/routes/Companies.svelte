<script lang="ts">
  import EmptyState from '$lib/components/EmptyState.svelte';
  import CompanyModal from '$lib/components/CompanyModal.svelte';
  import CompanyDetail from '$lib/components/CompanyDetail.svelte';
  import CompanyCard from '$lib/components/CompanyCard.svelte';
  import ResultsCounter from '$lib/components/ResultsCounter.svelte';
  import { paginatedCompaniesStore, companiesActions } from '$lib/stores';
  import type { PaginatedStoreState } from '$lib/stores/pagination';
  import { createFilterFunction, getUniqueFieldValues, hasActiveFilters, clearAllFilters } from '$lib/utils/filters';
  import { createCompanyFilterConfig } from '$lib/utils/search';
  import { onMount } from 'svelte';
  import type { Company } from '../types';

  // Filter states
  let searchQuery = $state('');
  let filters = $state({
    country: '',
    city: ''
  });

  // Scroll container ref for infinite scroll
  let scrollContainer: HTMLDivElement | null = $state(null);

  // Pagination state - synced from store via $effect
  let companies: Company[] = $state([]);
  let isLoading = $state(false);
  let hasMore = $state(true);
  let totalRecords = $state(0);
  let initialized = $state(false);

  // Effect to sync paginated store state to local runes
  $effect(() => {
    const unsubscribe = paginatedCompaniesStore.store.subscribe((state: PaginatedStoreState<Company>) => {
      companies = state.items;
      isLoading = state.pagination.isLoading;
      hasMore = state.pagination.hasMore;
      totalRecords = state.pagination.totalRecords;
      initialized = state.initialized;
    });
    return unsubscribe;
  });

  // Scroll handler for infinite scroll
  function handleScroll() {
    if (!scrollContainer || isLoading || !hasMore) return;

    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    const scrollPercentage = (scrollTop + clientHeight) / scrollHeight;

    // Load more when scrolled past 80%
    if (scrollPercentage >= 0.8) {
      paginatedCompaniesStore.actions.loadNextPage();
    }
  }

  // Set up scroll listener when container is available
  $effect(() => {
    if (scrollContainer) {
      scrollContainer.addEventListener('scroll', handleScroll);
      return () => scrollContainer?.removeEventListener('scroll', handleScroll);
    }
  });

  // Filter configuration for companies - uses unified search module
  const filterConfig = (() => {
    const baseConfig = createCompanyFilterConfig();
    // Add filter fields for dropdowns
    return {
      ...baseConfig,
      filterFields: {
        country: (company: Company) => company.country,
        city: (company: Company) => company.city
      }
    };
  })();

  // Reactive filtered companies using optimized filter function
  const filteredCompanies = $derived(createFilterFunction(companies, searchQuery, filters, filterConfig));

  // Get unique values for filters using optimized functions
  const uniqueCountries = $derived(getUniqueFieldValues(companies, (company) => company.country).filter(Boolean));
  const uniqueCities = $derived(getUniqueFieldValues(companies, (company) => company.city).filter(Boolean));
  
  // Modal states
  let isCompanyModalOpen = $state(false);
  let isCompanyDetailOpen = $state(false);
  let selectedCompany: Company | null = $state(null);
  let modalMode: 'create' | 'edit' = $state('create');
  
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
    // Check store state directly to avoid race condition with $effect subscription
    const storeState = paginatedCompaniesStore.actions.getState();
    if (!storeState.initialized) {
      paginatedCompaniesStore.actions.loadInitialPage();
    }
  });

  // Check if any filters are active
  const hasFiltersActive = $derived(hasActiveFilters(filters, searchQuery));
</script>

<div class="p-8">
  <!-- Search, Counter, and Add Button Row -->
  <div class="flex justify-between items-center mb-6 gap-4">
    <div class="flex items-center gap-2 flex-1">
      <div class="relative flex-1 max-w-md">
        <input
          type="text"
          placeholder="Search companies..."
          bind:value={searchQuery}
          class="emittiv-search-input"
        />
      </div>
      <button
        class="emittiv-search-button"
        aria-label="Search"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
      </button>
      <ResultsCounter
        totalItems={totalRecords}
        filteredItems={filteredCompanies.length}
        hasFilters={hasFiltersActive}
        entityName="companies"
        loadedItems={companies.length}
        hasMore={hasMore}
        inline={true}
        on:clear-filters={clearFilters}
      />
    </div>
    <button
      class="emittiv-fab flex-shrink-0"
      onclick={handleAddCompany}
      aria-label="Add new company"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
  </div>
  
  <!-- Filter Options -->
  <div class="flex flex-wrap items-center gap-2 mb-4">
    <!-- Country Filter -->
    <select 
      bind:value={filters.country} 
      class="emittiv-filter-select"
    >
      <option value="">All Countries</option>
      {#each uniqueCountries as country}
        <option value={country}>{country}</option>
      {/each}
    </select>
    
    <!-- City Filter -->
    <select 
      bind:value={filters.city} 
      class="emittiv-filter-select"
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
  
  
  {#if isLoading && companies.length === 0}
    <!-- Initial loading state -->
    <div class="flex flex-col items-center justify-center py-12">
      <div class="animate-spin rounded-full h-8 w-8 border-2 border-emittiv-splash border-t-transparent mb-4"></div>
      <p class="text-emittiv-light text-sm">Loading companies...</p>
    </div>
  {:else if companies.length === 0}
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
        onclick={clearFilters}
        class="text-sm text-emittiv-splash hover:text-orange-400 transition-smooth"
      >
        Clear all filters
      </button>
    </div>
  {:else}
    <!-- Scrollable container for infinite scroll -->
    <div
      bind:this={scrollContainer}
      class="grid gap-3 max-h-scroll overflow-y-auto pr-2 pt-1"
    >
      {#each filteredCompanies as company}
        <CompanyCard
          {company}
          on:edit={(e) => handleEditCompany(e.detail)}
          on:view={(e) => handleViewCompany(e.detail)}
        />
      {/each}

      <!-- Footer indicator -->
      {#if companies.length > 0}
        <div class="text-center py-4 text-emittiv-light text-xs opacity-60">
          {#if isLoading && companies.length > 0}
            <div class="flex items-center justify-center gap-2">
              <div class="animate-spin rounded-full h-4 w-4 border-2 border-emittiv-splash border-t-transparent"></div>
              <span>Loading more...</span>
            </div>
          {:else if hasMore}
            <span>Showing {companies.length} of {totalRecords} companies</span>
          {:else}
            <span>{totalRecords} companies</span>
          {/if}
        </div>
      {/if}
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
  isOpen={isCompanyDetailOpen}
  company={selectedCompany}
  on:close={handleCloseDetail}
  on:edit={handleEditFromDetail}
/>