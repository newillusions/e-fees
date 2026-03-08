<script lang="ts">
  import './scope.css';
  import { onMount } from 'svelte';
  import {
    getStages,
    getDeliverables,
    assembleDeliverables,
    saveScopeBuilder,
  } from '$lib/api/scope';
  import StageSection from './StageSection.svelte';
  import DeliverableLibrary from './DeliverableLibrary.svelte';
  import DeliverableDetail from './DeliverableDetail.svelte';

  let {
    params = {},
  }: {
    params?: { id?: string };
  } = $props();

  let feeId = $derived(params.id || '');

  // Data state
  let stages: any[] = $state([]);
  let activeByStage: Record<string, any[]> = $state({});
  let libraryDeliverables: any[] = $state([]);
  let selectedDeliverable: any | null = $state(null);

  // UI state
  let loading = $state(true);
  let saving = $state(false);
  let error: string | null = $state(null);
  let saveMessage: string | null = $state(null);

  // Derived: set of active IDs for library filtering
  let activeIds = $derived(() => {
    const ids = new Set<string>();
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
      .filter((s) => s.status === 'active')
      .sort((a, b) => a.sort_order - b.sort_order)
      .map((s) => ({
        canonical_name: s.canonical_name,
        label: s.default_label || s.canonical_name,
        sort_order: s.sort_order,
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
      const [stagesResp, deliverablesResp] = await Promise.all([
        getStages(),
        getDeliverables(),
      ]);

      stages = stagesResp.data || stagesResp;
      libraryDeliverables = deliverablesResp.data || deliverablesResp;

      // Assemble initial deliverables for this fee
      await runAssemble();
    } catch (err: any) {
      error = err.message || 'Failed to load scope data';
      console.error('ScopeBuilder load error:', err);
    } finally {
      loading = false;
    }
  }

  async function runAssemble() {
    try {
      const result = await assembleDeliverables({
        fee_id: feeId,
        disciplines: ['lighting'],
      });
      const byStage: Record<string, any[]> = {};

      const assemblyData = result.data || result;
      const stageList = assemblyData.stages || assemblyData;
      if (Array.isArray(stageList)) {
        for (const s of stageList) {
          byStage[s.canonical_name || s.stage] = s.deliverables || [];
        }
      }

      activeByStage = byStage;
    } catch (err: any) {
      console.error('Assemble error:', err);
      // If assemble fails (e.g. no saved state), start with empty stages
      activeByStage = {};
    }
  }

  async function handleSave() {
    saving = true;
    saveMessage = null;

    try {
      const deliverables: any[] = [];
      for (const [stageName, items] of Object.entries(activeByStage)) {
        for (const [idx, d] of items.entries()) {
          deliverables.push({
            deliverable_id: d.id,
            stage: stageName,
            sort_order: idx + 1,
            wording_override: null,
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
        stage_labels: stageLabels,
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

  function handleRemove(deliverable: any) {
    const updated = { ...activeByStage };
    for (const [stage, items] of Object.entries(updated)) {
      updated[stage] = items.filter((d) => d.id !== deliverable.id);
    }
    activeByStage = updated;
  }

  function handleExpand(deliverable: any) {
    selectedDeliverable = deliverable;
  }

  function handleDetailClose() {
    selectedDeliverable = null;
  }

  function handleDetailSave(updated: any) {
    // Update the deliverable in the active set
    const newByStage = { ...activeByStage };
    for (const [stage, items] of Object.entries(newByStage)) {
      const idx = items.findIndex((d) => d.id === updated.id);
      if (idx >= 0) {
        newByStage[stage] = [...items];
        newByStage[stage][idx] = updated;
      }
    }
    activeByStage = newByStage;
    selectedDeliverable = null;
  }

  function handleReorder(stage: any, reordered: any[]) {
    activeByStage = { ...activeByStage, [stage.canonical_name]: reordered };
  }

  function handleDropFromLibrary(stage: any, deliverable: any) {
    // Add to this stage if not already active
    if (activeIds().has(deliverable.id)) return;

    const current = activeByStage[stage.canonical_name] || [];
    activeByStage = {
      ...activeByStage,
      [stage.canonical_name]: [...current, deliverable],
    };
  }

  function handleAddFromLibrary(deliverable: any, _targetStage?: string) {
    // Add to the deliverable's native stage, or the first available stage
    const targetStage =
      _targetStage || deliverable.stage || stageConfigs[0]?.canonical_name;
    if (!targetStage) return;
    if (activeIds().has(deliverable.id)) return;

    const current = activeByStage[targetStage] || [];
    activeByStage = {
      ...activeByStage,
      [targetStage]: [...current, deliverable],
    };
  }

  onMount(() => {
    loadData();
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
      <span style="font-size: 11px; color: #4caf50;">{saveMessage}</span>
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
    <div style="display: flex; align-items: center; justify-content: center; flex: 1; color: var(--emittiv-light); font-size: 14px;">
      Loading scope data...
    </div>
  {:else if error}
    <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1; gap: 12px;">
      <p style="color: #ef5350; font-size: 14px; margin: 0;">{error}</p>
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
          <p style="font-size: 13px; color: var(--emittiv-light); text-align: center; padding: 32px;">
            No stage configurations found.
          </p>
        {/if}
      </div>

      <!-- Library (right panel) -->
      <DeliverableLibrary
        allDeliverables={libraryDeliverables}
        activeIds={activeIds()}
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
