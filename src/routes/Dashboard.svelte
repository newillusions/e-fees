<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
  import QuickActions from '$lib/components/dashboard/QuickActions.svelte';
  import ActivityFeed from '$lib/components/dashboard/ActivityFeed.svelte';
  import PendingProposals from '$lib/components/dashboard/PendingProposals.svelte';
  import { statisticsStore, isLoadingStore, loadAllData } from '$lib/stores';
  import { onMount } from 'svelte';
import { push } from 'svelte-spa-router';

  // Modal handling
  let showProjectModal = false;
  let showFeeModal = false;
  let showCompanyModal = false;

  function handleModalOpen(event: CustomEvent) {
    const { type } = event.detail;
    
    switch (type) {
      case 'project':
        showProjectModal = true;
        break;
      case 'fee':
        showFeeModal = true;
        break;
      case 'company':
        showCompanyModal = true;
        break;
    }
  }

  const statCards = [
    {
      title: 'Total Projects',
      key: 'totalProjects',
      icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10',
      color: 'text-blue-400',
      route: '/projects'
    },
    {
      title: 'Active Fees',
      key: 'activeFees',
      icon: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
      color: 'status-connected',
      route: '/proposals'
    },
    {
      title: 'Companies',
      key: 'totalCompanies',
      icon: 'M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4',
      color: 'text-green-400',
      route: '/companies'
    },
    {
      title: 'Contacts',
      key: 'totalContacts',
      icon: 'M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z',
      color: 'text-purple-400',
      route: '/contacts'
    }
  ];

  function handleStatCardClick(route: string) {
    push(route);
  }
</script>

<div class="dashboard-container">
  
  <!-- Quick Actions -->
  <section class="dashboard-section">
    <QuickActions on:openModal={handleModalOpen} />
  </section>
  
  <!-- Stats Grid -->
  <section class="dashboard-section">
    <div class="stats-grid">
      {#each statCards as card}
        <div class="stat-card" on:click={() => handleStatCardClick(card.route)} on:keydown={(e) => e.key === 'Enter' && handleStatCardClick(card.route)} role="button" tabindex="0">
          {#if $isLoadingStore}
            <LoadingSkeleton rows={2} />
          {:else}
            <div class="stat-card-content">
              <div class="stat-icon">
                <svg class="{card.color}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={card.icon} />
                </svg>
              </div>
              <div class="stat-info">
                <div class="stat-number">{($statisticsStore as any)[card.key]}</div>
                <div class="stat-label">{card.title}</div>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </section>
  
  <!-- Content Grid -->
  <section class="dashboard-section">
    <div class="content-grid">
      <div class="content-panel">
        <ActivityFeed isLoading={$isLoadingStore} />
      </div>
      
      <div class="content-panel">
        <PendingProposals isLoading={$isLoadingStore} />
      </div>
    </div>
  </section>
</div>

<!-- Modal Placeholders - These would be implemented when creating the modals -->
{#if showProjectModal}
  <!-- ProjectModal component would go here -->
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" on:click={() => showProjectModal = false}>
    <div class="bg-emittiv-black border border-emittiv-dark rounded-lg p-6">
      <p class="text-emittiv-white">Project Modal - To be implemented</p>
      <button class="mt-4 px-4 py-2 bg-emittiv-splash text-white rounded" on:click={() => showProjectModal = false}>
        Close
      </button>
    </div>
  </div>
{/if}

{#if showFeeModal}
  <!-- FeeModal component would go here -->
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" on:click={() => showFeeModal = false}>
    <div class="bg-emittiv-black border border-emittiv-dark rounded-lg p-6">
      <p class="text-emittiv-white">Fee Modal - To be implemented</p>
      <button class="mt-4 px-4 py-2 bg-emittiv-splash text-white rounded" on:click={() => showFeeModal = false}>
        Close
      </button>
    </div>
  </div>
{/if}

{#if showCompanyModal}
  <!-- CompanyModal component would go here -->
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" on:click={() => showCompanyModal = false}>
    <div class="bg-emittiv-black border border-emittiv-dark rounded-lg p-6">
      <p class="text-emittiv-white">Company Modal - To be implemented</p>
      <button class="mt-4 px-4 py-2 bg-emittiv-splash text-white rounded" on:click={() => showCompanyModal = false}>
        Close
      </button>
    </div>
  </div>
{/if}

<style>
  .dashboard-container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 24px;
    min-height: 100vh;
  }

  .dashboard-section {
    margin-bottom: 32px;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 20px;
  }

  .stat-card {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 12px;
    padding: 20px;
    transition: all 300ms cubic-bezier(0.4, 0, 0.2, 1);
    cursor: pointer;
    min-width: 0;
  }

  .stat-card:hover {
    border-color: var(--emittiv-light);
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(255, 153, 0, 0.1);
  }

  .stat-card:focus {
    outline: none;
    border-color: var(--emittiv-splash);
    box-shadow: 0 0 0 2px rgba(255, 153, 0, 0.2);
  }

  .stat-card-content {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .stat-icon {
    width: 48px;
    height: 48px;
    padding: 12px;
    background: rgba(255, 153, 0, 0.1);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .stat-icon svg {
    width: 24px;
    height: 24px;
  }

  .stat-info {
    flex: 1;
  }

  .stat-number {
    font-size: 28px;
    font-weight: 700;
    color: var(--emittiv-white);
    line-height: 1;
    margin-bottom: 4px;
  }

  .stat-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--emittiv-lighter);
    line-height: 1.2;
  }

  .content-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
  }

  .content-panel {
    min-height: 400px;
  }

  @media (max-width: 1024px) {
    .content-grid {
      grid-template-columns: 1fr;
      gap: 20px;
    }
  }

  @media (max-width: 768px) {
    .dashboard-container {
      padding: 16px;
    }
    
    .dashboard-section {
      margin-bottom: 24px;
    }
    
    .stats-grid {
      grid-template-columns: 1fr;
      gap: 16px;
    }
    
    .stat-card {
      padding: 16px;
    }
    
    .stat-number {
      font-size: 24px;
    }
  }
</style>