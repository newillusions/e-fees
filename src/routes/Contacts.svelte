<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { contactsStore, companiesStore, contactsActions, companiesActions } from '$lib/stores';
  import { onMount } from 'svelte';
  
  // Filter states  
  let searchQuery = '';
  let companyFilter = '';
  let countryFilter = '';
  let positionFilter = '';
  
  // Filter option arrays
  let uniqueCompanies: string[] = [];
  let uniqueCountries: string[] = [];
  let uniquePositions: string[] = [];
  
  // Reactive filtered contacts - updates when any filter changes
  $: filteredContacts = $contactsStore.filter(contact => {
    // Search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      const matchesSearch = 
        contact.full_name.toLowerCase().includes(query) ||
        contact.first_name.toLowerCase().includes(query) ||
        contact.last_name.toLowerCase().includes(query) ||
        contact.email.toLowerCase().includes(query) ||
        contact.phone.toLowerCase().includes(query) ||
        contact.position.toLowerCase().includes(query);
      if (!matchesSearch) return false;
    }
    
    // Company filter - compare using the company short name
    if (companyFilter && getCompanyShortName(contact.company) !== companyFilter) {
      return false;
    }
    
    // Country filter - get country from company data
    if (countryFilter) {
      const companyCountry = getCompanyCountry(contact.company);
      if (companyCountry !== countryFilter) return false;
    }
    
    // Position filter
    if (positionFilter && contact.position !== positionFilter) return false;
    
    return true;
  }).sort((a, b) => new Date(b.time.updated_at).getTime() - new Date(a.time.updated_at).getTime());
  
  // Get unique values for filters from actual contact data
  $: {
    const companyNames = $contactsStore
      .map(c => getCompanyShortName(c.company))
      .filter(company => company && company !== 'N/A');
    uniqueCompanies = [...new Set(companyNames)].sort((a, b) => a.localeCompare(b));
  }
  
  $: {
    const countryNames = $contactsStore
      .map(c => getCompanyCountry(c.company))
      .filter(country => country && country !== 'N/A');
    uniqueCountries = [...new Set(countryNames)].sort((a, b) => a.localeCompare(b));
  }
  
  $: uniquePositions = [...new Set($contactsStore.map(c => c.position).filter(Boolean))].sort((a, b) => a.localeCompare(b));
  
  function handleAddContact() {
    console.log('Add contact clicked');
  }
  
  function clearFilters() {
    searchQuery = '';
    companyFilter = '';
    countryFilter = '';
    positionFilter = '';
  }
  
  // Load contacts and companies on mount
  onMount(() => {
    contactsActions.load();
    companiesActions.load();
  });
  
  function getCompanyName(companyRef: any): string {
    if (!companyRef) return 'N/A';
    
    // Convert the company reference to a string ID
    let companyIdStr = '';
    
    if (typeof companyRef === 'string') {
      companyIdStr = companyRef;
    } else if (companyRef && typeof companyRef === 'object') {
      // Handle Thing object { tb: 'company', id: { String: 'ABBREVIATION' } }
      if (companyRef.tb && companyRef.id) {
        if (typeof companyRef.id === 'string') {
          companyIdStr = `${companyRef.tb}:${companyRef.id}`;
        } else if (companyRef.id.String) {
          companyIdStr = `${companyRef.tb}:${companyRef.id.String}`;
        }
      }
      // Handle simple { id: 'company:ABBREVIATION' } format
      else if (companyRef.id && typeof companyRef.id === 'string') {
        companyIdStr = companyRef.id;
      }
    }
    
    if (!companyIdStr) {
      return 'Unknown';
    }
    
    // Now find the company in the store
    const company = $companiesStore.find(c => {
      if (!c.id) return false;
      
      // Get the company's ID in various formats
      let cIdStr = '';
      if (typeof c.id === 'string') {
        cIdStr = c.id;
      } else if (c.id && typeof c.id === 'object') {
        if (c.id.tb && c.id.id) {
          if (typeof c.id.id === 'string') {
            cIdStr = `${c.id.tb}:${c.id.id}`;
          } else if (c.id.id.String) {
            cIdStr = `${c.id.tb}:${c.id.id.String}`;
          }
        }
      }
      
      // Compare IDs - handle all variations
      const id1 = cIdStr.replace('company:', '');
      const id2 = companyIdStr.replace('company:', '');
      
      return id1 === id2 || cIdStr === companyIdStr;
    });
    
    if (company) {
      return company.name;
    }
    
    // If not found, try to extract abbreviation from ID
    if (companyIdStr.includes(':')) {
      return companyIdStr.split(':')[1];
    }
    
    return companyIdStr;
  }
  
  function getCompanyCountry(companyRef: any): string {
    if (!companyRef) return 'N/A';
    
    // Convert the company reference to a string ID using same logic as getCompanyName
    let companyIdStr = '';
    
    if (typeof companyRef === 'string') {
      companyIdStr = companyRef;
    } else if (companyRef && typeof companyRef === 'object') {
      if (companyRef.tb && companyRef.id) {
        if (typeof companyRef.id === 'string') {
          companyIdStr = `${companyRef.tb}:${companyRef.id}`;
        } else if (companyRef.id.String) {
          companyIdStr = `${companyRef.tb}:${companyRef.id.String}`;
        }
      }
      else if (companyRef.id && typeof companyRef.id === 'string') {
        companyIdStr = companyRef.id;
      }
    }
    
    if (!companyIdStr) {
      return 'N/A';
    }
    
    // Find the company in the store
    const company = $companiesStore.find(c => {
      if (!c.id) return false;
      
      let cIdStr = '';
      if (typeof c.id === 'string') {
        cIdStr = c.id;
      } else if (c.id && typeof c.id === 'object') {
        if (c.id.tb && c.id.id) {
          if (typeof c.id.id === 'string') {
            cIdStr = `${c.id.tb}:${c.id.id}`;
          } else if (c.id.id.String) {
            cIdStr = `${c.id.tb}:${c.id.id.String}`;
          }
        }
      }
      
      const id1 = cIdStr.replace('company:', '');
      const id2 = companyIdStr.replace('company:', '');
      
      return id1 === id2 || cIdStr === companyIdStr;
    });
    
    return company ? company.country : 'N/A';
  }
  
  function getCompanyShortName(companyRef: any): string {
    if (!companyRef) return 'N/A';
    
    // Convert the company reference to a string ID using same logic as getCompanyName
    let companyIdStr = '';
    
    if (typeof companyRef === 'string') {
      companyIdStr = companyRef;
    } else if (companyRef && typeof companyRef === 'object') {
      if (companyRef.tb && companyRef.id) {
        if (typeof companyRef.id === 'string') {
          companyIdStr = `${companyRef.tb}:${companyRef.id}`;
        } else if (companyRef.id.String) {
          companyIdStr = `${companyRef.tb}:${companyRef.id.String}`;
        }
      }
      else if (companyRef.id && typeof companyRef.id === 'string') {
        companyIdStr = companyRef.id;
      }
    }
    
    if (!companyIdStr) {
      return 'N/A';
    }
    
    // Find the company in the store
    const company = $companiesStore.find(c => {
      if (!c.id) return false;
      
      let cIdStr = '';
      if (typeof c.id === 'string') {
        cIdStr = c.id;
      } else if (c.id && typeof c.id === 'object') {
        if (c.id.tb && c.id.id) {
          if (typeof c.id.id === 'string') {
            cIdStr = `${c.id.tb}:${c.id.id}`;
          } else if (c.id.id.String) {
            cIdStr = `${c.id.tb}:${c.id.id.String}`;
          }
        }
      }
      
      const id1 = cIdStr.replace('company:', '');
      const id2 = companyIdStr.replace('company:', '');
      
      return id1 === id2 || cIdStr === companyIdStr;
    });
    
    return company ? company.name_short : 'N/A';
  }
  
  // Check if any filters are active
  $: hasActiveFilters = searchQuery || companyFilter || countryFilter || positionFilter;
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
          on:click={() => console.log('Search triggered:', searchQuery)}
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
      bind:value={companyFilter} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Companies</option>
      {#each uniqueCompanies as company}
        <option value={company}>{company}</option>
      {/each}
    </select>
    
    <!-- Country Filter -->
    <select 
      bind:value={countryFilter} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Countries</option>
      {#each uniqueCountries as country}
        <option value={country}>{country}</option>
      {/each}
    </select>
    
    <!-- Position Filter -->
    <select 
      bind:value={positionFilter} 
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
  
  <!-- Results count and Clear button -->
  <div class="flex justify-between items-center mb-2">
    <div class="px-2 py-0.5 text-xs text-emittiv-light border border-emittiv-dark rounded bg-emittiv-darker">
      {#if hasActiveFilters}
        Showing {filteredContacts.length} of {$contactsStore.length} contacts
      {:else if $contactsStore.length > 0}
        {$contactsStore.length} contact{$contactsStore.length === 1 ? '' : 's'}
      {/if}
    </div>
    {#if hasActiveFilters}
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
                <div class="flex items-center space-x-2 ml-4 flex-shrink-0">
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
                <span>Company: {getCompanyName(contact.company)}</span>
              </div>
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>