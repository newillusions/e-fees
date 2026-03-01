<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import BaseListCard from './BaseListCard.svelte';
  import ActionButton from './ActionButton.svelte';
  import StatusBadge from './StatusBadge.svelte';
  import type { Project } from '../../types';
  
  const dispatch = createEventDispatcher();
  
  let { project, clickable = true, showFolderLink = true, onFolderClick = undefined, selectable = false, selected = false }: {
    project: Project;
    clickable?: boolean;
    showFolderLink?: boolean;
    onFolderClick?: ((project: Project) => void) | undefined;
    selectable?: boolean;
    selected?: boolean;
  } = $props();
  
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

<BaseListCard {clickable} {selectable} {selected} on:click={handleCardClick} on:select>
  <!-- Title -->
  <svelte:fragment slot="title">
    <h3 class="emittiv-card-title">
      {project.number?.id} - {project.name}
    </h3>
  </svelte:fragment>
  
  <!-- Subtitle -->
  <svelte:fragment slot="subtitle">
    <p class="text-sm text-emittiv-lighter">
      {project.area}, {project.city}, {project.country}
    </p>
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
  
  <!-- Extra - Full width body section with all metadata -->
  <svelte:fragment slot="extra">
    <div class="emittiv-card-meta">
      {#if showFolderLink && project.folder}
        <button 
          on:click={handleFolderClick}
          class="emittiv-link"
          title="Click to open in file explorer"
        >
          {project.folder}
        </button>
      {/if}
      <span>Short Name:<br/>{project.name_short || '—'}</span>
      <span>Created:<br/>{project.time?.created_at ? new Date(project.time.created_at).toISOString().slice(2,10).replace(/-/g,'') : '—'}</span>
      <span>Updated:<br/>{project.time?.updated_at ? new Date(project.time.updated_at).toISOString().slice(2,10).replace(/-/g,'') : '—'}</span>
    </div>
  </svelte:fragment>
</BaseListCard>