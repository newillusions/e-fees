<script lang="ts">
  import { onMount } from 'svelte';
  import { logo } from '../../assets';
  import { connectionStore } from '../stores';
  import { getConnectionStatus } from '../api';
  
  
  export let onComplete: () => void;
  
  let connectionText = 'Initializing...';
  let dots = '';
  let isConnecting = true;
  let progress = 0;
  
  // Animated dots for connecting message
  let dotInterval: number;
  let progressInterval: number;
  let statusCheckInterval: number;
  
  const connectionMessages = [
    'Initializing application...',
    'Loading configuration...',
    'Connecting to database...',
    'Establishing secure connection...',
    'Authenticating...',
    'Loading data...',
    'Almost ready...'
  ];
  
  let messageIndex = 0;
  
  onMount(() => {
    // Animate dots
    dotInterval = setInterval(() => {
      dots = dots.length >= 3 ? '' : dots + '.';
    }, 500) as any;
    
    // Progress simulation
    progressInterval = setInterval(() => {
      if (progress < 90) {
        progress += Math.random() * 15;
        if (progress > 90) progress = 90;
      }
    }, 300) as any;
    
    // Cycle through connection messages
    const messageInterval = setInterval(() => {
      if (messageIndex < connectionMessages.length - 1) {
        messageIndex++;
        connectionText = connectionMessages[messageIndex];
      }
    }, 800);
    
    // Check actual connection status
    statusCheckInterval = setInterval(async () => {
      try {
        const status = await getConnectionStatus();
        connectionStore.set({
          isConnected: status.is_connected,
          status: status.is_connected ? 'Connected' : 'Connecting',
          lastChecked: status.last_check ? new Date(status.last_check) : undefined,
          errorMessage: status.error_message || undefined
        });
        
        if (status.is_connected || messageIndex >= connectionMessages.length - 1) {
          // Connection successful or timeout reached
          progress = 100;
          connectionText = status.is_connected ? 'Connected successfully!' : 'Using offline mode...';
          
          setTimeout(() => {
            isConnecting = false;
            onComplete();
          }, 1000);
        }
      } catch (error) {
        // If we can't check status, assume we need to continue loading
        if (messageIndex >= connectionMessages.length - 1) {
          progress = 100;
          connectionText = 'Ready!';
          setTimeout(() => {
            isConnecting = false;
            onComplete();
          }, 1000);
        }
      }
    }, 1000) as any;
    
    // Fallback timeout to ensure splash screen doesn't hang
    setTimeout(() => {
      if (isConnecting) {
        progress = 100;
        connectionText = 'Ready!';
        isConnecting = false;
        onComplete();
      }
    }, 8000); // 8 second max
    
    // Cleanup intervals
    return () => {
      clearInterval(dotInterval);
      clearInterval(progressInterval);
      clearInterval(statusCheckInterval);
      clearInterval(messageInterval);
    };
  });
</script>

<div 
  class="fixed inset-0 z-50 flex items-center justify-center transition-all duration-500"
  style="background: linear-gradient(135deg, #000, #333);"
  class:opacity-0={!isConnecting}
  class:pointer-events-none={!isConnecting}
>
  <!-- Background Pattern -->
  <div class="absolute inset-0 opacity-5">
    <div class="absolute inset-0" style="background-image: radial-gradient(circle at 25% 25%, #ff9900 2px, transparent 2px); background-size: 50px 50px;"></div>
  </div>
  
  <!-- Main Content -->
  <div class="relative z-10 text-center px-8 max-w-md">
    <!-- Logo -->
    <div class="mb-8 transform transition-transform duration-1000" class:scale-110={progress > 50}>
      {#if logo}
        <img 
          src={logo} 
          alt="Company Logo" 
          class="w-32 h-16 mx-auto filter brightness-110 transition-all duration-1000"
          class:animate-pulse={progress < 30}
          on:error={() => console.error('Failed to load logo:', logo)}
        />
      {:else}
        <!-- Fallback logo placeholder -->
        <div class="w-32 h-16 mx-auto bg-gray-700 rounded flex items-center justify-center border border-gray-600">
          <span class="text-xs text-gray-400">FeePro</span>
        </div>
      {/if}
    </div>
    
    <!-- Company Name -->
    <h1 class="text-2xl font-bold mb-2 text-white">
      Fee<span class="text-orange-500">Pro</span>
    </h1>
    <p class="text-sm mb-8 text-gray-400">by emittiv</p>
    
    <!-- Progress Bar -->
    <div class="mb-6">
      <div class="w-full h-1 rounded-full bg-gray-700">
        <div 
          class="h-full rounded-full transition-all duration-300 ease-out bg-gradient-to-r from-orange-500 to-orange-400"
          style="width: {progress}%;"
        ></div>
      </div>
    </div>
    
    <!-- Connection Status -->
    <div class="min-h-[2rem] flex items-center justify-center">
      <p class="text-sm transition-all duration-300 text-gray-300">
        {connectionText}<span class="inline-block w-6 text-left">{dots}</span>
      </p>
    </div>
    
    <!-- Status Indicator -->
    <div class="mt-4 flex items-center justify-center space-x-2">
      <div 
        class="w-2 h-2 rounded-full transition-all duration-500"
        class:bg-orange-500={$connectionStore.isConnected}
        class:bg-gray-400={!$connectionStore.isConnected}
        class:animate-pulse={!$connectionStore.isConnected}
      ></div>
      <span class="text-xs text-gray-400">
        {$connectionStore.isConnected ? 'Database Connected' : 'Connecting to Database'}
      </span>
    </div>
  </div>
  
  <!-- Animated Background Elements -->
  <div class="absolute inset-0 overflow-hidden pointer-events-none">
    <!-- Floating circles -->
    <div class="absolute top-1/4 left-1/4 w-2 h-2 rounded-full animate-ping bg-orange-500 opacity-30" style="animation-delay: 0s; animation-duration: 3s;"></div>
    <div class="absolute top-3/4 right-1/4 w-1 h-1 rounded-full animate-ping bg-orange-500 opacity-30" style="animation-delay: 1s; animation-duration: 4s;"></div>
    <div class="absolute top-1/2 left-1/6 w-1.5 h-1.5 rounded-full animate-ping bg-orange-500 opacity-30" style="animation-delay: 2s; animation-duration: 3.5s;"></div>
  </div>
</div>

<style>
  /* Component uses Tailwind classes for styling */
</style>