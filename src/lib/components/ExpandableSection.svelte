<script lang="ts">
  import { slide } from 'svelte/transition';
  
  export let title: string;
  export let totalCount: number;
  export let showAll: boolean = false;
  export let items: any[] = [];
  export let maxItems: number = 3;
  
  function toggleExpanded() {
    showAll = !showAll;
  }
</script>

<section>
  <div class="flex items-center justify-between mb-2">
    <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider">{title}</h2>
    <span class="text-xs text-emittiv-light bg-emittiv-darker px-2 py-1 rounded-lg">
      {totalCount} total
    </span>
  </div>
  
  {#if items.length === 0}
    <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
      <slot name="empty-state" />
    </div>
  {:else}
    <div class="grid gap-3">
      <!-- Always show first few items -->
      <slot name="items" {items} visibleItems={items.slice(0, maxItems)} />
      
      <!-- Expandable additional items -->
      {#if showAll && items.length > maxItems}
        <div transition:slide={{ duration: 400, axis: 'y' }}>
          <slot name="additional-items" {items} additionalItems={items.slice(maxItems)} />
        </div>
      {/if}
      
      <!-- Expand/collapse button -->
      {#if items.length > maxItems}
        <button 
          on:click={toggleExpanded}
          class="text-sm text-emittiv-splash hover:text-orange-400 transition-colors text-center py-2"
        >
          {showAll ? '↑ Show less' : `View all ${items.length} ${title.toLowerCase()} →`}
        </button>
      {/if}
    </div>
  {/if}
</section>