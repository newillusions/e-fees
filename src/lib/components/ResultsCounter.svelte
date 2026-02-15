<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  let { totalItems = 0, filteredItems = 0, hasFilters = false, entityName = 'items', loadedItems = undefined as number | undefined, hasMore = false, inline = false }: {
    totalItems?: number;
    filteredItems?: number;
    hasFilters?: boolean;
    entityName?: string;
    loadedItems?: number | undefined;
    hasMore?: boolean;
    inline?: boolean;
  } = $props();

  function handleClearFilters() {
    dispatch('clear-filters');
  }

  const entityNameSingular = $derived(entityName.endsWith('ies')
    ? entityName.slice(0, -3) + 'y'
    : entityName.slice(0, -1));

  // Determine if we're in paginated mode
  const isPaginated = $derived(loadedItems !== undefined && totalItems > 0);
</script>

<!-- Results count and Clear button -->
<div class="flex items-center gap-2 {inline ? '' : 'justify-between mb-2'}">
  <div class="px-2 py-0.5 text-xs text-emittiv-light border border-emittiv-dark rounded bg-emittiv-darker whitespace-nowrap">
    {#if hasFilters}
      Showing {filteredItems} of {totalItems} {entityName}
    {:else if isPaginated && hasMore}
      Showing {loadedItems} of {totalItems} {entityName}
    {:else if totalItems > 0}
      {totalItems} {totalItems === 1 ? entityNameSingular : entityName}
    {/if}
  </div>
  {#if hasFilters}
    <button
      on:click={handleClearFilters}
      class="px-2 py-0.5 text-xs text-emittiv-light hover:text-emittiv-white border border-emittiv-dark hover:border-emittiv-light rounded bg-emittiv-darker transition-smooth flex items-center gap-1 whitespace-nowrap"
    >
      <svg style="width: 18px; height: 18px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
      Clear
    </button>
  {/if}
</div>