<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import BaseModal from './BaseModal.svelte';
  import {
    scanFolderSync,
    resolveByUpdatingDatabase,
    resolveByMovingFolder,
    resolveByCreatingFolder,
    resolveByIgnoring,
    getInconsistencyDescription,
    getSuggestedResolution
  } from '$lib/api/folderSync';
  import { settingsStore } from '$lib/stores/settings';
  import { logActivity } from '$lib/services/activityLogger';
  import { projectsActions } from '$lib/stores';
  import type { FolderSyncResult, FolderInconsistency } from '../../types/folderSync';

  const dispatch = createEventDispatcher();

  let { isOpen = false }: {
    isOpen?: boolean;
  } = $props();

  // Scan state
  let isScanning = $state(false);
  let scanResult: FolderSyncResult | null = $state(null);
  let scanError: string | null = $state(null);

  // Resolution state
  let resolvingId: string | null = $state(null);
  let resolveError: string | null = $state(null);

  // Get base path from settings
  const basePath = $derived($settingsStore?.project_folder_path || '');

  // Reset state when modal opens
  $effect(() => {
    if (isOpen) {
      scanResult = null;
      scanError = null;
      resolveError = null;
    }
  });

  async function startScan() {
    if (!basePath) {
      scanError = 'Project folder path is not configured. Please set it in Settings.';
      return;
    }

    isScanning = true;
    scanError = null;
    scanResult = null;

    try {
      scanResult = await scanFolderSync(basePath);
    } catch (error) {
      scanError = error instanceof Error ? error.message : String(error);
    } finally {
      isScanning = false;
    }
  }

  async function resolveInconsistency(
    inconsistency: FolderInconsistency,
    action: 'update_db' | 'move_folder' | 'create_folder' | 'ignore'
  ) {
    const id = `${inconsistency.projectNumber}-${inconsistency.type}`;
    resolvingId = id;
    resolveError = null;

    try {
      switch (action) {
        case 'update_db':
          await resolveByUpdatingDatabase(inconsistency);
          break;
        case 'move_folder':
          await resolveByMovingFolder(inconsistency);
          break;
        case 'create_folder':
          await resolveByCreatingFolder(inconsistency);
          break;
        case 'ignore':
          await resolveByIgnoring(inconsistency);
          break;
      }

      // Log the folder sync action to activity feed
      const entityName = `${inconsistency.projectNumber} ${inconsistency.projectName || inconsistency.folderName}`;
      let description = '';
      let actionType: 'update' | 'status_change' | 'create' = 'update';

      switch (action) {
        case 'update_db':
          actionType = 'status_change';
          description = `Folder sync: Updated status from "${inconsistency.dbStatus}" to "${inconsistency.actualStatus}" for ${entityName}`;
          break;
        case 'move_folder':
          description = `Folder sync: Moved folder to correct location for ${entityName}`;
          break;
        case 'create_folder':
          actionType = 'create';
          description = `Folder sync: Created missing folder for ${entityName}`;
          break;
        case 'ignore':
          description = `Folder sync: Ignored inconsistency for ${entityName}`;
          break;
      }

      // Fire-and-forget activity log
      logActivity({
        action: actionType,
        entity_type: 'project',
        entity_id: inconsistency.projectId || inconsistency.projectNumber,
        entity_name: entityName,
        description,
        old_value: action === 'update_db' ? inconsistency.dbStatus || undefined : undefined,
        new_value: action === 'update_db' ? inconsistency.actualStatus || undefined : undefined,
        metadata: {
          source: 'folder_sync',
          action_type: action,
          inconsistency_type: inconsistency.type,
          actual_path: inconsistency.actualPath,
          expected_path: inconsistency.expectedPath
        }
      });

      // Refresh projects store to reflect DB changes in the UI
      // This ensures the Projects list view shows updated status values
      if (action === 'update_db') {
        await projectsActions.refresh();
      }

      // Remove resolved item from the list
      if (scanResult) {
        scanResult = {
          ...scanResult,
          inconsistencies: scanResult.inconsistencies.filter(
            (i) => !(i.projectNumber === inconsistency.projectNumber && i.type === inconsistency.type)
          )
        };
      }
    } catch (error) {
      resolveError = `Failed to resolve: ${error instanceof Error ? error.message : String(error)}`;
    } finally {
      resolvingId = null;
    }
  }

  function getTypeBadgeStyle(type: string): string {
    switch (type) {
      case 'wrong_location':
        return 'background-color: #ca8a04; color: #fef9c3;';
      case 'missing':
        return 'background-color: #dc2626; color: #fecaca;';
      case 'orphan':
        return 'background-color: #9333ea; color: #e9d5ff;';
      case 'duplicate':
        return 'background-color: #ea580c; color: #ffedd5;';
      default:
        return 'background-color: #4b5563; color: #e5e7eb;';
    }
  }

  function closeModal() {
    dispatch('close');
  }

  /**
   * Truncate path by removing the base path section and showing relative path
   * Also returns the status folder portion for highlighting
   */
  function formatPathForDisplay(fullPath: string | null | undefined): { prefix: string; statusFolder: string; suffix: string } | null {
    if (!fullPath) return null;

    // Try to find the base path in the full path and remove it
    // The base path is typically like /Volumes/NAS01/01 Projects
    // We want to show from the status folder (01 RFPs, 11 Current, etc.)
    const statusFolders = ['00 Inactive', '01 RFPs', '11 Current', '99 Completed'];

    for (const folder of statusFolders) {
      const idx = fullPath.indexOf(folder);
      if (idx !== -1) {
        // Found the status folder - split into three parts
        const prefix = '...';
        const statusFolder = folder;
        const suffix = fullPath.substring(idx + folder.length);
        return { prefix, statusFolder, suffix };
      }
    }

    // Fallback: just truncate long paths
    if (fullPath.length > 50) {
      return { prefix: '...', statusFolder: '', suffix: fullPath.slice(-50) };
    }

    return { prefix: '', statusFolder: '', suffix: fullPath };
  }
</script>

<BaseModal {isOpen} on:close={closeModal} title="Folder Sync" maxWidth="700px">
  <div style="padding: 16px;">
    <!-- Base Path Display -->
    <div style="margin-bottom: 16px;">
      <span class="text-emittiv-lighter" style="display: block; font-size: 12px; margin-bottom: 4px;">Project Folder Path</span>
      <div class="bg-emittiv-dark text-emittiv-white" style="border-radius: 4px; padding: 8px 12px; font-size: 12px; font-family: monospace;">
        {basePath || '(Not configured)'}
      </div>
    </div>

    <!-- Scan Button -->
    <div style="margin-bottom: 16px;">
      <button
        type="button"
        on:click={startScan}
        disabled={isScanning || !basePath}
        class="bg-emittiv-splash text-emittiv-black transition-smooth"
        style="padding: 6px 16px; font-size: 12px; height: 32px; border-radius: 4px; border: none; cursor: pointer; display: inline-flex; align-items: center; gap: 6px;"
        style:opacity={isScanning || !basePath ? '0.5' : '1'}
        style:cursor={isScanning || !basePath ? 'not-allowed' : 'pointer'}
      >
        {#if isScanning}
          <div class="border-emittiv-black" style="width: 12px; height: 12px; border: 2px solid; border-top-color: transparent; border-radius: 50%; animation: spin 1s linear infinite;"></div>
          <span>Scanning...</span>
        {:else}
          <span>Scan Now</span>
        {/if}
      </button>
    </div>

    <!-- Error Display -->
    {#if scanError}
      <div class="text-red-300" style="margin-bottom: 16px; padding: 12px; background-color: rgba(127, 29, 29, 0.3); border: 1px solid #dc2626; border-radius: 4px; font-size: 12px;">
        {scanError}
      </div>
    {/if}

    {#if resolveError}
      <div class="text-red-300" style="margin-bottom: 16px; padding: 12px; background-color: rgba(127, 29, 29, 0.3); border: 1px solid #dc2626; border-radius: 4px; font-size: 12px;">
        {resolveError}
      </div>
    {/if}

    <!-- Results -->
    {#if scanResult}
      <!-- Statistics -->
      <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin-bottom: 16px;">
        <div class="bg-emittiv-dark" style="border-radius: 4px; padding: 12px; text-align: center;">
          <div class="text-emittiv-white" style="font-size: 20px; font-weight: 600;">{scanResult.totalProjects}</div>
          <div class="text-emittiv-light" style="font-size: 11px;">DB Projects</div>
        </div>
        <div class="bg-emittiv-dark" style="border-radius: 4px; padding: 12px; text-align: center;">
          <div class="text-emittiv-white" style="font-size: 20px; font-weight: 600;">{scanResult.totalFolders}</div>
          <div class="text-emittiv-light" style="font-size: 11px;">Folders Found</div>
        </div>
        <div class="bg-emittiv-dark" style="border-radius: 4px; padding: 12px; text-align: center;">
          <div style="font-size: 20px; font-weight: 600; color: {scanResult.inconsistencies.length === 0 ? '#4ade80' : '#facc15'};">
            {scanResult.inconsistencies.length}
          </div>
          <div class="text-emittiv-light" style="font-size: 11px;">Issues</div>
        </div>
      </div>

      <!-- Inconsistencies List -->
      {#if scanResult.inconsistencies.length > 0}
        <div class="border-emittiv-dark" style="border: 1px solid; border-radius: 4px; overflow: hidden;">
          <div class="bg-emittiv-dark text-emittiv-lighter" style="padding: 8px 12px; font-size: 12px; font-weight: 500;">
            Inconsistencies Found
          </div>
          <div style="max-height: 240px; overflow-y: auto;">
            {#each scanResult.inconsistencies as inconsistency}
              {@const itemId = `${inconsistency.projectNumber}-${inconsistency.type}`}
              {@const isResolving = resolvingId === itemId}
              <div class="border-emittiv-dark" style="border-top: 1px solid; padding: 12px;">
                <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 12px;">
                  <div style="flex: 1; min-width: 0;">
                    <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
                      <span class="text-emittiv-white" style="font-family: monospace; font-size: 12px;">{inconsistency.projectNumber}</span>
                      <span style="padding: 2px 8px; font-size: 10px; border-radius: 4px; {getTypeBadgeStyle(inconsistency.type)}">
                        {inconsistency.type.replace('_', ' ')}
                      </span>
                    </div>
                    <div class="text-emittiv-light" style="font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                      {inconsistency.projectName || inconsistency.folderName}
                    </div>
                    <div class="text-emittiv-light" style="font-size: 10px; margin-top: 4px;">
                      {getInconsistencyDescription(inconsistency.type)}
                    </div>
                    {#if inconsistency.type === 'wrong_location'}
                      {@const actualParts = formatPathForDisplay(inconsistency.actualPath)}
                      {@const expectedParts = formatPathForDisplay(inconsistency.expectedPath)}
                      <div style="margin-top: 8px; font-size: 10px;">
                        <div class="text-emittiv-light" style="display: flex; align-items: baseline; gap: 4px;">
                          <span style="color: #f87171; flex-shrink: 0;">Found:</span>
                          <span style="font-family: monospace;">
                            {#if actualParts}
                              <span style="opacity: 0.5;">{actualParts.prefix}</span><span style="background-color: rgba(248, 113, 113, 0.3); color: #f87171; padding: 0 2px; border-radius: 2px;">{actualParts.statusFolder}</span><span>{actualParts.suffix}</span>
                            {:else}
                              (unknown)
                            {/if}
                          </span>
                        </div>
                        <div class="text-emittiv-light" style="display: flex; align-items: baseline; gap: 4px;">
                          <span style="color: #4ade80; flex-shrink: 0;">Expected:</span>
                          <span style="font-family: monospace;">
                            {#if expectedParts}
                              <span style="opacity: 0.5;">{expectedParts.prefix}</span><span style="background-color: rgba(74, 222, 128, 0.3); color: #4ade80; padding: 0 2px; border-radius: 2px;">{expectedParts.statusFolder}</span><span>{expectedParts.suffix}</span>
                            {:else}
                              (unknown)
                            {/if}
                          </span>
                        </div>
                      </div>
                    {/if}
                  </div>

                  <!-- Actions -->
                  <div style="display: flex; flex-direction: column; gap: 6px;">
                    {#if inconsistency.type === 'wrong_location'}
                      <button
                        type="button"
                        on:click={() => resolveInconsistency(inconsistency, 'update_db')}
                        disabled={isResolving}
                        class="transition-smooth"
                        style="padding: 4px 10px; font-size: 10px; background-color: #2563eb; color: white; border: none; border-radius: 4px; cursor: pointer;"
                        style:opacity={isResolving ? '0.5' : '1'}
                        title="Update database status to match folder location"
                      >
                        Update DB
                      </button>
                      <button
                        type="button"
                        on:click={() => resolveInconsistency(inconsistency, 'move_folder')}
                        disabled={isResolving}
                        class="transition-smooth"
                        style="padding: 4px 10px; font-size: 10px; background-color: #ca8a04; color: white; border: none; border-radius: 4px; cursor: pointer;"
                        style:opacity={isResolving ? '0.5' : '1'}
                        title="Move folder to expected location"
                      >
                        Move Folder
                      </button>
                    {:else if inconsistency.type === 'missing'}
                      <button
                        type="button"
                        on:click={() => resolveInconsistency(inconsistency, 'create_folder')}
                        disabled={isResolving}
                        class="transition-smooth"
                        style="padding: 4px 10px; font-size: 10px; background-color: #16a34a; color: white; border: none; border-radius: 4px; cursor: pointer;"
                        style:opacity={isResolving ? '0.5' : '1'}
                        title="Create folder from template"
                      >
                        Create
                      </button>
                    {:else}
                      <button
                        type="button"
                        on:click={() => resolveInconsistency(inconsistency, 'ignore')}
                        disabled={isResolving}
                        class="transition-smooth"
                        style="padding: 4px 10px; font-size: 10px; background-color: #4b5563; color: white; border: none; border-radius: 4px; cursor: pointer;"
                        style:opacity={isResolving ? '0.5' : '1'}
                        title="Ignore this inconsistency"
                      >
                        Ignore
                      </button>
                    {/if}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div style="text-align: center; padding: 32px 0; color: #4ade80;">
          <svg style="width: 40px; height: 40px; margin: 0 auto 8px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <div style="font-size: 14px; font-weight: 500;">All folders in sync!</div>
          <div class="text-emittiv-light" style="font-size: 11px;">No inconsistencies detected</div>
        </div>
      {/if}

      <!-- Scan Errors -->
      {#if scanResult.errors.length > 0}
        <div style="margin-top: 16px; padding: 12px; background-color: rgba(113, 63, 18, 0.3); border: 1px solid #ca8a04; border-radius: 4px;">
          <div style="font-size: 12px; font-weight: 500; color: #fcd34d; margin-bottom: 8px;">Scan Warnings</div>
          <ul style="font-size: 10px; color: #fef08a; list-style-type: disc; padding-left: 16px; margin: 0;">
            {#each scanResult.errors as error}
              <li>{error}</li>
            {/each}
          </ul>
        </div>
      {/if}

      <!-- Scan Time -->
      <div class="text-emittiv-light" style="margin-top: 16px; font-size: 10px; text-align: right;">
        Scanned at: {new Date(scanResult.scannedAt).toLocaleString()}
      </div>
    {:else if !isScanning && !scanError}
      <div class="text-emittiv-light" style="text-align: center; padding: 32px 0;">
        <svg style="width: 40px; height: 40px; margin: 0 auto 8px; opacity: 0.5;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
        <div style="font-size: 12px;">Click "Scan Now" to check folder consistency</div>
      </div>
    {/if}

    <!-- Footer -->
    <div class="border-emittiv-dark" style="margin-top: 16px; padding-top: 16px; border-top: 1px solid; display: flex; justify-content: flex-end;">
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
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
