<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import BaseListCard from './BaseListCard.svelte';
  import ActionButton from './ActionButton.svelte';
  import StatusBadge from './StatusBadge.svelte';
  import type { FeeProposal } from '../../types';
  
  const dispatch = createEventDispatcher();
  
  export let proposal: FeeProposal;
  export let clickable = true;
  export let projectName: string = '';
  export let companyName: string = '';
  export let contactName: string = '';
  
  function handleCardClick() {
    if (clickable) {
      dispatch('view', proposal);
    }
  }
  
  function handleEdit(event: Event) {
    event.stopPropagation();
    dispatch('edit', proposal);
  }
  
  function handleView(event: Event) {
    event.stopPropagation();
    dispatch('view', proposal);
  }
</script>

<BaseListCard {clickable} on:click={handleCardClick}>
  <!-- Title -->
  <svelte:fragment slot="title">
    <h3 class="text-base font-medium text-emittiv-white group-hover:text-emittiv-splash transition-colors truncate">
      {proposal.number} - {projectName || 'Unknown Project'}
    </h3>
  </svelte:fragment>
  
  <!-- Subtitle -->
  <svelte:fragment slot="subtitle">
    <p class="text-sm text-emittiv-lighter">
      {companyName || 'N/A'}
      {#if contactName}
        • {contactName}
      {/if}
    </p>
  </svelte:fragment>
  
  <!-- Badge -->
  <svelte:fragment slot="badge">
    <StatusBadge status={proposal.status} type="proposal" />
  </svelte:fragment>
  
  <!-- Actions -->
  <svelte:fragment slot="actions">
    <ActionButton 
      type="edit" 
      ariaLabel="Edit proposal"
      on:click={handleEdit}
    />
    <ActionButton 
      type="view" 
      ariaLabel="View proposal details"
      on:click={handleView}
    />
  </svelte:fragment>
  
  <!-- Extra - Full width body section with all metadata -->
  <svelte:fragment slot="extra">
    <div class="space-y-1">
      {#if proposal.package}
        <div class="text-sm text-emittiv-light">
          Package: {proposal.package}
        </div>
      {/if}
      <div class="flex items-center gap-4 text-xs text-emittiv-light">
        <span>Rev:<br/>{proposal.rev}</span>
        <span>Staff:<br/>{proposal.staff_name || 'N/A'}</span>
        <span>Issue Date:<br/>{proposal.issue_date.length === 6 ? proposal.issue_date : new Date(proposal.issue_date).toISOString().slice(2,10).replace(/-/g,'')}</span>
        <span>Created:<br/>{new Date(proposal.time.created_at).toISOString().slice(2,10).replace(/-/g,'')}</span>
      </div>
    </div>
  </svelte:fragment>
</BaseListCard>