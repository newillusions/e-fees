<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import ContactCard from '$lib/components/ContactCard.svelte';
  import ContactDetail from '$lib/components/ContactDetail.svelte';
  import ContactModal from '$lib/components/ContactModal.svelte';
  import ResultsCounter from '$lib/components/ResultsCounter.svelte';
  import { contactsStore, companiesStore, contactsActions, companiesActions } from '$lib/stores';
  import { onMount } from 'svelte';
  import { createFilterFunction, getUniqueFieldValues, hasActiveFilters, clearAllFilters, type FilterConfig } from '$lib/utils/filters';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import type { Contact } from '../types';
  
  // Filter states  
  let searchQuery = '';
  let filters = {
    company: '',
    country: '',
    position: ''
  };
  
  // Modal states
  let isContactDetailOpen = false;
  let isContactModalOpen = false;
  let selectedContact: Contact | null = null;
  let modalMode: 'create' | 'edit' = 'create';
  
  // Create optimized company lookup when companies data changes
  $: companyLookup = createCompanyLookup($companiesStore);
  
  // Filter configuration for contacts
  const filterConfig: FilterConfig<Contact> = {
    searchFields: ['full_name', 'first_name', 'last_name', 'email', 'phone', 'position'],
    filterFields: {
      company: (contact) => companyLookup.getCompanyShortName(contact.company),
      country: (contact) => companyLookup.getCompanyCountry(contact.company),
      position: (contact) => contact.position
    }
  };
  
  // Reactive filtered contacts using optimized filter function
  $: filteredContacts = createFilterFunction($contactsStore, searchQuery, filters, filterConfig);
  
  // Get unique values for filters using optimized functions
  $: uniqueCompanies = getUniqueFieldValues($contactsStore, (contact) => 
    companyLookup.getCompanyShortName(contact.company)
  ).filter(name => name !== 'N/A');
  
  $: uniqueCountries = getUniqueFieldValues($contactsStore, (contact) => 
    companyLookup.getCompanyCountry(contact.company)
  ).filter(country => country !== 'N/A');
  
  $: uniquePositions = getUniqueFieldValues($contactsStore, (contact) => contact.position);
  
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
    contactsActions.load();
    
    // If we have a selected contact, update it with fresh data from the store
    if (selectedContact) {
      setTimeout(() => {
        if (selectedContact) {
          const updatedContact = $contactsStore.find(c => c.id === selectedContact!.id);
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
  $: hasFiltersActive = hasActiveFilters(filters, searchQuery);
  
  // Load contacts and companies on mount
  onMount(() => {
    contactsActions.load();
    companiesActions.load();
  });
</script>

<div class="p-8">
  <!-- Search and Add Button Row -->
  <div class="flex justify-between items-center mb-6">
    <div class="flex-1 max-w-2xl">
      <div class="flex items-center gap-2">
        <div class="relative flex-1">
          <input
            type="text"
            placeholder="Search contacts..."
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
      on:click={handleAddContact}
      aria-label="Add new contact"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
  </div>
  
  <!-- Filter Options -->
  <div class="flex flex-wrap items-center gap-2 mb-2">
    <!-- Company Filter -->
    <select 
      bind:value={filters.company} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Companies</option>
      {#each uniqueCompanies as company}
        <option value={company}>{company}</option>
      {/each}
    </select>
    
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
    
    <!-- Position Filter -->
    <select 
      bind:value={filters.position} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
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
  
  <ResultsCounter 
    totalItems={$contactsStore.length}
    filteredItems={filteredContacts.length}
    hasFilters={hasFiltersActive}
    entityName="contacts"
    on:clear-filters={clearFilters}
  />
  
  {#if $contactsStore.length === 0}
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
        on:click={clearFilters}
        class="text-sm text-emittiv-splash hover:text-orange-400 transition-smooth"
      >
        Clear all filters
      </button>
    </div>
  {:else}
    <div class="grid gap-2">
      {#each filteredContacts as contact}
        <ContactCard 
          {contact}
          companyName={companyLookup.getCompanyName(contact.company)}
          on:edit={(e) => handleEditContact(e.detail)}
          on:view={(e) => handleViewContact(e.detail)}
        />
      {/each}
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
  bind:isOpen={isContactDetailOpen}
  contact={selectedContact}
  on:close={handleCloseDetail}
  on:edit={handleEditFromDetail}
/>