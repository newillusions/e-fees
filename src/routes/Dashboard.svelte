<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
  import { statisticsStore, isLoadingStore, recentProposalsStore } from '$lib/stores';
  import { onMount } from 'svelte';
  
  const statCards = [
    {
      title: 'Total Projects',
      key: 'totalProjects',
      icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10',
      color: 'text-blue-400'
    },
    {
      title: 'Active Proposals',
      key: 'activeProposals',
      icon: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
      color: 'status-connected'
    },
    {
      title: 'Companies',
      key: 'totalCompanies',
      icon: 'M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4',
      color: 'text-green-400'
    },
    {
      title: 'Contacts',
      key: 'totalContacts',
      icon: 'M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z',
      color: 'text-purple-400'
    }
  ];
</script>

<div class="p-8">
  <div class="mb-8">
    <h1 class="text-3xl font-heading font-bold text-emittiv-white mb-2">Dashboard</h1>
    <p class="text-emittiv-lighter">Welcome to your Fee Proposal Management System</p>
  </div>
  
  <!-- Stats Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
    {#each statCards as card}
      <Card className="emittiv-card hover-lift">
        {#if $isLoadingStore}
          <LoadingSkeleton rows={2} />
        {:else}
          <div class="flex items-center justify-between mb-4">
            <svg class="w-8 h-8 {card.color}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={card.icon} />
            </svg>
            <span class="text-2xl font-bold" style="color: var(--emittiv-white);">{$statisticsStore[card.key]}</span>
          </div>
          <h3 class="font-medium" style="color: var(--emittiv-lighter);">{card.title}</h3>
        {/if}
      </Card>
    {/each}
  </div>
  
  <!-- Recent Activity -->
  <Card className="emittiv-card mb-6">
    <h2 class="text-xl font-heading font-semibold mb-4" style="color: var(--emittiv-white);">Recent Activity</h2>
    {#if $isLoadingStore}
      <LoadingSkeleton rows={4} />
    {:else if $recentProposalsStore.length > 0}
      <div class="space-y-3">
        {#each $recentProposalsStore.slice(0, 5) as proposal}
          <div class="flex items-center space-x-3 py-2">
            <div class="w-2 h-2 rounded-full status-connected"></div>
            <span style="color: var(--emittiv-lighter);">Proposal "{proposal.title}" for {proposal.client}</span>
            <span class="text-sm ml-auto" style="color: var(--emittiv-light);">{proposal.createdAt.toLocaleDateString()}</span>
          </div>
        {/each}
      </div>
    {:else}
      <div class="text-center py-8">
        <svg class="w-12 h-12 mx-auto mb-4" style="color: var(--emittiv-light);" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
        </svg>
        <p style="color: var(--emittiv-light);">No recent activity</p>
      </div>
    {/if}
  </Card>
</div>