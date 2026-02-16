<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  interface CustomAction {
    handler: () => void;
    label: string;
    disabled?: boolean;
    tooltip?: string;
    icon: string;
  }

  const dispatch = createEventDispatcher();

  let { isOpen = false, title = '', canEdit = true, customActions = [], show = true }: {
    isOpen?: boolean;
    title?: string;
    canEdit?: boolean;
    customActions?: CustomAction[];
    show?: boolean;
  } = $props();

  function closePanel() {
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

{#if show && isOpen}
  <!-- Backdrop -->
  <div
    class="emittiv-backdrop emittiv-backdrop--blur"
    on:click={closePanel}
    on:keydown={e => e.key === 'Escape' && closePanel()}
    role="button"
    tabindex="-1"
    aria-label="Close detail view"
    in:fade={{ duration: 200 }}
    out:fade={{ duration: 200 }}
  ></div>

  <!-- Sliding Panel -->
  <div
    class="emittiv-detail-panel"
    style="width: calc(100vw - 240px); left: 240px;"
    in:fly={{ x: '100%', duration: 300, easing: cubicOut }}
    out:fly={{ x: '100%', duration: 250, easing: cubicOut }}
  >
    <!-- Header Section -->
    <div
      class="relative bg-gradient-to-br from-emittiv-darker to-emittiv-black border-b border-emittiv-dark"
    >
      <!-- Top Right Buttons -->
      <div class="absolute top-6 right-6 flex items-center gap-1">
        <!-- Custom Action Buttons -->
        {#each customActions as action}
          <button
            on:click={() => action.handler()}
            class="emittiv-icon-btn"
            aria-label={action.label}
            disabled={action.disabled}
            title={action.tooltip}
          >
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d={action.icon}
              />
            </svg>
          </button>
        {/each}
        {#if canEdit}
          <!-- Edit Button -->
          <button
            on:click={handleEdit}
            class="emittiv-icon-btn"
            aria-label="Edit {title}"
          >
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"
              />
            </svg>
          </button>
        {/if}
        <!-- Close Button -->
        <button
          on:click={closePanel}
          class="emittiv-icon-btn"
          aria-label="Close detail view"
        >
          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
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
