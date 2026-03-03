<script lang="ts">
  /**
   * Multi-select status chip row.
   * Toggle chips to filter by one or more statuses.
   * When no chips are selected, all items pass through (no filtering).
   */

  interface Props {
    /** Ordered list of status values to display as chips */
    statuses: readonly string[];
    /** Currently selected statuses (bindable) */
    selected: Set<string>;
    /** Optional count per status, renders as "Status (N)" */
    counts?: Record<string, number>;
  }

  let { statuses, selected = $bindable(), counts }: Props = $props();

  function toggle(status: string) {
    const next = new Set(selected);
    if (next.has(status)) {
      next.delete(status);
    } else {
      next.add(status);
    }
    selected = next;
  }
</script>

<div class="emittiv-status-chips">
  {#each statuses as status}
    <button
      type="button"
      class="emittiv-status-chip"
      class:emittiv-status-chip--active={selected.has(status)}
      onclick={() => toggle(status)}
    >
      {status}{#if counts && counts[status]}&nbsp;({counts[status]}){/if}
    </button>
  {/each}
</div>
