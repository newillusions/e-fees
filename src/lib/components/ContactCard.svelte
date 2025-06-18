<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import BaseListCard from './BaseListCard.svelte';
  import ActionButton from './ActionButton.svelte';
  import type { Contact } from '../../types';
  
  const dispatch = createEventDispatcher();
  
  export let contact: Contact;
  export let clickable = true;
  export let companyName: string = '';
  
  function handleCardClick() {
    if (clickable) {
      dispatch('view', contact);
    }
  }
  
  function handleEdit(event: Event) {
    event.stopPropagation();
    dispatch('edit', contact);
  }
  
  function handleView(event: Event) {
    event.stopPropagation();
    dispatch('view', contact);
  }
</script>

<BaseListCard {clickable} on:click={handleCardClick}>
  <!-- Title -->
  <svelte:fragment slot="title">
    <h3 class="text-base font-medium text-emittiv-white group-hover:text-emittiv-splash transition-colors truncate">
      {contact.full_name}
    </h3>
  </svelte:fragment>
  
  <!-- Subtitle -->
  <svelte:fragment slot="subtitle">
    <p class="text-sm text-emittiv-lighter">
      {#if companyName}
        {companyName}
      {/if}
    </p>
  </svelte:fragment>
  
  
  <!-- Badge -->
  <svelte:fragment slot="badge">
    {#if contact.position}
      <span class="px-2 py-1 rounded-lg text-xs font-medium text-purple-400 bg-purple-400/10">
        {contact.position}
      </span>
    {/if}
  </svelte:fragment>
  
  <!-- Actions -->
  <svelte:fragment slot="actions">
    <ActionButton 
      type="edit" 
      ariaLabel="Edit contact"
      on:click={handleEdit}
    />
    <ActionButton 
      type="view" 
      ariaLabel="View contact details"
      on:click={handleView}
    />
  </svelte:fragment>
  
  <!-- Extra - Full width phone and email -->
  <svelte:fragment slot="extra">
    <div class="flex text-xs text-emittiv-light">
      <div class="flex items-center gap-1" style="width: 180px;">
        <ActionButton 
          type="phone" 
          href="tel:{contact.phone || '#'}" 
          ariaLabel="Call {contact.full_name}"
          size={14}
          class={!contact.phone ? 'opacity-30' : ''}
        />
        <span>{contact.phone || '+000 000 000 000'}</span>
      </div>
      <div class="flex items-center gap-1 flex-1">
        <ActionButton 
          type="email" 
          href="mailto:{contact.email}" 
          ariaLabel="Email {contact.full_name}"
          size={14}
        />
        <span>{contact.email}</span>
      </div>
    </div>
  </svelte:fragment>
</BaseListCard>