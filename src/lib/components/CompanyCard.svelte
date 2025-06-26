<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import BaseListCard from './BaseListCard.svelte';
  import ActionButton from './ActionButton.svelte';
  import type { Company } from '../../types';
  
  const dispatch = createEventDispatcher();
  
  export let company: Company;
  export let clickable = true;
  
  function handleCardClick() {
    if (clickable) {
      dispatch('view', company);
    }
  }
  
  function handleEdit(event: Event) {
    event.stopPropagation();
    dispatch('edit', company);
  }
  
  function handleView(event: Event) {
    event.stopPropagation();
    dispatch('view', company);
  }
</script>

<BaseListCard {clickable} on:click={handleCardClick}>
  <!-- Title -->
  <svelte:fragment slot="title">
    <h3 class="text-base font-medium text-emittiv-white group-hover:text-emittiv-splash transition-colors truncate">
      {company.name}
    </h3>
  </svelte:fragment>
  
  <!-- Subtitle -->
  <svelte:fragment slot="subtitle">
    <p class="text-sm text-emittiv-lighter">
      {company.city}, {company.country}
    </p>
  </svelte:fragment>
  
  <!-- Badge -->
  <svelte:fragment slot="badge">
    <span class="px-2 py-1 rounded-lg text-xs font-medium text-emittiv-splash bg-emittiv-splash/10">
      {company.abbreviation}
    </span>
  </svelte:fragment>
  
  <!-- Actions -->
  <svelte:fragment slot="actions">
    <ActionButton 
      type="edit" 
      ariaLabel="Edit company"
      on:click={handleEdit}
    />
    <ActionButton 
      type="view" 
      ariaLabel="View company details"
      on:click={handleView}
    />
  </svelte:fragment>
  
  <!-- Extra - Full width body section with all metadata -->
  <svelte:fragment slot="extra">
    <div class="flex items-center gap-4 text-xs text-emittiv-light">
      <span>Short:<br/>{company.name_short}</span>
      {#if company.reg_no}
        <span>Reg:<br/>{company.reg_no}</span>
      {/if}
      {#if company.tax_no}
        <span>VAT:<br/>{company.tax_no}</span>
      {/if}
      <span>Created:<br/>{new Date(company.time.created_at).toISOString().slice(2,10).replace(/-/g,'')}</span>
      <span>Updated:<br/>{new Date(company.time.updated_at).toISOString().slice(2,10).replace(/-/g,'')}</span>
    </div>
  </svelte:fragment>
</BaseListCard>