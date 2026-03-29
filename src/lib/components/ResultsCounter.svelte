<script lang="ts">
  let {
    totalItems = 0,
    filteredItems = 0,
    hasFilters = false,
    entityName = 'items',
    loadedItems = undefined as number | undefined,
    hasMore = false,
    inline = false,
    onclearfilters
  }: {
    totalItems?: number;
    filteredItems?: number;
    hasFilters?: boolean;
    entityName?: string;
    loadedItems?: number | undefined;
    hasMore?: boolean;
    inline?: boolean;
    onclearfilters?: () => void;
  } = $props();

  function handleClearFilters() {
    onclearfilters?.();
  }

  const entityNameSingular = $derived(
    entityName.endsWith('ies') ? entityName.slice(0, -3) + 'y' : entityName.slice(0, -1)
  );

  // Determine if we're in paginated mode
  const isPaginated = $derived(loadedItems !== undefined && totalItems > 0);
</script>

<!-- Results count and Clear button -->
<div class="flex items-center gap-2 {inline ? '' : 'justify-between mb-2'}">
  <div class="emittiv-chip">
    {#if hasFilters}
      Showing {filteredItems} of {totalItems} {entityName}
    {:else if isPaginated && hasMore}
      Showing {loadedItems} of {totalItems} {entityName}
    {:else if totalItems > 0}
      {totalItems} {totalItems === 1 ? entityNameSingular : entityName}
    {/if}
  </div>
  {#if hasFilters}
    <button onclick={handleClearFilters} class="emittiv-chip emittiv-chip--btn">
      <svg style="width: 18px; height: 18px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M6 18L18 6M6 6l12 12"
        />
      </svg>
      Clear
    </button>
  {/if}
</div>
