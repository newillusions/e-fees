<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import SearchFilterBar from '$lib/components/SearchFilterBar.svelte';
  import { contactsStore, companiesStore, contactsActions, companiesActions } from '$lib/stores';
  import { onMount } from 'svelte';
  import { createFilterFunction, getUniqueFieldValues, type FilterConfig } from '$lib/utils/filters';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import type { Contact } from '../types';
  
  // Filter states  
  let searchQuery = '';
  let filters = {
    company: '',
    country: '',
    position: ''
  };
  
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
  
  // Filter options for SearchFilterBar
  $: filterOptions = [
    {
      key: 'company',
      label: 'All Companies',
      options: uniqueCompanies
    },
    {
      key: 'country', 
      label: 'All Countries',
      options: uniqueCountries
    },
    {
      key: 'position',
      label: 'All Positions', 
      options: uniquePositions
    }
  ];
  
  function handleAddContact() {
    // TODO: Implement add contact functionality
  }
  
  // Load contacts and companies on mount
  onMount(() => {
    contactsActions.load();
    companiesActions.load();
  });
  
  
  
  
</script>

<div class="p-8">
  <!-- Reusable Search and Filter Bar -->
  <SearchFilterBar
    bind:searchQuery
    bind:filters
    {filterOptions}
    placeholder="Search contacts..."
    onAdd={handleAddContact}
    addLabel="Add new contact"
    resultCount={filteredContacts.length}
    totalCount={$contactsStore.length}
    itemName="contacts"
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
    <div class="grid gap-6">
      {#each filteredContacts as contact}
        <Card>
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between mb-2">
                <h3 class="text-lg font-semibold text-emittiv-white truncate">{contact.full_name}</h3>
                <div class="flex items-center space-x-1 ml-4 flex-shrink-0">
                  <span class="px-2 py-1 rounded-full text-xs font-medium text-purple-400 bg-purple-400/10">
                    {contact.position}
                  </span>
                  <a href="mailto:{contact.email}" class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth inline-block" aria-label="Send email to {contact.full_name}">
                    <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 7.89a2 2 0 002.83 0L21 9M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                    </svg>
                  </a>
                  <button class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth" aria-label="Edit contact">
                    <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                    </svg>
                  </button>
                  <button class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth" aria-label="View contact details">
                    <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                    </svg>
                  </button>
                </div>
              </div>
              <p class="text-emittiv-lighter mb-3 truncate flex space-x-4">
                <span>{contact.email}</span>
                <span>{contact.phone}</span>
              </p>
              <div class="flex items-center space-x-4 text-sm text-emittiv-light">
                <span>Company: {companyLookup.getCompanyName(contact.company)}</span>
              </div>
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>