<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  const modalId = `warning-${Math.random().toString(36).substr(2, 9)}`;

  let {
    isOpen = false,
    title = 'Warning',
    message = '',
    confirmText = 'OK',
    cancelText = '',
    onConfirm = null,
    onCancel = null,
    onconfirm,
    onclose,
    oncancel
  }: {
    isOpen?: boolean;
    title?: string;
    message?: string;
    confirmText?: string;
    cancelText?: string;
    onConfirm?: (() => void) | null;
    onCancel?: (() => void) | null;
    onconfirm?: () => void;
    onclose?: () => void;
    oncancel?: () => void;
  } = $props();

  function handleConfirm() {
    if (onConfirm) {
      onConfirm();
    }
    onconfirm?.();
    onclose?.();
  }

  function handleCancel() {
    if (onCancel) {
      onCancel();
    }
    oncancel?.();
    onclose?.();
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
    class="emittiv-backdrop emittiv-backdrop--dark emittiv-backdrop--blur flex items-center justify-center"
    style="z-index: 80;"
    on:click={cancelText ? handleCancel : handleConfirm}
    on:keydown={e => e.key === 'Escape' && (cancelText ? handleCancel() : handleConfirm())}
    tabindex="-1"
    in:fade={{ duration: 200 }}
    out:fade={{ duration: 200 }}
  >
    <!-- Modal -->
    <div
      class="bg-emittiv-darker rounded-lg shadow-2xl border border-emittiv-dark max-w-sm w-full mx-4"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      aria-modal="true"
      aria-labelledby="{modalId}-title"
      in:scale={{ duration: 250, start: 0.95, easing: cubicOut }}
      out:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
    >
      <!-- Header -->
      <div class="px-4 py-3 border-b border-emittiv-dark">
        <h3
          id="{modalId}-title"
          class="text-sm font-heading font-semibold text-emittiv-white flex items-center gap-2"
        >
          <svg
            class="w-4 h-4 text-emittiv-splash"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
          {title}
        </h3>
      </div>

      <!-- Body -->
      <div class="px-4 py-3">
        <p
          class="text-sm text-emittiv-lighter whitespace-pre-line"
          style="overflow-wrap: break-word; word-break: break-word;"
        >
          {message}
        </p>
      </div>

      <!-- Footer -->
      <div class="px-3 py-2 bg-emittiv-black/50 rounded-b-lg flex justify-center gap-2">
        {#if cancelText}
          <button
            on:click={handleCancel}
            class="emittiv-confirm-btn emittiv-confirm-btn--secondary"
          >
            {cancelText}
          </button>
        {/if}
        <button on:click={handleConfirm} class="emittiv-confirm-btn emittiv-confirm-btn--primary">
          {confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}
