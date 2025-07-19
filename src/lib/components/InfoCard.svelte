<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let title: string = 'Information';
  export let fields: Array<{
    label: string;
    value: string;
    type?: 'text' | 'date' | 'id' | 'link';
    clickable?: boolean;
  }> = [];
  export let columns: number = 3;
  
  function formatValue(value: string, type: string = 'text'): string {
    if (!value) return '—';
    
    switch (type) {
      case 'date':
        return new Date(value).toLocaleDateString('en-US', { 
          month: 'short', 
          day: 'numeric', 
          year: 'numeric' 
        });
      case 'id':
        return value.replace(/^[^:]+:/, ''); // Remove table prefix from IDs
      default:
        return value;
    }
  }
</script>

<section>
  <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider mb-2">{title}</h2>
  <div class="bg-emittiv-black rounded-xl px-3 py-2 border border-emittiv-dark">
    <div class="grid grid-cols-{columns} gap-2 text-sm">
      {#each fields as field, index}
        <div>
          <div class="text-xs text-emittiv-light">{field.label}</div>
          {#if field.clickable && field.value && field.value !== '—'}
            <button 
              class="text-sm text-emittiv-splash hover:text-orange-400 underline hover:no-underline transition-all text-left"
              on:click={() => dispatch('field-click', { field, index })}
              title="Click to open"
            >
              {formatValue(field.value, field.type)}
            </button>
          {:else}
            <div class="text-sm text-emittiv-white">{formatValue(field.value, field.type)}</div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</section>