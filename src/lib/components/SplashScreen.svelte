<script lang="ts">
  import { onMount } from 'svelte';
  import { logo } from '../../assets';
  import { getAppVersion } from '../utils';
  import { logger } from '$lib/services/logger';
  
  
  let { onComplete }: {
    onComplete: () => void;
  } = $props();

  let connectionText = $state('Initializing...');
  let dots = $state('');
  let isConnecting = $state(true);
  let progress = $state(0);
  let appVersion = $state('...');
  
  // Animated dots for connecting message
  let dotInterval: number;
  let progressInterval: number;
  
  const connectionMessages = [
    'Initializing application...',
    'Loading configuration...',
    'Connecting to database...',
    'Establishing secure connection...',
    'Authenticating...',
    'Loading data...',
    'Almost ready...'
  ];
  
  let messageIndex = $state(0);
  
  onMount(() => {
    // Load app version (async but not blocking)
    getAppVersion().then(version => {
      appVersion = version;
    });

    // Animate dots
    dotInterval = window.setInterval(() => {
      dots = dots.length >= 3 ? '' : dots + '.';
    }, 500);

    // Progress simulation
    progressInterval = window.setInterval(() => {
      if (progress < 90) {
        progress += Math.random() * 15;
        if (progress > 90) progress = 90;
      }
    }, 300);
    
    // Cycle through connection messages
    const messageInterval = setInterval(() => {
      if (messageIndex < connectionMessages.length - 1) {
        messageIndex++;
        connectionText = connectionMessages[messageIndex];
      }
    }, 800);
    
    // Simple timeout-based completion (ConnectionStatus will handle actual connection monitoring)
    // Just show the splash for a reasonable amount of time
    setTimeout(() => {
      progress = 100;
      connectionText = 'Ready!';
      setTimeout(() => {
        isConnecting = false;
        onComplete();
      }, 500);
    }, 3000); // 3 second splash display
    
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
      clearInterval(messageInterval);
    };
  });
</script>

<div 
  class="fixed inset-0 z-90 flex items-center justify-center transition-all duration-500"
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
          on:error={() => logger.error('Failed to load logo', { src: logo })}
        />
      {:else}
        <!-- Fallback logo placeholder -->
        <div class="splash-logo-fallback">
          <span class="splash-logo-fallback__text">e-fees</span>
        </div>
      {/if}
    </div>

    <!-- Company Name -->
    <h1 class="splash-title">
      e-<span class="splash-accent">fees</span>
    </h1>
    <p class="splash-subtitle">by emittiv</p>
    <p class="splash-version">v{appVersion}</p>

    <!-- Progress Bar -->
    <div class="mb-6">
      <div class="splash-progress-track">
        <div
          class="splash-progress-fill"
          style="width: {progress}%;"
        ></div>
      </div>
    </div>

    <!-- Connection Status -->
    <div class="flex items-center justify-center" style="min-height: 2rem;">
      <p class="splash-status-text">
        {connectionText}<span class="inline-block w-6 text-left">{dots}</span>
      </p>
    </div>
  </div>

  <!-- Animated Background Elements -->
  <div class="absolute inset-0 overflow-hidden pointer-events-none">
    <!-- Floating circles -->
    <div class="splash-ping" style="top: 25%; left: 25%; width: 8px; height: 8px; animation-delay: 0s; animation-duration: 3s;"></div>
    <div class="splash-ping" style="top: 75%; right: 25%; width: 4px; height: 4px; animation-delay: 1s; animation-duration: 4s;"></div>
    <div class="splash-ping" style="top: 50%; left: 16.67%; width: 6px; height: 6px; animation-delay: 2s; animation-duration: 3.5s;"></div>
  </div>
</div>

<style>
  .splash-logo-fallback {
    width: 128px;
    height: 64px;
    margin: 0 auto;
    background: var(--emittiv-darker);
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--emittiv-dark);
  }

  .splash-logo-fallback__text {
    font-size: 12px;
    color: var(--emittiv-light);
  }

  .splash-title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: 8px;
    color: var(--emittiv-white);
  }

  .splash-accent {
    color: var(--emittiv-splash);
  }

  .splash-subtitle {
    font-size: 14px;
    color: var(--emittiv-light);
  }

  .splash-version {
    font-size: 12px;
    margin-bottom: 32px;
    color: var(--emittiv-dark);
  }

  .splash-progress-track {
    width: 100%;
    height: 4px;
    border-radius: 9999px;
    background: var(--emittiv-darker);
  }

  .splash-progress-fill {
    height: 100%;
    border-radius: 9999px;
    background: linear-gradient(to right, var(--emittiv-splash), #fb923c);
    transition: width 0.3s ease-out;
  }

  .splash-status-text {
    font-size: 14px;
    transition: all 0.3s;
    color: var(--emittiv-lighter);
  }

  .splash-ping {
    position: absolute;
    border-radius: 9999px;
    background: var(--emittiv-splash);
    opacity: 0.3;
    animation: ping 1s cubic-bezier(0, 0, 0.2, 1) infinite;
  }

  @keyframes ping {
    75%, 100% {
      transform: scale(2);
      opacity: 0;
    }
  }
</style>