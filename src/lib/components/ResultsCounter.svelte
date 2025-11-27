<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let totalItems: number = 0;
  export let filteredItems: number = 0;
  export let hasFilters: boolean = false;
  export let entityName: string = 'items';
  
  function handleClearFilters() {
    dispatch('clear-filters');
  }
  
  $: entityNameSingular = entityName.endsWith('ies') 
    ? entityName.slice(0, -3) + 'y' 
    : entityName.slice(0, -1);
</script>

<!-- Results count and Clear button -->
<div class="flex justify-between items-center mb-2">
  <div class="px-2 py-0.5 text-xs text-emittiv-light border border-emittiv-dark rounded bg-emittiv-darker">
    {#if hasFilters}
      Showing {filteredItems} of {totalItems} {entityName}
    {:else if totalItems > 0}
      {totalItems} {totalItems === 1 ? entityNameSingular : entityName}
    {/if}
  </div>
  {#if hasFilters}
    <button 
      on:click={handleClearFilters}
      class="px-2 py-0.5 text-xs text-emittiv-light hover:text-emittiv-white border border-emittiv-dark hover:border-emittiv-light rounded bg-emittiv-darker transition-smooth flex items-center gap-1"
    >
      <svg style="width: 18px; height: 18px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
      Clear
    </button>
  {/if}
</div>