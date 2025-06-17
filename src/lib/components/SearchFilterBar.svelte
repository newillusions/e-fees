<script lang="ts">
  import { hasActiveFilters, clearAllFilters } from '$lib/utils/filters';
  
  // Props
  export let searchQuery: string = '';
  export let filters: Record<string, string> = {};
  export let filterOptions: Array<{
    key: string;
    label: string;
    options: string[];
  }> = [];
  export let placeholder: string = 'Search...';
  export let onAdd: (() => void) | null = null;
  export let addLabel: string = 'Add Item';
  export let resultCount: number = 0;
  export let totalCount: number = 0;
  export let itemName: string = 'items';
  
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
          <input
            type="text"
            {placeholder}
            bind:value={searchQuery}
            class="w-full px-2 py-2.5 bg-emittiv-darker border border-emittiv-dark rounded-lg text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
          />
        </div>
        <button 
          class="p-2.5 bg-emittiv-darker border border-emittiv-dark rounded-lg text-emittiv-light hover:text-emittiv-white hover:border-emittiv-splash transition-all"
          aria-label="Search"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </button>
      </div>
    </div>
    {#if onAdd}
      <button
        class="ml-4 w-12 h-12 rounded-full bg-emittiv-splash hover:bg-orange-600 text-emittiv-black flex items-center justify-center transition-smooth hover:scale-105 active:scale-95 shadow-lg"
        on:click={onAdd}
        aria-label={addLabel}
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    {/if}
  </div>
  
  <!-- Filter Options -->
  {#if filterOptions.length > 0}
    <div class="flex flex-wrap items-center gap-2">
      {#each filterOptions as filterOption}
        <select 
          bind:value={filters[filterOption.key]} 
          class="px-2 py-1 pr-6 bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-white text-xs hover:border-emittiv-splash focus:outline-none focus:border-emittiv-splash transition-all cursor-pointer"
        >
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
    <div class="px-2 py-0.5 text-xs text-emittiv-light border border-emittiv-dark rounded bg-emittiv-darker">
      {#if hasFiltersActive}
        Showing {resultCount} of {totalCount} {itemName}
      {:else if totalCount > 0}
        {totalCount} {itemName.replace(/ies$/, 'y').replace(/s$/, '')}{totalCount === 1 ? '' : totalCount > 1 && itemName.endsWith('ies') ? 'ies' : 's'}
      {/if}
    </div>
    {#if hasFiltersActive}
      <button 
        on:click={clearFilters}
        class="px-2 py-0.5 text-xs text-emittiv-light hover:text-emittiv-white border border-emittiv-dark hover:border-emittiv-light rounded bg-emittiv-darker transition-smooth flex items-center gap-1"
      >
        <svg style="width: 18px; height: 18px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
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