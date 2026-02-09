<script lang="ts">
  import EmptyState from '$lib/components/EmptyState.svelte';
  import ProposalCard from '$lib/components/ProposalCard.svelte';
  import ProposalModal from '$lib/components/ProposalModal.svelte';
  import ProposalDetail from '$lib/components/ProposalDetail.svelte';
  import ImportWizard from '$lib/components/ImportWizard.svelte';
  import ResultsCounter from '$lib/components/ResultsCounter.svelte';
  import { paginatedFeesStore, projectsStore, companiesStore, contactsStore, projectsActions, companiesActions, contactsActions } from '$lib/stores';
import { get } from 'svelte/store';
  import type { PaginatedStoreState } from '$lib/stores/pagination';
  import { createFilterFunction, getUniqueFieldValues, hasActiveFilters, clearAllFilters } from '$lib/utils/filters';
  import { createFeeFilterConfig, createProjectLookup } from '$lib/utils/search';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import { extractId } from '$lib/utils';
  import { createThrottled } from '$lib/utils/crud';
  import { PROPOSAL_STATUSES, getStatusColor } from '$lib/constants';
  import { onMount } from 'svelte';
  import type { Fee, UnknownSurrealThing } from '../types';

  // Modal states
  let showProposalModal = $state(false);
  let showImportWizard = $state(false);
  let proposalModalMode: 'create' | 'edit' = $state('create');
  let isProposalDetailOpen = $state(false);
  let selectedProposal: Fee | null = $state(null);

  // Filter states
  let searchQuery = $state('');
  let filters = $state({
    status: '',
    staff: ''
  });

  // Scroll container ref for infinite scroll
  let scrollContainer: HTMLDivElement | null = $state(null);

  // Pagination state - synced from store via $effect
  let fees: Fee[] = $state([]);
  let isLoading = $state(false);
  let hasMore = $state(true);
  let totalRecords = $state(0);
  let initialized = $state(false);

  // Effect to sync paginated store state to local runes
  $effect(() => {
    const unsubscribe = paginatedFeesStore.store.subscribe((state: PaginatedStoreState<Fee>) => {
      fees = state.items;
      isLoading = state.pagination.isLoading;
      hasMore = state.pagination.hasMore;
      totalRecords = state.pagination.totalRecords;
      initialized = state.initialized;
    });
    return unsubscribe;
  });

  // Scroll handler for infinite scroll (unthrottled - called by throttled wrapper)
  function checkScrollPosition() {
    if (!scrollContainer || isLoading || !hasMore) return;

    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    const scrollPercentage = (scrollTop + clientHeight) / scrollHeight;

    // Load more when scrolled past 80%
    if (scrollPercentage >= 0.8) {
      paginatedFeesStore.actions.loadNextPage();
    }
  }

  // PERF-M4: Throttle scroll handler to max once per 100ms for better performance
  const handleScroll = createThrottled(checkScrollPosition, 100);

  // Set up scroll listener when container is available
  $effect(() => {
    if (scrollContainer) {
      scrollContainer.addEventListener('scroll', handleScroll);
      return () => scrollContainer?.removeEventListener('scroll', handleScroll);
    }
  });

  // Create lookups for company/project/contact search - O(1) lookups instead of O(n)
  const companyLookup = $derived(createCompanyLookup($companiesStore));
  const projectLookup = $derived(createProjectLookup($projectsStore));
  const contactLookup = $derived(new Map($contactsStore.map(c => [extractId(c.id), c])));

  // Filter configuration for proposals - uses unified search module
  // This enables searching by company code (e.g., "ptg") and project name
  const filterConfig = $derived((() => {
    const baseConfig = createFeeFilterConfig({ companyLookup, projectLookup });
    // Add filter fields for dropdowns
    return {
      ...baseConfig,
      filterFields: {
        status: (proposal: Fee) => proposal.status,
        staff: (proposal: Fee) => proposal.staff_name || ''
      }
    };
  })());

  // Reactive filtered proposals using optimized filter function
  const filteredProposals = $derived(createFilterFunction(fees, searchQuery, filters, filterConfig));

  // Get unique values for filters using optimized functions
  const uniqueStatuses = $derived(getUniqueFieldValues(fees, (proposal) => proposal.status).filter(Boolean));
  const uniqueStaff = $derived(getUniqueFieldValues(fees, (proposal) => proposal.staff_name || '').filter(Boolean));

  // Count proposals per status for styling (bold for non-empty)
  // Uses single-pass O(n) instead of O(n*statuses)
  const statusCounts = $derived(
    fees.reduce((acc, fee) => {
      if (fee.status && fee.status in acc) {
        acc[fee.status]++;
      }
      return acc;
    }, Object.fromEntries(PROPOSAL_STATUSES.map(s => [s, 0])) as Record<string, number>)
  );
  
  function handleNewProposal() {
    selectedProposal = null;
    proposalModalMode = 'create';
    showProposalModal = true;
  }

  function handleImportClick() {
    showImportWizard = true;
  }

  function handleImportComplete() {
    // Refresh data after import
    paginatedFeesStore.actions.reset();
    paginatedFeesStore.actions.loadInitialPage();
    projectsActions.load();
    companiesActions.load();
  }

  function handleEditProposal(proposal: Fee) {
    selectedProposal = proposal;
    proposalModalMode = 'edit';
    showProposalModal = true;
  }

  function handleViewProposal(proposal: Fee) {
    selectedProposal = proposal;
    isProposalDetailOpen = true;
  }
  
  function handleCloseDetail() {
    isProposalDetailOpen = false;
    selectedProposal = null;
  }
  
  function handleEditFromDetail(event: CustomEvent) {
    // Close detail panel and open edit modal
    isProposalDetailOpen = false;
    selectedProposal = event.detail;
    proposalModalMode = 'edit';
    showProposalModal = true;
  }
  
  function clearFilters() {
    searchQuery = clearAllFilters(filters);
  }
  
  // Load proposals on mount
  onMount(() => {
    // Check store state directly to avoid race condition with $effect subscription
    const storeState = paginatedFeesStore.actions.getState();
    if (!storeState.initialized) {
      paginatedFeesStore.actions.loadInitialPage();
    }
    // Only load related data if not already loaded (performance optimization)
    if (!get(projectsStore).length) projectsActions.load();
    if (!get(companiesStore).length) companiesActions.load();
    if (!get(contactsStore).length) contactsActions.load();
  });
  
  // Check if any filters are active
  const hasFiltersActive = $derived(hasActiveFilters(filters, searchQuery));
  
  // O(1) lookup functions using pre-computed Maps (replaces O(n) .find() calls)
  function getProjectName(projectRef: UnknownSurrealThing): string {
    if (!projectRef) return 'N/A';
    const id = extractId(projectRef);
    const project = projectLookup.get(id);
    return project?.name || (id || 'Unknown Project');
  }

  function getCompanyName(companyRef: UnknownSurrealThing): string {
    return companyLookup.getCompanyName(companyRef);
  }

  function getContactName(contactRef: UnknownSurrealThing): string {
    if (!contactRef) return '';
    const id = extractId(contactRef);
    const contact = contactLookup.get(id);
    if (contact) {
      return contact.full_name || `${contact.first_name || ''} ${contact.last_name || ''}`.trim();
    }
    return id || '';
  }
</script>

<div class="p-8">
  <!-- Search, Counter, and Add Button Row -->
  <div class="flex justify-between items-center mb-6 gap-4">
    <div class="flex items-center gap-2 flex-1">
      <div class="relative flex-1 max-w-md">
        <input
          type="text"
          placeholder="Search proposals..."
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
      <ResultsCounter
        totalItems={totalRecords}
        filteredItems={filteredProposals.length}
        hasFilters={hasFiltersActive}
        entityName="proposals"
        loadedItems={fees.length}
        hasMore={hasMore}
        inline={true}
        on:clear-filters={clearFilters}
      />
    </div>
    <div class="flex items-center gap-2 flex-shrink-0">
      <button
        class="import-btn"
        onclick={handleImportClick}
        aria-label="Import RFPs data"
        title="Import from RFPs JSON"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
        </svg>
        <span>Import</span>
      </button>
      <button
        class="w-12 h-12 rounded-full bg-emittiv-splash hover:bg-orange-600 text-emittiv-black flex items-center justify-center transition-smooth hover:scale-105 active:scale-95 shadow-lg"
        onclick={handleNewProposal}
        aria-label="Add new proposal"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    </div>
  </div>
  
  <!-- Filter Options -->
  <div class="flex flex-wrap items-center gap-2 mb-2">
    <!-- Status Filter -->
    <select
      bind:value={filters.status}
      class="status-filter px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Status</option>
      {#each PROPOSAL_STATUSES as status}
        <option value={status} class:has-items={statusCounts[status] > 0}>
          {status}{statusCounts[status] > 0 ? ` (${statusCounts[status]})` : ''}
        </option>
      {/each}
    </select>
    
    <!-- Staff Filter -->
    <select 
      bind:value={filters.staff} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Staff</option>
      {#each uniqueStaff as staff}
        <option value={staff}>{staff}</option>
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

  /* Style options with items as bold (browser support varies) */
  select option.has-items {
    font-weight: 600;
  }

  select option:not(.has-items) {
    font-weight: 400;
    color: #999;
  }

  .import-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    background-color: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 8px;
    color: var(--emittiv-lighter);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease-in-out;
  }

  .import-btn:hover {
    border-color: var(--emittiv-splash);
    color: var(--emittiv-white);
  }
</style>
  
  
  {#if isLoading && fees.length === 0}
    <!-- Initial loading state -->
    <div class="flex flex-col items-center justify-center py-12">
      <div class="animate-spin rounded-full h-8 w-8 border-2 border-emittiv-splash border-t-transparent mb-4"></div>
      <p class="text-emittiv-light text-sm">Loading proposals...</p>
    </div>
  {:else if fees.length === 0}
    <EmptyState
      icon="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
      title="No Proposals Yet"
      description="Create your first fee proposal to start managing professional fee requests and proposals."
      actionText="Create Proposal"
      onAction={handleNewProposal}
    />
  {:else if filteredProposals.length === 0}
    <div class="text-center py-12">
      <svg class="w-16 h-16 mx-auto mb-4 text-emittiv-light opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <h3 class="text-lg font-medium text-emittiv-light mb-2">No proposals found</h3>
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
      class="grid gap-2 max-h-[calc(100vh-280px)] overflow-y-auto pr-2 pt-1"
    >
      {#each filteredProposals as proposal}
        <ProposalCard
          {proposal}
          projectName={getProjectName(proposal.project_id)}
          companyName={getCompanyName(proposal.company_id)}
          contactName={getContactName(proposal.contact_id)}
          on:edit={(e) => handleEditProposal(e.detail)}
          on:view={(e) => handleViewProposal(e.detail)}
        />
      {/each}

      <!-- Footer indicator -->
      {#if fees.length > 0}
        <div class="text-center py-4 text-emittiv-light text-xs opacity-60">
          {#if isLoading && fees.length > 0}
            <div class="flex items-center justify-center gap-2">
              <div class="animate-spin rounded-full h-4 w-4 border-2 border-emittiv-splash border-t-transparent"></div>
              <span>Loading more...</span>
            </div>
          {:else if hasMore}
            <span>Showing {fees.length} of {totalRecords} proposals</span>
          {:else}
            <span>{totalRecords} proposals</span>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- FP Modal -->
<ProposalModal 
  bind:isOpen={showProposalModal}
  proposal={selectedProposal}
  mode={proposalModalMode}
  on:close={() => showProposalModal = false}
/>

<!-- Proposal Detail Panel -->
<ProposalDetail
  isOpen={isProposalDetailOpen}
  proposal={selectedProposal}
  on:close={handleCloseDetail}
  on:edit={handleEditFromDetail}
/>

<!-- Import Wizard Modal -->
<ImportWizard
  bind:isOpen={showImportWizard}
  on:close={() => showImportWizard = false}
  on:imported={handleImportComplete}
/>