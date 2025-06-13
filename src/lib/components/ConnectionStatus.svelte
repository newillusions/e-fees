<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { connectionStore } from '../stores';
  import { getConnectionStatus, getDbInfo } from '../api';
  
  let checkInterval: number;
  let showDebugInfo = false;
  
  async function checkConnection() {
    try {
      const status = await getConnectionStatus();
      connectionStore.set({
        isConnected: status.is_connected,
        status: status.is_connected ? 'Connected' : 'Disconnected',
        lastChecked: status.last_check ? new Date(status.last_check) : undefined,
        errorMessage: status.error_message || undefined
      });
    } catch (error) {
      console.error('Failed to check connection:', error);
      connectionStore.set({
        isConnected: false,
        status: 'Connection Error',
        lastChecked: new Date(),
        errorMessage: error?.toString() || 'Unknown error'
      });
    }
  }
  
  onMount(() => {
    // Initial check
    checkConnection();
    
    // Check connection every 30 seconds
    checkInterval = setInterval(checkConnection, 30000);
  });
  
  onDestroy(() => {
    if (checkInterval) {
      clearInterval(checkInterval);
    }
  });

  async function logDebugInfo() {
    const dbInfo = await getDbInfo();
    console.log('=== SurrealDB Connection Debug Info ===');
    console.log(JSON.stringify(dbInfo, null, 2));
    console.log('=======================================');
    showDebugInfo = !showDebugInfo;
  }
</script>

<div class="flex items-center space-x-2">
  <div class="relative">
    <div 
      class="w-2 h-2 rounded-full transition-smooth"
      class:status-connected={$connectionStore.isConnected}
      class:status-disconnected={!$connectionStore.isConnected}
    ></div>
    {#if $connectionStore.isConnected}
      <div class="absolute inset-0 w-2 h-2 rounded-full status-connected animate-pulse"></div>
    {/if}
  </div>
  <div class="flex flex-col">
    <span class="text-sm" style="color: var(--emittiv-light);">
      Database: 
      <span 
        class="transition-smooth font-medium"
        class:status-connected={$connectionStore.isConnected}
        class:status-disconnected={!$connectionStore.isConnected}
      >
        {$connectionStore.status}
      </span>
    </span>
    {#if $connectionStore.lastChecked}
      <span class="text-xs" style="color: var(--emittiv-light); opacity: 0.7;">
        Last check: {$connectionStore.lastChecked.toLocaleTimeString()}
      </span>
    {/if}
    {#if $connectionStore.errorMessage && !$connectionStore.isConnected}
      <span class="text-xs status-disconnected" title={$connectionStore.errorMessage}>
        {$connectionStore.errorMessage.substring(0, 30)}{$connectionStore.errorMessage.length > 30 ? '...' : ''}
      </span>
    {/if}
  </div>
  
  <!-- Debug button (only shown when disconnected) -->
  {#if !$connectionStore.isConnected}
    <button 
      on:click={logDebugInfo}
      class="text-xs px-2 py-1 rounded transition-smooth"
      style="background-color: var(--emittiv-dark); color: var(--emittiv-light);"
      title="Log detailed connection info to console"
    >
      Debug
    </button>
  {/if}
</div>