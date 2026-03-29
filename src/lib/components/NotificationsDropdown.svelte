<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  let {
    isOpen = $bindable(false)
  }: {
    isOpen?: boolean;
  } = $props();

  // Close dropdown when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (isOpen && !target.closest('.notifications-container')) {
      isOpen = false;
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
  });

  onDestroy(() => {
    document.removeEventListener('click', handleClickOutside);
  });
</script>

<div class="notifications-container">
  <!-- Bell Button -->
  <button
    class="notification-button"
    on:click|stopPropagation={() => (isOpen = !isOpen)}
    aria-label="Notifications"
    aria-expanded={isOpen}
  >
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
      />
    </svg>
  </button>

  <!-- Dropdown -->
  {#if isOpen}
    <div class="dropdown">
      <div class="dropdown-header">
        <h3 class="dropdown-title">Notifications</h3>
      </div>

      <div class="dropdown-content">
        <div class="coming-soon-state">
          <svg class="coming-soon-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
            />
          </svg>
          <p class="coming-soon-title">Coming Soon</p>
          <p class="coming-soon-text">System notifications will appear here in a future update.</p>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .notifications-container {
    position: relative;
  }

  .notification-button {
    position: relative;
    padding: 4px;
    border-radius: 8px;
    transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
    color: var(--emittiv-light);
    background: transparent;
    border: none;
    cursor: pointer;
  }

  .notification-button:hover {
    background-color: var(--emittiv-darker);
    color: var(--emittiv-white);
    transform: translateY(-1px);
  }

  .dropdown {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    width: 300px;
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 12px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
    z-index: 1000;
    overflow: hidden;
  }

  .dropdown-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    border-bottom: 1px solid var(--emittiv-dark);
  }

  .dropdown-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--emittiv-white);
    margin: 0;
  }

  .dropdown-content {
    padding: 24px 16px;
  }

  .coming-soon-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
  }

  .coming-soon-icon {
    width: 48px;
    height: 48px;
    color: var(--emittiv-dark);
    margin-bottom: 16px;
  }

  .coming-soon-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--emittiv-lighter);
    margin: 0 0 8px 0;
  }

  .coming-soon-text {
    font-size: 12px;
    color: var(--emittiv-light);
    margin: 0;
    line-height: 1.5;
  }
</style>
