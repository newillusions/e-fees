<script lang="ts">
  import type { ScopeClauseItem as ClauseType } from '$lib/types/scope';

  let {
    clause,
    onupdate
  }: {
    clause: ClauseType;
    onupdate?: (field: 'title' | 'body', value: string) => void;
  } = $props();

  let editing = $state(false);
  let editTitle = $state('');
  let editBody = $state('');

  function startEdit() {
    editTitle = clause.title;
    editBody = clause.body;
    editing = true;
  }

  function saveEdit() {
    if (editTitle !== clause.title) onupdate?.('title', editTitle);
    if (editBody !== clause.body) onupdate?.('body', editBody);
    editing = false;
  }

  function cancelEdit() {
    editing = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    // BaseModal uses svelte:window for Escape — stopPropagation won't help.
    // Use preventDefault so BaseModal can check event.defaultPrevented.
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      cancelEdit();
    }
  }
</script>

{#if editing}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="emittiv-scope-clause emittiv-scope-clause--editing" onkeydown={handleKeydown}>
    <div class="emittiv-scope-clause__number">{clause.number}</div>
    <div class="emittiv-scope-clause__edit-fields">
      <input class="emittiv-input" bind:value={editTitle} placeholder="Clause title" />
      <textarea
        class="emittiv-input emittiv-scope-clause__body-input"
        bind:value={editBody}
        placeholder="Clause body"
        rows="3"
      ></textarea>
      <div class="emittiv-scope-clause__edit-actions">
        <button class="emittiv-btn emittiv-btn--primary emittiv-btn--sm" onclick={saveEdit}>
          Save
        </button>
        <button class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm" onclick={cancelEdit}>
          Cancel
        </button>
      </div>
    </div>
  </div>
{:else}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="emittiv-scope-clause" onclick={startEdit} title="Click to edit">
    <div class="emittiv-scope-clause__number">{clause.number}</div>
    <div class="emittiv-scope-clause__content">
      <span class="emittiv-scope-clause__title">{clause.title}</span>
      <span class="emittiv-scope-clause__separator"> — </span>
      <span class="emittiv-scope-clause__body">{clause.body}</span>
    </div>
  </div>
{/if}
