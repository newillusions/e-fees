<script lang="ts">
  import Layout from '$lib/components/Layout.svelte';
  import SplashScreen from '$lib/components/SplashScreen.svelte';
  import FirstRunSetup from '$lib/components/FirstRunSetup.svelte';
  import UpdateNotification from '$lib/components/UpdateNotification.svelte';
  import Router, { location } from 'svelte-spa-router';
  import Dashboard from './routes/Dashboard.svelte';
  import Projects from './routes/Projects.svelte';
  import Proposals from './routes/Proposals.svelte';
  import Companies from './routes/Companies.svelte';
  import Contacts from './routes/Contacts.svelte';
  import ProjectDetailPage from './routes/ProjectDetailPage.svelte';
  import ProposalDetailPage from './routes/ProposalDetailPage.svelte';
  import DevMode from './routes/DevMode.svelte';
  import ScopeBuilderPage from './routes/ScopeBuilderPage.svelte';
  import { onMount, onDestroy } from 'svelte';
  import { loadAllData } from '$lib/stores';
  import { refreshExchangeRates } from '$lib/stores/exchangeRates';
  import { fade } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import {
    setupPluginListeners,
    cleanupPluginListeners
  } from '../tauri-plugin-mcp/dist-js/index.js';

  let showSplash = $state(true);
  let appReady = $state(false);
  let showFirstRun = $state(false);

  // Startup connection gate. Previously the app declared itself "ready"
  // (and mounted Dashboard/Proposals/etc., which immediately query the DB)
  // as soon as settings existed - it never waited for the backend's async
  // DB connect to actually finish. That connect is a real network round
  // trip (up to 3 sequential signin attempts for a root-level user), so a
  // query could - and did, live - fire before a client existed, fail
  // instantly, and leave pages silently showing "0"/empty with no error.
  // 'connecting' polls check_connection_status for real readiness;
  // 'failed' means the bounded window ran out without a definitive
  // success - shown with the backend's real error message and a Retry,
  // never faked into 'ready'.
  let connectionPhase: 'connecting' | 'ready' | 'failed' = $state('connecting');
  let connectionErrorMessage = $state('');

  const CONNECT_POLL_INTERVAL_MS = 500;
  const CONNECT_POLL_TIMEOUT_MS = 15000;

  async function waitForConnection(): Promise<void> {
    connectionPhase = 'connecting';
    connectionErrorMessage = '';

    const { getConnectionStatus } = await import('$lib/api');
    const deadline = Date.now() + CONNECT_POLL_TIMEOUT_MS;

    while (Date.now() < deadline) {
      const status = await getConnectionStatus();
      if (status.is_connected) {
        connectionPhase = 'ready';
        appReady = true;
        return;
      }
      await new Promise(resolve => setTimeout(resolve, CONNECT_POLL_INTERVAL_MS));
    }

    // Timed out - one last check to capture whatever error the backend
    // most recently reported, so the user sees a real reason, not a
    // generic timeout.
    const finalStatus = await getConnectionStatus();
    connectionErrorMessage =
      finalStatus.error_message ||
      'Could not connect to the database. Please check your connection settings.';
    connectionPhase = 'failed';
  }

  // Reset scroll position on route change
  $effect(() => {
    if ($location && appReady) {
      // Use setTimeout to ensure the DOM has updated
      setTimeout(() => {
        // Find the main content scroll container
        const contentContainer = document.querySelector('.glass-content.overflow-auto');
        if (contentContainer) {
          contentContainer.scrollTo({ top: 0, behavior: 'smooth' });
        }
      }, 50);
    }
  });

  // Define routes for SPA
  const routes = {
    '/': Dashboard,
    '/projects': Projects,
    '/projects/:id': ProjectDetailPage,
    '/proposals': Proposals,
    '/proposals/:id': ProposalDetailPage,
    '/companies': Companies,
    '/contacts': Contacts,
    '/scope/:id': ScopeBuilderPage,
    '/dev': DevMode
  };

  async function handleSplashComplete() {
    showSplash = false;

    try {
      const { getSettings } = await import('$lib/api');
      const settings = await getSettings();

      const isFirstRun =
        !settings ||
        settings.surrealdb_user === 'placeholder' ||
        !settings.surrealdb_user ||
        !settings.surrealdb_url ||
        settings.surrealdb_url === 'placeholder';

      if (isFirstRun) {
        showFirstRun = true;
        return;
      }

      await waitForConnection();
    } catch (error) {
      console.error('Failed during app initialization:', error);
      showFirstRun = true;
    }
  }

  function handleFirstRunComplete() {
    showFirstRun = false;
    appReady = true;
    // Reload to apply new settings
    window.location.reload();
  }

  onMount(async () => {
    // Initialize app-wide logic

    // Fetch exchange rates on startup (fire and forget — don't block app loading)
    refreshExchangeRates().catch(err => console.warn('Exchange rates unavailable:', err));

    // Set up MCP plugin event listeners
    try {
      await setupPluginListeners();
    } catch (error) {
      console.error('Failed to set up MCP plugin listeners:', error);
    }

    // The splash screen will handle the initialization timing
    // Data loading will happen after splash completes
  });

  // Cleanup function for when component is destroyed
  onDestroy(() => {
    cleanupPluginListeners();
  });
</script>

{#if showSplash}
  <SplashScreen onComplete={handleSplashComplete} />
{:else if showFirstRun}
  <FirstRunSetup bind:isOpen={showFirstRun} oncomplete={handleFirstRunComplete} />
{:else if connectionPhase === 'connecting'}
  <div class="startup-gate">
    <div class="emittiv-spinner emittiv-spinner--page"></div>
    <p class="startup-gate-text">Connecting to database...</p>
  </div>
{:else if connectionPhase === 'failed'}
  <div class="startup-gate">
    <div class="emittiv-alert emittiv-alert--error" style="max-width: 480px;">
      {connectionErrorMessage}
      <button
        type="button"
        class="emittiv-link"
        onclick={waitForConnection}
        style="margin-left: 8px;"
      >
        Retry
      </button>
    </div>
  </div>
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
  <!-- Update notification - checks for updates on app startup -->
  <UpdateNotification />
{/if}

<style>
  .startup-gate {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100vh;
    background: var(--emittiv-black);
    gap: 12px;
  }

  .startup-gate-text {
    color: var(--emittiv-light);
    font-size: 14px;
  }

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
