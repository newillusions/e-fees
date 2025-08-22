<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { connectionStore } from '../stores';
  import { getConnectionStatus, getDbInfo } from '../api';
  import { createComponentLogger } from '../services/logger';
  
  const logger = createComponentLogger('ConnectionStatus');
  
  let checkInterval: number;
  let showDebugInfo = false;
  
  async function checkConnection(retries = 0) {
    try {
      const status = await getConnectionStatus();
      const wasConnected = $connectionStore.isConnected;
      const nowConnected = status.is_connected;
      
      connectionStore.set({
        isConnected: status.is_connected,
        status: status.is_connected ? 'Connected' : 'Disconnected',
        lastChecked: status.last_check ? new Date(status.last_check) : undefined,
        errorMessage: status.error_message || undefined
      });
      
      // If we just connected for the first time, load data
      if (!wasConnected && nowConnected) {
        await logger.info('Database connected via status check - loading data', {
          action: 'loadDataOnConnection'
        });
        const { loadAllData } = await import('../stores');
        await loadAllData();
      }
      
      // If still not connected and we have retries left, try again
      if (!status.is_connected && retries < 5) {
        await logger.debug(`Connection check ${retries + 1}/5 failed, retrying in ${2 + retries}s`, {
          action: 'retryConnection',
          attempt: retries + 1,
          maxAttempts: 5,
          retryDelay: 2 + retries
        });
        setTimeout(() => checkConnection(retries + 1), (2 + retries) * 1000);
      }
      
      // FALLBACK: If connection status check fails but this is first time, try loading data anyway
      // (backend might be working even if status check fails due to timing issues)
      if (!status.is_connected && retries === 0 && !wasConnected) {
        await logger.info('Connection status shows disconnected, attempting data load as fallback', {
          action: 'fallbackDataLoad',
          reason: 'status_check_failed'
        });
        try {
          const { loadAllData } = await import('../stores');
          await loadAllData();
          await logger.info('Fallback data loading succeeded - backend is actually working', {
            action: 'fallbackDataLoad',
            result: 'success'
          });
          // Update connection status to reflect reality
          connectionStore.set({
            isConnected: true,
            status: 'Connected',
            lastChecked: new Date(),
            errorMessage: undefined
          });
        } catch (dataError) {
          await logger.warn('Fallback data loading failed', {
            action: 'fallbackDataLoad',
            attempt: 'first'
          });
        }
      }
    } catch (error) {
      await logger.error('Failed to check database connection', {
        action: 'checkConnection',
        attempt: retries + 1
      }, error as Error);
      
      // On initial startup, retry a few times before showing error
      if (retries < 5) {
        await logger.debug(`Connection error ${retries + 1}/5, retrying in ${2 + retries}s`, {
          action: 'retryAfterError',
          attempt: retries + 1,
          maxAttempts: 5,
          retryDelay: 2 + retries
        });
        connectionStore.set({
          isConnected: false,
          status: 'Connecting...',
          lastChecked: new Date(),
          errorMessage: undefined
        });
        setTimeout(() => checkConnection(retries + 1), (2 + retries) * 1000);
      } else {
        // FALLBACK: Even if connection checks completely fail, try loading data
        await logger.warn('All connection checks failed, attempting final fallback data load', {
          action: 'finalFallbackDataLoad',
          totalRetries: retries,
          reason: 'all_checks_failed'
        });
        try {
          const { loadAllData } = await import('../stores');
          await loadAllData();
          await logger.info('Final fallback data loading succeeded despite connection check failures', {
            action: 'finalFallbackDataLoad',
            result: 'success',
            totalRetries: retries
          });
          connectionStore.set({
            isConnected: true,
            status: 'Connected',
            lastChecked: new Date(),
            errorMessage: undefined
          });
        } catch (dataError) {
          await logger.error('Final fallback data loading failed - all connection attempts exhausted', {
            action: 'finalFallbackDataLoad',
            attempt: 'final',
            totalRetries: retries
          }, dataError as Error);
          connectionStore.set({
            isConnected: false,
            status: 'Connection Error',
            lastChecked: new Date(),
            errorMessage: error?.toString() || 'Unknown error'
          });
        }
      }
    }
  }
  
  onMount(() => {
    // Delay initial check to allow backend startup time
    setTimeout(() => {
      checkConnection();
    }, 2000); // 2 second delay for backend initialization
    
    // Check connection every 30 seconds (without retries on periodic checks)
    checkInterval = setInterval(() => checkConnection(0), 30000) as any;
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
      <!-- Version Display -->
      <span class="text-xs" style="color: var(--emittiv-light); opacity: 0.5;">
        v0.9.0
      </span>
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