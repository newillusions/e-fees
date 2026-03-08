<script lang="ts">
  import './scope.css';

  let {
    deliverable,
    onRemove,
    onExpand,
    isDragging = false,
  }: {
    deliverable: any;
    onRemove?: (d: any) => void;
    onExpand?: (d: any) => void;
    isDragging?: boolean;
  } = $props();

  function handleDragStart(e: DragEvent) {
    if (e.dataTransfer) {
      e.dataTransfer.setData('text/plain', JSON.stringify(deliverable));
      e.dataTransfer.effectAllowed = 'move';
    }
  }

  function layerLabel(layer: string): string {
    if (layer === 'generic') return 'G';
    if (layer === 'discipline') return deliverable.discipline?.[0]?.toUpperCase() || 'D';
    if (layer === 'conditional') return 'C';
    return '?';
  }
</script>

<div
  class="emittiv-deliverable-card"
  class:dragging={isDragging}
  draggable="true"
  ondragstart={handleDragStart}
  ondblclick={() => onExpand?.(deliverable)}
  role="listitem"
>
  <span class="layer-badge {deliverable.layer}">{layerLabel(deliverable.layer)}</span>
  <span style="flex: 1;">{deliverable.short_name}</span>
  {#if onRemove}
    <button
      class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm"
      onclick={() => onRemove?.(deliverable)}
      title="Remove deliverable"
    >
      &times;
    </button>
  {/if}
</div>
