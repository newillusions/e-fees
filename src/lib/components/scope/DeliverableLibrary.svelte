<script lang="ts">
  import './scope.css';
  import type { Deliverable } from '$lib/types/scope';

  let {
    allDeliverables = [],
    activeIds = new Set(),
    onAdd,
  }: {
    allDeliverables: Deliverable[];
    activeIds: Set<string>;
    onAdd?: (d: Deliverable, targetStage?: string) => void;
  } = $props();

  let searchQuery = $state('');

  let availableDeliverables = $derived(
    allDeliverables.filter((d) => !activeIds.has(d.id))
  );

  let filteredDeliverables = $derived.by(() => {
    if (!searchQuery.trim()) return availableDeliverables;

    const q = searchQuery.toLowerCase();
    return availableDeliverables.filter((d) => {
      const fields = [
        d.title,
        d.short_name,
        d.body,
        ...(d.tags || []),
      ].filter(Boolean);
      return fields.some((f: string) => f.toLowerCase().includes(q));
    });
  });

  let groupedByStage = $derived.by(() => {
    const groups: Record<string, Deliverable[]> = {};
    for (const d of filteredDeliverables) {
      const stage = d.stage || 'unassigned';
      if (!groups[stage]) groups[stage] = [];
      groups[stage].push(d);
    }
    // Sort groups alphabetically
    return Object.entries(groups).sort(([a], [b]) => a.localeCompare(b));
  });

  function handleDragStart(e: DragEvent, deliverable: Deliverable) {
    if (e.dataTransfer) {
      e.dataTransfer.setData('text/plain', JSON.stringify(deliverable));
      e.dataTransfer.effectAllowed = 'copy';
    }
  }

  function layerLabel(d: Deliverable): string {
    if (d.layer === 'generic') return 'G';
    if (d.layer === 'discipline') return d.discipline?.[0]?.toUpperCase() || 'D';
    if (d.layer === 'conditional') return 'C';
    return '?';
  }
</script>

<div class="emittiv-scope-library">
  <div class="emittiv-library-search">
    <input
      class="emittiv-input"
      type="text"
      placeholder="Search deliverables..."
      bind:value={searchQuery}
    />
  </div>

  {#each groupedByStage as [stageName, items] (stageName)}
    <div class="emittiv-library-group-title">{stageName}</div>
    {#each items as deliverable (deliverable.id)}
      <div
        class="emittiv-library-item"
        draggable="true"
        ondragstart={(e) => handleDragStart(e, deliverable)}
      >
        <span class="layer-badge {deliverable.layer}" style="font-size: 9px; font-weight: 700; padding: 1px 5px; border-radius: 3px; text-transform: uppercase;">
          {layerLabel(deliverable)}
        </span>
        <span style="flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
          {deliverable.short_name}
        </span>
        <button
          class="add-btn"
          onclick={() => onAdd?.(deliverable)}
          title="Add to scope"
        >
          +
        </button>
      </div>
    {/each}
  {/each}

  {#if groupedByStage.length === 0}
    <p style="font-size: 12px; color: var(--emittiv-light); text-align: center; padding: 24px 0;">
      {searchQuery ? 'No matching deliverables' : 'All deliverables are in use'}
    </p>
  {/if}
</div>
