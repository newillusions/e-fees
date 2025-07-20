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
    console.log('=== SurrealDB Connection Debug Info ===');
    console.log(JSON.stringify(dbInfo, null, 2));
    console.log('=======================================');
    showDebugInfo = !showDebugInfo;
  }

  async function testRfpCreation() {
    try {
      // Use the actual RFP creation through the modal/API
      const { rfpsActions } = await import('../stores');
      
      const testRfpData = {
        name: "DELETE ME - Test RFP",
        number: "25-97107-FP-DELETE-ME",
        rev: 1,
        status: "Draft",
        stage: "Draft",
        issue_date: "250720",
        activity: "Design and Consultancy",
        package: "Complete",
        staff_name: "Martin Robert",
        staff_email: "martin@emittiv.com",
        staff_phone: "+971 5858 555 69",
        staff_position: "Lighting Director",
        strap_line: "sensory design studio",
        revisions: [],
        project_id: 'projects:25_97107',
        company_id: 'company:EMT',                         // Correct EMITTIV company ID
        contact_id: 'contacts:hqpz6h9z1v5w6uj46tl2'        // Correct Martin Robert contact ID
      };
      
      await rfpsActions.create(testRfpData);
      console.log('Test RFP created successfully via stores');
      alert('Test RFP created successfully!');
    } catch (error) {
      console.error('Test RFP Error:', error);
      alert(`Test RFP failed: ${error}`);
    }
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
    
    <!-- Test button for debugging -->
    <button
      on:click={testRfpCreation}
      class="ml-4 px-2 py-1 text-xs bg-orange-600 hover:bg-orange-700 text-white rounded transition-colors"
      title="Test RFP Creation"
    >
      Test RFP
    </button>
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