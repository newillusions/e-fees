<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
  import { statisticsStore, isLoadingStore, recentFeesStore, loadAllData, projectsStore, companiesStore } from '$lib/stores';
  import { onMount } from 'svelte';
  import { extractId, findEntityById, getEntityDisplayName } from '$lib/utils';
  
  // Helper functions for activity display
  function getProjectName(projectId: any): string {
    const project = findEntityById($projectsStore, projectId);
    return getEntityDisplayName(project) || 'Unknown Project';
  }
  
  function getCompanyName(companyId: any): string {
    const company = findEntityById($companiesStore, companyId);
    return getEntityDisplayName(company) || 'Unknown Company';
  }
  
  
  
  const statCards = [
    {
      title: 'Total Projects',
      key: 'totalProjects',
      icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10',
      color: 'text-blue-400'
    },
    {
      title: 'Active Fees',
      key: 'activeFees',
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
            <span class="text-2xl font-bold" style="color: var(--emittiv-white);">{($statisticsStore as any)[card.key]}</span>
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
    {:else if $recentFeesStore.length > 0}
      <div class="space-y-1">
        {#each $recentFeesStore.slice(0, 5) as fee}
          <div class="flex items-center gap-2 py-1 px-2 rounded hover:bg-emittiv-darker/50 transition-colors cursor-pointer text-xs">
            <!-- Status indicator -->
            {#if fee.status === 'Awarded'}
              <div class="w-1.5 h-1.5 rounded-full bg-green-500 flex-shrink-0"></div>
            {:else if fee.status === 'Lost' || fee.status === 'Cancelled'}
              <div class="w-1.5 h-1.5 rounded-full bg-red-500 flex-shrink-0"></div>
            {:else if fee.status === 'Active' || fee.status === 'Sent'}
              <div class="w-1.5 h-1.5 rounded-full bg-blue-500 flex-shrink-0"></div>
            {:else}
              <div class="w-1.5 h-1.5 rounded-full bg-gray-500 flex-shrink-0"></div>
            {/if}
            
            <!-- Fee number -->
            <span class="text-emittiv-white font-medium">{fee.number}</span>
            
            <!-- Status badge -->
            <span class="px-1.5 py-0.5 rounded bg-emittiv-dark text-emittiv-lighter text-xs">{fee.status}</span>
            
            <!-- Project and company -->
            <span class="text-emittiv-light truncate flex-1">
              {getProjectName(fee.project_id)} • {getCompanyName(fee.company_id)}
            </span>
            
            <!-- Date -->
            <span class="text-emittiv-light flex-shrink-0">
              {new Date(fee.time.created_at).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
            </span>
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