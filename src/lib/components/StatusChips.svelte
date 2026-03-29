<script lang="ts">
  /**
   * Multi-select status chip row with optional "Active" preset toggle.
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
    /** Optional preset statuses for the "Active" toggle */
    activePreset?: readonly string[];
  }

  import { SvelteSet } from 'svelte/reactivity';

  let { statuses, selected = $bindable(), counts, activePreset }: Props = $props();

  function toggle(status: string) {
    const next = new SvelteSet(selected);
    if (next.has(status)) {
      next.delete(status);
    } else {
      next.add(status);
    }
    selected = next;
  }

  const isActivePresetOn = $derived(
    activePreset != null &&
      activePreset.length > 0 &&
      selected.size === activePreset.length &&
      activePreset.every(s => selected.has(s))
  );

  function toggleActivePreset() {
    if (!activePreset) return;
    if (isActivePresetOn) {
      selected = new SvelteSet();
    } else {
      selected = new SvelteSet(activePreset);
    }
  }
</script>

<div class="emittiv-status-chips">
  {#if activePreset}
    <button
      type="button"
      class="emittiv-status-chip emittiv-status-chip--preset"
      class:emittiv-status-chip--active={isActivePresetOn}
      onclick={toggleActivePreset}
    >
      Active
    </button>
    <span class="emittiv-status-chips__sep"></span>
  {/if}
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
