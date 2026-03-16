<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  let { isOpen = $bindable(false), title = '', maxWidth = '', size = 'md' as 'sm' | 'md' | 'lg' | 'xl', showCloseButton = true, customClass = '', zIndex = 100 }: {
    isOpen?: boolean;
    title?: string;
    maxWidth?: string;
    size?: 'sm' | 'md' | 'lg' | 'xl';
    showCloseButton?: boolean;
    customClass?: string;
    zIndex?: number;
  } = $props();

  const dispatch = createEventDispatcher();
  const modalId = `modal-${Math.random().toString(36).substr(2, 9)}`;

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

  function trapFocus(node: HTMLElement) {
    function getFocusable() {
      return node.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
    }

    function handleTab(e: KeyboardEvent) {
      if (e.key !== 'Tab') return;
      const focusable = getFocusable();
      if (focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];

      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }

    node.addEventListener('keydown', handleTab);
    requestAnimationFrame(() => {
      const focusable = getFocusable();
      if (focusable.length > 0) focusable[0].focus();
    });

    return {
      destroy() { node.removeEventListener('keydown', handleTab); }
    };
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
    tabindex="-1"
    in:fade={{ duration: 200 }}
    out:fade={{ duration: 200 }}
  ></div>

  <!-- Modal container -->
  <div
    class="fixed inset-0 flex items-center justify-center p-4 pointer-events-none {customClass}"
    style="z-index: {zIndex + 1};"
    role="dialog"
    aria-modal="true"
    aria-labelledby={title ? `${modalId}-title` : undefined}
  >
    <div
      class="emittiv-modal emittiv-modal--{size} w-full pointer-events-auto"
      style={maxWidth ? `max-width: ${maxWidth};` : undefined}
      on:click={e => e.stopPropagation()}
      in:scale={{ duration: 250, start: 0.95, easing: cubicOut }}
      out:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
      use:trapFocus
    >
      {#if title || showCloseButton}
        <!-- Modal Header -->
        <div class="emittiv-modal__header">
          {#if title}
            <h2 id="{modalId}-title" class="emittiv-modal__title">
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
