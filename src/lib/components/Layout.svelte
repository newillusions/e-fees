<script lang="ts">
  import Navigation from './Navigation.svelte';
  import ConnectionStatus from './ConnectionStatus.svelte';
  import SettingsModal from './SettingsModal.svelte';
  import NotificationsDropdown from './NotificationsDropdown.svelte';
  import GlobalSearchModal from './GlobalSearchModal.svelte';
  import { location } from 'svelte-spa-router';
  import { logoLight } from '../../assets';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  // Settings modal state
  let isSettingsOpen = $state(false);

  // Notifications state
  let isNotificationsOpen = $state(false);

  // Search modal state
  let isSearchOpen = $state(false);

  // Blur/privacy mode state
  let isBlurMode = $state(false);

  function toggleBlurMode() {
    isBlurMode = !isBlurMode;
  }

  function openSettings() {
    isSettingsOpen = true;
  }

  function closeSettings() {
    isSettingsOpen = false;
  }

  function openSearch() {
    isSearchOpen = true;
  }

  function closeSearch() {
    isSearchOpen = false;
  }

  // Keyboard shortcuts handler
  function handleKeydown(event: KeyboardEvent) {
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    const modKey = isMac ? event.metaKey : event.ctrlKey;

    // Cmd/Ctrl + K = Open Search (standard spotlight convention)
    if (modKey && event.key === 'k') {
      event.preventDefault();
      openSearch();
      return;
    }

    // Cmd/Ctrl + , = Open Settings (standard macOS convention)
    if (modKey && event.key === ',') {
      event.preventDefault();
      openSettings();
      return;
    }

    // Escape = Close modals (search handles its own escape)
    if (event.key === 'Escape' && isSettingsOpen) {
      event.preventDefault();
      closeSettings();
      return;
    }
  }

  // Dynamic page titles
  const pageTitles: Record<string, string> = {
    '/': 'Dashboard',
    '/projects': 'Projects',
    '/proposals': 'Proposals',
    '/companies': 'Companies',
    '/contacts': 'Contacts',
    '/venues': 'Venues',
    '/dev': 'Dev Mode'
  };

  const pageTitle = $derived(pageTitles[$location] || 'Dashboard');
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex h-screen overflow-hidden glass-container">
  <!-- Fixed Sidebar -->
  <aside class="w-64 flex flex-col flex-shrink-0 glass-sidebar relative">
    <!-- Company Logo and Branding -->
    <div
      class="h-20 px-6 border-b flex items-center flex-shrink-0"
      style="border-color: var(--emittiv-darker);"
    >
      <div class="flex items-center space-x-3">
        <div class="w-12 h-12 rounded-lg flex items-center justify-center overflow-hidden">
          <img
            src={logoLight}
            alt="emittiv Logo"
            class="w-full h-full object-contain filter brightness-110"
          />
        </div>
        <div>
          <h1 class="text-xl font-heading font-bold" style="color: var(--emittiv-white);">
            e-<span style="color: var(--emittiv-splash);">fees</span>
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
    <div
      class="absolute bottom-0 left-0 right-0 border-t glass-sidebar"
      style="border-color: var(--emittiv-darker);"
    >
      <ConnectionStatus />
    </div>
  </aside>

  <!-- Main Content Area -->
  <main class="flex-1 flex flex-col overflow-hidden">
    <!-- Header -->
    <header
      class="h-20 px-8 flex-shrink-0 glass-content border-b flex items-center"
      style="border-color: rgba(255, 255, 255, 0.1);"
    >
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-2">
          <h2 class="text-lg font-medium" style="color: var(--emittiv-white);">
            {pageTitle}
          </h2>
        </div>
        <div class="flex items-center space-x-1">
          <!-- Search button -->
          <button
            class="search-button p-1 rounded-lg transition-smooth hover-lift"
            style="color: var(--emittiv-light);"
            onclick={openSearch}
            aria-label="Search (Cmd+K)"
            title="Search (Cmd+K)"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
          </button>

          <!-- Blur/Privacy mode toggle -->
          <button
            class="p-1 rounded-lg transition-smooth hover-lift"
            class:blur-active={isBlurMode}
            style="color: var(--emittiv-light);"
            onclick={toggleBlurMode}
            aria-label={isBlurMode ? "Show content" : "Hide sensitive content"}
            title={isBlurMode ? "Show content (privacy mode on)" : "Hide sensitive content"}
          >
            {#if isBlurMode}
              <!-- Eye with slash - privacy mode ON -->
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
                />
              </svg>
            {:else}
              <!-- Eye - privacy mode OFF -->
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                />
              </svg>
            {/if}
          </button>

          <!-- Notifications dropdown -->
          <NotificationsDropdown
            bind:isOpen={isNotificationsOpen}
          />

          <!-- Settings button -->
          <button
            class="p-1 rounded-lg transition-smooth hover-lift"
            style="color: var(--emittiv-light);"
            onclick={openSettings}
            aria-label="Settings"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
              />
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
            </svg>
          </button>
        </div>
      </div>
    </header>

    <!-- Content with enhanced gradient background -->
    <div class="flex-1 main-content-scroll glass-content" class:privacy-blur={isBlurMode}>
      <div class="page-enter">
        {@render children()}
      </div>
    </div>
  </main>
</div>

<!-- Settings Modal -->
<SettingsModal bind:isOpen={isSettingsOpen} />

<!-- Global Search Modal -->
<GlobalSearchModal
  bind:isOpen={isSearchOpen}
  on:close={closeSearch}
/>

<style>
  /* Custom hover effects for header buttons */
  .hover-lift:hover {
    background-color: var(--emittiv-darker);
    color: var(--emittiv-white);
    transform: translateY(-1px);
  }

  /* Blur/privacy mode active state */
  .blur-active {
    color: var(--emittiv-splash) !important;
    background-color: rgba(255, 153, 0, 0.15);
  }

  /* Privacy blur effect on content */
  .privacy-blur {
    filter: blur(8px);
    pointer-events: none;
    user-select: none;
    transition: filter 0.3s ease;
  }

  .privacy-blur::after {
    content: '';
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.1);
    pointer-events: none;
  }
</style>
