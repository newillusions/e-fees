<script lang="ts">
  import { hasActiveFilters, clearAllFilters } from '$lib/utils/filters';

  // Props using Svelte 5 syntax
  let {
    searchQuery = $bindable(''),
    filters = $bindable({}),
    filterOptions = [],
    placeholder = 'Search...',
    onAdd = null,
    addLabel = 'Add Item',
    resultCount = 0,
    totalCount = 0,
    itemName = 'items'
  }: {
    searchQuery: string;
    filters: Record<string, string>;
    filterOptions?: Array<{
      key: string;
      label: string;
      options: string[];
    }>;
    placeholder?: string;
    onAdd?: (() => void) | null;
    addLabel?: string;
    resultCount?: number;
    totalCount?: number;
    itemName?: string;
  } = $props();

  // Internal state
  let hasFiltersActive = $derived(hasActiveFilters(filters, searchQuery));

  function clearFilters() {
    searchQuery = clearAllFilters(filters);
  }
</script>

<div class="space-y-4">
  <!-- Search and Add Button Row -->
  <div class="flex justify-between items-center">
    <div class="flex-1 max-w-2xl">
      <div class="flex items-center gap-2">
        <div class="relative flex-1">
          <input type="text" {placeholder} bind:value={searchQuery} class="emittiv-search-input" />
        </div>
        {#if searchQuery}
          <button
            class="emittiv-icon-btn"
            onclick={() => (searchQuery = '')}
            aria-label="Clear search"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        {/if}
      </div>
    </div>
    {#if onAdd}
      <button class="emittiv-fab ml-4" onclick={onAdd} aria-label={addLabel}>
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 4v16m8-8H4"
          />
        </svg>
      </button>
    {/if}
  </div>

  <!-- Filter Options -->
  {#if filterOptions.length > 0}
    <div class="flex flex-wrap items-center gap-2">
      {#each filterOptions as filterOption}
        <select bind:value={filters[filterOption.key]} class="emittiv-filter-select">
          <option value="">{filterOption.label}</option>
          {#each filterOption.options as option}
            <option value={option}>{option}</option>
          {/each}
        </select>
      {/each}
    </div>
  {/if}

  <!-- Results count and Clear button -->
  <div class="flex justify-between items-center">
    <div class="emittiv-chip">
      {#if hasFiltersActive}
        Showing {resultCount} of {totalCount} {itemName}
      {:else if totalCount > 0}
        {totalCount}
        {itemName.replace(/ies$/, 'y').replace(/s$/, '')}{totalCount === 1
          ? ''
          : totalCount > 1 && itemName.endsWith('ies')
            ? 'ies'
            : 's'}
      {/if}
    </div>
    {#if hasFiltersActive}
      <button onclick={clearFilters} class="emittiv-chip emittiv-chip--btn">
        <svg
          style="width: 18px; height: 18px;"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
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
