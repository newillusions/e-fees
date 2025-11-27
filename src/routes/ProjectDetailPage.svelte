<script lang="ts">
  import { onMount } from 'svelte';
  import { projectsStore, loadAllData } from '$lib/stores';
  import { extractId, findEntityById } from '$lib/utils';
  import { push, location } from 'svelte-spa-router';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import ProjectDetail from '$lib/components/ProjectDetail.svelte';
  import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
  import type { Project } from '../types';

  export let params: { id: string } = { id: '' };
  
  // Track the previous page to return to the correct location
  let previousPage = '/projects'; // Default fallback
  
  let project: Project | null = null;
  let loading = true;
  let error: string | null = null;

  // Extract the project ID from params
  $: projectId = params.id;

  // Find the project when the store updates or ID changes
  $: {
    if ($projectsStore.length > 0 && projectId) {
      const foundProject = findEntityById($projectsStore, projectId);
      if (foundProject) {
        project = foundProject as Project;
        loading = false;
        error = null;
      } else {
        project = null;
        loading = false;
        error = `Project with ID "${projectId}" not found`;
      }
    }
  }

  onMount(async () => {
    // Check if we came from the dashboard
    const referrer = document.referrer;
    const urlParams = new URLSearchParams(window.location.search);
    const fromParam = urlParams.get('from');
    
    if (fromParam === 'dashboard' || window.history.length > 1) {
      // If there's a 'from' parameter or browser history, check where we came from
      if (fromParam === 'dashboard') {
        previousPage = '/';
      } else {
        // Try to determine from browser history or default to dashboard if uncertain
        previousPage = window.history.length > 1 ? '/' : '/projects';
      }
    }

    // If stores are empty, load data
    if ($projectsStore.length === 0) {
      try {
        await loadAllData();
      } catch (err) {
        console.error('Failed to load projects:', err);
        error = 'Failed to load project data';
        loading = false;
      }
    } else {
      // Data already loaded, check if project exists
      const foundProject = findEntityById($projectsStore, projectId);
      if (foundProject) {
        project = foundProject as Project;
      } else {
        error = `Project with ID "${projectId}" not found`;
      }
      loading = false;
    }
  });

  function handleClose() {
    // Use browser history to go back if possible, otherwise use previousPage
    if (window.history.length > 1) {
      window.history.back();
    } else {
      push(previousPage);
    }
  }

  function handleProjectUpdate() {
    // Reload data to get the updated project
    loadAllData();
  }
</script>

<!-- Backdrop -->
<div 
  class="fixed inset-0 bg-black bg-opacity-50 backdrop-blur-sm z-40"
  on:click={handleClose}
  on:keydown={(e) => e.key === 'Escape' && handleClose()}
  role="button"
  tabindex="-1"
  aria-label="Close detail view"
  in:fade={{ duration: 200 }}
  out:fade={{ duration: 200 }}
></div>

<!-- Sliding Panel -->
<div 
  class="fixed top-0 right-0 h-full bg-emittiv-black z-50 overflow-hidden shadow-2xl flex flex-col project-detail-page" 
  style="width: calc(100vw - 240px); left: 240px;"
  in:fly={{ x: '100%', duration: 300, easing: cubicOut }}
  out:fly={{ x: '100%', duration: 250, easing: cubicOut }}
>
  {#if loading}
    <div class="loading-container">
      <LoadingSkeleton rows={8} />
    </div>
  {:else if error}
    <div class="error-container">
      <div class="error-card">
        <h2>Project Not Found</h2>
        <p>{error}</p>
        <button class="back-button" on:click={handleClose}>
          ← Back to Projects
        </button>
      </div>
    </div>
  {:else if project}
    <ProjectDetail 
      {project} 
      isOpen={true} 
      on:close={handleClose}
      on:projectUpdated={handleProjectUpdate}
    />
  {/if}
</div>

<style>
  .project-detail-page {
    min-height: 100vh;
    background: var(--emittiv-black);
    color: var(--emittiv-white);
  }

  .loading-container {
    padding: 48px 24px;
    max-width: 800px;
    margin: 0 auto;
  }

  .error-container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 50vh;
    padding: 24px;
  }

  .error-card {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 16px;
    padding: 48px;
    text-align: center;
    max-width: 400px;
  }

  .error-card h2 {
    color: var(--emittiv-white);
    margin: 0 0 16px 0;
    font-size: 24px;
    font-weight: 600;
  }

  .error-card p {
    color: var(--emittiv-lighter);
    margin: 0 0 32px 0;
    font-size: 16px;
    line-height: 1.5;
  }

  .back-button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 12px 24px;
    background: var(--emittiv-splash);
    color: var(--emittiv-black);
    border: none;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: all 200ms ease;
  }

  .back-button:hover {
    background: rgba(255, 153, 0, 0.9);
    transform: translateY(-1px);
  }
</style>