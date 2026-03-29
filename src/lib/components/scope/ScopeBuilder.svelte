<script lang="ts">
  import './scope.css';
  import {
    getStages,
    getDeliverables,
    assembleDeliverables,
    saveScopeBuilder
  } from '$lib/api/scope';
  import { logApiError } from '$lib/services/logger';
  import type {
    StageConfig,
    Deliverable,
    StageDisplay,
    ScopeDeliverableEntry
  } from '$lib/types/scope';
  import StageSection from './StageSection.svelte';
  import DeliverableLibrary from './DeliverableLibrary.svelte';
  import DeliverableDetail from './DeliverableDetail.svelte';
  import { SvelteSet } from 'svelte/reactivity';

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

  // Derived: set of active IDs for library filtering
  let activeIds = $derived.by(() => {
    const ids = new SvelteSet<string>();
    for (const items of Object.values(activeByStage)) {
      for (const d of items) {
        ids.add(d.id);
      }
    }
    return ids;
  });

  // Derived: stages with labels for display
  let stageConfigs = $derived(
    stages
      .filter(s => s.status === 'active')
      .sort((a, b) => a.sort_order - b.sort_order)
      .map(s => ({
        canonical_name: s.canonical_name,
        label: s.default_label || s.canonical_name,
        sort_order: s.sort_order
      }))
  );

  async function loadData() {
    if (!feeId) {
      error = 'No fee ID provided';
      loading = false;
      return;
    }

    loading = true;
    error = null;

    try {
      // Fetch stages and all deliverables in parallel
      // API returns { data: [...], total: N } wrapper
      const [stagesResp, deliverablesResp] = await Promise.all([getStages(), getDeliverables()]);

      stages = stagesResp.data;
      libraryDeliverables = deliverablesResp.data;

      // Assemble initial deliverables for this fee
      await runAssemble();
    } catch (err: any) {
      error = err.message || 'Failed to load scope data';
      logApiError('ScopeBuilder load', err as Error);
    } finally {
      loading = false;
    }
  }

  async function runAssemble() {
    try {
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
    } catch (err: any) {
      logApiError('assemble scope', err as Error);
      // If assemble fails (e.g. no saved state), start with empty stages
      activeByStage = {};
    }
  }

  async function handleSave() {
    saving = true;
    saveMessage = null;

    try {
      const deliverables: ScopeDeliverableEntry[] = [];
      for (const [stageName, items] of Object.entries(activeByStage)) {
        for (const [idx, d] of items.entries()) {
          deliverables.push({
            deliverable_id: d.id,
            stage: stageName,
            sort_order: idx + 1,
            wording_override: null
          });
        }
      }

      // Build stage labels from current config
      const stageLabels: Record<string, string> = {};
      for (const sc of stageConfigs) {
        stageLabels[sc.canonical_name] = sc.label;
      }

      await saveScopeBuilder({
        fee_id: feeId,
        deliverables,
        stage_labels: stageLabels
      });

      saveMessage = 'Scope saved successfully';
      setTimeout(() => (saveMessage = null), 3000);
    } catch (err: any) {
      error = err.message || 'Failed to save scope';
    } finally {
      saving = false;
    }
  }

  async function handleReset() {
    loading = true;
    error = null;
    saveMessage = null;
    try {
      await runAssemble();
    } catch (err: any) {
      error = err.message || 'Failed to reset scope';
    } finally {
      loading = false;
    }
  }

  function handleRemove(deliverable: Deliverable) {
    const updated = { ...activeByStage };
    for (const [stage, items] of Object.entries(updated)) {
      updated[stage] = items.filter(d => d.id !== deliverable.id);
    }
    activeByStage = updated;
  }

  function handleExpand(deliverable: Deliverable) {
    selectedDeliverable = deliverable;
  }

  function handleDetailClose() {
    selectedDeliverable = null;
  }

  function handleDetailSave(updated: Deliverable) {
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
    selectedDeliverable = null;
  }

  function handleReorder(stage: StageDisplay, reordered: Deliverable[]) {
    activeByStage = { ...activeByStage, [stage.canonical_name]: reordered };
  }

  function handleDropFromLibrary(stage: StageDisplay, deliverable: Deliverable) {
    // Add to this stage if not already active
    if (activeIds.has(deliverable.id)) return;

    const current = activeByStage[stage.canonical_name] || [];
    activeByStage = {
      ...activeByStage,
      [stage.canonical_name]: [...current, deliverable]
    };
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
    {#if saveMessage}
      <span style="font-size: 11px; color: var(--color-success);">{saveMessage}</span>
    {/if}
    <button
      class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm"
      onclick={handleReset}
      disabled={loading}
    >
      Reset
    </button>
    <button
      class="emittiv-btn emittiv-btn--primary emittiv-btn--sm"
      onclick={handleSave}
      disabled={saving || loading}
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
