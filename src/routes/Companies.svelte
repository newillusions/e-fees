<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { companies } from '$lib/stores/data';
  
  function handleAddCompany() {
    console.log('Add company clicked');
  }
</script>

<div class="p-8">
  <div class="flex justify-between items-center mb-8">
    <div>
      <h1 class="text-3xl font-heading font-bold text-emittiv-white mb-2">Companies</h1>
      <p class="text-emittiv-lighter">Manage company information and details</p>
    </div>
    <Button variant="primary" on:click={handleAddCompany}>
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      Add Company
    </Button>
  </div>
  
  {#if $companies.length === 0}
    <EmptyState 
      icon="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"
      title="No Companies Yet"
      description="Add your first company to start organizing your clients and managing relationships."
      actionText="Add Company"
      onAction={handleAddCompany}
    />
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each $companies as company}
        <Card>
          <div class="flex items-start justify-between mb-4">
            <div class="flex-1">
              <h3 class="text-lg font-semibold text-emittiv-white mb-1">{company.name}</h3>
              <p class="text-emittiv-lighter mb-2">{company.industry}</p>
              {#if company.website}
                <a 
                  href={company.website} 
                  target="_blank" 
                  rel="noopener noreferrer" 
                  class="text-emittiv-splash hover:text-orange-600 text-sm transition-smooth"
                >
                  Visit Website
                </a>
              {/if}
            </div>
            <Button variant="ghost" size="sm">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
              </svg>
            </Button>
          </div>
          
          <div class="flex items-center justify-between text-sm text-emittiv-light pt-4 border-t border-emittiv-dark">
            <span>{company.contactCount} contacts</span>
            <span>Added {company.createdAt.toLocaleDateString()}</span>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>