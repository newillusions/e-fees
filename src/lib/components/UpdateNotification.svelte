<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { check, type DownloadEvent } from '@tauri-apps/plugin-updater';
  import type { Update } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { logMessage, getDevMode, type LogLevel } from '$lib/api/system';

  let showModal = false;
  let updateAvailable = false;
  let updateInfo: { version: string; notes: string } | null = null;
  let downloading = false;
  let downloadProgress = 0;
  let downloadedBytes = 0;
  let totalBytes = 0;
  let readyToInstall = false;
  let error: string | null = null;
  let updateObject: Update | null = null;
  let devMode = false;

  // Enhanced logging function - logs to console and backend
  // ARCH-L1: Routes invoke call through API layer
  async function log(level: LogLevel, message: string, data?: unknown) {
    const timestamp = new Date().toISOString();
    const formattedMessage = `[${timestamp}] [UPDATER] ${message}`;

    if (devMode || level === 'error') {
      if (data) {
        console[level](formattedMessage, data);
      } else {
        console[level](formattedMessage);
      }

      // Also log to backend for file logging via API layer
      await logMessage(level, 'updater', message, data ? JSON.stringify(data) : null);
    }
  }

  // Check for updates on mount
  onMount(async () => {
    // Check dev mode first via API layer
    devMode = await getDevMode();
    await log('info', `Dev mode: ${devMode}`);

    // Wait a bit after app starts to check for updates
    setTimeout(checkForUpdates, 3000);
  });

  async function checkForUpdates() {
    try {
      await log('info', 'Starting update check...');
      await log('debug', 'Calling Tauri updater plugin check()');

      const update = await check();

      if (update) {
        await log('info', `Update available: ${update.version}`, {
          version: update.version,
          currentVersion: update.currentVersion,
          date: update.date,
          body: update.body?.substring(0, 100) // First 100 chars of notes
        });

        updateObject = update;
        updateAvailable = true;
        updateInfo = {
          version: update.version,
          notes: update.body || `New version ${update.version} is available`
        };
        showModal = true;
      } else {
        await log('info', 'No updates available - app is up to date');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      await log('error', `Failed to check for updates: ${errorMessage}`, err);
      // Don't show error to user for update check failures
    }
  }

  async function downloadAndInstall() {
    if (!updateObject) {
      await log('warn', 'downloadAndInstall called but no updateObject available');
      return;
    }

    await log('info', 'Starting download and install process...');
    downloading = true;
    error = null;

    try {
      await log('debug', 'Calling updateObject.downloadAndInstall()');

      // Download the update with progress tracking
      await updateObject!.downloadAndInstall((event: DownloadEvent) => {
        switch (event.event) {
          case 'Started':
            totalBytes = event.data.contentLength || 0;
            downloadedBytes = 0;
            downloadProgress = 0;
            log('info', `Download started - Total size: ${totalBytes} bytes`);
            break;
          case 'Progress':
            downloadedBytes += event.data.chunkLength;
            if (totalBytes > 0) {
              downloadProgress = Math.round((downloadedBytes / totalBytes) * 100);
            }
            // Log progress every 10%
            if (downloadProgress % 10 === 0) {
              log(
                'debug',
                `Download progress: ${downloadProgress}% (${downloadedBytes}/${totalBytes} bytes)`
              );
            }
            break;
          case 'Finished':
            downloadProgress = 100;
            readyToInstall = true;
            log('info', 'Download finished successfully');
            break;
        }
      });

      await log('info', 'downloadAndInstall completed - ready to restart');
      // After download completes, prompt for restart
      readyToInstall = true;
      downloading = false;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      const errorStack = err instanceof Error ? err.stack : undefined;
      await log('error', `DOWNLOAD FAILED: ${errorMessage}`, {
        error: errorMessage,
        stack: errorStack,
        downloadedBytes,
        totalBytes,
        downloadProgress
      });
      error = errorMessage || 'Failed to download update';
      downloading = false;
    }
  }

  async function installAndRestart() {
    try {
      await log('info', 'Initiating app restart to apply update...');
      // Relaunch the app to apply the update
      await relaunch();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      await log('error', `Failed to relaunch: ${errorMessage}`, err);
      error = errorMessage || 'Failed to restart application';
    }
  }

  function closeModal() {
    showModal = false;
  }

  function remindLater() {
    showModal = false;
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }
</script>

{#if showModal && updateAvailable}
  <!-- Modal backdrop -->
  <div
    class="emittiv-backdrop"
    style="z-index: 200;"
    on:click={closeModal}
    on:keydown={() => {}}
    role="button"
    tabindex="-1"
    aria-label="Close modal"
    in:fade={{ duration: 200 }}
    out:fade={{ duration: 200 }}
  ></div>

  <!-- Modal container -->
  <div
    class="fixed inset-0 flex items-center justify-center p-4 pointer-events-none"
    style="z-index: 201;"
    role="dialog"
    aria-modal="true"
    aria-labelledby="update-modal-title"
  >
    <div
      class="bg-emittiv-darker border border-emittiv-dark rounded w-full max-w-md pointer-events-auto"
      style="padding: 24px;"
      on:click={e => e.stopPropagation()}
      in:scale={{ duration: 250, start: 0.95, easing: cubicOut }}
      out:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
    >
      <!-- Header -->
      <div class="flex items-center gap-3 mb-4">
        <div class="p-2 bg-emittiv-accent bg-opacity-20 rounded-lg">
          <svg
            class="w-6 h-6 text-emittiv-accent"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
            />
          </svg>
        </div>
        <div>
          <h2 id="update-modal-title" class="font-semibold text-emittiv-white text-lg">
            Update Available
          </h2>
          <p class="text-sm text-emittiv-light">
            Version {updateInfo?.version}
          </p>
        </div>
      </div>

      <!-- Release notes -->
      {#if updateInfo?.notes}
        <div
          class="mb-4 p-3 bg-emittiv-dark rounded text-sm text-emittiv-light max-h-32 overflow-y-auto"
        >
          {updateInfo.notes}
        </div>
      {/if}

      <!-- Error message -->
      {#if error}
        <div class="emittiv-alert emittiv-alert--error mb-4">
          {error}
        </div>
      {/if}

      <!-- Download progress -->
      {#if downloading}
        <div class="mb-4">
          <div class="flex justify-between text-sm text-emittiv-light mb-2">
            <span>Downloading...</span>
            <span>{downloadProgress}%</span>
          </div>
          <div class="w-full bg-emittiv-dark rounded-full h-2">
            <div
              class="bg-emittiv-accent h-2 rounded-full transition-all duration-300"
              style="width: {downloadProgress}%"
            ></div>
          </div>
          {#if totalBytes > 0}
            <div class="text-xs text-emittiv-light mt-1 text-right">
              {formatBytes(downloadedBytes)} / {formatBytes(totalBytes)}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Actions -->
      <div class="flex gap-3 justify-end">
        {#if readyToInstall}
          <button
            on:click={installAndRestart}
            class="px-4 py-2 bg-emittiv-accent text-white rounded hover:bg-opacity-90 transition-colors font-medium"
          >
            Restart Now
          </button>
        {:else if downloading}
          <button
            disabled
            class="px-4 py-2 bg-emittiv-dark text-emittiv-light rounded cursor-not-allowed"
          >
            Downloading...
          </button>
        {:else}
          <button
            on:click={remindLater}
            class="px-4 py-2 text-emittiv-light hover:text-emittiv-white transition-colors"
          >
            Later
          </button>
          <button
            on:click={downloadAndInstall}
            class="px-4 py-2 bg-emittiv-accent text-white rounded hover:bg-opacity-90 transition-colors font-medium"
          >
            Update Now
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
