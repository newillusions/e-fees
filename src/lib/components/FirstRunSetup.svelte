<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { connectionStore } from '../stores';
  import { checkDbConnection, saveSettings, getSettings } from '../api';
  import { fade, slide } from 'svelte/transition';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  
  // Setup steps
  let currentStep = 1;
  const totalSteps = 3;
  
  // Form data
  let dbConfig = {
    url: 'ws://10.0.1.17:8000',
    namespace: 'emittiv',
    database: 'projects',
    username: '',
    password: ''
  };
  
  let staffInfo = {
    name: '',
    email: '',
    phone: '',
    position: ''
  };
  
  let projectPath = '/Volumes/base/mms/DevTest/';
  
  // State
  let isTestingConnection = false;
  let connectionTestResult: 'pending' | 'success' | 'error' = 'pending';
  let connectionTestMessage = '';
  let isSaving = false;
  
  onMount(async () => {
    // Check if this is first run by testing connection
    const isConnected = await checkDbConnection();
    if (!isConnected) {
      // Check if we have placeholder config
      const settings = await getSettings();
      if (settings?.surrealdb_user === 'placeholder' || !settings?.surrealdb_user) {
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
      
      // Test connection
      const isConnected = await checkDbConnection();
      
      if (isConnected) {
        connectionTestResult = 'success';
        connectionTestMessage = 'Connection successful!';
      } else {
        connectionTestResult = 'error';
        connectionTestMessage = 'Failed to connect. Please check your settings.';
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
      
      dispatch('complete');
      isOpen = false;
      
      // Reload the page to apply new settings
      window.location.reload();
    } catch (error) {
      console.error('Failed to save configuration:', error);
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
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4" style="background: rgba(0, 0, 0, 0.8);">
    <div 
      class="relative w-full max-w-2xl max-h-[90vh] overflow-hidden rounded-lg shadow-2xl"
      style="background: var(--emittiv-darker); border: 1px solid var(--emittiv-dark);"
      transition:fade={{ duration: 200 }}
    >
      <!-- Header -->
      <div class="px-6 py-4 border-b" style="border-color: var(--emittiv-dark);">
        <h2 class="text-xl font-bold" style="color: var(--emittiv-white);">
          Welcome to Fee Proposal Management
        </h2>
        <p class="text-sm mt-1" style="color: var(--emittiv-light);">
          Let's set up your application for first use
        </p>
      </div>
      
      <!-- Progress Indicator -->
      <div class="px-6 py-3 border-b" style="border-color: var(--emittiv-dark);">
        <div class="flex items-center justify-between">
          {#each Array(totalSteps) as _, i}
            <div class="flex items-center">
              <div 
                class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium transition-all"
                style="background: {i + 1 <= currentStep ? 'var(--emittiv-splash)' : 'var(--emittiv-dark)'}; 
                       color: {i + 1 <= currentStep ? 'var(--emittiv-black)' : 'var(--emittiv-light)'};"
              >
                {i + 1}
              </div>
              {#if i < totalSteps - 1}
                <div 
                  class="w-24 h-0.5 mx-2"
                  style="background: {i + 1 < currentStep ? 'var(--emittiv-splash)' : 'var(--emittiv-dark)'};"
                ></div>
              {/if}
            </div>
          {/each}
        </div>
        <div class="flex justify-between mt-2">
          <span class="text-xs" style="color: var(--emittiv-light);">Database</span>
          <span class="text-xs" style="color: var(--emittiv-light);">Staff Info</span>
          <span class="text-xs" style="color: var(--emittiv-light);">Projects</span>
        </div>
      </div>
      
      <!-- Content -->
      <div class="px-6 py-4 overflow-y-auto" style="max-height: 400px;">
        {#if currentStep === 1}
          <div transition:slide={{ duration: 300 }}>
            <h3 class="text-lg font-semibold mb-4" style="color: var(--emittiv-white);">
              Database Configuration
            </h3>
            
            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium mb-1" style="color: var(--emittiv-lighter);">
                  Database URL
                </label>
                <input
                  type="text"
                  bind:value={dbConfig.url}
                  placeholder="ws://localhost:8000"
                  class="w-full px-3 py-2 rounded"
                  style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                />
              </div>
              
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm font-medium mb-1" style="color: var(--emittiv-lighter);">
                    Namespace
                  </label>
                  <input
                    type="text"
                    bind:value={dbConfig.namespace}
                    placeholder="emittiv"
                    class="w-full px-3 py-2 rounded"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>
                
                <div>
                  <label class="block text-sm font-medium mb-1" style="color: var(--emittiv-lighter);">
                    Database
                  </label>
                  <input
                    type="text"
                    bind:value={dbConfig.database}
                    placeholder="projects"
                    class="w-full px-3 py-2 rounded"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>
              </div>
              
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label for="db-username" class="block text-sm font-medium mb-1" style="color: var(--emittiv-lighter);">
                    Username
                  </label>
                  <input
                    id="db-username"
                    type="text"
                    bind:value={dbConfig.username}
                    placeholder="username"
                    class="w-full px-3 py-2 rounded"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>
                
                <div>
                  <label for="db-password" class="block text-sm font-medium mb-1" style="color: var(--emittiv-lighter);">
                    Password
                  </label>
                  <input
                    id="db-password"
                    type="password"
                    bind:value={dbConfig.password}
                    placeholder="••••••••"
                    class="w-full px-3 py-2 rounded"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>
              </div>
              
              {#if connectionTestResult !== 'pending'}
                <div 
                  class="p-3 rounded text-sm"
                  style="background: {connectionTestResult === 'success' ? 'rgba(34, 197, 94, 0.1)' : 'rgba(239, 68, 68, 0.1)'}; 
                         color: {connectionTestResult === 'success' ? '#22c55e' : '#ef4444'};"
                >
                  {connectionTestMessage}
                </div>
              {/if}
            </div>
          </div>
        {:else if currentStep === 2}
          <div transition:slide={{ duration: 300 }}>
            <h3 class="text-lg font-semibold mb-4" style="color: var(--emittiv-white);">
              Staff Information
            </h3>
            
            <div class="space-y-4">
              <div>
                <label for="staff-name" class="block text-sm font-medium mb-1" style="color: var(--emittiv-lighter);">
                  Your Name
                </label>
                <input
                  id="staff-name"
                  type="text"
                  bind:value={staffInfo.name}
                  placeholder="John Doe"
                  class="w-full px-3 py-2 rounded"
                  style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                />
              </div>
              
              <div>
                <label for="staff-email" class="block text-sm font-medium mb-1" style="color: var(--emittiv-lighter);">
                  Email Address
                </label>
                <input
                  id="staff-email"
                  type="email"
                  bind:value={staffInfo.email}
                  placeholder="john.doe@company.com"
                  class="w-full px-3 py-2 rounded"
                  style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                />
              </div>
              
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label for="staff-phone" class="block text-sm font-medium mb-1" style="color: var(--emittiv-lighter);">
                    Phone Number
                  </label>
                  <input
                    id="staff-phone"
                    type="tel"
                    bind:value={staffInfo.phone}
                    placeholder="+971 50 123 4567"
                    class="w-full px-3 py-2 rounded"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>
                
                <div>
                  <label for="staff-position" class="block text-sm font-medium mb-1" style="color: var(--emittiv-lighter);">
                    Position
                  </label>
                  <input
                    id="staff-position"
                    type="text"
                    bind:value={staffInfo.position}
                    placeholder="Project Manager"
                    class="w-full px-3 py-2 rounded"
                    style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                  />
                </div>
              </div>
            </div>
          </div>
        {:else if currentStep === 3}
          <div transition:slide={{ duration: 300 }}>
            <h3 class="text-lg font-semibold mb-4" style="color: var(--emittiv-white);">
              Project Configuration
            </h3>
            
            <div class="space-y-4">
              <div>
                <label for="project-path" class="block text-sm font-medium mb-1" style="color: var(--emittiv-lighter);">
                  Project Folder Path
                </label>
                <input
                  id="project-path"
                  type="text"
                  bind:value={projectPath}
                  placeholder="/path/to/projects/"
                  class="w-full px-3 py-2 rounded"
                  style="background: var(--emittiv-dark); color: var(--emittiv-white); border: 1px solid var(--emittiv-dark);"
                />
                <p class="text-xs mt-1" style="color: var(--emittiv-light);">
                  This is where project folders will be created and managed
                </p>
              </div>
              
              <div class="p-4 rounded" style="background: var(--emittiv-dark);">
                <h4 class="text-sm font-semibold mb-2" style="color: var(--emittiv-white);">
                  Setup Complete!
                </h4>
                <p class="text-sm" style="color: var(--emittiv-light);">
                  Your Fee Proposal Management system is ready to use. Click "Finish Setup" to save your configuration and start using the application.
                </p>
              </div>
            </div>
          </div>
        {/if}
      </div>
      
      <!-- Footer -->
      <div class="px-6 py-4 border-t flex justify-between" style="border-color: var(--emittiv-dark);">
        <button
          onclick={previousStep}
          disabled={currentStep === 1}
          class="px-4 py-2 rounded text-sm font-medium transition-colors disabled:opacity-50"
          style="background: var(--emittiv-dark); color: var(--emittiv-light);"
        >
          Previous
        </button>
        
        <button
          onclick={saveAndContinue}
          disabled={isTestingConnection || isSaving}
          class="px-4 py-2 rounded text-sm font-medium transition-colors"
          style="background: var(--emittiv-splash); color: var(--emittiv-black);"
        >
          {#if currentStep === 1}
            {isTestingConnection ? 'Testing...' : 'Test Connection & Continue'}
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