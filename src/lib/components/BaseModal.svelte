<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  let { isOpen = $bindable(false), title = '', maxWidth = '450px', showCloseButton = true, customClass = '', zIndex = 100 }: {
    isOpen?: boolean;
    title?: string;
    maxWidth?: string;
    showCloseButton?: boolean;
    customClass?: string;
    zIndex?: number;
  } = $props();

  const dispatch = createEventDispatcher();

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
    class="emittiv-backdrop {customClass}"
    style="z-index: {zIndex};"
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
    style="z-index: {zIndex + 1};"
    role="dialog"
    aria-modal="true"
    aria-labelledby={title ? 'modal-title' : undefined}
  >
    <div
      class="emittiv-modal w-full pointer-events-auto"
      style="max-width: {maxWidth};"
      on:click={e => e.stopPropagation()}
      in:scale={{ duration: 250, start: 0.95, easing: cubicOut }}
      out:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
    >
      {#if title || showCloseButton}
        <!-- Modal Header -->
        <div class="emittiv-modal__header">
          {#if title}
            <h2 id="modal-title" class="emittiv-modal__title">
              {title}
            </h2>
          {:else}
            <div></div>
          {/if}

          {#if showCloseButton}
            <button on:click={closeModal} class="emittiv-modal__close" aria-label="Close modal">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
      {/if}

      <!-- Modal Content -->
      <div>
        <slot />
      </div>
    </div>
  </div>
{/if}

