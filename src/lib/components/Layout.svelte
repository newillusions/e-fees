<script lang="ts">
  import Navigation from './Navigation.svelte';
  import ConnectionStatus from './ConnectionStatus.svelte';
  import SettingsModal from './SettingsModal.svelte';
  import { location } from 'svelte-spa-router';
  import { logoLight } from '../../assets';
  
  // Settings modal state
  let isSettingsOpen = false;
  
  function openSettings() {
    isSettingsOpen = true;
  }
  
  // Dynamic page titles
  const pageTitles: Record<string, string> = {
    '/': 'Dashboard',
    '/projects': 'Projects',
    '/proposals': 'Proposals',
    '/companies': 'Companies',
    '/contacts': 'Contacts'
  };
  
  $: pageTitle = pageTitles[$location] || 'Dashboard';
</script>

<div class="flex h-screen overflow-hidden glass-container">
  <!-- Fixed Sidebar -->
  <aside class="w-64 flex flex-col flex-shrink-0 glass-sidebar relative">
    <!-- Company Logo and Branding -->
    <div class="h-20 px-6 border-b flex items-center flex-shrink-0" style="border-color: var(--emittiv-darker);">
      <div class="flex items-center space-x-3">
        <div class="w-10 h-10 rounded-lg flex items-center justify-center overflow-hidden">
          <img 
            src={logoLight} 
            alt="Company Logo" 
            class="w-full h-full object-contain filter brightness-110"
          />
        </div>
        <div>
          <h1 class="text-xl font-heading font-bold" style="color: var(--emittiv-white);">
            Fee<span style="color: var(--emittiv-splash);">Pro</span>
          </h1>
          <p class="text-xs" style="color: var(--emittiv-light);">by emittiv</p>
        </div>
      </div>
    </div>
    
    <!-- Navigation -->
    <div class="flex-1 overflow-y-auto pb-20">
      <Navigation />
    </div>
    
    <!-- Connection Status - Fixed at bottom -->
    <div class="absolute bottom-0 left-0 right-0 border-t glass-sidebar" style="border-color: var(--emittiv-darker);">
      <ConnectionStatus />
    </div>
  </aside>
  
  <!-- Main Content Area -->
  <main class="flex-1 flex flex-col overflow-hidden">
    <!-- Header -->
    <header class="h-20 px-8 flex-shrink-0 glass-content border-b flex items-center" 
            style="border-color: rgba(255, 255, 255, 0.1);">
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-2">
          <span class="text-sm font-medium" style="color: var(--emittiv-white);">
            24-97108
          </span>
          <span class="text-sm" style="color: var(--emittiv-light);">
            Museum of Future
          </span>
        </div>
        <div class="flex items-center space-x-1">
          <!-- Search button -->
          <button 
            class="p-1 rounded-lg transition-smooth hover-lift" 
            style="color: var(--emittiv-light);"
            aria-label="Search"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </button>
          
          <!-- Notifications button -->
          <button 
            class="p-1 rounded-lg transition-smooth hover-lift" 
            style="color: var(--emittiv-light);"
            aria-label="Notifications"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
            </svg>
          </button>
          
          <!-- Settings button -->
          <button 
            class="p-1 rounded-lg transition-smooth hover-lift" 
            style="color: var(--emittiv-light);"
            on:click={openSettings}
            aria-label="Settings"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </div>
      </div>
    </header>
    
    <!-- Content with enhanced gradient background -->
    <div class="flex-1 overflow-auto glass-content">
      <div class="page-enter">
        <slot />
      </div>
    </div>
  </main>
</div>

<!-- Settings Modal -->
<SettingsModal bind:isOpen={isSettingsOpen} />

<style>
  /* Custom hover effects for header buttons */
  .hover-lift:hover {
    background-color: var(--emittiv-darker);
    color: var(--emittiv-white);
    transform: translateY(-1px);
  }
</style>