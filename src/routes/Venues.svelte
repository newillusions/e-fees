<script lang="ts">
  import EmptyState from '$lib/components/EmptyState.svelte';
  import VenueModal from '$lib/components/VenueModal.svelte';
  import ResultsCounter from '$lib/components/ResultsCounter.svelte';
  import { paginatedVenuesStore, venuesActions } from '$lib/stores';
  import type { PaginatedStoreState } from '$lib/stores/pagination';
  import { createFilterFunction, getUniqueFieldValues, hasActiveFilters, clearAllFilters } from '$lib/utils/filters';
  import type { FilterConfig } from '$lib/utils/filters';
  import { onMount } from 'svelte';
  import type { Venue } from '../types';

  // Filter states
  let searchQuery = $state('');
  let filters = $state({
    country: ''
  });

  // Scroll container ref for infinite scroll
  let scrollContainer: HTMLDivElement | null = $state(null);

  // Pagination state - synced from store via $effect
  let venues: Venue[] = $state([]);
  let isLoading = $state(false);
  let hasMore = $state(true);
  let totalRecords = $state(0);
  let initialized = $state(false);

  // Effect to sync paginated store state to local runes
  $effect(() => {
    const unsubscribe = paginatedVenuesStore.store.subscribe((state: PaginatedStoreState<Venue>) => {
      venues = state.items;
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
      paginatedVenuesStore.actions.loadNextPage();
    }
  }

  // Set up scroll listener when container is available
  $effect(() => {
    if (scrollContainer) {
      scrollContainer.addEventListener('scroll', handleScroll);
      return () => scrollContainer?.removeEventListener('scroll', handleScroll);
    }
  });

  // Venue search fields
  const VENUE_SEARCH_FIELDS: (keyof Venue)[] = ['name', 'name_short', 'notes'];

  // Filter configuration for venues
  const filterConfig: FilterConfig<Venue> & { filterFields?: Record<string, (v: Venue) => string> } = {
    searchFields: VENUE_SEARCH_FIELDS,
    customSearchFunctions: [
      (venue: Venue) => venue.location?.city || '',
      (venue: Venue) => venue.location?.country || '',
      (venue: Venue) => venue.location?.area || '',
      (venue: Venue) => (venue.tags || []).join(' '),
    ],
    filterFields: {
      country: (venue: Venue) => venue.location?.country || ''
    }
  };

  // Reactive filtered venues using optimized filter function
  const filteredVenues = $derived(createFilterFunction(venues, searchQuery, filters, filterConfig));

  // Get unique values for filters
  const uniqueCountries = $derived(
    getUniqueFieldValues(venues, (venue) => venue.location?.country || '').filter(Boolean)
  );

  // Modal states
  let isVenueModalOpen = $state(false);
  let selectedVenue: Venue | null = $state(null);
  let modalMode: 'create' | 'edit' = $state('create');

  function handleAddVenue() {
    selectedVenue = null;
    modalMode = 'create';
    isVenueModalOpen = true;
  }

  function handleEditVenue(venue: Venue) {
    selectedVenue = venue;
    modalMode = 'edit';
    isVenueModalOpen = true;
  }

  function handleCloseModal() {
    isVenueModalOpen = false;
    selectedVenue = null;
    // Refresh venues list after modal closes
    venuesActions.load();
  }

  function clearFilters() {
    searchQuery = clearAllFilters(filters);
  }

  // Load venues on mount
  onMount(() => {
    const storeState = paginatedVenuesStore.actions.getState();
    if (!storeState.initialized) {
      paginatedVenuesStore.actions.loadInitialPage();
    }
  });

  // Check if any filters are active
  const hasFiltersActive = $derived(hasActiveFilters(filters, searchQuery));

  /**
   * Formats a venue's location into a readable string.
   */
  function formatLocation(venue: Venue): string {
    const parts: string[] = [];
    if (venue.location?.city) parts.push(venue.location.city);
    if (venue.location?.country) parts.push(venue.location.country);
    return parts.join(', ');
  }
</script>

<div class="p-8">
  <!-- Search, Counter, and Add Button Row -->
  <div class="flex justify-between items-center mb-6 gap-4">
    <div class="flex items-center gap-2 flex-1">
      <div class="relative flex-1 max-w-md">
        <input
          type="text"
          placeholder="Search venues..."
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
        filteredItems={filteredVenues.length}
        hasFilters={hasFiltersActive}
        entityName="venues"
        loadedItems={venues.length}
        hasMore={hasMore}
        inline={true}
        on:clear-filters={clearFilters}
      />
    </div>
    <button
      class="emittiv-fab flex-shrink-0"
      onclick={handleAddVenue}
      aria-label="Add new venue"
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

  {#if isLoading && venues.length === 0}
    <!-- Initial loading state -->
    <div class="flex flex-col items-center justify-center py-12">
      <div class="emittiv-spinner emittiv-spinner--page"></div>
      <p class="text-emittiv-light text-sm">Loading venues...</p>
    </div>
  {:else if venues.length === 0}
    <EmptyState
      icon="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z M15 11a3 3 0 11-6 0 3 3 0 016 0z"
      title="No Venues Yet"
      description="Add your first venue to start tracking project locations and sites."
      actionText="Add Venue"
      onAction={handleAddVenue}
    />
  {:else if filteredVenues.length === 0}
    <div class="text-center py-12">
      <svg class="w-16 h-16 mx-auto mb-4 text-emittiv-light opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <h3 class="text-lg font-medium text-emittiv-light mb-2">No venues found</h3>
      <p class="text-emittiv-light opacity-60 mb-4">Try adjusting your search or filters</p>
      <button
        onclick={clearFilters}
        class="emittiv-link text-sm"
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
      {#each filteredVenues as venue}
        <button
          class="emittiv-card group w-full text-left"
          onclick={() => handleEditVenue(venue)}
        >
          <div class="flex justify-between items-start">
            <div>
              <h3 class="emittiv-card-title">{venue.name}</h3>
              {#if venue.name_short}
                <span class="text-emittiv-light text-xs">({venue.name_short})</span>
              {/if}
              {#if formatLocation(venue)}
                <p class="text-sm text-emittiv-lighter mt-1">
                  {formatLocation(venue)}
                  {#if venue.location?.area}
                    <span class="text-emittiv-light"> -- {venue.location.area}</span>
                  {/if}
                </p>
              {/if}
            </div>
            {#if venue.tags && venue.tags.length > 0}
              <div class="flex gap-1 flex-wrap ml-4">
                {#each venue.tags.slice(0, 3) as tag}
                  <span class="emittiv-badge emittiv-badge--splash">{tag}</span>
                {/each}
                {#if venue.tags.length > 3}
                  <span class="emittiv-badge emittiv-badge--gray">+{venue.tags.length - 3}</span>
                {/if}
              </div>
            {/if}
          </div>
          {#if venue.notes}
            <p class="text-emittiv-light text-xs mt-2 line-clamp-2">{venue.notes}</p>
          {/if}
          <div class="emittiv-card-meta mt-2">
            <span>
              Created: {venue.time ? new Date(venue.time.created_at).toISOString().slice(2, 10).replace(/-/g, '') : '--'}
            </span>
            <span>
              Updated: {venue.time ? new Date(venue.time.updated_at).toISOString().slice(2, 10).replace(/-/g, '') : '--'}
            </span>
          </div>
        </button>
      {/each}

      <!-- Footer indicator -->
      {#if venues.length > 0}
        <div class="text-center py-4 text-emittiv-light text-xs opacity-60">
          {#if isLoading && venues.length > 0}
            <div class="flex items-center justify-center gap-2">
              <div class="emittiv-spinner"></div>
              <span>Loading more...</span>
            </div>
          {:else if hasMore}
            <span>Showing {venues.length} of {totalRecords} venues</span>
          {:else}
            <span>{totalRecords} venues</span>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- Venue Modal -->
<VenueModal
  bind:isOpen={isVenueModalOpen}
  venue={selectedVenue}
  mode={modalMode}
  on:close={handleCloseModal}
/>
