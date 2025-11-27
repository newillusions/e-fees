<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let title = 'Warning';
  export let message = '';
  export let confirmText = 'OK';
  export let cancelText = '';
  export let onConfirm: (() => void) | null = null;
  export let onCancel: (() => void) | null = null;
  
  function handleConfirm() {
    if (onConfirm) {
      onConfirm();
    }
    dispatch('confirm');
    dispatch('close');
  }
  
  function handleCancel() {
    if (onCancel) {
      onCancel();
    }
    dispatch('cancel');
    dispatch('close');
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      if (cancelText) {
        handleCancel();
      } else {
        handleConfirm();
      }
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 bg-black/60 backdrop-blur-sm z-80 flex items-center justify-center"
    on:click={cancelText ? handleCancel : handleConfirm}
    on:keydown={(e) => e.key === 'Escape' && (cancelText ? handleCancel() : handleConfirm())}
    role="button"
    tabindex="-1"
    aria-label="Close modal"
    in:fade={{ duration: 200 }}
    out:fade={{ duration: 200 }}
  >
    <!-- Modal -->
    <div 
      class="bg-emittiv-darker rounded-lg shadow-2xl border border-emittiv-dark max-w-xs w-full mx-4"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      in:scale={{ duration: 250, start: 0.95, easing: cubicOut }}
      out:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
    >
      <!-- Header -->
      <div class="px-4 py-3 border-b border-emittiv-dark">
        <h3 id="modal-title" class="text-sm font-heading font-semibold text-emittiv-white flex items-center gap-2">
          <svg class="w-4 h-4 text-emittiv-splash" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          {title}
        </h3>
      </div>
      
      <!-- Body -->
      <div class="px-4 py-3">
        <p class="text-sm text-emittiv-lighter whitespace-pre-line">
          {message}
        </p>
      </div>
      
      <!-- Footer -->
      <div class="px-3 py-2 bg-emittiv-black/50 rounded-b-lg flex justify-center gap-2">
        {#if cancelText}
          <button
            on:click={handleCancel}
            class="px-2 py-1 bg-emittiv-dark text-emittiv-lighter rounded text-xs font-medium hover:bg-emittiv-dark/80 hover:text-emittiv-white transition-colors"
          >
            {cancelText}
          </button>
        {/if}
        <button
          on:click={handleConfirm}
          class="px-2 py-1 bg-emittiv-splash text-emittiv-black rounded text-xs font-medium hover:bg-emittiv-splash/90 transition-colors"
        >
          {confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}

