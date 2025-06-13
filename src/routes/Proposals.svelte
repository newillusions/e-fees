<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { proposals } from '$lib/stores/data';
  
  function handleNewProposal() {
    console.log('New proposal clicked');
  }
  
  function getStatusColor(status: string) {
    switch (status) {
      case 'draft': return 'text-yellow-400 bg-yellow-400/10';
      case 'sent': return 'text-blue-400 bg-blue-400/10';
      case 'approved': return 'text-green-400 bg-green-400/10';
      case 'rejected': return 'text-red-400 bg-red-400/10';
      default: return 'text-emittiv-light bg-emittiv-dark';
    }
  }
  
  function formatCurrency(amount: number) {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0
    }).format(amount);
  }
</script>

<div class="p-8">
  <div class="flex justify-between items-center mb-8">
    <div>
      <h1 class="text-3xl font-heading font-bold text-emittiv-white mb-2">Proposals</h1>
      <p class="text-emittiv-lighter">Create and manage fee proposals</p>
    </div>
    <Button variant="primary" on:click={handleNewProposal}>
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      New Proposal
    </Button>
  </div>
  
  {#if $proposals.length === 0}
    <EmptyState 
      icon="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
      title="No Proposals Yet"
      description="Create your first proposal to start sending professional fee estimates to your clients."
      actionText="Create Proposal"
      onAction={handleNewProposal}
    />
  {:else}
    <div class="grid gap-6">
      {#each $proposals as proposal}
        <Card>
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center space-x-3 mb-2">
                <h3 class="text-lg font-semibold text-emittiv-white">{proposal.title}</h3>
                <span class="px-2 py-1 rounded-full text-xs font-medium {getStatusColor(proposal.status)}">
                  {proposal.status}
                </span>
              </div>
              <p class="text-emittiv-lighter mb-1">{proposal.client}</p>
              <div class="flex items-center space-x-4 text-sm text-emittiv-light">
                <span>Created: {proposal.createdAt.toLocaleDateString()}</span>
                {#if proposal.dueDate}
                  <span>Due: {proposal.dueDate.toLocaleDateString()}</span>
                {/if}
              </div>
            </div>
            <div class="text-right">
              <div class="text-2xl font-bold text-emittiv-splash mb-2">
                {formatCurrency(proposal.amount)}
              </div>
              <div class="flex space-x-2">
                <Button variant="ghost" size="sm">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                  </svg>
                </Button>
                <Button variant="ghost" size="sm">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                </Button>
              </div>
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>