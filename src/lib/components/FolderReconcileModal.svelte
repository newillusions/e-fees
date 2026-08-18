<!--
  Folder Reconcile Modal

  Optional Step 5 following a project merge: the DB-level merge
  (MergeProjectModal / db/project_lifecycle.rs) never touches the
  filesystem, so this reconciles the two projects' on-disk folders
  separately and decoupled - closing or cancelling this modal never
  affects the already-committed DB merge.

  Flow: A) resolve both projects' on-disk folders by number, B) classify
  every file as new / conflict / identical (size-first, hash on a size
  match), C) resolve conflicts per-file or via bulk shortcuts, D) dry-run
  summary requiring an explicit confirm before any write actually happens.
-->
<script lang="ts">
  import { previewFolderReconcile, executeFolderReconcile } from '$lib/api/folderReconcile';
  import { getProjectFolderLocation } from '$lib/api/folderManagement';
  import { logApiError } from '$lib/services/logger';
  import BaseModal from './BaseModal.svelte';
  import Button from './Button.svelte';
  import type {
    FolderReconcilePreview,
    ReconcileAction,
    ReconcileFileEntry,
    ReconcileFileStatus,
    ReconcileResolution,
    FolderReconcileOutcome
  } from '../../types';

  let {
    isOpen = $bindable(false),
    sourceNumber = '',
    targetNumber = '',
    sourceLabel = '',
    targetLabel = '',
    onclose
  }: {
    isOpen?: boolean;
    /** Display-form project number, e.g. "26-97105" - matches the on-disk
     * folder naming convention (folder_management.rs::find_project_folder). */
    sourceNumber?: string;
    targetNumber?: string;
    sourceLabel?: string;
    targetLabel?: string;
    onclose?: () => void;
  } = $props();

  type Step = 'resolving' | 'no-folders' | 'reviewing' | 'confirming' | 'done' | 'error';

  let step: Step = $state('resolving');
  let sourcePath = $state('');
  let targetPath = $state('');
  let preview: FolderReconcilePreview | null = $state(null);
  let resolutions: Record<string, ReconcileAction> = $state({});
  let dryRunOutcome: FolderReconcileOutcome | null = $state(null);
  let finalOutcome: FolderReconcileOutcome | null = $state(null);
  let error = $state('');
  let busy = $state(false);

  function entriesByStatus(status: ReconcileFileStatus): ReconcileFileEntry[] {
    if (!preview) return [];
    return preview.entries.filter(entry => entry.status === status);
  }
  const conflictEntries = $derived(entriesByStatus('conflict'));
  const newEntries = $derived(entriesByStatus('new'));

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  async function resolveAndPreview() {
    step = 'resolving';
    error = '';
    try {
      const [sourceInfo, targetInfo] = await Promise.all([
        getProjectFolderLocation(sourceNumber),
        getProjectFolderLocation(targetNumber)
      ]);

      if (!sourceInfo.exists || !targetInfo.exists) {
        step = 'no-folders';
        return;
      }

      sourcePath = sourceInfo.full_path;
      targetPath = targetInfo.full_path;

      preview = await previewFolderReconcile(sourcePath, targetPath, sourceNumber, targetNumber);

      // Defaults: NEW files copy over by default (nothing to lose - the
      // destination doesn't have them); CONFLICTs default to skip, forcing
      // an explicit per-file or bulk decision before anything is applied.
      const defaults: Record<string, ReconcileAction> = {};
      for (const entry of preview.entries) {
        if (entry.status === 'new') defaults[entry.relative_path] = 'copy';
        else if (entry.status === 'conflict') defaults[entry.relative_path] = 'skip';
      }
      resolutions = defaults;

      step = 'reviewing';
    } catch (e) {
      logApiError('resolveAndPreview', e as Error, { component: 'FolderReconcileModal' });
      error = e instanceof Error ? e.message : String(e);
      step = 'error';
    }
  }

  $effect(() => {
    if (isOpen && sourceNumber && targetNumber) {
      resolveAndPreview();
    }
  });

  function buildResolutions(): ReconcileResolution[] {
    return Object.entries(resolutions)
      .filter(([, action]) => action !== undefined)
      .map(([relative_path, action]) => ({ relative_path, action }));
  }

  function bulkTakeAllFromSource() {
    const next: Record<string, ReconcileAction> = { ...resolutions };
    for (const entry of newEntries) next[entry.relative_path] = 'copy';
    for (const entry of conflictEntries) next[entry.relative_path] = 'overwrite';
    resolutions = next;
  }

  function bulkKeepAllDestination() {
    const next: Record<string, ReconcileAction> = { ...resolutions };
    for (const entry of newEntries) next[entry.relative_path] = 'skip';
    for (const entry of conflictEntries) next[entry.relative_path] = 'skip';
    resolutions = next;
  }

  function bulkKeepBothForConflicts() {
    const next: Record<string, ReconcileAction> = { ...resolutions };
    for (const entry of conflictEntries) next[entry.relative_path] = 'keep_both';
    resolutions = next;
  }

  async function handleReviewDryRun() {
    if (!preview) return;
    busy = true;
    error = '';
    try {
      dryRunOutcome = await executeFolderReconcile(
        sourcePath,
        targetPath,
        buildResolutions(),
        true,
        sourceNumber,
        targetNumber
      );
      step = 'confirming';
    } catch (e) {
      logApiError('executeFolderReconcile(dry_run)', e as Error, {
        component: 'FolderReconcileModal'
      });
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function handleConfirmApply() {
    busy = true;
    error = '';
    try {
      finalOutcome = await executeFolderReconcile(
        sourcePath,
        targetPath,
        buildResolutions(),
        false,
        sourceNumber,
        targetNumber
      );
      step = 'done';
    } catch (e) {
      logApiError('executeFolderReconcile', e as Error, { component: 'FolderReconcileModal' });
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function handleClose() {
    step = 'resolving';
    sourcePath = '';
    targetPath = '';
    preview = null;
    resolutions = {};
    dryRunOutcome = null;
    finalOutcome = null;
    error = '';
    onclose?.();
  }
</script>

<BaseModal {isOpen} title="Reconcile Project Folders" size="lg" onclose={busy ? undefined : handleClose}>
  <div class="flex flex-col gap-3">
    <p class="text-sm text-emittiv-lighter">
      The database merge is already complete. This optional step reconciles the on-disk folders
      for <strong class="text-emittiv-white">{sourceLabel || sourceNumber}</strong> and
      <strong class="text-emittiv-white">{targetLabel || targetNumber}</strong> - skip it and both
      folders are simply left as they are.
    </p>

    {#if step === 'resolving'}
      <div class="flex items-center gap-2 text-sm text-emittiv-light">
        <div class="emittiv-spinner"></div>
        <span>Locating project folders...</span>
      </div>
    {:else if step === 'no-folders'}
      <div class="emittiv-alert">
        No on-disk folder was found for one or both projects - nothing to reconcile.
      </div>
    {:else if step === 'error'}
      <div class="emittiv-alert emittiv-alert--error">{error}</div>
    {:else if step === 'reviewing' && preview}
      <div class="emittiv-card" style="padding: 12px;">
        <p class="text-xs text-emittiv-light">
          Source: <code class="text-emittiv-lighter">{sourcePath}</code>
        </p>
        <p class="text-xs text-emittiv-light mt-1">
          Target: <code class="text-emittiv-lighter">{targetPath}</code>
        </p>
        <p class="text-sm text-emittiv-white mt-2">
          {preview.new_count} new file{preview.new_count === 1 ? '' : 's'}, {preview.conflict_count}
          conflict{preview.conflict_count === 1 ? '' : 's'}, {preview.identical_count} identical
          (skipped automatically).
        </p>
      </div>

      {#if preview.new_count === 0 && preview.conflict_count === 0}
        <p class="text-sm text-emittiv-light">
          Nothing to reconcile - every file already matches or the source folder is empty.
        </p>
      {:else}
        <div class="flex gap-2 flex-wrap">
          <Button variant="secondary" size="sm" on:click={bulkTakeAllFromSource}
            >Take all from source</Button
          >
          <Button variant="secondary" size="sm" on:click={bulkKeepAllDestination}
            >Keep all destination</Button
          >
          {#if conflictEntries.length > 0}
            <Button variant="secondary" size="sm" on:click={bulkKeepBothForConflicts}
              >Keep both for all conflicts</Button
            >
          {/if}
        </div>

        {#if conflictEntries.length > 0}
          <div class="flex flex-col gap-1" style="max-height: 260px; overflow-y: auto;">
            <h3 class="text-xs font-medium text-emittiv-light uppercase tracking-wider">
              Conflicts ({conflictEntries.length})
            </h3>
            {#each conflictEntries as entry (entry.relative_path)}
              <div class="flex items-center justify-between gap-2 text-xs py-1">
                <span class="text-emittiv-lighter truncate" title={entry.relative_path}>
                  {entry.relative_path}
                  {#if entry.dest_relative_path !== entry.relative_path}
                    <span class="text-emittiv-light"> &rarr; {entry.dest_relative_path}</span>
                  {/if}
                </span>
                <select
                  class="emittiv-select"
                  style="width: auto; padding: 2px 8px; font-size: 12px;"
                  value={resolutions[entry.relative_path] ?? 'skip'}
                  onchange={e =>
                    (resolutions = {
                      ...resolutions,
                      [entry.relative_path]: (e.currentTarget as HTMLSelectElement)
                        .value as ReconcileAction
                    })}
                >
                  <option value="skip">Skip (keep destination)</option>
                  <option value="overwrite">Overwrite (backup then replace)</option>
                  <option value="keep_both">Keep both</option>
                </select>
              </div>
            {/each}
          </div>
        {/if}

        {#if newEntries.length > 0}
          <p class="text-xs text-emittiv-light">
            {newEntries.length} new file{newEntries.length === 1 ? '' : 's'} will be copied over
            by default.
          </p>
        {/if}
      {/if}

      {#if error}
        <div class="emittiv-alert emittiv-alert--error">{error}</div>
      {/if}
    {:else if step === 'confirming' && dryRunOutcome}
      <div class="emittiv-card" style="padding: 12px;">
        <p class="text-sm text-emittiv-white font-medium mb-1">Dry-run summary</p>
        <ul class="text-xs text-emittiv-lighter" style="padding-left: 16px;">
          <li>{dryRunOutcome.copied} file{dryRunOutcome.copied === 1 ? '' : 's'} copied</li>
          <li>
            {dryRunOutcome.overwritten} file{dryRunOutcome.overwritten === 1 ? '' : 's'} overwritten
            (existing file backed up to .reconcile-backups first)
          </li>
          <li>{dryRunOutcome.kept_both} file{dryRunOutcome.kept_both === 1 ? '' : 's'} kept both</li>
          <li>{dryRunOutcome.skipped} file{dryRunOutcome.skipped === 1 ? '' : 's'} skipped</li>
        </ul>
        {#if dryRunOutcome.errors.length > 0}
          <p class="text-xs text-emittiv-splash mt-2">
            {dryRunOutcome.errors.length} issue{dryRunOutcome.errors.length === 1 ? '' : 's'} found
            - review before proceeding:
          </p>
          <ul class="text-xs text-emittiv-lighter" style="padding-left: 16px;">
            {#each dryRunOutcome.errors as err, i (i)}
              <li>{err}</li>
            {/each}
          </ul>
        {/if}
      </div>
      <p class="text-xs text-emittiv-light">
        Nothing has been written yet. Confirm to apply these changes for real.
      </p>
      {#if error}
        <div class="emittiv-alert emittiv-alert--error">{error}</div>
      {/if}
    {:else if step === 'done' && finalOutcome}
      <div class="emittiv-card" style="padding: 12px;">
        <p class="text-sm text-emittiv-white font-medium mb-1">Reconcile complete</p>
        <ul class="text-xs text-emittiv-lighter" style="padding-left: 16px;">
          <li>{finalOutcome.copied} file{finalOutcome.copied === 1 ? '' : 's'} copied</li>
          <li>
            {finalOutcome.overwritten} file{finalOutcome.overwritten === 1 ? '' : 's'} overwritten
          </li>
          <li>{finalOutcome.kept_both} file{finalOutcome.kept_both === 1 ? '' : 's'} kept both</li>
          <li>{finalOutcome.skipped} file{finalOutcome.skipped === 1 ? '' : 's'} skipped</li>
        </ul>
        {#if finalOutcome.backups_created.length > 0}
          <p class="text-xs text-emittiv-light mt-2">
            {finalOutcome.backups_created.length} backup file{finalOutcome.backups_created
              .length === 1
              ? ''
              : 's'} written to .reconcile-backups.
          </p>
        {/if}
        {#if finalOutcome.errors.length > 0}
          <p class="text-xs text-emittiv-splash mt-2">
            {finalOutcome.errors.length} file{finalOutcome.errors.length === 1 ? '' : 's'} could
            not be reconciled:
          </p>
          <ul class="text-xs text-emittiv-lighter" style="padding-left: 16px;">
            {#each finalOutcome.errors as err, i (i)}
              <li>{err}</li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}
  </div>

  <div class="flex justify-end gap-2 mt-4">
    {#if step === 'reviewing'}
      <Button variant="secondary" disabled={busy} on:click={handleClose}>Skip</Button>
      <Button
        variant="primary"
        disabled={busy || !preview || (preview.new_count === 0 && preview.conflict_count === 0)}
        loading={busy}
        on:click={handleReviewDryRun}
      >
        Review Changes
      </Button>
    {:else if step === 'confirming'}
      <Button variant="secondary" disabled={busy} on:click={handleClose}>Cancel</Button>
      <Button variant="danger" disabled={busy} loading={busy} on:click={handleConfirmApply}>
        {busy ? 'Applying...' : 'Confirm and Apply'}
      </Button>
    {:else if step === 'no-folders' || step === 'error' || step === 'done'}
      <Button variant="primary" on:click={handleClose}>Close</Button>
    {/if}
  </div>
</BaseModal>
