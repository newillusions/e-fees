<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { check } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';

  let showModal = false;
  let updateAvailable = false;
  let updateInfo: { version: string; notes: string } | null = null;
  let downloading = false;
  let downloadProgress = 0;
  let downloadedBytes = 0;
  let totalBytes = 0;
  let readyToInstall = false;
  let error: string | null = null;
  let updateObject: any = null;

  // Check for updates on mount
  onMount(async () => {
    // Wait a bit after app starts to check for updates
    setTimeout(checkForUpdates, 3000);
  });

  async function checkForUpdates() {
    try {
      if (import.meta.env.DEV) {
        console.log('Checking for updates...');
      }

      const update = await check();

      if (update) {
        updateObject = update;
        updateAvailable = true;
        updateInfo = {
          version: update.version,
          notes: update.body || `New version ${update.version} is available`
        };
        showModal = true;

        if (import.meta.env.DEV) {
          console.log('Update available:', update.version);
        }
      } else {
        if (import.meta.env.DEV) {
          console.log('No updates available');
        }
      }
    } catch (err) {
      console.error('Failed to check for updates:', err);
      // Don't show error to user for update check failures
    }
  }

  async function downloadAndInstall() {
    if (!updateObject) return;

    downloading = true;
    error = null;

    try {
      // Download the update with progress tracking
      await updateObject.downloadAndInstall((event: any) => {
        switch (event.event) {
          case 'Started':
            totalBytes = event.data.contentLength || 0;
            downloadedBytes = 0;
            downloadProgress = 0;
            if (import.meta.env.DEV) {
              console.log('Download started, total size:', totalBytes);
            }
            break;
          case 'Progress':
            downloadedBytes += event.data.chunkLength;
            if (totalBytes > 0) {
              downloadProgress = Math.round((downloadedBytes / totalBytes) * 100);
            }
            break;
          case 'Finished':
            downloadProgress = 100;
            readyToInstall = true;
            if (import.meta.env.DEV) {
              console.log('Download finished');
            }
            break;
        }
      });

      // After download completes, prompt for restart
      readyToInstall = true;
      downloading = false;

    } catch (err) {
      console.error('Failed to download update:', err);
      error = err instanceof Error ? err.message : 'Failed to download update';
      downloading = false;
    }
  }

  async function installAndRestart() {
    try {
      // Relaunch the app to apply the update
      await relaunch();
    } catch (err) {
      console.error('Failed to relaunch:', err);
      error = err instanceof Error ? err.message : 'Failed to restart application';
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
    class="fixed inset-0 bg-black bg-opacity-50"
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
      on:click={(e) => e.stopPropagation()}
      in:scale={{ duration: 250, start: 0.95, easing: cubicOut }}
      out:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
    >
      <!-- Header -->
      <div class="flex items-center gap-3 mb-4">
        <div class="p-2 bg-emittiv-accent bg-opacity-20 rounded-lg">
          <svg class="w-6 h-6 text-emittiv-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
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
        <div class="mb-4 p-3 bg-emittiv-dark rounded text-sm text-emittiv-light max-h-32 overflow-y-auto">
          {updateInfo.notes}
        </div>
      {/if}

      <!-- Error message -->
      {#if error}
        <div class="mb-4 p-3 bg-red-900 bg-opacity-50 border border-red-700 rounded text-sm text-red-200">
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
