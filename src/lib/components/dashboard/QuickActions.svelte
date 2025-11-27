<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import { validateProjectBasePath } from '$lib/api/folderManagement';

  const dispatch = createEventDispatcher();

  function handleNewProject() {
    dispatch('openModal', { type: 'project' });
  }

  function handleNewFee() {
    dispatch('openModal', { type: 'fee' });
  }

  function handleNewCompany() {
    dispatch('openModal', { type: 'company' });
  }

  async function handleOpenFolders() {
    try {
      const basePath = await validateProjectBasePath();
      // For now, just show an alert with the base path
      // In the future, could open the folder externally
      alert(`Project base path: ${basePath}`);
    } catch (error) {
      console.error('Failed to access project folders:', error);
      alert('Failed to access project folders');
    }
  }

  const quickActions = [
    {
      id: 'project',
      title: 'New Project',
      icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10',
      color: 'text-blue-400',
      handler: handleNewProject
    },
    {
      id: 'fee',
      title: 'Create Proposal',
      icon: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
      color: 'text-splash',
      handler: handleNewFee
    },
    {
      id: 'company',
      title: 'Add Company',
      icon: 'M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4',
      color: 'text-green-400',
      handler: handleNewCompany
    },
    {
      id: 'folders',
      title: 'Open Folders',
      icon: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z',
      color: 'text-purple-400',
      handler: handleOpenFolders
    }
  ];
</script>

<div class="quick-actions-section">
  <h2 class="section-title">Quick Actions</h2>
  
  <div class="actions-grid">
    {#each quickActions as action}
      <button
        class="action-button"
        on:click={action.handler}
      >
        <div class="action-icon">
          <svg class="{action.color}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={action.icon} />
          </svg>
        </div>
        <span class="action-label">{action.title}</span>
      </button>
    {/each}
  </div>
</div>

<style>
  .quick-actions-section {
    /* No container styling - let buttons align with stats grid */
  }

  .section-title {
    font-family: Ubuntu, sans-serif;
    font-size: 20px;
    font-weight: 600;
    color: var(--emittiv-white);
    margin: 0 0 20px 0;
    letter-spacing: -0.02em;
  }

  .actions-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 20px;
  }

  .action-button {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px;
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 12px;
    color: var(--emittiv-lighter);
    text-decoration: none;
    transition: all 300ms cubic-bezier(0.4, 0, 0.2, 1);
    cursor: pointer;
    min-width: 0;
  }

  .action-button:hover {
    border-color: var(--emittiv-light);
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(255, 153, 0, 0.1);
  }

  .action-icon {
    width: 48px;
    height: 48px;
    padding: 12px;
    background: rgba(255, 153, 0, 0.1);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: transform 200ms ease;
  }

  .action-icon svg {
    width: 24px;
    height: 24px;
  }

  .action-button:hover .action-icon {
    transform: scale(1.05);
  }

  .action-label {
    flex: 1;
    font-size: 14px;
    font-weight: 500;
    color: var(--emittiv-lighter);
    line-height: 1.2;
  }

  @media (max-width: 768px) {
    .actions-grid {
      grid-template-columns: 1fr;
      gap: 12px;
    }
    
    .action-button {
      padding: 14px 16px;
      min-height: 56px;
    }
    
    .action-icon {
      width: 28px;
      height: 28px;
    }
  }
</style>