<script lang="ts">
  import './scope.css';
  import {
    getStages,
    getDeliverables,
    assembleDeliverables,
    getScopeDeliverables,
    saveScopeBuilder
  } from '$lib/api/scope';
  import { logApiError } from '$lib/services/logger';
  import type {
    StageConfig,
    Deliverable,
    StageDisplay,
    ScopeDeliverableEntry,
    SavedScopeEntry
  } from '$lib/types/scope';
  import StageSection from './StageSection.svelte';
  import DeliverableLibrary from './DeliverableLibrary.svelte';
  import DeliverableDetail from './DeliverableDetail.svelte';

  let {
    params = {}
  }: {
    params?: { id?: string };
  } = $props();

  let feeId = $derived(params.id || '');

  // Data state
  let stages: StageConfig[] = $state([]);
  let activeByStage: Record<string, Deliverable[]> = $state({});
  let libraryDeliverables: Deliverable[] = $state([]);
  let selectedDeliverable: Deliverable | null = $state(null);

  // UI state
  let loading = $state(true);
  let saving = $state(false);
  let error: string | null = $state(null);
  let saveMessage: string | null = $state(null);
  let saveError: string | null = $state(null);
  // True only after the saved scope (or library defaults) loaded successfully.
  // Save is disabled otherwise: a failed load leaves no scope on screen, and
  // saving then would replace the saved scope with an empty one.
  let loadedOk = $state(false);
  // Custom stage labels saved with this proposal (shown; never overwritten by
  // a save, which omits stage_labels so the server leaves them alone).
  let savedStageLabels: Record<string, string> = $state({});

  // Save-state tracking. editVersion counts edits; a save only counts as
  // "saved" if no edit happened while its request was in flight (the request
  // body is a snapshot taken before the await).
  let editVersion = 0;
  let dirty = $state(false);

  // Wording edited in the detail modal, by deliverable id. Sent as
  // wording_override on every save until Reset/reload discards it.
  let wordingEdits: Record<string, string> = {};

  function markEdited() {
    editVersion += 1;
    dirty = true;
    saveMessage = null;
  }

  // Derived: set of active IDs for library filtering
  let activeIds = $derived(
    new Set(Object.values(activeByStage).flatMap(items => items.map(d => d.id)))
  );

  // Derived: stages with labels for display
  let stageConfigs = $derived(
    stages
      .filter(s => s.status === 'active')
      .sort((a, b) => a.sort_order - b.sort_order)
      .map(s => ({
        canonical_name: s.canonical_name,
        label: savedStageLabels[s.canonical_name] || s.default_label || s.canonical_name,
        sort_order: s.sort_order
      }))
  );

  async function loadData() {
    if (!feeId) {
      error = 'No fee ID provided';
      loading = false;
      return;
    }

    wordingEdits = {};
    loadedOk = false;
    loading = true;
    error = null;

    try {
      // Fetch stages and all deliverables in parallel
      // API returns { data: [...], total: N } wrapper
      const [stagesResp, deliverablesResp] = await Promise.all([getStages(), getDeliverables()]);

      stages = stagesResp.data;
      libraryDeliverables = deliverablesResp.data;

      // Show what was last saved; library defaults only when nothing was saved.
      await loadSavedOrDefaults();
      loadedOk = true;
    } catch (err: any) {
      error = err.message || 'Failed to load scope data';
      logApiError('ScopeBuilder load', err as Error);
    } finally {
      loading = false;
    }
  }

  // Rebuild one saved row as a Deliverable. The library copy supplies the
  // metadata; the saved wording_snapshot is the body that was saved. A row whose
  // library deliverable is gone is rebuilt from the row so saved work is not
  // silently dropped.
  function deliverableFromSaved(entry: SavedScopeEntry): Deliverable {
    const key = entry.deliverable_id.replace(/^deliverable:/, '');
    const lib = libraryDeliverables.find(d => d.id === key);
    const orphan = !lib;
    const base: Deliverable = lib ?? {
      id: key,
      title: entry.short_name || key,
      short_name: entry.short_name || key,
      body: '',
      stage: entry.stage,
      layer: 'generic',
      sort_order: entry.sort_order,
      status: 'active',
      version: 1,
      created_at: '',
      updated_at: ''
    };
    const body = entry.wording_snapshot ?? base.body;
    // An orphan has no library body to fall back on: its saved wording is the
    // only copy, so it is always sent back as an override (snapshot-only row).
    if (orphan || body !== base.body) wordingEdits[key] = body;
    return orphan ? { ...base, body, orphan: true } : { ...base, body };
  }

  /**
   * Load the last saved scope for this fee. Returns false when nothing has
   * been saved. A failed fetch throws: falling back to library defaults there
   * would show (and on Save overwrite) the wrong scope.
   */
  async function applySavedScope(): Promise<boolean> {
    const saved = await getScopeDeliverables(feeId);
    const entries = saved?.deliverables_used ?? [];
    if (entries.length === 0) return false;

    wordingEdits = {};
    const byStage: Record<string, Deliverable[]> = {};
    for (const entry of [...entries].sort((a, b) => a.sort_order - b.sort_order)) {
      (byStage[entry.stage] ??= []).push(deliverableFromSaved(entry));
    }
    savedStageLabels = saved.stage_labels ?? {};
    activeByStage = byStage;
    dirty = false;
    return true;
  }

  async function loadSavedOrDefaults() {
    if (await applySavedScope()) return;
    wordingEdits = {};
    await runAssemble();
  }

  // Throws on failure: a failed assemble must never look like an empty scope.
  async function runAssemble() {
    const result = await assembleDeliverables({
      fee_id: feeId,
      disciplines: ['lighting']
    });
    const byStage: Record<string, Deliverable[]> = {};

    if (result.stages) {
      for (const s of result.stages) {
        byStage[s.canonical_name] = s.deliverables || [];
      }
    }

    activeByStage = byStage;
    dirty = false;
  }

  async function handleSave() {
    if (!loadedOk) return;
    // A row whose library item is gone and that has no saved wording has
    // nothing to keep. Nothing is sent until each has wording or is removed.
    const wordless = Object.values(activeByStage)
      .flat()
      .filter(d => d.orphan && !d.body.trim());
    if (wordless.length > 0) {
      saveMessage = null;
      saveError = `Add wording to, or remove, ${wordless.length} row(s) marked "wording missing" before saving.`;
      return;
    }
    saving = true;
    saveMessage = null;
    saveError = null;
    const savedVersion = editVersion;

    try {
      const deliverables: ScopeDeliverableEntry[] = [];
      for (const [stageName, items] of Object.entries(activeByStage)) {
        for (const [idx, d] of items.entries()) {
          deliverables.push({
            deliverable_id: d.id,
            stage: stageName,
            sort_order: idx + 1,
            wording_override: wordingEdits[d.id] ?? null,
            // Snapshot-only rows (library item removed) need their name.
            ...(d.orphan ? { short_name: d.short_name } : {})
          });
        }
      }

      await saveScopeBuilder({
        fee_id: feeId,
        // stage_labels deliberately omitted: there is no label editor, and an
        // omitted field leaves the saved labels untouched on the server.
        deliverables
      });

      if (editVersion === savedVersion) {
        dirty = false;
        saveMessage = 'Scope saved successfully';
        setTimeout(() => (saveMessage = null), 3000);
      }
      // else: edits landed after the request body was built - they are not
      // saved, so stay dirty and do not claim success.
    } catch (err: any) {
      // Keep the editor (and the user's edits) on screen: the page-level
      // `error` branch replaces the whole builder and Retry reloads it.
      saveError = err.message || 'Failed to save scope';
    } finally {
      saving = false;
    }
  }

  async function handleReset() {
    if (dirty && !confirm('Discard unsaved scope changes?')) return;
    loadedOk = false;
    loading = true;
    error = null;
    saveMessage = null;
    saveError = null;
    try {
      // Discard unsaved edits and return to the last saved state.
      await loadSavedOrDefaults();
      loadedOk = true;
    } catch (err: any) {
      error = err.message || 'Failed to reset scope';
    } finally {
      loading = false;
    }
  }

  async function handleLibraryDefaults() {
    if (!confirm('Replace the scope with the library defaults? Nothing is saved until you Save.'))
      return;
    saveMessage = null;
    saveError = null;
    try {
      await runAssemble();
      wordingEdits = {};
      markEdited();
    } catch (err: any) {
      // The editor keeps its current contents; only the toolbar reports it.
      saveError = err.message || 'Failed to load library defaults';
      logApiError('library defaults', err as Error);
    }
  }

  function handleRemove(deliverable: Deliverable) {
    const updated = { ...activeByStage };
    for (const [stage, items] of Object.entries(updated)) {
      updated[stage] = items.filter(d => d.id !== deliverable.id);
    }
    activeByStage = updated;
    markEdited();
  }

  function handleExpand(deliverable: Deliverable) {
    selectedDeliverable = deliverable;
  }

  function handleDetailClose() {
    selectedDeliverable = null;
  }

  function handleDetailSave(updated: Deliverable) {
    // Only a body that differs from the library body is an override; an
    // orphan has no library body, so its wording is always kept.
    const libBody = libraryDeliverables.find(d => d.id === updated.id)?.body;
    if (updated.orphan || libBody === undefined || updated.body !== libBody) {
      wordingEdits[updated.id] = updated.body;
    } else {
      delete wordingEdits[updated.id];
    }
    // Update the deliverable in the active set
    const newByStage = { ...activeByStage };
    for (const [stage, items] of Object.entries(newByStage)) {
      const idx = items.findIndex(d => d.id === updated.id);
      if (idx >= 0) {
        newByStage[stage] = [...items];
        newByStage[stage][idx] = updated;
      }
    }
    activeByStage = newByStage;
    markEdited();
    selectedDeliverable = null;
  }

  function handleReorder(stage: StageDisplay, reordered: Deliverable[]) {
    activeByStage = { ...activeByStage, [stage.canonical_name]: reordered };
    markEdited();
  }

  function handleDropFromLibrary(stage: StageDisplay, deliverable: Deliverable) {
    // Add to this stage if not already active
    if (activeIds.has(deliverable.id)) return;

    const current = activeByStage[stage.canonical_name] || [];
    activeByStage = {
      ...activeByStage,
      [stage.canonical_name]: [...current, deliverable]
    };
    markEdited();
  }

  function handleAddFromLibrary(deliverable: Deliverable, _targetStage?: string) {
    // Add to the deliverable's native stage, or the first available stage
    const targetStage = _targetStage || deliverable.stage || stageConfigs[0]?.canonical_name;
    if (!targetStage) return;
    if (activeIds.has(deliverable.id)) return;

    const current = activeByStage[targetStage] || [];
    activeByStage = {
      ...activeByStage,
      [targetStage]: [...current, deliverable]
    };
    markEdited();
  }

  $effect(() => {
    if (feeId) loadData();
  });
</script>

<div class="emittiv-scope-builder">
  <!-- Toolbar -->
  <div class="emittiv-scope-toolbar">
    <h2 style="font-size: 16px; font-weight: 600; color: var(--emittiv-white); margin: 0; flex: 1;">
      Scope Builder
    </h2>
    {#if feeId}
      <span style="font-size: 11px; color: var(--emittiv-light);">
        Fee: {feeId}
      </span>
    {/if}
    {#if saveError}
      <span style="font-size: 11px; color: var(--color-error);">{saveError}</span>
    {/if}
    {#if saveMessage}
      <span style="font-size: 11px; color: var(--color-success);">{saveMessage}</span>
    {/if}
    {#if dirty}
      <span style="font-size: 11px; color: var(--emittiv-light);">Unsaved changes</span>
    {/if}
    <button
      class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm"
      onclick={handleReset}
      disabled={loading || saving}
    >
      Reset
    </button>
    <button
      class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm"
      onclick={handleLibraryDefaults}
      disabled={loading || saving}
      title="Replace the scope with the library defaults (asks first)"
    >
      Library defaults
    </button>
    <button
      class="emittiv-btn emittiv-btn--primary emittiv-btn--sm"
      onclick={handleSave}
      disabled={saving || loading || !loadedOk}
    >
      {saving ? 'Saving...' : 'Save'}
    </button>
  </div>

  <!-- Content -->
  {#if loading}
    <div
      style="display: flex; align-items: center; justify-content: center; flex: 1; color: var(--emittiv-light); font-size: 14px;"
    >
      Loading scope data...
    </div>
  {:else if error}
    <div
      style="display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1; gap: 12px;"
    >
      <p style="color: var(--color-error); font-size: 14px; margin: 0;">{error}</p>
      <button class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm" onclick={loadData}>
        Retry
      </button>
    </div>
  {:else}
    <div class="emittiv-scope-content">
      <!-- Stages (left panel) -->
      <div class="emittiv-scope-stages">
        {#each stageConfigs as stageConfig (stageConfig.canonical_name)}
          <StageSection
            stage={stageConfig}
            deliverables={activeByStage[stageConfig.canonical_name] || []}
            onRemove={handleRemove}
            onExpand={handleExpand}
            onReorder={handleReorder}
            onDropFromLibrary={handleDropFromLibrary}
          />
        {/each}

        {#if stageConfigs.length === 0}
          <p
            style="font-size: 13px; color: var(--emittiv-light); text-align: center; padding: 32px;"
          >
            No stage configurations found.
          </p>
        {/if}
      </div>

      <!-- Library (right panel) -->
      <DeliverableLibrary
        allDeliverables={libraryDeliverables}
        {activeIds}
        onAdd={handleAddFromLibrary}
      />
    </div>
  {/if}

  <!-- Detail modal -->
  {#if selectedDeliverable}
    <DeliverableDetail
      deliverable={selectedDeliverable}
      onClose={handleDetailClose}
      onSave={handleDetailSave}
    />
  {/if}
</div>
