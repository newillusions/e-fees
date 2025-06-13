<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { contacts } from '$lib/stores/data';
  
  function handleAddContact() {
    console.log('Add contact clicked');
  }
</script>

<div class="p-8">
  <div class="flex justify-between items-center mb-8">
    <div>
      <h1 class="text-3xl font-heading font-bold text-emittiv-white mb-2">Contacts</h1>
      <p class="text-emittiv-lighter">Manage your contacts and client relationships</p>
    </div>
    <Button variant="primary" on:click={handleAddContact}>
      <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      Add Contact
    </Button>
  </div>
  
  {#if $contacts.length === 0}
    <EmptyState 
      icon="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"
      title="No Contacts Yet"
      description="Add your first contact to start building your professional network and managing client relationships."
      actionText="Add Contact"
      onAction={handleAddContact}
    />
  {:else}
    <div class="grid gap-4">
      {#each $contacts as contact}
        <Card>
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-4">
              <div class="w-12 h-12 bg-emittiv-splash/20 rounded-full flex items-center justify-center">
                <span class="text-emittiv-splash font-semibold text-lg">
                  {contact.name.split(' ').map(n => n[0]).join('')}
                </span>
              </div>
              <div>
                <h3 class="text-lg font-semibold text-emittiv-white">{contact.name}</h3>
                <p class="text-emittiv-lighter">{contact.email}</p>
                <div class="flex items-center space-x-3 text-sm text-emittiv-light mt-1">
                  {#if contact.position}
                    <span>{contact.position}</span>
                    <span>•</span>
                  {/if}
                  <span>{contact.company}</span>
                  {#if contact.phone}
                    <span>•</span>
                    <span>{contact.phone}</span>
                  {/if}
                </div>
              </div>
            </div>
            <div class="flex space-x-2">
              <Button variant="ghost" size="sm">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 7.89a2 2 0 002.83 0L21 9M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                </svg>
              </Button>
              <Button variant="ghost" size="sm">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                </svg>
              </Button>
              <Button variant="ghost" size="sm" aria-label="View contact details">
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