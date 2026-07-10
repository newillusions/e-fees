<script lang="ts">
  import { getClauseSelection, saveClauseSelection, getClauseSuggestions } from '$lib/api/scope';
  import type { ClauseSelectionItem, ClauseSuggestion } from '$lib/types/scope';

  let {
    feeId,
    onSaved
  }: {
    feeId: string;
    /** Called after selections are saved successfully. */
    onSaved?: () => void;
  } = $props();

  // ── State ──────────────────────────────────────────────────────────

  let loading = $state(true);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let saveError = $state<string | null>(null);
  let hasCustomSelection = $state(false);

  /** Working copy of selections - mutated by user interactions. */
  let selections = $state<ClauseSelectionItem[]>([]);

  /** Stage 3: ranked clause suggestions mined from historical proposal usage. */
  let suggestions = $state<ClauseSuggestion[]>([]);

  /** Track which clause bodies the user has expanded to edit. */
  let expandedIds = $state<Set<string>>(new Set());

  let searchQuery = $state('');

  // ── Derived ────────────────────────────────────────────────────────

  let filteredSelections = $derived.by(() => {
    if (!searchQuery.trim()) return selections;
    const q = searchQuery.toLowerCase();
    return selections.filter((s) => {
      return (
        s.title.toLowerCase().includes(q) ||
        s.category.toLowerCase().includes(q) ||
        s.body.toLowerCase().includes(q)
      );
    });
  });

  let groupedByCategory = $derived.by(() => {
    const groups: Record<string, ClauseSelectionItem[]> = {};
    for (const s of filteredSelections) {
      if (!groups[s.category]) groups[s.category] = [];
      groups[s.category].push(s);
    }
    return Object.entries(groups).sort(([a], [b]) => a.localeCompare(b));
  });

  let includedCount = $derived(selections.filter((s) => s.included).length);

  /** Suggestions not already included in the current working selection. */
  let visibleSuggestions = $derived(
    suggestions.filter((sug) => {
      const sel = selections.find((s) => s.clause_id === sug.clause_id);
      return !sel || !sel.included;
    })
  );

  // ── Init ───────────────────────────────────────────────────────────

  $effect(() => {
    if (feeId) {
      loadSelections();
      loadSuggestions();
    }
  });

  async function loadSelections() {
    loading = true;
    error = null;
    try {
      const resp = await getClauseSelection(feeId);
      hasCustomSelection = resp.has_custom_selection;
      selections = resp.selections.map((s) => ({ ...s }));
    } catch (e: any) {
      error = e.message || 'Failed to load clause selection';
    } finally {
      loading = false;
    }
  }

  /**
   * Best-effort load: suggestions are a supplementary affordance, not a
   * required part of the picker, so a failure here does not block or error
   * out the main selection UI. Empty (unmined) is the expected steady state
   * before the mining job has run - not treated as an error either way.
   */
  async function loadSuggestions() {
    try {
      const resp = await getClauseSuggestions(feeId);
      suggestions = resp.suggestions;
    } catch {
      suggestions = [];
    }
  }

  // ── Actions ────────────────────────────────────────────────────────

  function toggleClause(clauseId: string) {
    const idx = selections.findIndex((s) => s.clause_id === clauseId);
    if (idx !== -1) {
      selections[idx] = { ...selections[idx], included: !selections[idx].included };
    }
  }

  /** Explicit opt-in add from the Suggested section - never auto-included. */
  function includeSuggestedClause(clauseId: string) {
    const idx = selections.findIndex((s) => s.clause_id === clauseId);
    if (idx !== -1) {
      selections[idx] = { ...selections[idx], included: true };
    }
  }

  function toggleExpanded(clauseId: string) {
    const next = new Set(expandedIds);
    if (next.has(clauseId)) {
      next.delete(clauseId);
    } else {
      next.add(clauseId);
    }
    expandedIds = next;
  }

  function setOverrideBody(clauseId: string, value: string) {
    const idx = selections.findIndex((s) => s.clause_id === clauseId);
    if (idx !== -1) {
      selections[idx] = {
        ...selections[idx],
        override_body: value.trim() === '' ? null : value
      };
    }
  }

  function selectAll() {
    selections = selections.map((s) => ({ ...s, included: true }));
  }

  function deselectAll() {
    selections = selections.map((s) => ({ ...s, included: false }));
  }

  async function handleSave() {
    saving = true;
    saveError = null;
    try {
      await saveClauseSelection({
        fee_id: feeId,
        selections: selections.map((s) => ({
          clause_id: s.clause_id,
          included: s.included,
          override_body: s.override_body
        }))
      });
      hasCustomSelection = true;
      onSaved?.();
    } catch (e: any) {
      saveError = e.message || 'Failed to save clause selection';
    } finally {
      saving = false;
    }
  }
</script>

<div class="clause-picker">
  <div class="clause-picker-header">
    <div class="clause-picker-title">
      <span>Clause Selection</span>
      {#if !loading && selections.length > 0}
        <span class="clause-count-badge">{includedCount}/{selections.length}</span>
      {/if}
    </div>

    {#if !loading && !error}
      <div class="clause-picker-actions">
        <button class="emittiv-btn btn-xs btn-ghost" onclick={selectAll} disabled={saving}>
          All
        </button>
        <button class="emittiv-btn btn-xs btn-ghost" onclick={deselectAll} disabled={saving}>
          None
        </button>
      </div>
    {/if}
  </div>

  {#if loading}
    <div class="clause-picker-status">Loading clauses...</div>
  {:else if error}
    <div class="clause-picker-error">{error}</div>
  {:else if selections.length === 0}
    <div class="clause-picker-status">No active clauses in library.</div>
  {:else}
    {#if !hasCustomSelection}
      <div class="clause-picker-hint">
        Default clauses are pre-selected from the clause library. Toggle any clause to customize
        the selection for this proposal.
      </div>
    {/if}

    {#if visibleSuggestions.length > 0}
      <div class="clause-suggestions">
        <div class="clause-suggestions-label">Suggested from past proposals</div>
        {#each visibleSuggestions.slice(0, 5) as sug (sug.clause_id)}
          <div class="clause-suggestion-row">
            <span class="clause-suggestion-title">{sug.title}</span>
            <span class="clause-suggestion-usage">used in {sug.usage_count} past proposal{sug.usage_count === 1 ? '' : 's'}</span>
            <button
              class="emittiv-btn btn-xs btn-ghost"
              onclick={() => includeSuggestedClause(sug.clause_id)}
              disabled={saving}
            >
              + Add
            </button>
          </div>
        {/each}
      </div>
    {/if}

    <div class="clause-picker-search">
      <input
        class="emittiv-input"
        type="text"
        placeholder="Filter clauses..."
        bind:value={searchQuery}
      />
    </div>

    <div class="clause-picker-list">
      {#each groupedByCategory as [category, items] (category)}
        <div class="clause-category">
          <div class="clause-category-label">{category}</div>
          {#each items as item (item.clause_id)}
            {@const isExpanded = expandedIds.has(item.clause_id)}
            <div class="clause-row" class:excluded={!item.included}>
              <label class="clause-row-check">
                <input
                  type="checkbox"
                  checked={item.included}
                  onchange={() => toggleClause(item.clause_id)}
                />
                <span class="clause-row-title">{item.title}</span>
              </label>

              <button
                class="clause-expand-btn"
                onclick={() => toggleExpanded(item.clause_id)}
                title={isExpanded ? 'Collapse' : 'Expand / Override body'}
                aria-expanded={isExpanded}
              >
                {isExpanded ? '▲' : '▼'}
              </button>
            </div>

            {#if isExpanded}
              <div class="clause-body-panel">
                <div class="clause-body-label">
                  {#if item.override_body !== null && item.override_body !== undefined}
                    Body (overridden - structural, preserved)
                  {:else}
                    Body (from library)
                  {/if}
                </div>
                <textarea
                  class="emittiv-input clause-body-textarea"
                  value={item.override_body ?? item.body}
                  rows={4}
                  placeholder="Leave empty to use the library body"
                  oninput={(e) =>
                    setOverrideBody(item.clause_id, (e.target as HTMLTextAreaElement).value)}
                ></textarea>
                {#if item.override_body !== null && item.override_body !== undefined}
                  <button
                    class="emittiv-btn btn-xs btn-ghost"
                    onclick={() => setOverrideBody(item.clause_id, '')}
                  >
                    Clear override
                  </button>
                {/if}
              </div>
            {/if}
          {/each}
        </div>
      {/each}
    </div>

    {#if saveError}
      <div class="clause-picker-error save-error">{saveError}</div>
    {/if}

    <div class="clause-picker-footer">
      <button
        class="emittiv-btn btn-primary"
        onclick={handleSave}
        disabled={saving || includedCount === 0}
      >
        {saving ? 'Saving...' : 'Save Selection'}
      </button>
      {#if includedCount === 0}
        <span class="clause-picker-hint">Include at least one clause to generate scope.</span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .clause-picker {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: var(--darker);
    border: 1px solid var(--dark);
    border-radius: 6px;
    padding: 12px;
    font-size: 12px;
  }

  .clause-picker-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .clause-picker-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--lighter);
  }

  .clause-count-badge {
    background: var(--splash);
    color: var(--black);
    border-radius: 10px;
    padding: 1px 7px;
    font-size: 10px;
    font-weight: 700;
  }

  .clause-picker-actions {
    display: flex;
    gap: 4px;
  }

  .clause-picker-status {
    color: var(--light);
    font-size: 11px;
    padding: 8px 0;
  }

  .clause-picker-hint {
    color: var(--light);
    font-size: 11px;
    font-style: italic;
  }

  .clause-picker-error {
    color: #f44;
    font-size: 11px;
  }

  .save-error {
    margin-top: 4px;
  }

  .clause-suggestions {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 6px 8px;
    background: var(--black);
    border-radius: 4px;
    border-left: 2px solid var(--splash);
  }

  .clause-suggestions-label {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--light);
    margin-bottom: 2px;
  }

  .clause-suggestion-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .clause-suggestion-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 11px;
    color: var(--lighter);
  }

  .clause-suggestion-usage {
    font-size: 10px;
    color: var(--light);
    flex-shrink: 0;
  }

  .clause-picker-search {
    margin: 4px 0;
  }

  .clause-picker-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 320px;
    overflow-y: auto;
  }

  .clause-category {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .clause-category-label {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--light);
    padding: 6px 0 2px;
    border-bottom: 1px solid var(--dark);
    margin-bottom: 2px;
  }

  .clause-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 4px;
    border-radius: 3px;
    transition: background 150ms;
  }

  .clause-row:hover {
    background: var(--dark);
  }

  .clause-row.excluded {
    opacity: 0.45;
  }

  .clause-row-check {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    cursor: pointer;
    min-width: 0;
  }

  .clause-row-check input[type='checkbox'] {
    width: 13px;
    height: 13px;
    flex-shrink: 0;
    accent-color: var(--splash);
    cursor: pointer;
  }

  .clause-row-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--lighter);
    font-size: 12px;
  }

  .clause-expand-btn {
    background: none;
    border: none;
    color: var(--dark);
    cursor: pointer;
    font-size: 9px;
    padding: 2px 4px;
    border-radius: 3px;
    flex-shrink: 0;
    transition: color 150ms;
  }

  .clause-expand-btn:hover {
    color: var(--light);
  }

  .clause-body-panel {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin: 2px 0 4px 19px;
    padding: 8px;
    background: var(--black);
    border-radius: 3px;
    border-left: 2px solid var(--splash);
  }

  .clause-body-label {
    font-size: 10px;
    color: var(--light);
    font-style: italic;
  }

  .clause-body-textarea {
    font-size: 11px;
    line-height: 1.5;
    resize: vertical;
    min-height: 70px;
    font-family: inherit;
  }

  .clause-picker-footer {
    display: flex;
    align-items: center;
    gap: 10px;
    padding-top: 4px;
    border-top: 1px solid var(--dark);
    margin-top: 4px;
  }

  /* btn-xs modifier */
  :global(.btn-xs) {
    padding: 2px 8px;
    font-size: 10px;
  }

  :global(.btn-ghost) {
    background: transparent;
    border-color: var(--dark);
    color: var(--light);
  }

  :global(.btn-ghost:hover) {
    background: var(--dark);
    color: var(--lighter);
  }
</style>
