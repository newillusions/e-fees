<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { contactsStore, contactsActions } from '$lib/stores';
  import { onMount } from 'svelte';
  import type { Company, Contact } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let company: Company | null = null;
  
  // Filter contacts for this company
  $: companyContacts = company ? $contactsStore.filter(contact => {
    if (!contact.company || !company?.id) return false;
    
    // Handle various ID formats like in the contacts page
    let contactCompanyId = '';
    if (typeof contact.company === 'string') {
      contactCompanyId = contact.company;
    } else if (contact.company && typeof contact.company === 'object') {
      // Handle Thing object formats
      if ((contact.company as any).tb && (contact.company as any).id) {
        if (typeof (contact.company as any).id === 'string') {
          contactCompanyId = `${(contact.company as any).tb}:${(contact.company as any).id}`;
        } else if ((contact.company as any).id.String) {
          contactCompanyId = `${(contact.company as any).tb}:${(contact.company as any).id.String}`;
        }
      }
    }
    
    // Compare with company ID
    let companyIdStr = '';
    if (typeof company.id === 'string') {
      companyIdStr = company.id;
    } else if (company.id && typeof company.id === 'object') {
      if ((company.id as any).tb && (company.id as any).id) {
        if (typeof (company.id as any).id === 'string') {
          companyIdStr = `${(company.id as any).tb}:${(company.id as any).id}`;
        } else if ((company.id as any).id.String) {
          companyIdStr = `${(company.id as any).tb}:${(company.id as any).id.String}`;
        }
      }
    }
    
    // Match IDs
    const id1 = contactCompanyId.replace('company:', '');
    const id2 = companyIdStr.replace('company:', '');
    return id1 === id2 || contactCompanyId === companyIdStr;
  }) : [];
  
  // Load contacts when component mounts
  onMount(() => {
    contactsActions.load();
  });
  
  function closeDetail() {
    isOpen = false;
    dispatch('close');
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeDetail();
    }
  }
  
  function handleEdit() {
    dispatch('edit', company);
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen && company}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 bg-black bg-opacity-30 z-40"
    on:click={closeDetail}
    role="button"
    tabindex="-1"
    aria-label="Close detail view"
  ></div>
  
  <!-- Sliding Panel -->
  <div 
    class="fixed top-0 right-0 h-full bg-emittiv-darker border-l border-emittiv-dark z-50 overflow-y-auto slide-in-right"
    style="width: 500px; padding: 24px;"
  >
    <!-- Header -->
    <div class="flex items-center justify-between" style="margin-bottom: 24px;">
      <div>
        <h1 class="font-semibold text-emittiv-white" style="font-size: 20px; margin-bottom: 4px;">
          {company.name}
        </h1>
        <p class="text-emittiv-light" style="font-size: 12px;">
          Company Details
        </p>
      </div>
      <div class="flex items-center" style="gap: 8px;">
        <button 
          on:click={handleEdit}
          class="p-2 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth"
          aria-label="Edit company"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
          </svg>
        </button>
        <button 
          on:click={closeDetail}
          class="p-2 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth"
          aria-label="Close detail view"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Company Information Section -->
    <div style="margin-bottom: 32px;">
      <h2 class="font-medium text-emittiv-white" style="font-size: 16px; margin-bottom: 16px;">Company Information</h2>
      
      <div class="bg-emittiv-dark rounded-lg" style="padding: 16px;">
        <div class="grid grid-cols-2" style="gap: 16px; margin-bottom: 16px;">
          <div>
            <label class="block text-emittiv-light" style="font-size: 11px; margin-bottom: 4px;">FULL NAME</label>
            <p class="text-emittiv-white" style="font-size: 13px;">{company.name}</p>
          </div>
          <div>
            <label class="block text-emittiv-light" style="font-size: 11px; margin-bottom: 4px;">SHORT NAME</label>
            <p class="text-emittiv-white" style="font-size: 13px;">{company.name_short}</p>
          </div>
        </div>
        
        <div class="grid grid-cols-2" style="gap: 16px; margin-bottom: 16px;">
          <div>
            <label class="block text-emittiv-light" style="font-size: 11px; margin-bottom: 4px;">ABBREVIATION</label>
            <div class="flex items-center" style="gap: 8px;">
              <span class="px-2 py-1 rounded-full text-xs font-medium text-blue-400 bg-blue-400/10">
                {company.abbreviation}
              </span>
            </div>
          </div>
          <div>
            <label class="block text-emittiv-light" style="font-size: 11px; margin-bottom: 4px;">CREATED</label>
            <p class="text-emittiv-white" style="font-size: 13px;">
              {new Date(company.time.created_at).toLocaleDateString()}
            </p>
          </div>
        </div>
        
        <div class="grid grid-cols-2" style="gap: 16px;">
          <div>
            <label class="block text-emittiv-light" style="font-size: 11px; margin-bottom: 4px;">CITY</label>
            <p class="text-emittiv-white" style="font-size: 13px;">{company.city}</p>
          </div>
          <div>
            <label class="block text-emittiv-light" style="font-size: 11px; margin-bottom: 4px;">COUNTRY</label>
            <p class="text-emittiv-white" style="font-size: 13px;">{company.country}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Registration Details Section -->
    {#if company.reg_no || company.tax_no}
      <div style="margin-bottom: 32px;">
        <h2 class="font-medium text-emittiv-white" style="font-size: 16px; margin-bottom: 16px;">Registration Details</h2>
        
        <div class="bg-emittiv-dark rounded-lg" style="padding: 16px;">
          <div class="grid grid-cols-1" style="gap: 16px;">
            {#if company.reg_no}
              <div>
                <label class="block text-emittiv-light" style="font-size: 11px; margin-bottom: 4px;">REGISTRATION NUMBER</label>
                <p class="text-emittiv-white" style="font-size: 13px;">{company.reg_no}</p>
              </div>
            {/if}
            {#if company.tax_no}
              <div>
                <label class="block text-emittiv-light" style="font-size: 11px; margin-bottom: 4px;">TAX NUMBER</label>
                <p class="text-emittiv-white" style="font-size: 13px;">{company.tax_no}</p>
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}

    <!-- Contacts Section -->
    <div style="margin-bottom: 32px;">
      <div class="flex items-center justify-between" style="margin-bottom: 16px;">
        <h2 class="font-medium text-emittiv-white" style="font-size: 16px;">Contacts</h2>
        <span class="px-2 py-1 rounded-full text-xs font-medium text-purple-400 bg-purple-400/10">
          {companyContacts.length} contact{companyContacts.length === 1 ? '' : 's'}
        </span>
      </div>
      
      {#if companyContacts.length === 0}
        <div class="bg-emittiv-dark rounded-lg text-center" style="padding: 24px;">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-light opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
          </svg>
          <p class="text-emittiv-light" style="font-size: 13px;">No contacts found for this company</p>
        </div>
      {:else}
        <div style="display: flex; flex-direction: column; gap: 12px;">
          {#each companyContacts as contact}
            <div class="bg-emittiv-dark rounded-lg" style="padding: 16px;">
              <div class="flex items-start justify-between" style="margin-bottom: 12px;">
                <div>
                  <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 4px;">
                    {contact.full_name}
                  </h3>
                  <p class="text-emittiv-lighter" style="font-size: 12px;">{contact.position}</p>
                </div>
                <a 
                  href="mailto:{contact.email}" 
                  class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-darker transition-smooth"
                  aria-label="Send email to {contact.full_name}"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 7.89a2 2 0 002.83 0L21 9M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                  </svg>
                </a>
              </div>
              <div style="display: flex; flex-direction: column; gap: 4px;">
                <div class="flex items-center" style="gap: 8px;">
                  <svg class="w-3 h-3 text-emittiv-light" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 7.89a2 2 0 002.83 0L21 9M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                  </svg>
                  <p class="text-emittiv-light" style="font-size: 11px;">{contact.email}</p>
                </div>
                <div class="flex items-center" style="gap: 8px;">
                  <svg class="w-3 h-3 text-emittiv-light" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z" />
                  </svg>
                  <p class="text-emittiv-light" style="font-size: 11px;">{contact.phone}</p>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Metadata Section -->
    <div>
      <h2 class="font-medium text-emittiv-white" style="font-size: 16px; margin-bottom: 16px;">Metadata</h2>
      
      <div class="bg-emittiv-dark rounded-lg" style="padding: 16px;">
        <div class="grid grid-cols-1" style="gap: 12px;">
          <div>
            <label class="block text-emittiv-light" style="font-size: 11px; margin-bottom: 4px;">LAST UPDATED</label>
            <p class="text-emittiv-white" style="font-size: 13px;">
              {new Date(company.time.updated_at).toLocaleDateString()} at {new Date(company.time.updated_at).toLocaleTimeString()}
            </p>
          </div>
          {#if company.id}
            <div>
              <label class="block text-emittiv-light" style="font-size: 11px; margin-bottom: 4px;">COMPANY ID</label>
              <p class="text-emittiv-light font-mono" style="font-size: 11px;">{company.id}</p>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .slide-in-right {
    animation: slideInRight 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }
  
  @keyframes slideInRight {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }
</style>