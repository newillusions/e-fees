<script lang="ts">
  import type { ScopeSection } from '$lib/types/scope';
  import ScopeClauseItem from './ScopeClauseItem.svelte';

  let {
    section,
    onupdate,
  }: {
    section: ScopeSection;
    onupdate?: (clauseId: string, field: 'title' | 'body', value: string) => void;
  } = $props();

  let isExpanded = $state(true);

  function toggleExpand() {
    isExpanded = !isExpanded;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      toggleExpand();
    }
  }
</script>

<div class="emittiv-scope-section">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="emittiv-scope-section__header"
    onclick={toggleExpand}
    onkeydown={handleKeydown}
    role="button"
    tabindex="0"
    title="Click to expand/collapse"
  >
    <span class="emittiv-scope-section__chevron" class:emittiv-scope-section__chevron--collapsed={!isExpanded}>
      ▼
    </span>
    <span class="emittiv-scope-section__number">{section.number}</span>
    <span class="emittiv-scope-section__title">{section.title}</span>
    <span class="emittiv-scope-section__count">{section.clauses.length}</span>
  </div>

  {#if isExpanded}
    <div class="emittiv-scope-section__body">
      {#if section.clauses.length > 0}
        {#each section.clauses as clause (clause.clause_id)}
          <ScopeClauseItem
            {clause}
            onupdate={(field, value) => onupdate?.(clause.clause_id, field, value)}
          />
        {/each}
      {:else}
        <p class="emittiv-scope-section__empty">No clauses in this section.</p>
      {/if}
    </div>
  {/if}
</div>
