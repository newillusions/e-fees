<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  
  export let isOpen = false;
  export let title = '';
  export let maxWidth = '450px';
  export let showCloseButton = true;
  export let customClass = '';
  
  const dispatch = createEventDispatcher();
  
  // Debug logging
  $: {
    console.log('BaseModal - isOpen changed:', isOpen, 'title:', title);
  }
  
  function closeModal() {
    dispatch('close');
  }
  
  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      closeModal();
    }
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeModal();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Modal backdrop -->
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 {customClass}"
    style="z-index: 100;"
    on:click={handleBackdropClick}
    on:keydown={() => {}}
    role="button"
    tabindex="-1"
    aria-label="Close modal"
    in:fade={{ duration: 200 }}
    out:fade={{ duration: 200 }}
  ></div>
  
  <!-- Modal container -->
  <div 
    class="fixed inset-0 flex items-center justify-center p-4 pointer-events-none {customClass}"
    style="z-index: 101;"
    role="dialog"
    aria-modal="true"
    aria-labelledby={title ? 'modal-title' : undefined}
  >
    <div 
      class="bg-emittiv-darker border border-emittiv-dark rounded w-full max-h-[90vh] overflow-y-auto pointer-events-auto"
      style="padding: 16px; max-width: {maxWidth};"
      on:click={(e) => e.stopPropagation()}
      in:scale={{ duration: 250, start: 0.95, easing: cubicOut }}
      out:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
    >
      {#if title || showCloseButton}
        <!-- Modal Header -->
        <div class="flex items-center justify-between" style="margin-bottom: 20px;">
          {#if title}
            <h2 id="modal-title" class="font-semibold text-emittiv-white" style="font-size: 16px;">
              {title}
            </h2>
          {:else}
            <div></div>
          {/if}
          
          {#if showCloseButton}
            <button 
              on:click={closeModal}
              class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth"
              aria-label="Close modal"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          {/if}
        </div>
      {/if}
      
      <!-- Modal Content -->
      <div>
        <slot />
      </div>
    </div>
  </div>
{/if}

<style>
  .transition-smooth {
    transition: all 300ms cubic-bezier(0.4, 0, 0.2, 1);
  }
</style>