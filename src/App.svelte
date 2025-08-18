<script lang="ts">
  import Layout from '$lib/components/Layout.svelte';
  import SplashScreen from '$lib/components/SplashScreen.svelte';
  import FirstRunSetup from '$lib/components/FirstRunSetup.svelte';
  import Router, { location } from 'svelte-spa-router';
  import Dashboard from './routes/Dashboard.svelte';
  import Projects from './routes/Projects.svelte';
  import Proposals from './routes/Proposals.svelte';
  import Companies from './routes/Companies.svelte';
  import Contacts from './routes/Contacts.svelte';
  import { onMount } from 'svelte';
  import { loadAllData } from '$lib/stores';
  import { fade } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import { setupPluginListeners, cleanupPluginListeners } from '../tauri-plugin-mcp/dist-js/index.js';
  
  let showSplash = true;
  let appReady = false;
  let showFirstRun = false;
  
  // Reset scroll position on route change
  $: if ($location && appReady) {
    // Use setTimeout to ensure the DOM has updated
    setTimeout(() => {
      // Find the main content scroll container
      const contentContainer = document.querySelector('.glass-content.overflow-auto');
      if (contentContainer) {
        contentContainer.scrollTo({ top: 0, behavior: 'smooth' });
      }
    }, 50);
  }
  
  // Define routes for SPA
  const routes = {
    '/': Dashboard,
    '/projects': Projects,
    '/proposals': Proposals,
    '/companies': Companies,
    '/contacts': Contacts
  };
  
  async function handleSplashComplete() {
    showSplash = false;
    
    // Simplified startup: just check settings and start the app
    // The ConnectionStatus component will handle database connectivity display
    setTimeout(async () => {
      try {
        console.log('Starting app initialization...');
        const { getSettings } = await import('$lib/api');
        
        const settings = await getSettings();
        console.log('Settings loaded:', settings ? 'found' : 'not found');
        
        const isFirstRun = !settings || 
                          settings.surrealdb_user === 'placeholder' || 
                          !settings.surrealdb_user ||
                          !settings.surrealdb_url ||
                          settings.surrealdb_url === 'placeholder';
        
        if (isFirstRun) {
          console.log('First run detected - showing setup');
          showFirstRun = true;
        } else {
          console.log('Settings found - starting app');
          appReady = true;
          // Data will be loaded automatically when ConnectionStatus detects first connection
          console.log('App ready - data loading will happen after database connection confirmed');
          
          // FALLBACK: Also attempt data loading directly after a delay
          // This ensures data loads even if ConnectionStatus fails
          setTimeout(async () => {
            console.log('FALLBACK: Starting unconditional data loading after 5s delay...');
            try {
              await loadAllData();
              console.log('FALLBACK: Unconditional data loading completed successfully');
            } catch (error) {
              console.log('FALLBACK: Unconditional data loading failed:', error);
            }
          }, 5000); // 5 second delay to allow ConnectionStatus to try first
        }
      } catch (error) {
        console.error('Failed during app initialization:', error);
        // If we can't load settings at all, show first run setup
        console.log('Settings load failed - showing first run setup');
        showFirstRun = true;
      }
    }, 800); // Reduced delay since we're not trying to test connections
  }
  
  function handleFirstRunComplete() {
    showFirstRun = false;
    appReady = true;
    // Reload to apply new settings
    window.location.reload();
  }
  
  onMount(async () => {
    // Initialize app-wide logic
    
    // Set up MCP plugin event listeners
    try {
      await setupPluginListeners();
      console.log('MCP plugin listeners set up successfully');
    } catch (error) {
      console.error('Failed to set up MCP plugin listeners:', error);
    }
    
    // The splash screen will handle the initialization timing
    // Data loading will happen after splash completes
    
    // Cleanup function for when component is destroyed
    return () => {
      cleanupPluginListeners();
    };
  });
</script>

{#if showSplash}
  <SplashScreen onComplete={handleSplashComplete} />
{:else if showFirstRun}
  <FirstRunSetup bind:isOpen={showFirstRun} on:complete={handleFirstRunComplete} />
{:else if appReady}
  <Layout>
    <div class="route-content">
      {#key $location}
        <div
          in:fade={{ duration: 300, delay: 150, easing: quintOut }}
          out:fade={{ duration: 200, easing: quintOut }}
        >
          <Router {routes} />
        </div>
      {/key}
    </div>
  </Layout>
{/if}

<style>
  .route-content {
    position: relative;
    width: 100%;
    height: 100%;
  }
  
  .route-content > :global(div) {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }
</style>