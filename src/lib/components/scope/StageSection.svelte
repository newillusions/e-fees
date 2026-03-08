<script lang="ts">
  import type { StageDisplay, Deliverable } from '$lib/types/scope';
  import DeliverableCard from './DeliverableCard.svelte';

  let {
    stage,
    deliverables = [],
    onRemove,
    onExpand,
    onReorder,
    onDropFromLibrary,
  }: {
    stage: StageDisplay;
    deliverables: Deliverable[];
    onRemove?: (d: Deliverable) => void;
    onExpand?: (d: Deliverable) => void;
    onReorder?: (stage: StageDisplay, reordered: Deliverable[]) => void;
    onDropFromLibrary?: (stage: StageDisplay, d: Deliverable) => void;
  } = $props();

  let collapsed = $state(false);
  let dropActive = $state(false);
  let dragOverIndex = $state<number | null>(null);

  function handleHeaderClick() {
    collapsed = !collapsed;
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'move';
    }
    dropActive = true;
  }

  function handleDragLeave() {
    dropActive = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    dropActive = false;
    dragOverIndex = null;

    if (!e.dataTransfer) return;

    try {
      const data = JSON.parse(e.dataTransfer.getData('text/plain'));

      // Check if this deliverable is already in this stage (reorder)
      const existingIndex = deliverables.findIndex((d) => d.id === data.id);
      if (existingIndex >= 0) {
        // Reorder within stage — move to end
        const reordered = [...deliverables];
        const [moved] = reordered.splice(existingIndex, 1);
        reordered.push(moved);
        onReorder?.(stage, reordered);
      } else {
        // Dropped from library
        onDropFromLibrary?.(stage, data);
      }
    } catch {
      // Invalid drag data, ignore
    }
  }

  function handleCardDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    dragOverIndex = index;
  }

  function handleCardDrop(e: DragEvent, targetIndex: number) {
    e.preventDefault();
    e.stopPropagation();
    dropActive = false;
    dragOverIndex = null;

    if (!e.dataTransfer) return;

    try {
      const data = JSON.parse(e.dataTransfer.getData('text/plain'));
      const sourceIndex = deliverables.findIndex((d) => d.id === data.id);

      if (sourceIndex >= 0 && sourceIndex !== targetIndex) {
        // Reorder within stage
        const reordered = [...deliverables];
        const [moved] = reordered.splice(sourceIndex, 1);
        reordered.splice(targetIndex, 0, moved);
        onReorder?.(stage, reordered);
      } else if (sourceIndex < 0) {
        // From library — insert at position
        onDropFromLibrary?.(stage, data);
      }
    } catch {
      // Invalid drag data
    }
  }
</script>

<div class="emittiv-stage-section">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="emittiv-stage-header"
    onclick={handleHeaderClick}
    onkeydown={(e) => e.key === 'Enter' && handleHeaderClick()}
    role="button"
    tabindex="0"
  >
    <span style="font-size: 10px; color: var(--emittiv-light); transition: transform 0.2s; transform: rotate({collapsed ? '-90deg' : '0deg'});">
      &#9660;
    </span>
    <span class="emittiv-stage-label">{stage.label}</span>
    <span class="emittiv-stage-count">{deliverables.length}</span>
  </div>

  {#if !collapsed}
    <div
      class="emittiv-stage-body"
      ondragover={handleDragOver}
      ondragleave={handleDragLeave}
      ondrop={handleDrop}
      role="list"
    >
      {#if deliverables.length > 0}
        {#each deliverables as deliverable, index (deliverable.id)}
          <div
            ondragover={(e) => handleCardDragOver(e, index)}
            ondrop={(e) => handleCardDrop(e, index)}
            style={dragOverIndex === index ? 'border-top: 2px solid var(--emittiv-splash);' : ''}
          >
            <DeliverableCard
              {deliverable}
              {onRemove}
              {onExpand}
            />
          </div>
        {/each}
      {:else}
        <div class="emittiv-drop-zone" class:active={dropActive}>
          Drop deliverables here
        </div>
      {/if}
    </div>
  {/if}
</div>
