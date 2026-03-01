<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  let {
    selectedCount = 0,
    entityType = 'items',
    statuses = [] as string[],
  }: {
    selectedCount: number;
    entityType: string;
    statuses?: string[];
  } = $props();

  let selectedStatus = $state('');
  let confirming: 'delete' | null = $state(null);

  function handleStatusChange() {
    if (!selectedStatus) return;
    dispatch('status-change', selectedStatus);
    selectedStatus = '';
  }

  function handleDelete() {
    if (confirming === 'delete') {
      dispatch('delete');
      confirming = null;
    } else {
      confirming = 'delete';
    }
  }

  function handleClear() {
    confirming = null;
    dispatch('clear');
  }

  // Reset confirmation when selection changes
  $effect(() => {
    if (selectedCount === 0) confirming = null;
  });
</script>

{#if selectedCount > 0}
  <div class="emittiv-bulk-bar">
    <span class="emittiv-bulk-bar__count">
      {selectedCount} {entityType} selected
    </span>

    {#if statuses.length > 0}
      <div class="emittiv-bulk-bar__action">
        <select
          bind:value={selectedStatus}
          class="emittiv-bulk-bar__select"
          onchange={handleStatusChange}
        >
          <option value="">Change status...</option>
          {#each statuses as status}
            <option value={status}>{status}</option>
          {/each}
        </select>
      </div>
    {/if}

    <button
      class="emittiv-btn emittiv-btn--sm {confirming === 'delete' ? 'emittiv-btn--danger' : 'emittiv-btn--secondary'}"
      onclick={handleDelete}
    >
      {confirming === 'delete' ? 'Confirm Delete' : 'Delete'}
    </button>

    <button
      class="emittiv-btn emittiv-btn--sm emittiv-btn--ghost"
      onclick={handleClear}
    >
      Clear
    </button>
  </div>
{/if}
