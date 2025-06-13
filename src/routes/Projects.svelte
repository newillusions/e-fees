<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { projects } from '$lib/stores/data';
  
  function handleNewProject() {
    console.log('New project clicked');
    // In a real app, this would open a modal or navigate to a form
  }
  
  function getStatusColor(status: string) {
    switch (status) {
      case 'active': return 'text-green-400 bg-green-400/10';
      case 'completed': return 'text-blue-400 bg-blue-400/10';
      case 'on-hold': return 'text-yellow-400 bg-yellow-400/10';
      default: return 'text-emittiv-light bg-emittiv-dark';
    }
  }
</script>

<div class="p-8">
  <div class="flex justify-between items-center mb-8">
    <div>
      <h1 class="text-3xl font-heading font-bold text-emittiv-white mb-2">Projects</h1>
      <p class="text-emittiv-lighter">Manage your projects and track progress</p>
    </div>
    <Button variant="primary" on:click={handleNewProject}>
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      New Project
    </Button>
  </div>
  
  {#if $projects.length === 0}
    <EmptyState 
      icon="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
      title="No Projects Yet"
      description="Create your first project to get started with managing your proposals and tracking progress."
      actionText="Create Project"
      onAction={handleNewProject}
    />
  {:else}
    <div class="grid gap-6">
      {#each $projects as project}
        <Card>
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center space-x-3 mb-2">
                <h3 class="text-lg font-semibold text-emittiv-white">{project.name}</h3>
                <span class="px-2 py-1 rounded-full text-xs font-medium {getStatusColor(project.status)}">
                  {project.status}
                </span>
              </div>
              <p class="text-emittiv-lighter mb-3">{project.description}</p>
              <div class="flex items-center space-x-4 text-sm text-emittiv-light">
                <span>Created: {project.createdAt.toLocaleDateString()}</span>
                <span>Updated: {project.updatedAt.toLocaleDateString()}</span>
              </div>
            </div>
            <div class="flex space-x-2 ml-4">
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
        </Card>
      {/each}
    </div>
  {/if}
</div>