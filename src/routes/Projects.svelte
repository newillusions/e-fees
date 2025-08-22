<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import ProjectModal from '$lib/components/ProjectModal.svelte';
  import NewProjectModal from '$lib/components/NewProjectModal.svelte';
  import ListCard from '$lib/components/ListCard.svelte';
  import ResultsCounter from '$lib/components/ResultsCounter.svelte';
  import ActionButton from '$lib/components/ActionButton.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import ProjectCard from '$lib/components/ProjectCard.svelte';
  import ProjectDetail from '$lib/components/ProjectDetail.svelte';
  import { projectsStore, projectsActions } from '$lib/stores';
  import { settingsStore, settingsActions } from '$lib/stores/settings';
  import { openFolderInExplorer } from '$lib/api';
  import { createFilterFunction, getUniqueFieldValues, hasActiveFilters, clearAllFilters, type FilterConfig } from '$lib/utils/filters';
  import type { Project } from '../types';
  import { onMount } from 'svelte';
  
  // Modal states
  let showProjectModal = $state(false);
  let showNewProjectModal = $state(false);
  let projectModalMode: 'create' | 'edit' = $state('create');
  let isProjectDetailOpen = $state(false);
  let selectedProject: Project | null = $state(null);
  
  // Filter states
  let searchQuery = $state('');
  let filters = $state({
    status: '',
    country: '',
    city: ''
  });
  
  
  // Filter configuration for projects
  const filterConfig: FilterConfig<Project> = {
    searchFields: ['name', 'name_short', 'area', 'city', 'country', 'folder'],
    filterFields: {
      status: (project) => project.status,
      country: (project) => project.country,
      city: (project) => project.city
    },
    sortFunction: (a, b) => {
      // Sort by project number descending (newest numbers first)
      const aNumber = a.number?.id || '';
      const bNumber = b.number?.id || '';
      return bNumber.localeCompare(aNumber);
    }
  };

  // Reactive filtered projects using optimized filter function
  const filteredProjects = $derived(createFilterFunction($projectsStore, searchQuery, filters, filterConfig));
  

  
  function handleNewProject() {
    selectedProject = null;
    showNewProjectModal = true;
  }
  
  function handleNewProjectClosed() {
    showNewProjectModal = false;
  }
  
  function handleEditProject(project: Project) {
    selectedProject = project;
    projectModalMode = 'edit';
    showProjectModal = true;
  }
  
  function handleViewProject(project: Project) {
    selectedProject = project;
    isProjectDetailOpen = true;
  }
  
  function handleCloseDetail() {
    isProjectDetailOpen = false;
    selectedProject = null;
  }
  
  function handleEditFromDetail(event: CustomEvent) {
    // Close detail panel and open edit modal
    isProjectDetailOpen = false;
    selectedProject = event.detail;
    projectModalMode = 'edit';
    showProjectModal = true;
  }
  
  function clearFilters() {
    searchQuery = clearAllFilters(filters);
  }
  
  // Load projects and settings on mount
  onMount(() => {
    projectsActions.load();
    settingsActions.load();
  });
  
  
  // Check if any filters are active
  const hasFiltersActive = $derived(hasActiveFilters(filters, searchQuery));
  
  // Get unique values for filters using optimized functions
  const uniqueStatuses = $derived(getUniqueFieldValues($projectsStore, (project) => project.status));
  const uniqueCountries = $derived(getUniqueFieldValues($projectsStore, (project) => project.country));
  const uniqueCities = $derived(getUniqueFieldValues($projectsStore, (project) => project.city));
  
  // Function to get full project folder path
  function getFullProjectPath(project: Project): string {
    const basePath = $settingsStore.project_folder_path;
    if (!basePath) return project.folder;
    
    // Use appropriate path separator based on platform
    const separator = basePath.includes('\\') ? '\\' : '/';
    return `${basePath}${separator}${project.folder}`;
  }
  
  // Function to open project folder in explorer
  async function openProjectFolder(project: Project) {
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
      onclick={handleNewProject}
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
      bind:value={filters.status} 
      class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
    >
      <option value="">All Status</option>
      {#each uniqueStatuses as status}
        <option value={status}>{status}</option>
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
    
    <!-- City Filter -->
    <select 
      bind:value={filters.city} 
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
  
  <ResultsCounter 
    totalItems={$projectsStore.length}
    filteredItems={filteredProjects.length}
    hasFilters={hasFiltersActive}
    entityName="projects"
    on:clear-filters={clearFilters}
  />
  
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
        onclick={clearFilters}
        class="text-sm text-emittiv-splash hover:text-orange-400 transition-smooth"
      >
        Clear all filters
      </button>
    </div>
  {:else}
    <div class="grid gap-2">
      {#each filteredProjects as project}
        <ProjectCard 
          {project}
          onFolderClick={openProjectFolder}
          on:edit={(e) => handleEditProject(e.detail)}
          on:view={(e) => handleViewProject(e.detail)}
        />
      {/each}
    </div>
  {/if}
</div>

<!-- New Project Modal -->
<NewProjectModal 
  bind:isOpen={showNewProjectModal}
  on:close={handleNewProjectClosed}
/>

<!-- Edit Project Modal -->
<ProjectModal 
  bind:isOpen={showProjectModal}
  project={selectedProject}
  mode={projectModalMode}
  on:close={() => showProjectModal = false}
/>

<!-- Project Detail Panel -->
<ProjectDetail 
  isOpen={isProjectDetailOpen}
  project={selectedProject}
  on:close={handleCloseDetail}
  on:edit={handleEditFromDetail}
/>