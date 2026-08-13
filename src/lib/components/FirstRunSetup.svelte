<script lang="ts">
  import { onMount } from 'svelte';
  import { connectionStore } from '../stores';
  import { checkDbConnection, saveSettings, getSettings, reloadDatabaseConfig } from '../api';
  import { fade, slide } from 'svelte/transition';
  import { logApiError } from '$lib/services/logger';

  let {
    isOpen = $bindable(false),
    oncomplete
  }: {
    isOpen?: boolean;
    oncomplete?: () => void;
  } = $props();

  // Setup steps
  let currentStep = $state(1);
  const totalSteps = 3;

  // Form data
  let dbConfig = $state({
    url: '',
    namespace: '',
    database: '',
    username: '',
    password: ''
  });

  let staffInfo = $state({
    name: '',
    email: '',
    phone: '',
    position: ''
  });

  let projectPath = $state('');

  // State
  let isTestingConnection = $state(false);
  let connectionTestResult: 'pending' | 'success' | 'error' = $state('pending');
  let connectionTestMessage = $state('');
  let isSaving = $state(false);

  onMount(async () => {
    // Check if this is first run by testing connection
    const isConnected = await checkDbConnection();
    if (!isConnected) {
      // Check if we have valid configuration
      // Settings now return has_password boolean instead of actual password
      const settings = await getSettings();
      const hasValidConfig =
        settings?.surrealdb_user &&
        settings.surrealdb_user !== 'placeholder' &&
        settings.has_password;
      if (!hasValidConfig) {
        isOpen = true;
      }
    }
  });

  async function testConnection() {
    isTestingConnection = true;
    connectionTestResult = 'pending';
    connectionTestMessage = 'Testing connection...';

    try {
      // Save temporary config
      await saveSettings({
        surrealdb_url: dbConfig.url,
        surrealdb_ns: dbConfig.namespace,
        surrealdb_db: dbConfig.database,
        surrealdb_user: dbConfig.username,
        surrealdb_pass: dbConfig.password,
        staff_name: staffInfo.name || 'User',
        staff_email: staffInfo.email || 'user@example.com',
        staff_phone: staffInfo.phone || '+000 00 000 0000',
        staff_position: staffInfo.position || 'User',
        project_folder_path: projectPath
      });

      // check_db_connection only reports whether a DB client already exists -
      // it never attempts a connection. reconnect_database re-initializes
      // using the manager's EXISTING config, which is stale until the app
      // restarts (it never re-reads the settings we just saved). Use
      // reload_database_config instead - it re-reads the settings file into
      // a fresh DatabaseConfig via DatabaseManager::reconfigure() before
      // reconnecting, mirroring the app's own startup config-loading path.
      let reloadMessage = '';
      try {
        reloadMessage = await reloadDatabaseConfig();
      } catch (reloadError) {
        connectionTestResult = 'error';
        connectionTestMessage = `${reloadError}`;
        return;
      }

      const isConnected = await checkDbConnection();

      if (isConnected) {
        connectionTestResult = 'success';
        connectionTestMessage = 'Connection successful!';
      } else {
        // reload_database_config does not throw when the inner reconnect
        // fails - it resolves with the real reason embedded in the message
        // (e.g. "...but connection failed: Invalid URL: http://"). Prefer
        // that over the generic fallback when present.
        connectionTestResult = 'error';
        connectionTestMessage = reloadMessage || 'Failed to connect. Please check your settings.';
      }
    } catch (error) {
      connectionTestResult = 'error';
      connectionTestMessage = `Connection error: ${error}`;
    } finally {
      isTestingConnection = false;
    }
  }

  async function saveAndContinue() {
    if (currentStep === 1) {
      // Test connection before proceeding
      await testConnection();
      if (connectionTestResult === 'success') {
        currentStep = 2;
      }
    } else if (currentStep === 2) {
      currentStep = 3;
    } else if (currentStep === 3) {
      await saveConfiguration();
    }
  }

  async function saveConfiguration() {
    isSaving = true;

    try {
      await saveSettings({
        surrealdb_url: dbConfig.url,
        surrealdb_ns: dbConfig.namespace,
        surrealdb_db: dbConfig.database,
        surrealdb_user: dbConfig.username,
        surrealdb_pass: dbConfig.password,
        staff_name: staffInfo.name,
        staff_email: staffInfo.email,
        staff_phone: staffInfo.phone,
        staff_position: staffInfo.position,
        project_folder_path: projectPath
      });

      oncomplete?.();
      isOpen = false;

      // Reload the page to apply new settings
      window.location.reload();
    } catch (error) {
      logApiError('save configuration', error as Error);
    } finally {
      isSaving = false;
    }
  }

  function previousStep() {
    if (currentStep > 1) {
      currentStep--;
    }
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-2"
    style="background: rgba(0, 0, 0, 0.8);"
  >
    <div
      class="relative w-full overflow-hidden rounded-lg shadow-2xl flex flex-col"
      style="background: var(--emittiv-darker); border: 1px solid var(--emittiv-dark); max-width: 500px; max-height: 95vh;"
      transition:fade={{ duration: 200 }}
    >
      <!-- Header -->
      <div class="p-3 pb-2 border-b flex-shrink-0" style="border-color: var(--emittiv-dark);">
        <h2 class="text-lg font-bold" style="color: var(--emittiv-white);">Welcome to E-Fees</h2>
        <p class="text-xs mt-1" style="color: var(--emittiv-light);">
          Let's set up your application for first use
        </p>
      </div>

      <!-- Progress Indicator -->
      <div class="p-3 pb-2 border-b flex-shrink-0" style="border-color: var(--emittiv-dark);">
        <div class="flex items-center justify-between">
          {#each Array(totalSteps) as _, i}
            <div class="flex items-center">
              <div
                class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium transition-all"
                style="background: {i + 1 <= currentStep
                  ? 'var(--emittiv-splash)'
                  : 'var(--emittiv-dark)'}; 
                       color: {i + 1 <= currentStep
                  ? 'var(--emittiv-black)'
                  : 'var(--emittiv-light)'};"
              >
                {i + 1}
              </div>
              {#if i < totalSteps - 1}
                <div
                  class="w-16 h-0.5 mx-1"
                  style="background: {i + 1 < currentStep
                    ? 'var(--emittiv-splash)'
                    : 'var(--emittiv-dark)'};"
                ></div>
              {/if}
            </div>
          {/each}
        </div>
        <div class="flex justify-between mt-1">
          <span class="text-xs" style="color: var(--emittiv-light);">Database</span>
          <span class="text-xs" style="color: var(--emittiv-light);">Staff Info</span>
          <span class="text-xs" style="color: var(--emittiv-light);">Projects</span>
        </div>
      </div>

      <!-- Content -->
      <div class="overflow-y-auto flex-grow p-3" style="min-height: 0;">
        {#if currentStep === 1}
          <div transition:slide={{ duration: 300 }}>
            <h3 class="text-base font-semibold mb-3" style="color: var(--emittiv-white);">
              Database Configuration
            </h3>

            <div class="space-y-3">
              <div>
                <label
                  for="db-url"
                  class="block text-xs font-medium mb-1"
                  style="color: var(--emittiv-lighter);"
                >
                  Database URL
                </label>
                <input
                  id="db-url"
                  type="text"
                  bind:value={dbConfig.url}
                  placeholder="e.g., ws://localhost:8000"
                  class="emittiv-input"
                />
              </div>

              <div class="grid grid-cols-2 gap-3">
                <div>
                  <label
                    for="db-namespace"
                    class="block text-xs font-medium mb-1"
                    style="color: var(--emittiv-lighter);"
                  >
                    Namespace
                  </label>
                  <input
                    id="db-namespace"
                    type="text"
                    bind:value={dbConfig.namespace}
                    placeholder="your-namespace"
                    class="w-full px-2 py-1.5 rounded text-sm"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>

                <div>
                  <label
                    for="db-database"
                    class="block text-xs font-medium mb-1"
                    style="color: var(--emittiv-lighter);"
                  >
                    Database
                  </label>
                  <input
                    id="db-database"
                    type="text"
                    bind:value={dbConfig.database}
                    placeholder="your-database"
                    class="w-full px-2 py-1.5 rounded text-sm"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>
              </div>

              <div class="grid grid-cols-2 gap-3">
                <div>
                  <label
                    for="db-username"
                    class="block text-xs font-medium mb-1"
                    style="color: var(--emittiv-lighter);"
                  >
                    Username
                  </label>
                  <input
                    id="db-username"
                    type="text"
                    bind:value={dbConfig.username}
                    placeholder="username"
                    class="w-full px-2 py-1.5 rounded text-sm"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>

                <div>
                  <label
                    for="db-password"
                    class="block text-xs font-medium mb-1"
                    style="color: var(--emittiv-lighter);"
                  >
                    Password
                  </label>
                  <input
                    id="db-password"
                    type="password"
                    bind:value={dbConfig.password}
                    placeholder="••••••••"
                    class="w-full px-2 py-1.5 rounded text-sm"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>
              </div>

              {#if connectionTestResult !== 'pending'}
                <div
                  class="p-3 rounded text-sm"
                  style="background: {connectionTestResult === 'success'
                    ? 'rgba(34, 197, 94, 0.1)'
                    : 'rgba(239, 68, 68, 0.1)'}; 
                         color: {connectionTestResult === 'success' ? '#22c55e' : '#ef4444'};"
                >
                  {connectionTestMessage}
                </div>
              {/if}
            </div>
          </div>
        {:else if currentStep === 2}
          <div transition:slide={{ duration: 300 }}>
            <h3 class="text-base font-semibold mb-3" style="color: var(--emittiv-white);">
              Staff Information
            </h3>

            <div class="space-y-3">
              <div>
                <label
                  for="staff-name"
                  class="block text-xs font-medium mb-1"
                  style="color: var(--emittiv-lighter);"
                >
                  Your Name
                </label>
                <input
                  id="staff-name"
                  type="text"
                  bind:value={staffInfo.name}
                  placeholder="John Doe"
                  class="emittiv-input"
                />
              </div>

              <div>
                <label
                  for="staff-email"
                  class="block text-xs font-medium mb-1"
                  style="color: var(--emittiv-lighter);"
                >
                  Email Address
                </label>
                <input
                  id="staff-email"
                  type="email"
                  bind:value={staffInfo.email}
                  placeholder="john.doe@company.com"
                  class="emittiv-input"
                />
              </div>

              <div class="grid grid-cols-2 gap-3">
                <div>
                  <label
                    for="staff-phone"
                    class="block text-xs font-medium mb-1"
                    style="color: var(--emittiv-lighter);"
                  >
                    Phone Number
                  </label>
                  <input
                    id="staff-phone"
                    type="tel"
                    bind:value={staffInfo.phone}
                    placeholder="+971 50 123 4567"
                    class="w-full px-2 py-1.5 rounded text-sm"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>

                <div>
                  <label
                    for="staff-position"
                    class="block text-xs font-medium mb-1"
                    style="color: var(--emittiv-lighter);"
                  >
                    Position
                  </label>
                  <input
                    id="staff-position"
                    type="text"
                    bind:value={staffInfo.position}
                    placeholder="Project Manager"
                    class="w-full px-2 py-1.5 rounded text-sm"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>
              </div>
            </div>
          </div>
        {:else if currentStep === 3}
          <div transition:slide={{ duration: 300 }}>
            <h3 class="text-base font-semibold mb-3" style="color: var(--emittiv-white);">
              Project Configuration
            </h3>

            <div class="space-y-3">
              <div>
                <label
                  for="project-path"
                  class="block text-xs font-medium mb-1"
                  style="color: var(--emittiv-lighter);"
                >
                  Project Folder Path
                </label>
                <input
                  id="project-path"
                  type="text"
                  bind:value={projectPath}
                  placeholder="e.g., /Users/username/Projects/"
                  class="emittiv-input"
                />
                <p class="text-xs mt-1" style="color: var(--emittiv-light);">
                  This is where project folders will be created and managed
                </p>
              </div>

              <div class="p-3 rounded" style="background: var(--emittiv-dark);">
                <h4 class="text-sm font-semibold mb-1" style="color: var(--emittiv-white);">
                  Setup Complete!
                </h4>
                <p class="text-xs" style="color: var(--emittiv-light);">
                  Your E-Fees workspace is ready.
                </p>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div
        class="p-3 border-t flex justify-between flex-shrink-0"
        style="border-color: var(--emittiv-dark);"
      >
        <button
          onclick={previousStep}
          disabled={currentStep === 1}
          class="px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-50"
          style="background: var(--emittiv-dark); color: var(--emittiv-light);"
        >
          Previous
        </button>

        <button
          onclick={saveAndContinue}
          disabled={isTestingConnection || isSaving}
          class="px-3 py-1.5 rounded text-sm font-medium transition-colors"
          style="background: var(--emittiv-splash); color: var(--emittiv-black);"
        >
          {#if currentStep === 1}
            {isTestingConnection ? 'Testing...' : 'Test & Continue'}
          {:else if currentStep === 2}
            Continue
          {:else}
            {isSaving ? 'Saving...' : 'Finish Setup'}
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  input:focus {
    outline: none;
    border-color: var(--emittiv-splash) !important;
  }

  button:not(:disabled):hover {
    opacity: 0.9;
  }
</style>
