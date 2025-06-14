<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { projectsStore, projectsActions } from '$lib/stores';
  import { settingsStore, settingsActions } from '$lib/stores/settings';
  import { openFolderInExplorer } from '$lib/api';
  import { derived, writable } from 'svelte/store';
  import { onMount } from 'svelte';
  
  // Filter states
  let searchQuery = '';
  let statusFilter = '';
  let countryFilter = '';
  let cityFilter = '';
  
  // Search state
  // const isSearching = writable(false);
  
  // Reactive filtered projects - updates when any filter changes
  $: filteredProjects = $projectsStore.filter(project => {
    // Search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      const matchesSearch = 
        project.name.toLowerCase().includes(query) ||
        project.name_short.toLowerCase().includes(query) ||
        project.area.toLowerCase().includes(query) ||
        project.city.toLowerCase().includes(query) ||
        project.country.toLowerCase().includes(query) ||
        project.number.id.toLowerCase().includes(query) ||
        project.folder.toLowerCase().includes(query);
      if (!matchesSearch) return false;
    }
    
    // Status filter
    if (statusFilter && project.status !== statusFilter) return false;
    
    // Country filter
    if (countryFilter && project.country !== countryFilter) return false;
    
    // City filter
    if (cityFilter && project.city !== cityFilter) return false;
    
    return true;
  }).sort((a, b) => new Date(b.time.updated_at).getTime() - new Date(a.time.updated_at).getTime());
  

  
  function handleNewProject() {
    console.log('New project clicked');
    // In a real app, this would open a modal or navigate to a form
  }
  
  function clearFilters() {
    searchQuery = '';
    statusFilter = '';
    countryFilter = '';
    cityFilter = '';
  }
  
  // Load projects and settings on mount
  onMount(() => {
    projectsActions.load();
    settingsActions.load();
  });
  
  function getStatusColor(status: string) {
    switch (status) {
      case 'Active': return 'text-green-400 bg-green-400/10';
      case 'Completed': return 'text-blue-400 bg-blue-400/10';
      case 'On Hold': return 'text-yellow-400 bg-yellow-400/10';
      case 'RFP': return 'text-orange-400 bg-orange-400/10';
      case 'Draft': return 'text-gray-400 bg-gray-400/10';
      case 'Cancelled': return 'text-red-400 bg-red-400/10';
      default: return 'text-emittiv-light bg-emittiv-dark';
    }
  }
  
  // Check if any filters are active
  $: hasActiveFilters = searchQuery || statusFilter || countryFilter || cityFilter;
  
  // Get unique countries from actual project data
  $: uniqueCountries = [...new Set($projectsStore.map(project => project.country))].sort();
  
  // Get unique cities from actual project data
  $: uniqueCities = [...new Set($projectsStore.map(project => project.city))].sort();
  
  // Get unique statuses from actual project data
  $: uniqueStatuses = [...new Set($projectsStore.map(project => project.status))].sort();
  
  // Function to get full project folder path
  function getFullProjectPath(project: any): string {
    const basePath = $settingsStore.project_folder_path;
    if (!basePath) return project.folder;
    
    // Use appropriate path separator based on platform
    const separator = basePath.includes('\\') ? '\\' : '/';
    return `${basePath}${separator}${project.folder}`;
  }
  
  // Function to open project folder in explorer
  async function openProjectFolder(project: any) {
    const projectFolderPath = $settingsStore.project_folder_path;
    if (!projectFolderPath) {
      alert('Project folder path not configured. Please set it in Settings.');
      return;
    }
    
    const fullPath = getFullProjectPath(project);
    
    try {
      await openFolderInExplorer(fullPath);
    } catch (error) {
      console.error('Failed to open project folder:', error);
      alert('Failed to open project folder. Please check the path exists.');
    }
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
            placeholder="Search projects..."
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
      on:click={handleNewProject}
      aria-label="Add new project"
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
    
    <!-- City Filter -->
    <select 
      bind:value={cityFilter} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
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
  
  <!-- Results count and Clear button -->
  <div class="flex justify-between items-center mb-2">
    <div class="px-2 py-0.5 text-xs text-emittiv-light border border-emittiv-dark rounded bg-emittiv-darker">
      {#if hasActiveFilters}
        Showing {filteredProjects.length} of {$projectsStore.length} projects
      {:else if $projectsStore.length > 0}
        {$projectsStore.length} project{$projectsStore.length === 1 ? '' : 's'}
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
  
  {#if $projectsStore.length === 0}
    <EmptyState 
      icon="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
      title="No Projects Yet"
      description="Create your first project to get started with managing your proposals and tracking progress."
      actionText="Create Project"
      onAction={handleNewProject}
    />
  {:else if filteredProjects.length === 0}
    <div class="text-center py-12">
      <svg class="w-16 h-16 mx-auto mb-4 text-emittiv-light opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <h3 class="text-lg font-medium text-emittiv-light mb-2">No projects found</h3>
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
      {#each filteredProjects as project}
        <Card>
          <div>
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-lg font-semibold text-emittiv-white truncate">{project.name}</h3>
              <div class="flex items-center space-x-2 ml-4 flex-shrink-0">
                <span class="px-2 py-1 rounded-full text-xs font-medium {getStatusColor(project.status)}">
                  {project.status}
                </span>
                <button class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth" aria-label="Edit project">
                  <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                  </svg>
                </button>
                <button class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth" aria-label="View project details">
                  <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                </button>
              </div>
            </div>
            <p class="text-emittiv-lighter mb-3">
              {project.area}, {project.city}, {project.country}
            </p>
            <div class="flex items-center justify-between text-sm text-emittiv-light">
              <div class="flex items-center space-x-4">
                <span class="flex items-center space-x-1">
                  <span>Folder:</span>
                  <button 
                    on:click={() => openProjectFolder(project)}
                    class="text-emittiv-splash hover:text-orange-400 underline hover:no-underline transition-all"
                    title="Click to open in file explorer: {getFullProjectPath(project)}"
                  >
                    {project.folder}
                  </button>
                </span>
              </div>
              <span>Created: {new Date(project.time.created_at).toLocaleDateString()}</span>
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>