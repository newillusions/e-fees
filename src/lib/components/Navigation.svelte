<script lang="ts">
  import type { NavItem } from '../../types';
  import { onMount } from 'svelte';
  import { location, push } from 'svelte-spa-router';
  // import { positionWindow4K } from '../api'; // DISABLED for new environment
  
  const navItems: NavItem[] = [
    {
      id: '/',
      label: 'Dashboard',
      icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6',
      shortcut: '1'
    },
    {
      id: '/projects',
      label: 'Projects',
      icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10',
      shortcut: '2'
    },
    {
      id: '/companies',
      label: 'Companies',
      icon: 'M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4',
      shortcut: '3'
    },
    {
      id: '/contacts',
      label: 'Contacts',
      icon: 'M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z',
      shortcut: '4'
    },
    {
      id: '/proposals',
      label: 'Proposals',
      icon: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
      shortcut: '5'
    }
  ];
  
  function handleNavClick(route: string) {
    push(route);
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey) {
      const key = event.key;
      
      // Handle navigation shortcuts
      const item = navItems.find(item => item.shortcut === key);
      if (item) {
        event.preventDefault();
        handleNavClick(item.id);
        return;
      }
      
      // Window positioning shortcut (Cmd/Ctrl + W) - DISABLED for new environment
      // if (key === 'w' || key === 'W') {
      //   event.preventDefault();
      //   positionWindow4K().then(result => {
      //     console.log('Window positioning result:', result);
      //   }).catch(error => {
      //     console.error('Failed to position window:', error);
      //   });
      // }
    }
  }
  
  onMount(() => {
    document.addEventListener('keydown', handleKeydown);
    return () => document.removeEventListener('keydown', handleKeydown);
  });
</script>

<nav class="flex-1">
  <ul class="space-y-1">
    {#each navItems as item}
      <li>
        <button
          on:click={() => handleNavClick(item.id)}
          class="w-full px-6 py-2 flex items-center text-left transition-smooth group"
          class:nav-active={$location === item.id}
          class:nav-inactive={$location !== item.id}
        >
          <div class="flex items-center space-x-3 transition-transform group-hover:scale-105">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={item.icon} />
            </svg>
            <span class="font-medium">{item.label}</span>
          </div>
        </button>
      </li>
    {/each}
  </ul>
</nav>