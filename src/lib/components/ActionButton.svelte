<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let type: 'edit' | 'view' | 'delete' | 'email' | 'phone' | 'custom' = 'custom';
  export let href: string = '';
  export let ariaLabel: string = '';
  export let size: number = 16;
  export let customIcon: string = '';
  
  function handleClick(event: MouseEvent) {
    event.stopPropagation();
    dispatch('click');
  }
  
  function getIcon(type: string): string {
    switch (type) {
      case 'edit':
        return 'M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z';
      case 'view':
        return 'M9 5l7 7-7 7';
      case 'delete':
        return 'M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16';
      case 'email':
        return 'M3 8l7.89 7.89a2 2 0 002.83 0L21 9M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z';
      case 'phone':
        return 'M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z';
      default:
        return customIcon;
    }
  }
</script>

{#if href}
  <a 
    {href}
    class="p-1 rounded text-emittiv-light hover:text-emittiv-splash hover:bg-emittiv-dark transition-all"
    aria-label={ariaLabel}
  >
    <svg style="width: {size}px; height: {size}px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getIcon(type)} />
    </svg>
  </a>
{:else}
  <button 
    on:click={handleClick}
    class="p-1 rounded text-emittiv-light hover:text-emittiv-splash hover:bg-emittiv-dark transition-all"
    aria-label={ariaLabel}
  >
    <svg style="width: {size}px; height: {size}px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getIcon(type)} />
    </svg>
  </button>
{/if}