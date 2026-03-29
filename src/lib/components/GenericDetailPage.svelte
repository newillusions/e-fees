<script lang="ts">
  import { onMount } from 'svelte';
  import { loadAllData } from '$lib/stores';
  import { findEntityById } from '$lib/utils';
  import { logApiError } from '$lib/services/logger';
  import { push } from 'svelte-spa-router';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
  import type { Writable } from 'svelte/store';

  interface Props {
    params?: { id: string };
    entityType: string;
    store: Writable<any[]>;
    backRoute: string;
    DetailComponent: any;
    ModalComponent: any;
    detailProps: (
      entity: any,
      callbacks: { onedit: () => void; onclose: () => void }
    ) => Record<string, unknown>;
    modalProps: (entity: any, callbacks: { onclose: () => void }) => Record<string, unknown>;
  }

  let {
    params = { id: '' },
    entityType,
    store,
    backRoute,
    DetailComponent,
    ModalComponent,
    detailProps,
    modalProps
  }: Props = $props();

  let previousPage = $state(backRoute);
  let entity: any | null = $state(null);
  let loading = $state(true);
  let error: string | null = $state(null);
  // Edit-only modal — create mode not supported from detail pages.
  // Wrappers pass mode: 'edit' via modalProps factory.
  let showModal = $state(false);

  let entityId = $derived(params.id);

  // Find the entity when the store updates or ID changes
  $effect(() => {
    if ($store.length > 0 && entityId) {
      const found = findEntityById($store, entityId);
      if (found) {
        entity = found;
        loading = false;
        error = null;
      } else {
        entity = null;
        loading = false;
        error = `${entityType} with ID "${entityId}" not found`;
      }
    }
  });

  onMount(async () => {
    const urlParams = new URLSearchParams(window.location.search);
    const fromParam = urlParams.get('from');

    if (fromParam === 'dashboard' || window.history.length > 1) {
      if (fromParam === 'dashboard') {
        previousPage = '/';
      } else {
        previousPage = window.history.length > 1 ? '/' : backRoute;
      }
    }

    if ($store.length === 0) {
      try {
        await loadAllData();
      } catch (err) {
        logApiError(`load ${entityType.toLowerCase()}s`, err as Error);
        error = `Failed to load ${entityType.toLowerCase()} data`;
        loading = false;
      }
    } else {
      const found = findEntityById($store, entityId);
      if (found) {
        entity = found;
      } else {
        error = `${entityType} with ID "${entityId}" not found`;
      }
      loading = false;
    }
  });

  function handleClose() {
    if (window.history.length > 1) {
      window.history.back();
    } else {
      push(previousPage);
    }
  }

  function handleEdit() {
    showModal = true;
  }

  function handleModalClose() {
    showModal = false;
    loadAllData();
  }
</script>

<!-- Backdrop -->
<div
  class="emittiv-backdrop emittiv-backdrop--blur"
  onclick={handleClose}
  onkeydown={e => e.key === 'Escape' && handleClose()}
  role="button"
  tabindex="-1"
  aria-label="Close detail view"
  in:fade={{ duration: 200 }}
  out:fade={{ duration: 200 }}
></div>

<!-- Sliding Panel -->
<div
  class="emittiv-detail-panel generic-detail-page"
  style="width: calc(100vw - 240px); left: 240px;"
  in:fly={{ x: '100%', duration: 300, easing: cubicOut }}
  out:fly={{ x: '100%', duration: 250, easing: cubicOut }}
>
  {#if loading}
    <div class="loading-container">
      <LoadingSkeleton rows={8} />
    </div>
  {:else if error}
    <div class="error-container">
      <div class="error-card">
        <h2>{entityType} Not Found</h2>
        <p>{error}</p>
        <button class="back-button" onclick={handleClose}>
          ← Back to {entityType}s
        </button>
      </div>
    </div>
  {:else if entity}
    <svelte:component
      this={DetailComponent}
      {...detailProps(entity, { onedit: handleEdit, onclose: handleClose })}
    />
  {/if}
</div>

<!-- Edit Modal -->
{#if showModal && entity}
  <svelte:component this={ModalComponent} {...modalProps(entity, { onclose: handleModalClose })} />
{/if}

<style>
  .generic-detail-page {
    min-height: 100vh;
    background: var(--emittiv-black);
    color: var(--emittiv-white);
  }

  .loading-container {
    padding: 48px 24px;
    max-width: 800px;
    margin: 0 auto;
  }

  .error-container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 50vh;
    padding: 24px;
  }

  .error-card {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 16px;
    padding: 48px;
    text-align: center;
    max-width: 400px;
  }

  .error-card h2 {
    color: var(--emittiv-white);
    margin: 0 0 16px 0;
    font-size: 24px;
    font-weight: 600;
  }

  .error-card p {
    color: var(--emittiv-lighter);
    margin: 0 0 32px 0;
    font-size: 16px;
    line-height: 1.5;
  }

  .back-button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 12px 24px;
    background: var(--emittiv-splash);
    color: var(--emittiv-black);
    border: none;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: all 200ms ease;
  }

  .back-button:hover {
    background: rgba(255, 153, 0, 0.9);
    transform: translateY(-1px);
  }
</style>
