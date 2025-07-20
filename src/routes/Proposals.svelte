<script lang="ts">
  import EmptyState from '$lib/components/EmptyState.svelte';
  import ProposalCard from '$lib/components/ProposalCard.svelte';
  import RfpModal from '$lib/components/RfpModal.svelte';
  import ProposalDetail from '$lib/components/ProposalDetail.svelte';
  import ResultsCounter from '$lib/components/ResultsCounter.svelte';
  import { rfpsStore, projectsStore, companiesStore, contactsStore, rfpsActions, projectsActions, companiesActions, contactsActions } from '$lib/stores';
  import { createFilterFunction, getUniqueFieldValues, hasActiveFilters, clearAllFilters, type FilterConfig } from '$lib/utils/filters';
  import { onMount } from 'svelte';
  import type { Rfp } from '../types';
  
  // Modal states
  let showProposalModal = $state(false);
  let proposalModalMode: 'create' | 'edit' = $state('create');
  let isProposalDetailOpen = $state(false);
  let selectedProposal: Rfp | null = $state(null);
  
  // Filter states
  let searchQuery = $state('');
  let filters = $state({
    status: '',
    stage: '',
    staff: ''
  });
  
  // Filter configuration for proposals
  const filterConfig: FilterConfig<Rfp> = {
    searchFields: ['name', 'number', 'activity', 'package', 'staff_name'],
    filterFields: {
      status: (proposal) => proposal.status,
      stage: (proposal) => proposal.stage,
      staff: (proposal) => proposal.staff_name
    },
    sortFunction: (a, b) => {
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
    }
  };
  
  // Reactive filtered proposals using optimized filter function
  const filteredProposals = $derived(createFilterFunction($rfpsStore, searchQuery, filters, filterConfig));
  
  // Get unique values for filters using optimized functions
  const uniqueStatuses = $derived(getUniqueFieldValues($rfpsStore, (proposal) => proposal.status).filter(Boolean));
  const uniqueStages = $derived(getUniqueFieldValues($rfpsStore, (proposal) => proposal.stage).filter(Boolean));
  const uniqueStaff = $derived(getUniqueFieldValues($rfpsStore, (proposal) => proposal.staff_name).filter(Boolean));
  
  function handleNewProposal() {
    selectedProposal = null;
    proposalModalMode = 'create';
    showProposalModal = true;
  }

  function handleEditProposal(proposal: Rfp) {
    selectedProposal = proposal;
    proposalModalMode = 'edit';
    showProposalModal = true;
  }

  function handleViewProposal(proposal: Rfp) {
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
    rfpsActions.load();
    projectsActions.load();
    companiesActions.load();
    contactsActions.load();
  });
  
  // Check if any filters are active
  const hasFiltersActive = $derived(hasActiveFilters(filters, searchQuery));
  
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
      onclick={handleNewProposal}
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
      bind:value={filters.status} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Status</option>
      {#each uniqueStatuses as status}
        <option value={status}>{status}</option>
      {/each}
    </select>
    
    <!-- Stage Filter -->
    <select 
      bind:value={filters.stage} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Stages</option>
      {#each uniqueStages as stage}
        <option value={stage}>{stage}</option>
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
</style>
  
  <ResultsCounter 
    totalItems={$rfpsStore.length}
    filteredItems={filteredProposals.length}
    hasFilters={hasFiltersActive}
    entityName="proposals"
    on:clear-filters={clearFilters}
  />
  
  {#if $rfpsStore.length === 0}
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
    <div class="grid gap-2">
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
    </div>
  {/if}
</div>

<!-- RFP Modal -->
<RfpModal 
  bind:isOpen={showProposalModal}
  rfp={selectedProposal}
  mode={proposalModalMode}
  on:close={() => showProposalModal = false}
/>

<!-- Proposal Detail Panel -->
<ProposalDetail 
  bind:isOpen={isProposalDetailOpen}
  proposal={selectedProposal}
  on:close={handleCloseDetail}
  on:edit={handleEditFromDetail}
/>