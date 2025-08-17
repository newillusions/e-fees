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
    
    // Test database connection first
    setTimeout(async () => {
      try {
        const { checkDbConnection, getDbInfo, getSettings } = await import('$lib/api');
        
        const isConnected = await checkDbConnection();
        
        if (isConnected) {
          const dbInfo = await getDbInfo();
          appReady = true;
        } else {
          console.warn('Database is not connected');
          // Check if this is first run
          const settings = await getSettings();
          if (!settings || settings.surrealdb_user === 'placeholder' || !settings.surrealdb_user) {
            showFirstRun = true;
          } else {
            appReady = true;
          }
        }
        
        // Load data regardless
        await loadAllData();
      } catch (error) {
        console.error('Failed to load initial data:', error);
        // Show first run on error
        showFirstRun = true;
      }
    }, 500);
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