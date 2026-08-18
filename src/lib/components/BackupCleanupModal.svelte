<!--
  Backup Cleanup Modal

  Maintenance follow-up to the two on-disk reconcile flows
  (FolderReconcileModal's overwrite path, and the cascade-delete "trash on
  delete" checkbox) - both write their backups under the same
  {PROJECT_FOLDER_PATH}/.reconcile-backups/{timestamp}/ root (see
  src-tauri/src/commands/folder_reconcile.rs), and neither ever cleans that
  directory up on its own. This is where those accumulated backups get
  reviewed and removed.

  Flow: A) scan lists every backup older than the cutoff (dry-run, no
  writes), B) an explicit confirm step shows the count + total size before
  anything is deleted, C) delete is genuinely permanent - these backups are
  the LAST safety net for an overwrite or a trashed project folder, so
  there is no further backup taken here.
-->
<script lang="ts">
  import BaseModal from './BaseModal.svelte';
  import Button from './Button.svelte';
  import { previewBackupCleanup, executeBackupCleanup } from '$lib/api/folderReconcile';
  import { settingsStore } from '$lib/stores/settings';
  import type { BackupCleanupPreview, BackupCleanupOutcome } from '../../types';

  /** Backups exactly this many days old are kept - only backups strictly
   * older are eligible. Matches the backend default
   * (DEFAULT_BACKUP_CLEANUP_CUTOFF_DAYS in folder_reconcile.rs). */
  const CUTOFF_DAYS = 30;

  let {
    isOpen = $bindable(false),
    onclose
  }: {
    isOpen?: boolean;
    onclose?: () => void;
  } = $props();

  const basePath = $derived($settingsStore?.project_folder_path || '');

  let isScanning = $state(false);
  let preview: BackupCleanupPreview | null = $state(null);
  let scanError: string | null = $state(null);

  let showConfirm = $state(false);
  let isDeleting = $state(false);
  let deleteError: string | null = $state(null);
  let outcome: BackupCleanupOutcome | null = $state(null);

  $effect(() => {
    if (isOpen) {
      preview = null;
      scanError = null;
      showConfirm = false;
      deleteError = null;
      outcome = null;
    }
  });

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  async function startScan() {
    if (!basePath) {
      scanError = 'Project folder path is not configured. Please set it in Settings.';
      return;
    }

    isScanning = true;
    scanError = null;
    preview = null;
    outcome = null;
    showConfirm = false;

    try {
      preview = await previewBackupCleanup(CUTOFF_DAYS);
    } catch (error) {
      scanError = error instanceof Error ? error.message : String(error);
    } finally {
      isScanning = false;
    }
  }

  function requestDelete() {
    deleteError = null;
    showConfirm = true;
  }

  function cancelDelete() {
    showConfirm = false;
  }

  async function confirmDelete() {
    isDeleting = true;
    deleteError = null;

    try {
      outcome = await executeBackupCleanup(false, CUTOFF_DAYS);
      showConfirm = false;
      preview = null;
    } catch (error) {
      deleteError = error instanceof Error ? error.message : String(error);
    } finally {
      isDeleting = false;
    }
  }

  function closeModal() {
    onclose?.();
  }
</script>

<BaseModal {isOpen} onclose={closeModal} title="Clean Up Old Backups" maxWidth="600px">
  <div style="padding: 16px;">
    <!-- Base Path Display -->
    <div style="margin-bottom: 16px;">
      <span
        class="text-emittiv-lighter"
        style="display: block; font-size: 12px; margin-bottom: 4px;">Project Folder Path</span
      >
      <div
        class="bg-emittiv-dark text-emittiv-white"
        style="border-radius: 4px; padding: 8px 12px; font-size: 12px; font-family: monospace;"
      >
        {basePath || '(Not configured)'}
      </div>
      <p class="text-emittiv-light" style="font-size: 11px; margin-top: 6px;">
        Scans <code>.reconcile-backups/</code> under this path for backups older than {CUTOFF_DAYS}
        days - created by overwriting a conflicting file during a folder reconcile, or by trashing
        a deleted project's on-disk folder.
      </p>
    </div>

    <!-- Scan Button -->
    <div style="margin-bottom: 16px;">
      <button
        type="button"
        on:click={startScan}
        disabled={isScanning || !basePath}
        class="bg-emittiv-splash text-emittiv-black transition-smooth"
        style="padding: 6px 16px; font-size: 12px; height: 32px; border-radius: 4px; border: none; display: inline-flex; align-items: center; gap: 6px;"
        style:opacity={isScanning || !basePath ? '0.5' : '1'}
        style:cursor={isScanning || !basePath ? 'not-allowed' : 'pointer'}
      >
        {#if isScanning}
          <div
            class="border-emittiv-black"
            style="width: 12px; height: 12px; border: 2px solid; border-top-color: transparent; border-radius: 50%; animation: spin 1s linear infinite;"
          ></div>
          <span>Scanning...</span>
        {:else}
          <span>Scan Now</span>
        {/if}
      </button>
    </div>

    <!-- Scan Error -->
    {#if scanError}
      <div
        class="text-red-300"
        style="margin-bottom: 16px; padding: 12px; background-color: rgba(127, 29, 29, 0.3); border: 1px solid #dc2626; border-radius: 4px; font-size: 12px;"
      >
        {scanError}
      </div>
    {/if}

    <!-- Delete Error -->
    {#if deleteError}
      <div
        class="text-red-300"
        style="margin-bottom: 16px; padding: 12px; background-color: rgba(127, 29, 29, 0.3); border: 1px solid #dc2626; border-radius: 4px; font-size: 12px;"
      >
        {deleteError}
      </div>
    {/if}

    <!-- Deletion Outcome -->
    {#if outcome}
      <div
        style="margin-bottom: 16px; padding: 12px; background-color: rgba(20, 83, 45, 0.3); border: 1px solid #16a34a; border-radius: 4px;"
      >
        <div style="font-size: 12px; font-weight: 500; color: #4ade80;">
          Deleted {outcome.deleted_count} backup{outcome.deleted_count === 1 ? '' : 's'} - freed {formatBytes(
            outcome.freed_bytes
          )}
        </div>
        {#if outcome.errors.length > 0}
          <ul
            style="margin-top: 8px; font-size: 10px; color: #fef08a; list-style-type: disc; padding-left: 16px;"
          >
            {#each outcome.errors as err, i (i)}
              <li>{err}</li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}

    <!-- Results -->
    {#if preview}
      <!-- Statistics -->
      <div
        style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; margin-bottom: 16px;"
      >
        <div class="bg-emittiv-dark" style="border-radius: 4px; padding: 12px; text-align: center;">
          <div
            style="font-size: 20px; font-weight: 600; color: {preview.eligible_count === 0
              ? '#4ade80'
              : '#facc15'};"
          >
            {preview.eligible_count}
          </div>
          <div class="text-emittiv-light" style="font-size: 11px;">Backups Older Than {CUTOFF_DAYS}d</div>
        </div>
        <div class="bg-emittiv-dark" style="border-radius: 4px; padding: 12px; text-align: center;">
          <div class="text-emittiv-white" style="font-size: 20px; font-weight: 600;">
            {formatBytes(preview.total_size_bytes)}
          </div>
          <div class="text-emittiv-light" style="font-size: 11px;">Reclaimable</div>
        </div>
      </div>

      {#if preview.eligible_count > 0}
        <div
          class="border-emittiv-dark"
          style="border: 1px solid; border-radius: 4px; overflow: hidden; margin-bottom: 16px;"
        >
          <div
            class="bg-emittiv-dark text-emittiv-lighter"
            style="padding: 8px 12px; font-size: 12px; font-weight: 500;"
          >
            Eligible for Deletion
          </div>
          <div style="max-height: 220px; overflow-y: auto;">
            {#each preview.eligible as entry (entry.timestamp)}
              <div class="border-emittiv-dark" style="border-top: 1px solid; padding: 10px 12px;">
                <div style="display: flex; align-items: center; justify-content: space-between; gap: 12px;">
                  <div style="min-width: 0;">
                    <div
                      class="text-emittiv-white"
                      style="font-family: monospace; font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;"
                    >
                      {entry.timestamp}
                    </div>
                    <div class="text-emittiv-light" style="font-size: 10px; margin-top: 2px;">
                      {entry.age_days} days old - {entry.file_count} file{entry.file_count === 1
                        ? ''
                        : 's'}
                    </div>
                  </div>
                  <div class="text-emittiv-lighter" style="font-size: 11px; flex-shrink: 0;">
                    {formatBytes(entry.total_size_bytes)}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>

        {#if !showConfirm}
          <Button variant="danger" size="sm" disabled={isDeleting} on:click={requestDelete}>
            Delete {preview.eligible_count} Backup{preview.eligible_count === 1 ? '' : 's'}
          </Button>
        {:else}
          <div
            style="padding: 12px; background-color: rgba(127, 29, 29, 0.3); border: 1px solid #dc2626; border-radius: 4px;"
          >
            <div class="text-red-300" style="font-size: 12px; margin-bottom: 10px;">
              This will permanently delete {preview.eligible_count} backup folder{preview.eligible_count ===
              1
                ? ''
                : 's'} ({formatBytes(preview.total_size_bytes)}). These backups are the last safety
              net for an overwritten or trashed file - this cannot be undone.
            </div>
            <div style="display: flex; gap: 8px;">
              <Button variant="secondary" size="sm" disabled={isDeleting} on:click={cancelDelete}>
                Cancel
              </Button>
              <Button
                variant="danger"
                size="sm"
                disabled={isDeleting}
                loading={isDeleting}
                on:click={confirmDelete}
              >
                Yes, Delete Permanently
              </Button>
            </div>
          </div>
        {/if}
      {:else}
        <div style="text-align: center; padding: 24px 0; color: #4ade80;">
          <svg
            style="width: 32px; height: 32px; margin: 0 auto 8px;"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <div style="font-size: 13px; font-weight: 500;">No backups older than {CUTOFF_DAYS} days</div>
        </div>
      {/if}
    {:else if !isScanning && !scanError && !outcome}
      <div class="text-emittiv-light" style="text-align: center; padding: 24px 0;">
        <div style="font-size: 12px;">Click "Scan Now" to list old reconcile backups</div>
      </div>
    {/if}

    <!-- Footer -->
    <div
      class="border-emittiv-dark"
      style="margin-top: 16px; padding-top: 16px; border-top: 1px solid; display: flex; justify-content: flex-end;"
    >
      <button
        type="button"
        on:click={closeModal}
        class="bg-emittiv-dark text-emittiv-light border-emittiv-dark transition-smooth"
        style="padding: 6px 12px; font-size: 12px; height: 28px; border: 1px solid; border-radius: 4px; cursor: pointer;"
      >
        Close
      </button>
    </div>
  </div>
</BaseModal>

<style>
  .transition-smooth {
    transition: all 300ms cubic-bezier(0.4, 0, 0.2, 1);
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
