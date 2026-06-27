<script lang="ts">
  import './scopeViewer.css';
  import { invoke } from '@tauri-apps/api/core';
  import {
    getScope,
    generateScope,
    updateScope,
    regenerateScope,
    assembleDeliverables
  } from '$lib/api/scope';
  import { logApiError } from '$lib/services/logger';
  import type { ScopeAssembly, ScopeSection, AssembleRequest } from '$lib/types/scope';
  import { formatSectionsAsText } from '$lib/utils/scopeFormatter';
  import ScopeSectionView from './ScopeSectionView.svelte';
  import ScopeAdvancedControls from './ScopeAdvancedControls.svelte';
  import ClausePicker from './ClausePicker.svelte';

  let {
    feeId,
    stages = [],
    projectName = '',
    projectNumber = '',
    ondirtychange
  }: {
    feeId: string;
    stages?: import('$lib/api/feeStages').FeeStage[];
    projectName?: string;
    projectNumber?: string;
    ondirtychange?: (isDirty: boolean) => void;
  } = $props();

  // Data state
  let scope: ScopeAssembly | null = $state(null);
  let sections: ScopeSection[] = $state([]);

  // UI state
  let loading = $state(true);
  let generating = $state(false);
  let saving = $state(false);
  let dirty = $state(false);
  let showAdvanced = $state(false);
  let showClausePicker = $state(false);
  let error: string | null = $state(null);
  let message: string | null = $state(null);

  async function loadScope() {
    loading = true;
    error = null;

    try {
      scope = await getScope(feeId);

      if (scope && Array.isArray(scope.clauses)) {
        sections = JSON.parse(JSON.stringify(scope.clauses));
      } else {
        sections = [];
      }
    } catch (err: any) {
      error = err.message || 'Failed to load scope data';
      logApiError('ScopeViewer load', err as Error);
    } finally {
      loading = false;
    }
  }

  async function handleGenerate() {
    generating = true;
    error = null;

    // AbortController with 90s timeout — signal is passed to generateScope → fetch
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 90_000);

    try {
      const result = await generateScope(
        {
          fee_id: feeId,
          polish: true,
          stages: stages.map(s => ({
            name: s.name,
            code: s.code,
            is_post_contract: s.is_post_contract,
            order: s.order
          }))
        },
        controller.signal
      );
      scope = result;
      sections = Array.isArray(result.clauses) ? JSON.parse(JSON.stringify(result.clauses)) : [];
      dirty = false;
      message = 'Scope generated successfully';
      setTimeout(() => (message = null), 3000);
    } catch (err: any) {
      if (err.name === 'AbortError') {
        error =
          'Generation timed out — the scope service may still be processing. Try again in a minute.';
      } else {
        error = err.message || 'Failed to generate scope';
        logApiError('ScopeViewer generate', err as Error);
      }
    } finally {
      clearTimeout(timeout);
      generating = false;
    }
  }

  async function handleRegenerate() {
    if (!confirm('This will replace your current scope text. Continue?')) return;

    generating = true;
    error = null;

    try {
      const result = await regenerateScope(feeId);
      scope = result;
      sections = Array.isArray(result.clauses) ? JSON.parse(JSON.stringify(result.clauses)) : [];
      dirty = false;
      message = 'Scope regenerated successfully';
      setTimeout(() => (message = null), 3000);
    } catch (err: any) {
      error = err.message || 'Failed to regenerate scope';
      logApiError('ScopeViewer regenerate', err as Error);
    } finally {
      generating = false;
    }
  }

  async function handleSave() {
    saving = true;
    error = null;

    try {
      const generatedText = formatSectionsAsText(sections);
      await updateScope(feeId, { clauses: sections, generated_text: generatedText });
      dirty = false;
      message = 'Scope saved successfully';

      // Export markdown to project folder if configured
      if (projectNumber) {
        try {
          const folderInfo = await invoke<{ full_path: string; exists: boolean }>(
            'get_project_folder_location',
            { projectNumber }
          );
          if (folderInfo.exists) {
            const revision = scope?.current_revision ?? 1;
            const stageNames = stages.map(s => s.name);
            const result = await invoke<string | null>('export_scope_markdown', {
              feeRef: feeId,
              projectName,
              projectFolder: folderInfo.full_path,
              revision,
              stages: stageNames,
              scopeText: generatedText
            });
            if (result) {
              message = 'Scope saved and exported to project folder';
            }
          }
        } catch (exportErr: any) {
          // Don't fail the save — export is best-effort
          logApiError('ScopeViewer export markdown', exportErr as Error);
        }
      }

      setTimeout(() => (message = null), 3000);
    } catch (err: any) {
      error = err.message || 'Failed to save scope';
      logApiError('ScopeViewer save', err as Error);
    } finally {
      saving = false;
    }
  }

  async function handleCopyAll() {
    const text = formatSectionsAsText(sections);
    try {
      await navigator.clipboard.writeText(text);
      message = 'Copied to clipboard';
      setTimeout(() => (message = null), 2000);
    } catch {
      error = 'Failed to copy to clipboard';
    }
  }

  function handleClauseUpdate(
    sectionIndex: number,
    clauseId: string,
    field: 'title' | 'body',
    value: string
  ) {
    const updated = JSON.parse(JSON.stringify(sections));
    const clause = updated[sectionIndex].clauses.find(
      (c: { clause_id: string }) => c.clause_id === clauseId
    );
    if (clause) {
      clause[field] = value;
      sections = updated;
      dirty = true;
    }
  }

  async function handleAdvancedAssemble(request: AssembleRequest) {
    // Call POST /scope/assemble with the discipline/stage selections
    generating = true;
    error = null;

    try {
      const result = await assembleDeliverables(request);
      // assembleDeliverables returns AssembleResponse with stages[] —
      // after assembly, trigger a full generate to get the numbered clauses
      await handleGenerate();
    } catch (err: any) {
      error = err.message || 'Failed to assemble scope';
      logApiError('ScopeViewer assemble', err as Error);
      generating = false;
    }
  }

  // Load on mount
  $effect(() => {
    if (feeId) loadScope();
  });

  // Notify parent when dirty state changes
  $effect(() => {
    ondirtychange?.(dirty);
  });
</script>

<div class="emittiv-scope-viewer">
  {#if loading}
    <div class="emittiv-scope-viewer__generating">
      <div class="emittiv-spinner-sm"></div>
      Loading scope data...
    </div>
  {:else if generating}
    <div class="emittiv-scope-viewer__generating">
      <div class="emittiv-spinner-sm"></div>
      Generating scope text... This may take up to 60 seconds.
    </div>
  {:else if error}
    <div class="emittiv-alert emittiv-alert--error">
      {error}
    </div>
    <button class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm" onclick={loadScope}>
      Retry
    </button>
  {:else if !scope}
    <!-- No existing scope — show generate controls -->
    <div class="emittiv-scope-viewer__empty">
      <p>No scope generated yet for this proposal.</p>

      <!-- Clause selection (Stage 1): shown before the generate button so users
           can customise their clause set before the first generation run. -->
      <button
        class="emittiv-scope-viewer__advanced-toggle"
        onclick={() => (showClausePicker = !showClausePicker)}
      >
        {showClausePicker ? '▼' : '▶'} Clause selection
      </button>

      {#if showClausePicker}
        <ClausePicker {feeId} onSaved={() => (message = 'Clause selection saved')} />
      {/if}

      <button class="emittiv-btn emittiv-btn--primary emittiv-btn--sm" onclick={handleGenerate}>
        Generate Scope
      </button>

      <button
        class="emittiv-scope-viewer__advanced-toggle"
        onclick={() => (showAdvanced = !showAdvanced)}
      >
        {showAdvanced ? '▼' : '▶'} Advanced options
      </button>

      {#if showAdvanced}
        <ScopeAdvancedControls
          {feeId}
          {stages}
          onassemble={handleAdvancedAssemble}
          loading={generating}
        />
      {/if}
    </div>
  {:else}
    <!-- Existing scope — structured view -->
    {#if message}
      <div class="emittiv-alert emittiv-alert--success emittiv-alert--sm">
        {message}
      </div>
    {/if}

    <div class="emittiv-scope-viewer__sections">
      {#each sections as section, i (i)}
        <ScopeSectionView
          {section}
          onupdate={(clauseId, field, value) => handleClauseUpdate(i, clauseId, field, value)}
        />
      {/each}

      {#if sections.length === 0}
        <p class="emittiv-scope-viewer__empty">No clauses generated. Try regenerating.</p>
      {/if}
    </div>

    {#if scope && stages.length > 0}
      {#each stages as stage}
        {#if !sections.some(s => s.clauses?.some(c => c.body?.includes(stage.name)))}
          <div class="emittiv-alert emittiv-alert--info emittiv-alert--sm">
            {stage.name}: no scope deliverables
          </div>
        {/if}
      {/each}
    {/if}

    {#if scope?.stages_snapshot && stages.length > 0}
      {#if stages.some(s => !scope?.stages_snapshot?.includes(s.name)) || scope?.stages_snapshot?.some(n => !stages.find(s => s.name === n))}
        <div class="emittiv-alert emittiv-alert--warning emittiv-alert--sm">
          Stages changed since last generation — regenerate to update scope text
        </div>
      {/if}
    {/if}

    <!-- Clause selection toggle (Stage 1) — access after scope exists -->
    <button
      class="emittiv-scope-viewer__advanced-toggle"
      onclick={() => (showClausePicker = !showClausePicker)}
    >
      {showClausePicker ? '▼' : '▶'} Clause selection
    </button>

    {#if showClausePicker}
      <ClausePicker
        {feeId}
        onSaved={() => (message = 'Clause selection saved — regenerate to apply')}
      />
    {/if}

    <!-- Advanced toggle -->
    <button
      class="emittiv-scope-viewer__advanced-toggle"
      onclick={() => (showAdvanced = !showAdvanced)}
    >
      {showAdvanced ? '▼' : '▶'} Advanced options
    </button>

    {#if showAdvanced}
      <ScopeAdvancedControls
        {feeId}
        {stages}
        onassemble={handleAdvancedAssemble}
        loading={generating}
      />
    {/if}

    <!-- Action bar -->
    <div class="emittiv-scope-viewer__actions">
      {#if dirty}
        <span class="emittiv-scope-viewer__status">Unsaved changes</span>
      {/if}
      <div class="emittiv-scope-viewer__actions-spacer"></div>
      <button
        class="emittiv-btn emittiv-btn--ghost emittiv-btn--sm"
        onclick={handleRegenerate}
        disabled={saving || generating}
      >
        Regenerate
      </button>
      <button
        class="emittiv-btn emittiv-btn--secondary emittiv-btn--sm"
        onclick={handleCopyAll}
        disabled={sections.length === 0}
      >
        Copy All
      </button>
      <button
        class="emittiv-btn emittiv-btn--primary emittiv-btn--sm"
        onclick={handleSave}
        disabled={saving || !dirty}
      >
        {saving ? 'Saving...' : 'Save'}
      </button>
    </div>
  {/if}
</div>
