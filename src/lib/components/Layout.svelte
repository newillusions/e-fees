<script lang="ts">
  import Navigation from './Navigation.svelte';
  import ConnectionStatus from './ConnectionStatus.svelte';
  import { location } from 'svelte-spa-router';
  import { logoLight } from '../../assets';
  
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

<div class="flex h-screen overflow-hidden" style="background: var(--gradient-main);">
  <!-- Fixed Sidebar -->
  <aside class="w-64 flex flex-col flex-shrink-0" style="background: var(--gradient-sidebar); border-right: 1px solid var(--emittiv-darker);">
    <!-- Company Logo and Branding -->
    <div class="p-6 border-b" style="border-color: var(--emittiv-darker);">
      <div class="flex items-center space-x-3 mb-3">
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
      <p class="text-sm" style="color: var(--emittiv-light);">Proposal Management System</p>
    </div>
    
    <!-- Navigation -->
    <Navigation />
    
    <!-- Connection Status -->
    <div class="mt-auto p-4 border-t" style="border-color: var(--emittiv-darker);">
      <ConnectionStatus />
    </div>
  </aside>
  
  <!-- Main Content Area -->
  <main class="flex-1 flex flex-col overflow-hidden">
    <!-- Header -->
    <header class="px-8 py-4 flex-shrink-0 backdrop-blur-md border-b" 
            style="background: rgba(0, 0, 0, 0.5); border-color: var(--emittiv-darker);">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-4">
          <h2 class="text-xl font-heading font-semibold" style="color: var(--emittiv-white);">
            {pageTitle}
          </h2>
          <div class="w-1 h-6 rounded-full" style="background: var(--emittiv-splash);"></div>
          <span class="text-sm" style="color: var(--emittiv-light);">
            {new Date().toLocaleDateString('en-US', { 
              weekday: 'long', 
              year: 'numeric', 
              month: 'long', 
              day: 'numeric' 
            })}
          </span>
        </div>
        <div class="flex items-center space-x-4">
          <!-- Search button -->
          <button 
            class="p-2 rounded-lg transition-smooth hover-lift" 
            style="color: var(--emittiv-light);"
            aria-label="Search"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </button>
          
          <!-- Notifications button -->
          <button 
            class="p-2 rounded-lg transition-smooth hover-lift" 
            style="color: var(--emittiv-light);"
            aria-label="Notifications"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-5 5v-5zM11 17H7l4 4v-4zM5.868 12A9.001 9.001 0 0012 3a9 9 0 005.132 8.868" />
            </svg>
          </button>
          
          <!-- Settings button -->
          <button 
            class="p-2 rounded-lg transition-smooth hover-lift" 
            style="color: var(--emittiv-light);"
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
    <div class="flex-1 overflow-auto" style="background: var(--gradient-content);">
      <div class="page-enter">
        <slot />
      </div>
    </div>
  </main>
</div>

<style>
  /* Custom hover effects for header buttons */
  .hover-lift:hover {
    background-color: var(--emittiv-darker);
    color: var(--emittiv-white);
    transform: translateY(-1px);
  }
</style>