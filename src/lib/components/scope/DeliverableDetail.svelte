<script lang="ts">
  import './scope.css';

  let {
    deliverable,
    onClose,
    onSave,
    onDuplicate,
    onSaveAsNew,
  }: {
    deliverable: any;
    onClose: () => void;
    onSave?: (d: any) => void;
    onDuplicate?: (d: any) => void;
    onSaveAsNew?: (d: any) => void;
  } = $props();

  let editBody = $state(deliverable?.body || '');
  let isEditing = $state(false);

  $effect(() => {
    if (deliverable) {
      editBody = deliverable.body || '';
      isEditing = false;
    }
  });

  function handleOverlayClick() {
    onClose();
  }

  function handleDialogClick(e: MouseEvent) {
    e.stopPropagation();
  }

  function handleCancel() {
    isEditing = false;
    editBody = deliverable.body || '';
  }

  function handleSave() {
    onSave?.({ ...deliverable, body: editBody });
    isEditing = false;
  }
</script>

{#if deliverable}
  <div
    class="emittiv-deliverable-detail-overlay"
    onclick={handleOverlayClick}
    role="dialog"
    aria-modal="true"
    aria-label="Deliverable detail"
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="emittiv-deliverable-detail" onclick={handleDialogClick}>
      <div class="emittiv-deliverable-detail-header">
        <div>
          <h3 style="font-size: 14px; font-weight: 600; color: var(--emittiv-white); margin: 0;">
            {deliverable.title}
          </h3>
          <div style="display: flex; gap: 8px; margin-top: 4px; align-items: center;">
            <span class="layer-badge {deliverable.layer}" style="font-size: 9px; font-weight: 700; padding: 1px 5px; border-radius: 3px; text-transform: uppercase;">
              {deliverable.layer}
            </span>
            {#if deliverable.discipline}
              <span style="font-size: 10px; color: var(--emittiv-light);">{deliverable.discipline}</span>
            {/if}
            <span style="font-size: 10px; color: var(--emittiv-light);">v{deliverable.version}</span>
          </div>
        </div>
        <button class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm" onclick={onClose}>
          &times;
        </button>
      </div>

      <div class="emittiv-deliverable-detail-body">
        {#if isEditing}
          <textarea
            class="emittiv-textarea"
            style="width: 100%; min-height: 200px; font-size: 12px;"
            bind:value={editBody}
          ></textarea>
        {:else}
          <p style="font-size: 12px; color: var(--emittiv-lighter); white-space: pre-wrap; margin: 0;">
            {editBody}
          </p>
        {/if}
      </div>

      <div class="emittiv-deliverable-detail-footer">
        <div style="display: flex; gap: 8px;">
          {#if onDuplicate}
            <button
              class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm"
              onclick={() => onDuplicate?.(deliverable)}
            >
              Duplicate
            </button>
          {/if}
          {#if onSaveAsNew}
            <button
              class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm"
              onclick={() => onSaveAsNew?.({ ...deliverable, body: editBody })}
            >
              Save as New
            </button>
          {/if}
        </div>
        <div style="display: flex; gap: 8px;">
          {#if isEditing}
            <button class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm" onclick={handleCancel}>
              Cancel
            </button>
            <button class="emittiv-btn emittiv-btn--primary emittiv-btn--sm" onclick={handleSave}>
              Save
            </button>
          {:else}
            <button class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm" onclick={() => (isEditing = true)}>
              Edit
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
