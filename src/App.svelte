<script lang="ts">
  import Layout from '$lib/components/Layout.svelte';
  import SplashScreen from '$lib/components/SplashScreen.svelte';
  import Router from 'svelte-spa-router';
  import Dashboard from './routes/Dashboard.svelte';
  import Projects from './routes/Projects.svelte';
  import Proposals from './routes/Proposals.svelte';
  import Companies from './routes/Companies.svelte';
  import Contacts from './routes/Contacts.svelte';
  import { onMount } from 'svelte';
  import { loadAllData } from '$lib/stores';
  
  let showSplash = true;
  let appReady = false;
  
  // Define routes for SPA
  const routes = {
    '/': Dashboard,
    '/projects': Projects,
    '/proposals': Proposals,
    '/companies': Companies,
    '/contacts': Contacts
  };
  
  // Route loaded function for transitions
  function routeLoaded(route: any) {
    // Trigger page transition
    if (typeof window !== 'undefined') {
      const content = document.querySelector('.route-content');
      if (content) {
        // Remove the class first to reset the animation
        content.classList.remove('page-enter');
        // Force a reflow to ensure the animation restarts
        void (content as HTMLElement).offsetWidth;
        // Add the class back to trigger the animation
        content.classList.add('page-enter');
      }
    }
  }
  
  async function handleSplashComplete() {
    showSplash = false;
    appReady = true;
    
    // Test database connection first
    setTimeout(async () => {
      try {
        console.log('Testing database connection...');
        const { checkDbConnection, getDbInfo } = await import('$lib/api');
        
        const isConnected = await checkDbConnection();
        console.log('Database connection status:', isConnected);
        
        if (isConnected) {
          console.log('Database is connected, fetching info...');
          const dbInfo = await getDbInfo();
          console.log('Database info:', dbInfo);
        } else {
          console.warn('Database is not connected');
        }
        
        // Load data regardless
        console.log('Loading all data...');
        await loadAllData();
        console.log('Data loading completed');
      } catch (error) {
        console.error('Failed to load initial data:', error);
      }
    }, 500);
  }
  
  onMount(() => {
    // Initialize app-wide logic
    
    // The splash screen will handle the initialization timing
    // Data loading will happen after splash completes
  });
</script>

{#if showSplash}
  <SplashScreen onComplete={handleSplashComplete} />
{:else}
  <Layout>
    <div class="route-content">
      <Router 
        {routes} 
        on:routeLoaded={routeLoaded}
      />
    </div>
  </Layout>
{/if}