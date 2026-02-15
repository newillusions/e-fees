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
  import { paginatedProjectsStore } from '$lib/stores';
  import type { PaginatedStoreState } from '$lib/stores/pagination';
  import { settingsStore, settingsActions } from '$lib/stores/settings';
  import { openFolderInExplorer } from '$lib/api';
  import { createFilterFunction, getUniqueFieldValues, hasActiveFilters, clearAllFilters } from '$lib/utils/filters';
import { getFolderForStatus } from '$lib/api/folderManagement';
  import { createProjectFilterConfig } from '$lib/utils/search';
  import type { Project } from '../types';
  import { onMount, onDestroy } from 'svelte';

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

  // Scroll container ref for infinite scroll
  let scrollContainer: HTMLDivElement | null = $state(null);

  // Pagination state - synced from store via $effect
  let projects: Project[] = $state([]);
  let isLoading = $state(false);
  let hasMore = $state(true);
  let totalRecords = $state(0);
  let initialized = $state(false);

  // Effect to sync paginated store state to local runes
  $effect(() => {
    const unsubscribe = paginatedProjectsStore.store.subscribe((state: PaginatedStoreState<Project>) => {
      projects = state.items;
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
      paginatedProjectsStore.actions.loadNextPage();
    }
  }

  // Set up scroll listener when container is available
  $effect(() => {
    if (scrollContainer) {
      scrollContainer.addEventListener('scroll', handleScroll);
      return () => scrollContainer?.removeEventListener('scroll', handleScroll);
    }
  });
  
  
  // Filter configuration for projects - uses unified search module
  const filterConfig = (() => {
    const baseConfig = createProjectFilterConfig();
    // Add filter fields for dropdowns
    return {
      ...baseConfig,
      filterFields: {
        status: (project: Project) => project.status,
        country: (project: Project) => project.country,
        city: (project: Project) => project.city
      }
    };
  })();

  // Reactive filtered projects using optimized filter function
  const filteredProjects = $derived(createFilterFunction(projects, searchQuery, filters, filterConfig));
  

  
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
    // Load first page of projects using pagination
    // Check store state directly to avoid race condition with $effect subscription
    const storeState = paginatedProjectsStore.actions.getState();
    if (!storeState.initialized) {
      paginatedProjectsStore.actions.loadInitialPage();
    }
    settingsActions.load();
  });
  
  
  // Check if any filters are active
  const hasFiltersActive = $derived(hasActiveFilters(filters, searchQuery));
  
  // Get unique values for filters using optimized functions
  const uniqueStatuses = $derived(getUniqueFieldValues(projects, (project) => project.status));
  const uniqueCountries = $derived(getUniqueFieldValues(projects, (project) => project.country));
  const uniqueCities = $derived(getUniqueFieldValues(projects, (project) => project.city));
  
  // Function to get full project folder path
  function getFullProjectPath(project: Project): string {
    const basePath = $settingsStore.project_folder_path;
    if (!basePath) return project.folder || '';

    // Normalize paths - remove trailing slashes from base and leading slashes from folder
    const isWindows = basePath.includes('\\');
    const separator = isWindows ? '\\' : '/';

    // Normalize base path (remove trailing separators)
    let normalizedBase = basePath;
    while (normalizedBase.endsWith(separator) || normalizedBase.endsWith('/') || normalizedBase.endsWith('\\')) {
      normalizedBase = normalizedBase.slice(0, -1);
    }

    // Normalize project folder (remove leading separators)
    let normalizedFolder = project.folder || '';
    while (normalizedFolder.startsWith('/') || normalizedFolder.startsWith('\\')) {
      normalizedFolder = normalizedFolder.slice(1);
    }

    if (!normalizedFolder) return normalizedBase;

    // Get the status-based subfolder (e.g., "01 RFPs", "11 Current")
    const statusFolder = getFolderForStatus(project.status || 'rfp');

    return `${normalizedBase}${separator}${statusFolder}${separator}${normalizedFolder}`;
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
      const result = await openFolderInExplorer(fullPath);
      if (result.includes('Failed')) {
        alert('Failed to open project folder. Please check the path exists.');
      }
    } catch (error) {
      console.error('Failed to open project folder:', error);
      alert('Failed to open project folder. Please check the path exists.');
    }
  }
</script>

<div class="p-8">
  <!-- Search, Counter, and Add Button Row -->
  <div class="flex justify-between items-center mb-6 gap-4">
    <div class="flex items-center gap-2 flex-1">
      <div class="relative flex-1 max-w-md">
        <input
          type="text"
          placeholder="Search projects..."
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
        filteredItems={filteredProjects.length}
        hasFilters={hasFiltersActive}
        entityName="projects"
        loadedItems={projects.length}
        hasMore={hasMore}
        inline={true}
        on:clear-filters={clearFilters}
      />
    </div>
    <button
      class="emittiv-fab flex-shrink-0"
      onclick={handleNewProject}
      aria-label="Add new project"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
  </div>
  
  <!-- Filter Options -->
  <div class="flex flex-wrap items-center gap-2 mb-4">
    <!-- Status Filter -->
    <select 
      bind:value={filters.status} 
      class="emittiv-filter-select"
    >
      <option value="">All Status</option>
      {#each uniqueStatuses as status}
        <option value={status}>{status}</option>
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
    
    <!-- City Filter -->
    <select 
      bind:value={filters.city} 
      class="emittiv-filter-select"
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
  

  {#if isLoading && projects.length === 0}
    <!-- Initial loading state -->
    <div class="flex flex-col items-center justify-center py-12">
      <div class="animate-spin rounded-full h-8 w-8 border-2 border-emittiv-splash border-t-transparent mb-4"></div>
      <p class="text-emittiv-light text-sm">Loading projects...</p>
    </div>
  {:else if projects.length === 0}
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
    <!-- Scrollable container for infinite scroll -->
    <div
      bind:this={scrollContainer}
      class="grid gap-3 max-h-scroll overflow-y-auto pr-2 pt-1"
    >
      {#each filteredProjects as project}
        <ProjectCard
          {project}
          onFolderClick={openProjectFolder}
          on:edit={(e) => handleEditProject(e.detail)}
          on:view={(e) => handleViewProject(e.detail)}
        />
      {/each}

      <!-- Footer indicator -->
      {#if projects.length > 0}
        <div class="text-center py-4 text-emittiv-light text-xs opacity-60">
          {#if isLoading && projects.length > 0}
            <div class="flex items-center justify-center gap-2">
              <div class="animate-spin rounded-full h-4 w-4 border-2 border-emittiv-splash border-t-transparent"></div>
              <span>Loading more...</span>
            </div>
          {:else if hasMore}
            <span>Showing {projects.length} of {totalRecords} projects</span>
          {:else}
            <span>{totalRecords} projects</span>
          {/if}
        </div>
      {/if}
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