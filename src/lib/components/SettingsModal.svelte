<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { settingsStore, settingsLoading, settingsError, settingsActions, type AppSettings } from '$lib/stores/settings';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  
  // Settings form data - initialize with defaults
  let settings: AppSettings = {
    surrealdb_url: '',
    surrealdb_ns: '',
    surrealdb_db: '',
    surrealdb_user: '',
    surrealdb_pass: '',
    staff_name: '',
    staff_email: '',
    staff_phone: '',
    staff_position: '',
    project_folder_path: ''
  };
  
  // Loading states
  let isSaving = false;
  let saveMessage = '';
  
  // Load settings when modal opens
  $: if (isOpen) {
    loadSettings();
  }
  
  // Update form when store changes
  $: if ($settingsStore) {
    settings = {
      surrealdb_url: $settingsStore.surrealdb_url || '',
      surrealdb_ns: $settingsStore.surrealdb_ns || '',
      surrealdb_db: $settingsStore.surrealdb_db || '',
      surrealdb_user: $settingsStore.surrealdb_user || '',
      surrealdb_pass: $settingsStore.surrealdb_pass || '',
      staff_name: $settingsStore.staff_name || '',
      staff_email: $settingsStore.staff_email || '',
      staff_phone: $settingsStore.staff_phone || '',
      staff_position: $settingsStore.staff_position || '',
      project_folder_path: $settingsStore.project_folder_path || ''
    };
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
      
      // Save settings
      await settingsActions.save(settings);
      saveMessage = 'Settings saved successfully! Please restart the application for changes to take effect.';
      
      // Auto-close after 2 seconds
      setTimeout(() => {
        closeModal();
      }, 2000);
      
    } catch (error) {
      saveMessage = `Error: ${error.message || error}`;
    } finally {
      isSaving = false;
    }
  }
  
  function closeModal() {
    isOpen = false;
    saveMessage = '';
    dispatch('close');
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
      console.error('Failed to select folder:', error);
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Modal Backdrop -->
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4"
    on:click={closeModal}
    on:keydown={(e) => e.key === 'Escape' && closeModal()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="settings-title"
    tabindex="-1"
  >
    <!-- Modal Content -->
    <div 
      class="bg-emittiv-darker border border-emittiv-dark rounded w-full max-h-[90vh] overflow-y-auto"
      style="padding: 24px; max-width: 700px;"
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
          <div class="w-6 h-6 border-2 border-emittiv-splash border-t-transparent rounded-full animate-spin"></div>
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
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
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
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
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
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
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
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
                  />
                </div>
                
                <div>
                  <label for="surrealdb_pass" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                    Password
                  </label>
                  <input
                    id="surrealdb_pass"
                    type="password"
                    bind:value={settings.surrealdb_pass}
                    placeholder="••••••••"
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
                  />
                </div>
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
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
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
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
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
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
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
                    class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                    style="padding: 8px 12px; font-size: 12px; height: 32px;"
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
                class="flex-1 bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all"
                style="padding: 8px 12px; font-size: 12px; height: 32px;"
              />
              <button
                type="button"
                on:click={selectFolder}
                class="bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-light hover:text-emittiv-white hover:border-emittiv-splash transition-all"
                style="padding: 8px 16px; font-size: 12px; height: 32px;"
              >
                Browse
              </button>
            </div>
          </div>

          <!-- Save Message -->
          {#if saveMessage}
            <div class="rounded-lg {saveMessage.startsWith('Error') ? 'bg-red-900/20 border border-red-500/30 text-red-300' : 'bg-green-900/20 border border-green-500/30 text-green-300'}" style="padding: 8px; font-size: 11px;">
              {saveMessage}
            </div>
          {/if}

          <!-- Action Buttons -->
          <div class="flex justify-end border-t border-emittiv-dark" style="gap: 12px; padding-top: 16px; margin-top: 8px;">
            <button
              type="button"
              on:click={closeModal}
              class="border border-emittiv-dark rounded text-emittiv-light hover:text-emittiv-white hover:border-emittiv-light transition-all"
              style="padding: 6px 12px; font-size: 12px; height: 28px;"
              disabled={isSaving}
            >
              Cancel
            </button>
            <button
              type="submit"
              class="bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
              style="padding: 6px 12px; font-size: 12px; height: 28px; gap: 4px;"
              disabled={isSaving}
            >
              {#if isSaving}
                <div class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin" style="width: 12px; height: 12px;"></div>
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