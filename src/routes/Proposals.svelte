<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { rfpsStore, projectsStore, companiesStore, contactsStore, rfpsActions } from '$lib/stores';
  import { onMount } from 'svelte';
  
  // Filter states
  let searchQuery = '';
  let statusFilter = '';
  let stageFilter = '';
  let staffFilter = '';
  
  // Reactive filtered RFPs - updates when any filter changes
  $: filteredRfps = $rfpsStore.filter(rfp => {
    // Search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      const matchesSearch = 
        rfp.name.toLowerCase().includes(query) ||
        rfp.number.toLowerCase().includes(query) ||
        rfp.activity?.toLowerCase().includes(query) ||
        rfp.package?.toLowerCase().includes(query) ||
        rfp.staff_name?.toLowerCase().includes(query);
      if (!matchesSearch) return false;
    }
    
    // Status filter
    if (statusFilter && rfp.status !== statusFilter) return false;
    
    // Stage filter
    if (stageFilter && rfp.stage !== stageFilter) return false;
    
    // Staff filter
    if (staffFilter && rfp.staff_name !== staffFilter) return false;
    
    return true;
  }).sort((a, b) => {
    // Primary sort: updated_at descending (most recent first)
    const aUpdated = new Date(a.time?.updated_at || a.time?.created_at || 0).getTime();
    const bUpdated = new Date(b.time?.updated_at || b.time?.created_at || 0).getTime();
    
    if (bUpdated !== aUpdated) {
      return bUpdated - aUpdated;
    }
    
    // Secondary sort: created_at descending (if updated_at is the same)
    const aCreated = new Date(a.time?.created_at || 0).getTime();
    const bCreated = new Date(b.time?.created_at || 0).getTime();
    return bCreated - aCreated;
  });
  
  // Get unique values for filters from actual RFP data
  $: uniqueStatuses = [...new Set($rfpsStore.map(rfp => rfp.status).filter(Boolean))].sort();
  $: uniqueStages = [...new Set($rfpsStore.map(rfp => rfp.stage).filter(Boolean))].sort();
  $: uniqueStaff = [...new Set($rfpsStore.map(rfp => rfp.staff_name).filter(Boolean))].sort();
  
  function handleNewRfp() {
    // TODO: Implement new RFP functionality
  }
  
  function clearFilters() {
    searchQuery = '';
    statusFilter = '';
    stageFilter = '';
    staffFilter = '';
  }
  
  // Load RFPs on mount
  onMount(() => {
    rfpsActions.load();
  });
  
  // Check if any filters are active
  $: hasActiveFilters = searchQuery || statusFilter || stageFilter || staffFilter;
  
  function getStatusColor(status: string) {
    switch (status) {
      case 'Draft': return 'text-yellow-400 bg-yellow-400/10';
      case 'Active': return 'text-blue-400 bg-blue-400/10';
      case 'Sent': return 'text-purple-400 bg-purple-400/10';
      case 'Awarded': return 'text-green-400 bg-green-400/10';
      case 'Lost': return 'text-red-400 bg-red-400/10';
      case 'Cancelled': return 'text-gray-400 bg-gray-400/10';
      default: return 'text-emittiv-light bg-emittiv-dark';
    }
  }

  function getProjectName(projectRef: any): string {
    if (!projectRef) return 'N/A';
    
    // Convert the project reference to a string ID
    let projectIdStr = '';
    
    if (typeof projectRef === 'string') {
      projectIdStr = projectRef;
    } else if (projectRef && typeof projectRef === 'object') {
      // Handle Thing object { tb: 'projects', id: { String: 'PROJECT_ID' } }
      if (projectRef.tb && projectRef.id) {
        if (typeof projectRef.id === 'string') {
          projectIdStr = `${projectRef.tb}:${projectRef.id}`;
        } else if (projectRef.id.String) {
          projectIdStr = `${projectRef.tb}:${projectRef.id.String}`;
        }
      }
      // Handle simple { id: 'projects:PROJECT_ID' } format
      else if (projectRef.id && typeof projectRef.id === 'string') {
        projectIdStr = projectRef.id;
      }
    }
    
    if (!projectIdStr) {
      return 'Unknown Project';
    }
    
    // Now find the project in the store
    const project = $projectsStore.find(p => {
      if (!p.id) return false;
      
      // Get the project's ID in various formats
      let pIdStr = '';
      if (typeof p.id === 'string') {
        pIdStr = p.id;
      } else if (p.id && typeof p.id === 'object') {
        if (p.id.tb && p.id.id) {
          if (typeof p.id.id === 'string') {
            pIdStr = `${p.id.tb}:${p.id.id}`;
          } else if (p.id.id.String) {
            pIdStr = `${p.id.tb}:${p.id.id.String}`;
          }
        }
      }
      
      // Compare IDs - handle all variations
      const id1 = pIdStr.replace('projects:', '');
      const id2 = projectIdStr.replace('projects:', '');
      
      return id1 === id2 || pIdStr === projectIdStr;
    });
    
    if (project) {
      return project.name;
    }
    
    // If not found, try to extract ID from reference
    if (projectIdStr.includes(':')) {
      return projectIdStr.split(':')[1];
    }
    
    return projectIdStr;
  }

  function getCompanyName(companyRef: any): string {
    if (!companyRef) return '';
    
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
      return '';
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

  function getContactName(contactRef: any): string {
    if (!contactRef) return '';
    
    // Convert the contact reference to a string ID
    let contactIdStr = '';
    
    if (typeof contactRef === 'string') {
      contactIdStr = contactRef;
    } else if (contactRef && typeof contactRef === 'object') {
      // Handle Thing object { tb: 'contacts', id: { String: 'CONTACT_ID' } }
      if (contactRef.tb && contactRef.id) {
        if (typeof contactRef.id === 'string') {
          contactIdStr = `${contactRef.tb}:${contactRef.id}`;
        } else if (contactRef.id.String) {
          contactIdStr = `${contactRef.tb}:${contactRef.id.String}`;
        }
      }
      // Handle simple { id: 'contacts:CONTACT_ID' } format
      else if (contactRef.id && typeof contactRef.id === 'string') {
        contactIdStr = contactRef.id;
      }
    }
    
    if (!contactIdStr) {
      return '';
    }
    
    // Now find the contact in the store
    const contact = $contactsStore.find(c => {
      if (!c.id) return false;
      
      // Get the contact's ID in various formats
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
      const id1 = cIdStr.replace('contacts:', '');
      const id2 = contactIdStr.replace('contacts:', '');
      
      return id1 === id2 || cIdStr === contactIdStr;
    });
    
    if (contact) {
      return contact.full_name;
    }
    
    // If not found, try to extract ID from reference
    if (contactIdStr.includes(':')) {
      return contactIdStr.split(':')[1];
    }
    
    return contactIdStr;
  }
</script>

<div class="p-8">
  <!-- Search and Add Button Row -->
  <div class="flex justify-between items-center mb-6">
    <div class="flex-1 max-w-2xl">
      <div class="flex items-center gap-2">
        <div class="relative flex-1">
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
      </div>
    </div>
    <button
      class="ml-4 w-12 h-12 rounded-full bg-emittiv-splash hover:bg-orange-600 text-emittiv-black flex items-center justify-center transition-smooth hover:scale-105 active:scale-95 shadow-lg"
      on:click={handleNewRfp}
      aria-label="Add new proposal"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
  </div>
  
  <!-- Filter Options -->
  <div class="flex flex-wrap items-center gap-2 mb-2">
    <!-- Status Filter -->
    <select 
      bind:value={statusFilter} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Status</option>
      {#each uniqueStatuses as status}
        <option value={status}>{status}</option>
      {/each}
    </select>
    
    <!-- Stage Filter -->
    <select 
      bind:value={stageFilter} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Stages</option>
      {#each uniqueStages as stage}
        <option value={stage}>{stage}</option>
      {/each}
    </select>
    
    <!-- Staff Filter -->
    <select 
      bind:value={staffFilter} 
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
</style>
  
  <!-- Results count and Clear button -->
  <div class="flex justify-between items-center mb-2">
    <div class="px-2 py-0.5 text-xs text-emittiv-light border border-emittiv-dark rounded bg-emittiv-darker">
      {#if hasActiveFilters}
        Showing {filteredRfps.length} of {$rfpsStore.length} proposals
      {:else if $rfpsStore.length > 0}
        {$rfpsStore.length} proposal{$rfpsStore.length === 1 ? '' : 's'}
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
  
  {#if $rfpsStore.length === 0}
    <EmptyState 
      icon="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
      title="No RFPs Yet"
      description="Create your first RFP to start managing professional fee requests and proposals."
      actionText="Create RFP"
      onAction={handleNewRfp}
    />
  {:else if filteredRfps.length === 0}
    <div class="text-center py-12">
      <svg class="w-16 h-16 mx-auto mb-4 text-emittiv-light opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <h3 class="text-lg font-medium text-emittiv-light mb-2">No proposals found</h3>
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
      {#each filteredRfps as rfp}
        <Card>
          <div>
            <div class="flex items-center justify-between mb-2">
              <div class="flex-1 min-w-0">
                <h3 class="text-lg font-semibold text-emittiv-white truncate">
                  {getProjectName(rfp.project_id)}
                </h3>
                {#if rfp.package}
                  <p class="text-sm text-emittiv-lighter mt-1 truncate">{rfp.package}</p>
                {/if}
              </div>
              <div class="flex items-center space-x-1 ml-4 flex-shrink-0">
                <span class="px-2 py-1 rounded-full text-xs font-medium {getStatusColor(rfp.status)}">
                  {rfp.status}
                </span>
                <button class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth" aria-label="Edit proposal">
                  <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                  </svg>
                </button>
                <button class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth" aria-label="View proposal details">
                  <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                </button>
              </div>
            </div>
            <p class="text-emittiv-lighter mb-2 truncate">
              {getCompanyName(rfp.company_id) || 'N/A'}
              {#if getContactName(rfp.contact_id)}
                <span class="ml-2">{getContactName(rfp.contact_id)}</span>
              {/if}
            </p>
            <div class="flex items-center justify-between text-sm text-emittiv-light">
              <div class="flex items-center space-x-4">
                <span>Number: {rfp.number}</span>
                <span>Rev: {rfp.rev}</span>
                <span>Stage: {rfp.stage}</span>
                <span>Staff: {rfp.staff_name || 'N/A'}</span>
              </div>
              <span>Created: {new Date(rfp.time.created_at).toLocaleDateString()}</span>
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>