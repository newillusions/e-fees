<!--
  Merge Project Modal

  Recovers from a duplicate or mistaken project (e.g. one created in error
  from PA RFP intake) by folding its fee proposals into another project,
  then deleting the source. Two steps: pick a target project (typeahead,
  excludes the source itself), then review a server-computed preview
  (fees moved + any revision renumbering) before confirming - the merge
  itself is destructive (the source project is permanently deleted), so
  nothing happens without an explicit confirm on the actual preview.
-->
<script lang="ts">
  import { projectsStore, paginatedProjectsStore, feesActions } from '$lib/stores';
  import { extractIdFromRelation } from '$lib/utils/surrealdb';
  import { previewProjectMerge, mergeProjects } from '$lib/api';
  import { logMerge } from '$lib/services/activityLogger';
  import { logApiError } from '$lib/services/logger';
  import BaseModal from './BaseModal.svelte';
  import TypeaheadSelect from './TypeaheadSelect.svelte';
  import Button from './Button.svelte';
  import FolderReconcileModal from './FolderReconcileModal.svelte';
  import type { Project, ProjectMergePreview, ProjectMergeResult } from '../../types';

  let {
    isOpen = $bindable(false),
    sourceProject = null,
    onclose,
    onmerged
  }: {
    isOpen?: boolean;
    sourceProject?: Project | null;
    onclose?: () => void;
    onmerged?: (result: ProjectMergeResult) => void;
  } = $props();

  const sourceId = $derived(sourceProject ? extractIdFromRelation(sourceProject.id || '') : '');

  let targetId = $state('');
  let targetSearchText = $state('');
  let targetOptions: Array<{ id: string; number: string; name: string; name_short?: string }> =
    $state([]);

  let preview: ProjectMergePreview | null = $state(null);
  let loadingPreview = $state(false);
  let merging = $state(false);
  let error = $state('');

  // Optional Step 5: on-disk folder reconcile, offered after the DB merge
  // above has already committed. Fully decoupled - onmerged() only fires
  // once this step is skipped or completed, but the DB merge itself is
  // final the moment mergeProjects() resolves, regardless of what happens
  // here.
  let showReconcileModal = $state(false);
  let pendingMergeResult: ProjectMergeResult | null = $state(null);
  let reconcileSourceNumber = $state('');
  let reconcileTargetNumber = $state('');
  let reconcileSourceLabel = $state('');
  let reconcileTargetLabel = $state('');

  const allTargetOptions = $derived(
    $projectsStore
      .map(project => ({
        id: extractIdFromRelation(project.id || ''),
        number: project.number?.id || 'No Number',
        name: project.name,
        name_short: project.name_short
      }))
      .filter(option => option.id && option.id !== sourceId)
      .sort((a, b) => a.number.localeCompare(b.number))
  );

  function handleTargetSearch(searchText: string) {
    if (!searchText) {
      targetOptions = allTargetOptions.slice(0, 20);
      return;
    }
    const search = searchText.toLowerCase();
    targetOptions = allTargetOptions
      .filter(
        p =>
          p.number.toLowerCase().includes(search) ||
          p.name.toLowerCase().includes(search) ||
          (p.name_short || '').toLowerCase().includes(search)
      )
      .slice(0, 20);
  }

  async function handleTargetSelect(data: { id: string }) {
    targetId = data.id;
    preview = null;
    error = '';
    if (!sourceId || !targetId) return;

    loadingPreview = true;
    try {
      preview = await previewProjectMerge(sourceId, targetId);
    } catch (e) {
      logApiError('previewProjectMerge', e as Error, { component: 'MergeProjectModal' });
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loadingPreview = false;
    }
  }

  async function handleConfirmMerge() {
    if (!sourceId || !targetId || !sourceProject) return;

    merging = true;
    error = '';
    try {
      const result = await mergeProjects(sourceId, targetId);

      // Source project is gone - drop it from the projects list in place
      // rather than forcing a full refetch.
      paginatedProjectsStore.actions.removeItem(sourceProject.id || '');
      // Moved fees changed project_id (and possibly rev) server-side -
      // refresh the fee list so the target project's proposal list is
      // accurate immediately.
      await feesActions.load();

      const sourceName = sourceProject.name || sourceProject.number?.id || 'Unknown Project';
      const targetName = result.target.name || result.target.number?.id || 'Unknown Project';
      await logMerge(targetId, targetName, sourceName, {
        feesMoved: result.fees_moved,
        revChanges: result.rev_changes
      });

      // Capture folder-reconcile identifiers now, while sourceProject is
      // still in scope - the DB merge above already deleted the source
      // project's record, so its number is only reachable from this local
      // reference from here on (get_project_folder_location itself works
      // fine post-merge, since it scans folder names on disk rather than
      // querying the DB - this is purely about the frontend not losing the
      // reference once paginatedProjectsStore drops the row).
      pendingMergeResult = result;
      reconcileSourceNumber = sourceProject.number?.id || '';
      reconcileTargetNumber = result.target.number?.id || '';
      reconcileSourceLabel = `${sourceProject.number?.id ?? ''} - ${sourceName}`;
      reconcileTargetLabel = `${result.target.number?.id ?? ''} - ${targetName}`;

      handleClose();

      if (reconcileSourceNumber && reconcileTargetNumber) {
        showReconcileModal = true;
      } else {
        finishMerge();
      }
    } catch (e) {
      logApiError('mergeProjects', e as Error, { component: 'MergeProjectModal' });
      error = e instanceof Error ? e.message : String(e);
    } finally {
      merging = false;
    }
  }

  // Fires onmerged() for the caller (e.g. ProjectDetail closes its panel
  // since the source project no longer exists) - deferred until the
  // optional folder-reconcile step is skipped or completed, never blocked
  // by it.
  function finishMerge() {
    const result = pendingMergeResult;
    pendingMergeResult = null;
    if (result) onmerged?.(result);
  }

  function handleReconcileClose() {
    showReconcileModal = false;
    finishMerge();
  }

  function handleClose() {
    targetId = '';
    targetSearchText = '';
    targetOptions = [];
    preview = null;
    error = '';
    onclose?.();
  }
</script>

<BaseModal
  {isOpen}
  title="Merge Project"
  size="md"
  onclose={merging ? undefined : handleClose}
>
  {#if sourceProject}
    <div class="flex flex-col gap-4">
      <p class="text-sm text-emittiv-lighter">
        Move every fee proposal from <strong class="text-emittiv-white"
          >{sourceProject.number?.id} - {sourceProject.name}</strong
        > onto another project, then permanently delete this one. This cannot be undone.
      </p>

      <TypeaheadSelect
        label="Merge into"
        bind:value={targetId}
        bind:searchText={targetSearchText}
        placeholder="Search projects by number or name..."
        options={targetOptions}
        displayFields={['number', 'name']}
        onfocus={() => handleTargetSearch(targetSearchText)}
        oninput={handleTargetSearch}
        onselect={handleTargetSelect}
        disabled={merging}
      />

      {#if loadingPreview}
        <div class="flex items-center gap-2 text-sm text-emittiv-light">
          <div class="emittiv-spinner"></div>
          <span>Checking fee proposals...</span>
        </div>
      {/if}

      {#if error}
        <div class="emittiv-alert emittiv-alert--error">{error}</div>
      {/if}

      {#if preview && !loadingPreview}
        <div class="emittiv-card" style="padding: 12px;">
          <p class="text-sm text-emittiv-white font-medium mb-2">
            {#if preview.fees_to_move === 0}
              No fee proposals to move - the source project will simply be deleted.
            {:else}
              {preview.fees_to_move} fee proposal{preview.fees_to_move === 1 ? '' : 's'} will move
              onto <strong>{preview.target.number?.id} - {preview.target.name}</strong>.
            {/if}
          </p>

          {#if preview.rev_changes.some(c => c.old_rev !== c.new_rev)}
            <p class="text-xs text-emittiv-light mb-1">
              Revision numbers below will be renumbered to avoid a collision with proposals
              already on the target project:
            </p>
            <ul class="text-xs text-emittiv-lighter" style="padding-left: 16px;">
              {#each preview.rev_changes.filter(c => c.old_rev !== c.new_rev) as change (change.fee_id)}
                <li>{change.fee_number}: rev {change.old_rev} &rarr; rev {change.new_rev}</li>
              {/each}
            </ul>
          {/if}

          <p class="text-xs text-emittiv-light mt-2">
            Source project <strong>{sourceProject.number?.id}</strong> will be permanently deleted
            once the merge completes.
          </p>
        </div>
      {/if}
    </div>

    <div class="flex justify-end gap-2 mt-4">
      <Button variant="secondary" disabled={merging} on:click={handleClose}>Cancel</Button>
      <Button
        variant="danger"
        disabled={!preview || merging || loadingPreview}
        loading={merging}
        on:click={handleConfirmMerge}
      >
        {merging ? 'Merging...' : 'Merge and Delete Source'}
      </Button>
    </div>
  {/if}
</BaseModal>

<FolderReconcileModal
  bind:isOpen={showReconcileModal}
  sourceNumber={reconcileSourceNumber}
  targetNumber={reconcileTargetNumber}
  sourceLabel={reconcileSourceLabel}
  targetLabel={reconcileTargetLabel}
  onclose={handleReconcileClose}
/>
