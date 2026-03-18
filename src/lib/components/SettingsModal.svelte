<script lang="ts">
  import { settingsStore, settingsLoading, settingsError, settingsActions, type AppSettings } from '$lib/stores/settings';
  import { invoke } from '@tauri-apps/api/core';
  import { reloadDatabaseConfig, reconnectDatabase } from '$lib/api';
  import { loadAllData } from '$lib/stores';
  import FolderSyncModal from './FolderSyncModal.svelte';
  import { logger, logApiError } from '$lib/services/logger';

  // Folder sync modal state
  let showFolderSyncModal = $state(false);

  let { isOpen = $bindable(false), onclose }: {
    isOpen?: boolean;
    onclose?: () => void;
  } = $props();

  // Settings form data - initialize with defaults
  // This uses AppSettings (with surrealdb_pass) for the form, even though
  // the store holds AppSettingsPublic (with has_password instead)
  let settings: AppSettings = $state({
    surrealdb_url: '',
    surrealdb_ns: '',
    surrealdb_db: '',
    surrealdb_user: '',
    surrealdb_pass: '',
    staff_name: '',
    staff_email: '',
    staff_phone: '',
    staff_position: '',
    project_folder_path: '',
    dev_mode: false,
    log_level: 'info'
  });

  // Track whether a password exists (from backend) and if user entered new one
  let hasExistingPassword = $state(false);
  let passwordChanged = $state(false);

  // Loading states
  let isSaving = $state(false);
  let saveMessage = $state('');
  let isTesting = $state(false);
  let testMessage = $state('');

  // Load settings when modal opens
  $effect(() => {
    if (isOpen) {
      loadSettings();
    }
  });

  // Update form when store changes (store holds public settings without password)
  $effect(() => {
    if ($settingsStore) {
      hasExistingPassword = $settingsStore.has_password;
      settings = {
        surrealdb_url: $settingsStore.surrealdb_url || '',
        surrealdb_ns: $settingsStore.surrealdb_ns || '',
        surrealdb_db: $settingsStore.surrealdb_db || '',
        surrealdb_user: $settingsStore.surrealdb_user || '',
        // Password is never returned from backend - start empty
        // User can enter new password or leave empty to keep existing
        surrealdb_pass: '',
        staff_name: $settingsStore.staff_name || '',
        staff_email: $settingsStore.staff_email || '',
        staff_phone: $settingsStore.staff_phone || '',
        staff_position: $settingsStore.staff_position || '',
        project_folder_path: $settingsStore.project_folder_path || '',
        dev_mode: $settingsStore.dev_mode || false,
        log_level: $settingsStore.log_level || 'info'
      };
      passwordChanged = false;
    }
  });

  // Track when user modifies the password field
  function onPasswordInput() {
    passwordChanged = true;
  }

  // Apply log level change immediately (no save needed)
  async function handleLogLevelChange() {
    try {
      await invoke('set_log_level', { level: settings.log_level || 'info' });
    } catch (e) {
      logApiError('set log level', e as Error);
    }
  }
  
  async function loadSettings() {
    await settingsActions.load();
  }
  
  async function saveSettingsForm() {
    isSaving = true;
    saveMessage = '';

    try {
      // Validate required fields
      if (!settings.surrealdb_url || !settings.surrealdb_ns || !settings.surrealdb_db) {
        throw new Error('SurrealDB connection fields are required');
      }

      // Prepare settings for save
      // If password wasn't changed and there's an existing password, don't send empty string
      // (the backend will preserve existing password if surrealdb_pass is undefined)
      const settingsToSave: AppSettings = { ...settings };
      if (!passwordChanged && hasExistingPassword && !settingsToSave.surrealdb_pass) {
        // Remove password field so backend preserves existing value
        delete settingsToSave.surrealdb_pass;
      }

      // Save settings to .env file
      await settingsActions.save(settingsToSave);
      saveMessage = 'Settings saved! Applying configuration...';

      // Reload database configuration in real-time
      try {
        await reloadDatabaseConfig();

        // Refresh data with new connection
        await loadAllData();

        saveMessage = 'Settings saved and applied successfully! Database connection updated.';
      } catch (reloadError) {
        logger.warn('Database reload failed');
        saveMessage = 'Settings saved but database reload failed. You may need to restart the app.';
      }

      // Auto-close after 3 seconds (longer to show the reload process)
      setTimeout(() => {
        closeModal();
      }, 3000);

    } catch (error) {
      saveMessage = `Error: ${error instanceof Error ? error.message : String(error)}`;
    } finally {
      isSaving = false;
    }
  }
  
  function closeModal() {
    isOpen = false;
    saveMessage = '';
    onclose?.();
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeModal();
    }
  }
  
  async function selectFolder() {
    try {
      // Import the API function
      const { selectFolder: apiSelectFolder } = await import('$lib/api');

      // Open native folder picker dialog
      const folderPath = await apiSelectFolder();

      if (folderPath) {
        settings.project_folder_path = folderPath;
      }
    } catch (error) {
      logApiError('select folder', error as Error);
    }
  }

  async function testConnection() {
    isTesting = true;
    testMessage = '';

    try {
      // Force reconnect to database using saved config
      await reconnectDatabase();

      // If successful, reload data
      await loadAllData();

      testMessage = 'Connected!';

      // Clear success message after 3 seconds
      setTimeout(() => {
        testMessage = '';
      }, 3000);
    } catch (error) {
      testMessage = `Connection failed: ${error instanceof Error ? error.message : String(error)}`;
    } finally {
      isTesting = false;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Modal Backdrop -->
  <div
    class="emittiv-backdrop flex items-center justify-center p-4"
    style="z-index: 80;"
    on:click={closeModal}
    on:keydown={(e) => e.key === 'Escape' && closeModal()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="settings-title"
    tabindex="-1"
  >
    <!-- Modal Content -->
    <div 
      class="bg-emittiv-darker border border-emittiv-dark rounded w-full overflow-y-auto"
      style="padding: 16px; max-width: 450px; max-height: 90vh;"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="presentation"
    >
      <!-- Header -->
      <div class="flex items-center justify-between" style="margin-bottom: 20px;">
        <h2 id="settings-title" class="font-semibold text-emittiv-white" style="font-size: 16px;">Application Settings</h2>
        <button 
          on:click={closeModal}
          class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth"
          aria-label="Close settings"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      {#if $settingsLoading}
        <div class="flex items-center justify-center py-8">
          <div class="emittiv-spinner emittiv-spinner--lg" style="border-color: var(--emittiv-splash); border-top-color: transparent;"></div>
          <span class="ml-2 text-emittiv-light">Loading settings...</span>
        </div>
      {:else}
        <form on:submit|preventDefault={saveSettingsForm} style="display: flex; flex-direction: column; gap: 24px;">
          
          <!-- SurrealDB Connection Section -->
          <div>
            <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">SurrealDB Connection</h3>
            <div style="display: flex; flex-direction: column; gap: 12px;">
              <div>
                <label for="surrealdb_url" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Database URL *
                </label>
                <input
                  id="surrealdb_url"
                  type="text"
                  bind:value={settings.surrealdb_url}
                  placeholder="ws://10.0.1.17:8000"
                  required
                  class="emittiv-input"
                />
              </div>
              
              <div class="grid grid-cols-2" style="gap: 12px;">
                <div>
                  <label for="surrealdb_ns" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                    Namespace *
                  </label>
                  <input
                    id="surrealdb_ns"
                    type="text"
                    bind:value={settings.surrealdb_ns}
                    placeholder="emittiv"
                    required
                    class="emittiv-input"
                  />
                </div>
                
                <div>
                  <label for="surrealdb_db" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                    Database *
                  </label>
                  <input
                    id="surrealdb_db"
                    type="text"
                    bind:value={settings.surrealdb_db}
                    placeholder="projects"
                    required
                    class="emittiv-input"
                  />
                </div>
              </div>
              
              <div class="grid grid-cols-2" style="gap: 12px;">
                <div>
                  <label for="surrealdb_user" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                    Username
                  </label>
                  <input
                    id="surrealdb_user"
                    type="text"
                    bind:value={settings.surrealdb_user}
                    placeholder="martin"
                    class="emittiv-input"
                  />
                </div>
                
                <div>
                  <label for="surrealdb_pass" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                    Password {hasExistingPassword && !passwordChanged ? '(configured)' : ''}
                  </label>
                  <input
                    id="surrealdb_pass"
                    type="password"
                    bind:value={settings.surrealdb_pass}
                    on:input={onPasswordInput}
                    placeholder={hasExistingPassword ? 'Leave empty to keep existing' : 'Enter password'}
                    class="emittiv-input"
                  />
                </div>
              </div>

              <!-- Reconnect Database Button -->
              <div class="flex items-center" style="gap: 12px; margin-top: 4px;">
                <button
                  type="button"
                  on:click={testConnection}
                  disabled={isTesting}
                  class="emittiv-btn emittiv-btn--sm emittiv-btn--dark"
                  style="gap: 6px;"
                >
                  {#if isTesting}
                    <div class="emittiv-spinner" style="border-color: var(--emittiv-light); border-top-color: transparent;"></div>
                    <span>Reconnecting...</span>
                  {:else}
                    <svg style="width: 14px; height: 14px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                    <span>Reconnect</span>
                  {/if}
                </button>
                {#if testMessage}
                  <span class="text-xs {testMessage.includes('failed') ? 'text-red-400' : 'text-green-400'}">
                    {testMessage}
                  </span>
                {/if}
              </div>
            </div>
          </div>

          <!-- Staff Information Section -->
          <div>
            <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 6px;">Staff Information</h3>
            <p class="text-emittiv-light" style="font-size: 11px; margin-bottom: 12px;">Used for RFP generation and signatures</p>
            <div style="display: flex; flex-direction: column; gap: 12px;">
              <div class="grid grid-cols-2" style="gap: 12px;">
                <div>
                  <label for="staff_name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                    Full Name
                  </label>
                  <input
                    id="staff_name"
                    type="text"
                    bind:value={settings.staff_name}
                    placeholder="John Smith"
                    class="emittiv-input"
                  />
                </div>
                
                <div>
                  <label for="staff_position" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                    Position
                  </label>
                  <input
                    id="staff_position"
                    type="text"
                    bind:value={settings.staff_position}
                    placeholder="Senior Designer"
                    class="emittiv-input"
                  />
                </div>
              </div>
              
              <div class="grid grid-cols-2" style="gap: 12px;">
                <div>
                  <label for="staff_email" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                    Email
                  </label>
                  <input
                    id="staff_email"
                    type="email"
                    bind:value={settings.staff_email}
                    placeholder="john@emittiv.com"
                    class="emittiv-input"
                  />
                </div>
                
                <div>
                  <label for="staff_phone" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                    Phone
                  </label>
                  <input
                    id="staff_phone"
                    type="tel"
                    bind:value={settings.staff_phone}
                    placeholder="+971501234567"
                    class="emittiv-input"
                  />
                </div>
              </div>
            </div>
          </div>

          <!-- Project Folder Path Section -->
          <div>
            <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 6px;">Project Folder Path</h3>
            <p class="text-emittiv-light" style="font-size: 11px; margin-bottom: 12px;">Root folder where project files are stored</p>
            <div class="flex" style="gap: 12px;">
              <input
                id="project_folder_path"
                type="text"
                bind:value={settings.project_folder_path}
                placeholder="/Users/username/Projects"
                class="emittiv-input flex-1"
              />
              <button
                type="button"
                on:click={selectFolder}
                class="emittiv-btn emittiv-btn--md emittiv-btn--dark"
              >
                Browse
              </button>
            </div>
            <!-- Folder Sync Button -->
            <button
              type="button"
              on:click={() => showFolderSyncModal = true}
              class="emittiv-btn emittiv-btn--md emittiv-btn--dark"
              style="margin-top: 12px; width: 100%; gap: 8px;"
              style:opacity={!settings.project_folder_path ? '0.5' : '1'}
              style:cursor={!settings.project_folder_path ? 'not-allowed' : 'pointer'}
              disabled={!settings.project_folder_path}
            >
              <svg style="width: 14px; height: 14px; flex-shrink: 0;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              Check Folder Consistency
            </button>
          </div>

          <!-- Developer Options Section -->
          <div>
            <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 6px;">Developer Options</h3>
            <p class="text-emittiv-light" style="font-size: 11px; margin-bottom: 12px;">Advanced settings for debugging and development</p>

            <div style="display: flex; flex-direction: column; gap: 12px;">
              <!-- Log Level -->
              <div>
                <label class="text-emittiv-white" style="font-size: 12px; display: block; margin-bottom: 4px;">Log Level</label>
                <select
                  bind:value={settings.log_level}
                  on:change={handleLogLevelChange}
                  class="emittiv-select"
                  style="width: 160px;"
                >
                  <option value="off">Off</option>
                  <option value="info">Info (Default)</option>
                  <option value="debug">Debug</option>
                  <option value="trace">Trace</option>
                </select>
                <p class="text-emittiv-light" style="font-size: 11px; margin-top: 4px;">
                  Controls log verbosity. Changes take effect immediately. Stream logs with: curl -N http://localhost:3100/api/logs/stream
                </p>
              </div>

              <!-- Dev Mode checkbox -->
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={settings.dev_mode}
                  class="emittiv-checkbox"
                />
                <div>
                  <span class="text-emittiv-white" style="font-size: 12px;">Updater Verbose Mode</span>
                  <p class="text-emittiv-light" style="font-size: 11px;">Enable verbose logging for auto-updater. Logs written to /tmp/e-fees-updater.log</p>
                </div>
              </label>
            </div>
          </div>

          <!-- Save Message -->
          {#if saveMessage}
            <div class="emittiv-alert emittiv-alert--sm {saveMessage.startsWith('Error') ? 'emittiv-alert--error' : 'emittiv-alert--success'}">
              {saveMessage}
            </div>
          {/if}

          <!-- Action Buttons -->
          <div class="flex justify-end border-t border-emittiv-dark" style="gap: 12px; padding-top: 16px; margin-top: 8px;">
            <button
              type="button"
              on:click={closeModal}
              class="emittiv-btn emittiv-btn--sm emittiv-btn--secondary"
              disabled={isSaving}
            >
              Cancel
            </button>
            <button
              type="submit"
              class="emittiv-btn emittiv-btn--sm emittiv-btn--primary"
              style="gap: 4px;"
              disabled={isSaving}
            >
              {#if isSaving}
                <div class="emittiv-spinner-sm"></div>
                <span>Saving...</span>
              {:else}
                <span>Save Settings</span>
              {/if}
            </button>
          </div>
        </form>
      {/if}
    </div>
  </div>
{/if}

<!-- Folder Sync Modal (rendered outside main modal for proper z-index) -->
<FolderSyncModal
  isOpen={showFolderSyncModal}
  onclose={() => showFolderSyncModal = false}
/>