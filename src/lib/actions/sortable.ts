/**
 * Pointer-based sortable action for Svelte 5.
 * Works in Tauri WebView where HTML5 DnD dragover events don't fire.
 *
 * Usage:
 *   <div use:sortable={{ items, onReorder }}>
 *     {#each items as item (item.id)}
 *       <div data-sortable-id={item.id}>...</div>
 *     {/each}
 *   </div>
 */

export interface SortableOptions<T extends { id: string }> {
  items: T[];
  onReorder: (reordered: T[]) => void;
  /** CSS class applied to the row being dragged */
  dragClass?: string;
  /** CSS class applied to the row being hovered over */
  overClass?: string;
  /** Selector for the drag handle within each item. If not set, entire row is draggable. */
  handleSelector?: string;
}

export function sortable<T extends { id: string }>(node: HTMLElement, options: SortableOptions<T>) {
  let opts = options;
  let draggedId: string | null = null;
  let overId: string | null = null;
  let isDragging = false;
  let startY = 0;
  const DRAG_THRESHOLD = 4; // px before drag activates

  function getItemElements(): HTMLElement[] {
    return Array.from(node.querySelectorAll('[data-sortable-id]'));
  }

  function getIdFromElement(el: HTMLElement | null): string | null {
    if (!el) return null;
    const sortableEl = el.closest('[data-sortable-id]') as HTMLElement | null;
    return sortableEl?.dataset.sortableId ?? null;
  }

  function handlePointerDown(e: PointerEvent) {
    const target = e.target as HTMLElement;

    // Don't drag from inputs, buttons, or interactive elements
    if (target.closest('input, button, select, textarea, [contenteditable]')) return;

    // If handleSelector is set, only drag from handle
    if (opts.handleSelector && !target.closest(opts.handleSelector)) return;

    const id = getIdFromElement(target);
    if (!id) return;

    draggedId = id;
    startY = e.clientY;
    isDragging = false;

    // Capture pointer for reliable tracking
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);

    e.preventDefault();
  }

  function handlePointerMove(e: PointerEvent) {
    if (!draggedId) return;

    // Check threshold before activating drag
    if (!isDragging) {
      if (Math.abs(e.clientY - startY) < DRAG_THRESHOLD) return;
      isDragging = true;

      // Apply drag class to source
      const dragEl = node.querySelector(`[data-sortable-id="${draggedId}"]`) as HTMLElement;
      if (dragEl && opts.dragClass) dragEl.classList.add(opts.dragClass);
    }

    // Find element under pointer (release capture momentarily)
    (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    const elementUnder = document.elementFromPoint(e.clientX, e.clientY);
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);

    const newOverId = getIdFromElement(elementUnder as HTMLElement);

    if (newOverId !== overId) {
      // Remove old hover
      if (overId) {
        const oldEl = node.querySelector(`[data-sortable-id="${overId}"]`) as HTMLElement;
        if (oldEl && opts.overClass) oldEl.classList.remove(opts.overClass);
      }

      overId = newOverId;

      // Apply new hover (skip if hovering own element)
      if (overId && overId !== draggedId) {
        const newEl = node.querySelector(`[data-sortable-id="${overId}"]`) as HTMLElement;
        if (newEl && opts.overClass) newEl.classList.add(opts.overClass);
      }
    }
  }

  function handlePointerUp(e: PointerEvent) {
    if (!draggedId) return;

    // Clean up classes
    if (opts.dragClass) {
      const dragEl = node.querySelector(`[data-sortable-id="${draggedId}"]`) as HTMLElement;
      if (dragEl) dragEl.classList.remove(opts.dragClass);
    }
    if (overId && opts.overClass) {
      const overEl = node.querySelector(`[data-sortable-id="${overId}"]`) as HTMLElement;
      if (overEl) overEl.classList.remove(opts.overClass);
    }

    // Commit reorder if we have a valid target
    if (isDragging && overId && overId !== draggedId) {
      const items = opts.items;
      const dragIndex = items.findIndex(i => i.id === draggedId);
      const overIndex = items.findIndex(i => i.id === overId);

      if (dragIndex !== -1 && overIndex !== -1) {
        const reordered = [...items];
        const [removed] = reordered.splice(dragIndex, 1);
        reordered.splice(overIndex, 0, removed);
        opts.onReorder(reordered);
      }
    }

    // Release capture
    try {
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    } catch {
      /* already released */
    }

    draggedId = null;
    overId = null;
    isDragging = false;
  }

  node.addEventListener('pointerdown', handlePointerDown);
  node.addEventListener('pointermove', handlePointerMove);
  node.addEventListener('pointerup', handlePointerUp);

  return {
    update(newOptions: SortableOptions<T>) {
      opts = newOptions;
    },
    destroy() {
      node.removeEventListener('pointerdown', handlePointerDown);
      node.removeEventListener('pointermove', handlePointerMove);
      node.removeEventListener('pointerup', handlePointerUp);
    }
  };
}
