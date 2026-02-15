<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import ContactCard from '$lib/components/ContactCard.svelte';
  import ContactDetail from '$lib/components/ContactDetail.svelte';
  import ContactModal from '$lib/components/ContactModal.svelte';
  import ResultsCounter from '$lib/components/ResultsCounter.svelte';
  import { paginatedContactsStore, companiesStore, companiesActions } from '$lib/stores';
  import type { PaginatedStoreState } from '$lib/stores/pagination';
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { createFilterFunction, getUniqueFieldValues, hasActiveFilters, clearAllFilters } from '$lib/utils/filters';
  import { createContactFilterConfig } from '$lib/utils/search';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import type { Contact } from '../types';

  // Filter states
  let searchQuery = $state('');
  let filters = $state({
    company: '',
    country: '',
    position: ''
  });

  // Scroll container ref for infinite scroll
  let scrollContainer: HTMLDivElement | null = $state(null);

  // Pagination state - synced from store via $effect
  let contacts: Contact[] = $state([]);
  let isLoading = $state(false);
  let hasMore = $state(true);
  let totalRecords = $state(0);
  let initialized = $state(false);

  // Modal states
  let isContactDetailOpen = $state(false);
  let isContactModalOpen = $state(false);
  let selectedContact: Contact | null = $state(null);
  let modalMode: 'create' | 'edit' = $state('create');

  // Effect to sync paginated store state to local runes
  $effect(() => {
    const unsubscribe = paginatedContactsStore.store.subscribe((state: PaginatedStoreState<Contact>) => {
      contacts = state.items;
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
      paginatedContactsStore.actions.loadNextPage();
    }
  }

  // Set up scroll listener when container is available
  $effect(() => {
    if (scrollContainer) {
      scrollContainer.addEventListener('scroll', handleScroll);
      return () => scrollContainer?.removeEventListener('scroll', handleScroll);
    }
  });

  // Create optimized company lookup when companies data changes
  const companyLookup = $derived(createCompanyLookup($companiesStore));

  // Filter configuration for contacts - uses unified search module
  // This enables searching by company code (e.g., "ptg") and company name
  const filterConfig = $derived(createContactFilterConfig({ companyLookup }));

  // Reactive filtered contacts using optimized filter function
  const filteredContacts = $derived(createFilterFunction(contacts, searchQuery, filters, filterConfig));

  // Get unique values for filters using optimized functions
  const uniqueCompanies = $derived(getUniqueFieldValues(contacts, (contact) =>
    companyLookup.getCompanyShortName(contact.company)
  ).filter(name => name !== 'N/A'));

  const uniqueCountries = $derived(getUniqueFieldValues(contacts, (contact) =>
    companyLookup.getCompanyCountry(contact.company)
  ).filter(country => country !== 'N/A'));

  const uniquePositions = $derived(getUniqueFieldValues(contacts, (contact) => contact.position));
  
  function handleAddContact() {
    selectedContact = null;
    modalMode = 'create';
    isContactModalOpen = true;
  }
  
  function handleEditContact(contact: Contact) {
    selectedContact = contact;
    modalMode = 'edit';
    isContactModalOpen = true;
  }
  
  function handleViewContact(contact: Contact) {
    selectedContact = contact;
    isContactDetailOpen = true;
  }
  
  function handleCloseDetail() {
    isContactDetailOpen = false;
    selectedContact = null;
  }
  
  function handleEditFromDetail(event: CustomEvent) {
    // Keep detail view open and show edit modal on top
    selectedContact = event.detail;
    modalMode = 'edit';
    isContactModalOpen = true;
  }
  
  function handleCloseModal() {
    isContactModalOpen = false;
    // Refresh contacts list after modal closes
    paginatedContactsStore.actions.refresh();

    // If we have a selected contact, update it with fresh data from the store
    if (selectedContact) {
      setTimeout(() => {
        if (selectedContact) {
          const updatedContact = contacts.find(c => c.id === selectedContact!.id);
          if (updatedContact) {
            selectedContact = updatedContact;
          }
        }
      }, 100);
    }
  }

  function clearFilters() {
    searchQuery = clearAllFilters(filters);
  }

  // Check if any filters are active
  const hasFiltersActive = $derived(hasActiveFilters(filters, searchQuery));

  // Load contacts and companies on mount
  onMount(() => {
    // Check store state directly to avoid race condition with $effect subscription
    const storeState = paginatedContactsStore.actions.getState();
    if (!storeState.initialized) {
      paginatedContactsStore.actions.loadInitialPage();
    }
    // Only load companies if not already loaded (performance optimization)
    if (!get(companiesStore).length) companiesActions.load();
  });
</script>

<div class="p-8">
  <!-- Search, Counter, and Add Button Row -->
  <div class="flex justify-between items-center mb-6 gap-4">
    <div class="flex items-center gap-2 flex-1">
      <div class="relative flex-1 max-w-md">
        <input
          type="text"
          placeholder="Search contacts..."
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
        filteredItems={filteredContacts.length}
        hasFilters={hasFiltersActive}
        entityName="contacts"
        loadedItems={contacts.length}
        hasMore={hasMore}
        inline={true}
        on:clear-filters={clearFilters}
      />
    </div>
    <button
      class="emittiv-fab flex-shrink-0"
      onclick={handleAddContact}
      aria-label="Add new contact"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
  </div>
  
  <!-- Filter Options -->
  <div class="flex flex-wrap items-center gap-2 mb-4">
    <!-- Company Filter -->
    <select 
      bind:value={filters.company} 
      class="emittiv-filter-select"
    >
      <option value="">All Companies</option>
      {#each uniqueCompanies as company}
        <option value={company}>{company}</option>
      {/each}
    </select>
    
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
    
    <!-- Position Filter -->
    <select 
      bind:value={filters.position} 
      class="emittiv-filter-select"
    >
      <option value="">All Positions</option>
      {#each uniquePositions as position}
        <option value={position}>{position}</option>
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
  
  
  {#if isLoading && contacts.length === 0}
    <!-- Initial loading state -->
    <div class="flex flex-col items-center justify-center py-12">
      <div class="animate-spin rounded-full h-8 w-8 border-2 border-emittiv-splash border-t-transparent mb-4"></div>
      <p class="text-emittiv-light text-sm">Loading contacts...</p>
    </div>
  {:else if contacts.length === 0}
    <EmptyState
      icon="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"
      title="No Contacts Yet"
      description="Add your first contact to start building your professional network and managing client relationships."
      actionText="Add Contact"
      onAction={handleAddContact}
    />
  {:else if filteredContacts.length === 0}
    <div class="text-center py-12">
      <svg class="w-16 h-16 mx-auto mb-4 text-emittiv-light opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <h3 class="text-lg font-medium text-emittiv-light mb-2">No contacts found</h3>
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
      {#each filteredContacts as contact}
        <ContactCard
          {contact}
          companyName={companyLookup.getCompanyName(contact.company)}
          on:edit={(e) => handleEditContact(e.detail)}
          on:view={(e) => handleViewContact(e.detail)}
        />
      {/each}

      <!-- Footer indicator -->
      {#if contacts.length > 0}
        <div class="text-center py-4 text-emittiv-light text-xs opacity-60">
          {#if isLoading && contacts.length > 0}
            <div class="flex items-center justify-center gap-2">
              <div class="animate-spin rounded-full h-4 w-4 border-2 border-emittiv-splash border-t-transparent"></div>
              <span>Loading more...</span>
            </div>
          {:else if hasMore}
            <span>Showing {contacts.length} of {totalRecords} contacts</span>
          {:else}
            <span>{totalRecords} contacts</span>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- Contact Modal -->
<ContactModal 
  bind:isOpen={isContactModalOpen}
  contact={selectedContact}
  mode={modalMode}
  on:close={handleCloseModal}
/>

<!-- Contact Detail Panel -->
<ContactDetail 
  isOpen={isContactDetailOpen}
  contact={selectedContact}
  on:close={handleCloseDetail}
  on:edit={handleEditFromDetail}
/>