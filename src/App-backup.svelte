<script lang="ts">
  import Layout from '$lib/components/Layout.svelte';
  import SplashScreen from '$lib/components/SplashScreen.svelte';
  import Router, { link, push } from 'svelte-spa-router';
  import Dashboard from './routes/Dashboard.svelte';
  import Projects from './routes/Projects.svelte';
  import Proposals from './routes/Proposals.svelte';
  import Companies from './routes/Companies.svelte';
  import Contacts from './routes/Contacts.svelte';
  import { onMount } from 'svelte';
  import { loadAllData } from '$lib/stores';
  
  let showSplash = true;
  let appReady = false;
  
  // Define routes
  const routes = {
    '/': Dashboard,
    '/projects': Projects,
    '/proposals': Proposals,
    '/companies': Companies,
    '/contacts': Contacts
  };
  
  // Route loading condition function
  function routeLoading(routeLoading: boolean) {
    // Add any loading logic here if needed
    return routeLoading;
  }
  
  // Route loaded function for transitions
  function routeLoaded(route: any) {
    // Trigger page transition
    if (typeof window !== 'undefined') {
      const content = document.querySelector('.route-content');
      if (content) {
        content.classList.remove('page-enter');
        requestAnimationFrame(() => {
          content.classList.add('page-enter');
        });
      }
    }
  }
  
  function handleSplashComplete() {
    showSplash = false;
    appReady = true;
    
    // Load data after splash is complete
    setTimeout(() => {
      loadAllData().catch(error => {
        console.error('Failed to load initial data:', error);
        // App will still work with empty data - the UI will show empty states
      });
    }, 500);
  }
  
  onMount(() => {
    // Initialize app-wide logic
    console.log('Fee Proposal Management App initialized');
    console.log('Show splash:', showSplash);
    
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
        on:routeLoading={routeLoading}
        on:routeLoaded={routeLoaded}
      />
    </div>
  </Layout>
{/if}