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
    checkInterval = setInterval(checkConnection, 30000) as any;
  });
  
  onDestroy(() => {
    if (checkInterval) {
      clearInterval(checkInterval);
    }
  });

  async function logDebugInfo() {
    const dbInfo = await getDbInfo();
    showDebugInfo = !showDebugInfo;
  }

</script>

<div class="flex items-center px-6 py-2">
  <!-- Left side: LED and Status Text aligned with nav items -->
  <div class="flex items-center space-x-3">
    <!-- LED Indicator -->
    <div class="relative flex items-center justify-center w-5 h-5">
      <!-- Base LED centered in the icon space -->
      <div 
        class="w-1 h-1 rounded-full transition-all duration-300 relative z-10"
        class:led-connected={$connectionStore.isConnected}
        class:led-disconnected={!$connectionStore.isConnected}
      ></div>
      <!-- Glow effect for connected state -->
      {#if $connectionStore.isConnected}
        <div class="absolute w-1.5 h-1.5 rounded-full led-glow-connected animate-pulse"></div>
      {/if}
    </div>
    
    <!-- Status Text -->
    <div class="flex flex-col">
      <span class="text-xs font-medium" style="color: var(--emittiv-light);">
        {$connectionStore.status}
      </span>
      {#if $connectionStore.lastChecked}
        <span class="text-xs" style="color: var(--emittiv-light); opacity: 0.6;">
          {$connectionStore.lastChecked.toLocaleTimeString()}
        </span>
      {/if}
      {#if $connectionStore.errorMessage && !$connectionStore.isConnected}
        <span class="text-xs" style="color: var(--emittiv-light); opacity: 0.7;" title={$connectionStore.errorMessage}>
          {$connectionStore.errorMessage.substring(0, 25)}{$connectionStore.errorMessage.length > 25 ? '...' : ''}
        </span>
      {/if}
    </div>
    
  </div>
</div>

<style>
  /* LED Base Styles */
  .led-connected {
    background-color: var(--emittiv-splash); /* Orange #f90 */
    box-shadow: 0 0 2px var(--emittiv-splash);
  }
  
  .led-disconnected {
    background-color: #ef4444; /* Red */
    box-shadow: 0 0 2px #ef4444;
  }
  
  /* Glow effect for connected state */
  .led-glow-connected {
    background-color: var(--emittiv-splash);
    filter: blur(2px);
    opacity: 0.5;
  }
  
  /* Override the pulse animation for a smoother glow */
  @keyframes pulse {
    0%, 100% {
      opacity: 0.5;
      transform: scale(1);
    }
    50% {
      opacity: 0.1;
      transform: scale(1.1);
    }
  }
</style>