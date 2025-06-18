<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import BaseListCard from './BaseListCard.svelte';
  import ActionButton from './ActionButton.svelte';
  import StatusBadge from './StatusBadge.svelte';
  import type { Project } from '../../types';
  
  const dispatch = createEventDispatcher();
  
  export let project: Project;
  export let clickable = true;
  export let showFolderLink = true;
  export let onFolderClick: ((project: Project) => void) | undefined = undefined;
  
  function handleCardClick() {
    if (clickable) {
      dispatch('view', project);
    }
  }
  
  function handleEdit(event: Event) {
    event.stopPropagation();
    dispatch('edit', project);
  }
  
  function handleView(event: Event) {
    event.stopPropagation();
    dispatch('view', project);
  }
  
  function handleFolderClick(event: Event) {
    event.stopPropagation();
    if (onFolderClick) {
      onFolderClick(project);
    }
  }
</script>

<BaseListCard {clickable} on:click={handleCardClick}>
  <!-- Title -->
  <svelte:fragment slot="title">
    <h3 class="text-base font-medium text-emittiv-white group-hover:text-emittiv-splash transition-colors truncate">
      {project.number?.id} - {project.name}
    </h3>
  </svelte:fragment>
  
  <!-- Subtitle -->
  <svelte:fragment slot="subtitle">
    <p class="text-sm text-emittiv-lighter">
      {project.area}, {project.city}, {project.country}
    </p>
  </svelte:fragment>
  
  <!-- Metadata -->
  <svelte:fragment slot="metadata">
    <div class="flex items-center gap-4 text-xs text-emittiv-light">
      {#if showFolderLink && project.folder}
        <button 
          on:click={handleFolderClick}
          class="text-emittiv-splash hover:text-orange-400 underline hover:no-underline transition-all"
          title="Click to open in file explorer"
        >
          {project.folder}
        </button>
      {/if}
      <span>Created: {new Date(project.time.created_at).toLocaleDateString()}</span>
    </div>
  </svelte:fragment>
  
  <!-- Badge -->
  <svelte:fragment slot="badge">
    <StatusBadge status={project.status} type="project" />
  </svelte:fragment>
  
  <!-- Actions -->
  <svelte:fragment slot="actions">
    <ActionButton 
      type="edit" 
      ariaLabel="Edit project"
      on:click={handleEdit}
    />
    <ActionButton 
      type="view" 
      ariaLabel="View project details"
      on:click={handleView}
    />
  </svelte:fragment>
</BaseListCard>