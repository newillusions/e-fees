<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let title = '';
  export let canEdit = true;
  export let customActions = [];
  
  function closePanel() {
    isOpen = false;
    dispatch('close');
  }
  
  function handleEdit() {
    dispatch('edit');
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closePanel();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 backdrop-blur-sm z-40 transition-opacity"
    on:click={closePanel}
    on:keydown={(e) => e.key === 'Escape' && closePanel()}
    role="button"
    tabindex="-1"
    aria-label="Close detail view"
  ></div>
  
  <!-- Sliding Panel -->
  <div class="fixed top-0 right-0 h-full bg-emittiv-black z-50 overflow-hidden slide-in-right shadow-2xl flex flex-col" style="width: calc(100vw - 240px); left: 240px;">
    <!-- Header Section -->
    <div class="relative bg-gradient-to-br from-emittiv-darker to-emittiv-black border-b border-emittiv-dark">
      <!-- Top Right Buttons -->
      <div class="absolute top-6 right-6 flex items-center gap-1">
        <!-- Custom Action Buttons -->
        {#each customActions as action}
          <button 
            on:click={action.handler}
            class="p-1 rounded text-emittiv-light hover:text-emittiv-splash hover:bg-emittiv-dark transition-all"
            aria-label={action.label}
            disabled={action.disabled}
            title={action.tooltip}
          >
            <svg style="width: 16px; height: 16px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={action.icon} />
            </svg>
          </button>
        {/each}
        {#if canEdit}
          <!-- Edit Button -->
          <button 
            on:click={handleEdit}
            class="p-1 rounded text-emittiv-light hover:text-emittiv-splash hover:bg-emittiv-dark transition-all"
            aria-label="Edit {title}"
          >
            <svg style="width: 16px; height: 16px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
            </svg>
          </button>
        {/if}
        <!-- Close Button -->
        <button 
          on:click={closePanel}
          class="p-1 rounded text-emittiv-light hover:text-emittiv-splash hover:bg-emittiv-dark transition-all"
          aria-label="Close detail view"
        >
          <svg style="width: 16px; height: 16px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
      
      <!-- Header Content Slot -->
      <slot name="header" />
    </div>
    
    <!-- Scrollable Content Area -->
    <div class="flex-1 overflow-y-auto">
      <div class="px-8 py-6 pb-16 space-y-8">
        <slot name="content" />
      </div>
    </div>
  </div>
{/if}

<style>
  .slide-in-right {
    animation: slideInRight 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }
  
  @keyframes slideInRight {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }
  
  /* Custom scrollbar for dark theme */
  .overflow-y-auto::-webkit-scrollbar {
    width: 8px;
  }
  
  .overflow-y-auto::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
  }
  
  .overflow-y-auto::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
  }
  
  .overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }
</style>