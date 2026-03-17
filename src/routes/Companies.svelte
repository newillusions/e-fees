<script lang="ts">
  import EmptyState from '$lib/components/EmptyState.svelte';
  import CompanyModal from '$lib/components/CompanyModal.svelte';
  import CompanyDetail from '$lib/components/CompanyDetail.svelte';
  import CompanyCard from '$lib/components/CompanyCard.svelte';
  import BulkActionBar from '$lib/components/BulkActionBar.svelte';
  import ResultsCounter from '$lib/components/ResultsCounter.svelte';
  import { paginatedCompaniesStore, companiesActions } from '$lib/stores';
  import type { PaginatedStoreState } from '$lib/stores/pagination';
  import DateRangeFilter from '$lib/components/DateRangeFilter.svelte';
  import { createFilterFunction, getUniqueFieldValues, hasActiveFilters, clearAllFilters } from '$lib/utils/filters';
  import type { AdvancedFilters } from '$lib/utils/filters';
  import { createCompanyFilterConfig } from '$lib/utils/search';
  import { extractIdFromRelation } from '$lib/utils/surrealdb';
  import { batchDeleteEntities } from '$lib/api/batch';
  import { logApiError } from '$lib/services/logger';
  import { onMount } from 'svelte';
  import { SvelteSet } from 'svelte/reactivity';
  import type { Company } from '../types';

  // Filter states
  let searchQuery = $state('');
  let filters = $state({
    country: '',
    city: ''
  });

  // Advanced filter states
  let dateFrom = $state('');
  let dateTo = $state('');

  // Bulk selection state
  let selectedIds: Set<string> = new SvelteSet();
  let selectMode = $state(false);

  function toggleSelect(id: string) {
    const next = new SvelteSet(selectedIds);
    if (next.has(id)) next.delete(id); else next.add(id);
    selectedIds = next;
    if (next.size === 0) selectMode = false;
  }

  function clearSelection() {
    selectedIds = new SvelteSet();
    selectMode = false;
  }

  async function handleBulkDelete() {
    const ids = [...selectedIds];
    try {
      await batchDeleteEntities('companies', ids);
      clearSelection();
      paginatedCompaniesStore.actions.refresh();
      companiesActions.load();
    } catch (e) {
      logApiError('bulk delete companies', e as Error);
    }
  }

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
    return {
      ...baseConfig,
      filterFields: {
        country: (company: Company) => company.country,
        city: (company: Company) => company.city
      },
      dateFieldExtractor: (company: Company) => company.time?.updated_at || '',
      dateFieldFormat: 'iso' as const,
    };
  })();

  // Build advanced filters from state
  const advanced: AdvancedFilters = $derived({
    dateRange: { from: dateFrom, to: dateTo }
  });

  // Reactive filtered companies using optimized filter function
  const filteredCompanies = $derived(createFilterFunction(companies, searchQuery, filters, filterConfig, advanced));

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
    dateFrom = '';
    dateTo = '';
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
  const hasFiltersActive = $derived(hasActiveFilters(filters, searchQuery, advanced));
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
      class="emittiv-btn emittiv-btn--sm {selectMode ? 'emittiv-btn--primary' : 'emittiv-btn--secondary'} flex-shrink-0"
      onclick={() => { selectMode = !selectMode; if (!selectMode) clearSelection(); }}
      aria-label="Toggle selection mode"
      title="Multi-select"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
      </svg>
    </button>
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

    <DateRangeFilter bind:from={dateFrom} bind:to={dateTo} />
  </div>
  
  
  {#if isLoading && companies.length === 0}
    <!-- Initial loading state -->
    <div class="flex flex-col items-center justify-center py-12">
      <div class="emittiv-spinner emittiv-spinner--page"></div>
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
        class="emittiv-link text-sm"
      >
        Clear all filters
      </button>
    </div>
  {:else}
    <!-- Bulk Action Bar -->
    <BulkActionBar
      selectedCount={selectedIds.size}
      entityType="companies"
      on:delete={handleBulkDelete}
      on:clear={clearSelection}
    />

    <!-- Scrollable container for infinite scroll -->
    <div
      bind:this={scrollContainer}
      class="grid gap-3 max-h-scroll overflow-y-auto pr-2 pt-1"
    >
      {#each filteredCompanies as company}
        <CompanyCard
          {company}
          selectable={selectMode}
          selected={selectedIds.has(extractIdFromRelation(company.id || ''))}
          onedit={(company) => handleEditCompany(company)}
          onview={(company) => handleViewCompany(company)}
          on:select={() => toggleSelect(extractIdFromRelation(company.id || ''))}
        />
      {/each}

      <!-- Footer indicator -->
      {#if companies.length > 0}
        <div class="text-center py-4 text-emittiv-light text-xs opacity-60">
          {#if isLoading && companies.length > 0}
            <div class="flex items-center justify-center gap-2">
              <div class="emittiv-spinner"></div>
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