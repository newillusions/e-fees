<script lang="ts">
  let {
    selectedCount = 0,
    entityType = 'items',
    statuses = [] as string[],
    onstatuschange,
    ondelete,
    onclear
  }: {
    selectedCount: number;
    entityType: string;
    statuses?: string[];
    onstatuschange?: (status: string) => void;
    ondelete?: () => void;
    onclear?: () => void;
  } = $props();

  let confirming: 'delete' | null = $state(null);
  let confirmingStatus = $state('');

  // Derive singular entity label for button text
  const entityLabel = $derived(
    entityType.endsWith('ies')
      ? entityType.slice(0, -3) + 'y'
      : entityType.endsWith('s')
        ? entityType.slice(0, -1)
        : entityType
  );

  function handleStatusSelect(event: Event) {
    const value = (event.target as HTMLSelectElement).value;
    confirmingStatus = value;
  }

  function handleApplyStatus() {
    if (!confirmingStatus) return;
    onstatuschange?.(confirmingStatus);
    confirmingStatus = '';
  }

  function handleDelete() {
    if (confirming === 'delete') {
      ondelete?.();
      confirming = null;
    } else {
      confirming = 'delete';
    }
  }

  function handleClear() {
    confirming = null;
    confirmingStatus = '';
    onclear?.();
  }

  // Reset confirmation when selection changes
  $effect(() => {
    if (selectedCount === 0) {
      confirming = null;
      confirmingStatus = '';
    }
  });
</script>

{#if selectedCount > 0}
  <div class="emittiv-bulk-bar">
    <span class="emittiv-bulk-bar__count">
      {selectedCount}
      {entityType} selected
    </span>

    {#if statuses.length > 0}
      <div class="emittiv-bulk-bar__action" style="display: flex; align-items: center; gap: 8px;">
        <select
          value={confirmingStatus}
          class="emittiv-bulk-bar__select"
          onchange={handleStatusSelect}
        >
          <option value="">Change status...</option>
          {#each statuses as status}
            <option value={status}>{status}</option>
          {/each}
        </select>
        {#if confirmingStatus}
          <button
            class="emittiv-btn emittiv-btn--sm emittiv-btn--primary"
            onclick={handleApplyStatus}
          >
            Apply
          </button>
        {/if}
      </div>
    {/if}

    <button
      class="emittiv-btn emittiv-btn--sm {confirming === 'delete'
        ? 'emittiv-btn--danger'
        : 'emittiv-btn--secondary'}"
      onclick={handleDelete}
    >
      {confirming === 'delete'
        ? `Delete ${selectedCount} ${selectedCount === 1 ? entityLabel : entityType}`
        : 'Delete'}
    </button>

    <button class="emittiv-btn emittiv-btn--sm emittiv-btn--ghost" onclick={handleClear}>
      Clear
    </button>
  </div>
{/if}
