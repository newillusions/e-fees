<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { contactsStore, contactsActions, rfpsStore, rfpsActions, projectsStore, projectsActions } from '$lib/stores';
  import { onMount } from 'svelte';
  import type { Company, Contact, Rfp, Project } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let company: Company | null = null;
  
  // Filter contacts for this company
  $: companyContacts = company ? $contactsStore.filter(contact => {
    if (!contact.company || !company?.id) return false;
    
    let contactCompanyId = '';
    if (typeof contact.company === 'string') {
      contactCompanyId = contact.company;
    } else if (contact.company && typeof contact.company === 'object') {
      if ((contact.company as any).tb && (contact.company as any).id) {
        if (typeof (contact.company as any).id === 'string') {
          contactCompanyId = `${(contact.company as any).tb}:${(contact.company as any).id}`;
        } else if ((contact.company as any).id.String) {
          contactCompanyId = `${(contact.company as any).tb}:${(contact.company as any).id.String}`;
        }
      }
    }
    
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
    
    const id1 = contactCompanyId.replace('company:', '');
    const id2 = companyIdStr.replace('company:', '');
    return id1 === id2 || contactCompanyId === companyIdStr;
  }) : [];
  
  // Filter RFPs for this company
  $: companyRfps = company ? $rfpsStore.filter(rfp => {
    if (!rfp.company_id || !company?.id) return false;
    
    let rfpCompanyId = '';
    if (typeof rfp.company_id === 'string') {
      rfpCompanyId = rfp.company_id;
    } else if (rfp.company_id && typeof rfp.company_id === 'object') {
      if ((rfp.company_id as any).tb && (rfp.company_id as any).id) {
        if (typeof (rfp.company_id as any).id === 'string') {
          rfpCompanyId = `${(rfp.company_id as any).tb}:${(rfp.company_id as any).id}`;
        } else if ((rfp.company_id as any).id.String) {
          rfpCompanyId = `${(rfp.company_id as any).tb}:${(rfp.company_id as any).id.String}`;
        }
      }
    }
    
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
    
    const id1 = rfpCompanyId.replace('company:', '');
    const id2 = companyIdStr.replace('company:', '');
    return id1 === id2 || rfpCompanyId === companyIdStr;
  }).sort((a, b) => new Date(b.time.created_at).getTime() - new Date(a.time.created_at).getTime()) : [];

  // Filter projects related to this company (through RFPs)
  $: companyProjects = company ? $projectsStore.filter(project => {
    return companyRfps.some(rfp => {
      if (!rfp.project_id || !project?.id) return false;
      
      let rfpProjectId = '';
      if (typeof rfp.project_id === 'string') {
        rfpProjectId = rfp.project_id;
      } else if (rfp.project_id && typeof rfp.project_id === 'object') {
        if ((rfp.project_id as any).tb && (rfp.project_id as any).id) {
          if (typeof (rfp.project_id as any).id === 'string') {
            rfpProjectId = `${(rfp.project_id as any).tb}:${(rfp.project_id as any).id}`;
          } else if ((rfp.project_id as any).id.String) {
            rfpProjectId = `${(rfp.project_id as any).tb}:${(rfp.project_id as any).id.String}`;
          }
        }
      }
      
      let projectIdStr = '';
      if (typeof project.id === 'string') {
        projectIdStr = project.id;
      } else if (project.id && typeof project.id === 'object') {
        if ((project.id as any).tb && (project.id as any).id) {
          if (typeof (project.id as any).id === 'string') {
            projectIdStr = `${(project.id as any).tb}:${(project.id as any).id}`;
          } else if ((project.id as any).id.String) {
            projectIdStr = `${(project.id as any).tb}:${(project.id as any).id.String}`;
          }
        }
      }
      
      const id1 = rfpProjectId.replace('projects:', '');
      const id2 = projectIdStr.replace('projects:', '');
      return id1 === id2 || rfpProjectId === projectIdStr;
    });
  }).sort((a, b) => new Date(b.time.updated_at).getTime() - new Date(a.time.updated_at).getTime()) : [];

  // Load all related data when component mounts
  onMount(() => {
    contactsActions.load();
    rfpsActions.load();
    projectsActions.load();
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
    class="fixed inset-0 bg-black bg-opacity-50 backdrop-blur-sm z-40 transition-opacity"
    on:click={closeDetail}
    on:keydown={(e) => e.key === 'Escape' && closeDetail()}
    role="button"
    tabindex="-1"
    aria-label="Close detail view"
  ></div>
  
  <!-- Sliding Panel -->
  <div class="fixed top-0 right-0 h-full bg-emittiv-black z-50 overflow-hidden slide-in-right shadow-2xl" style="width: 600px;">
    <!-- Header Section with Gradient Background -->
    <div class="relative bg-gradient-to-br from-emittiv-darker to-emittiv-black border-b border-emittiv-dark">
      <!-- Close Button -->
      <button 
        on:click={closeDetail}
        class="absolute top-6 right-6 p-2 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-darker/50 transition-all"
        aria-label="Close detail view"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
      
      <!-- Company Header Info -->
      <div class="px-8 pt-8 pb-6">
        <div class="flex items-start gap-4">
          <!-- Company Avatar/Icon -->
          <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-emittiv-splash/20 to-emittiv-splash/10 border border-emittiv-splash/30 flex items-center justify-center flex-shrink-0">
            <span class="text-2xl font-bold text-emittiv-splash">
              {company.abbreviation.substring(0, 2).toUpperCase()}
            </span>
          </div>
          
          <!-- Company Details -->
          <div class="flex-1 min-w-0">
            <h1 class="text-2xl font-heading font-bold text-emittiv-white mb-1">
              {company.name}
            </h1>
            <p class="text-sm text-emittiv-lighter mb-3">
              {company.name_short} • {company.abbreviation}
            </p>
            <div class="flex items-center gap-3 text-sm">
              <div class="flex items-center gap-1.5 text-emittiv-light">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
                <span>{company.city}, {company.country}</span>
              </div>
              <button 
                on:click={handleEdit}
                class="flex items-center gap-1.5 px-3 py-1 rounded-lg text-emittiv-splash hover:bg-emittiv-splash/10 transition-all"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                </svg>
                <span>Edit</span>
              </button>
            </div>
          </div>
        </div>
      </div>
      
      <!-- Quick Stats Bar -->
      <div class="px-8 pb-6">
        <div class="grid grid-cols-3 gap-4">
          <div class="text-center">
            <div class="text-2xl font-bold text-emittiv-white">{companyProjects.length}</div>
            <div class="text-xs text-emittiv-light uppercase tracking-wider">Projects</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold text-emittiv-white">{companyRfps.length}</div>
            <div class="text-xs text-emittiv-light uppercase tracking-wider">Proposals</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold text-emittiv-white">{companyContacts.length}</div>
            <div class="text-xs text-emittiv-light uppercase tracking-wider">Contacts</div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Scrollable Content Area -->
    <div class="h-full overflow-y-auto pb-32" style="height: calc(100% - 220px);">
      <div class="px-8 py-6 space-y-8">
        
        <!-- Company Details Section -->
        {#if company.reg_no || company.tax_no}
          <section>
            <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider mb-4">Registration Details</h2>
            <div class="bg-emittiv-black rounded-xl p-5 border border-emittiv-dark">
              <div class="grid grid-cols-2 gap-4">
                {#if company.reg_no}
                  <div>
                    <div class="text-xs text-emittiv-light mb-1">Registration Number</div>
                    <div class="text-sm text-emittiv-white font-medium">{company.reg_no}</div>
                  </div>
                {/if}
                {#if company.tax_no}
                  <div>
                    <div class="text-xs text-emittiv-light mb-1">Tax Number</div>
                    <div class="text-sm text-emittiv-white font-medium">{company.tax_no}</div>
                  </div>
                {/if}
              </div>
            </div>
          </section>
        {/if}
        
        <!-- Projects Section -->
        <section>
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider">Active Projects</h2>
            <span class="text-xs text-emittiv-light bg-emittiv-darker px-2 py-1 rounded-full">
              {companyProjects.length} total
            </span>
          </div>
          
          {#if companyProjects.length === 0}
            <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
              <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
              <p class="text-emittiv-light text-sm">No projects yet</p>
            </div>
          {:else}
            <div class="grid gap-3">
              {#each companyProjects.slice(0, 3) as project}
                <div class="group bg-emittiv-black hover:bg-emittiv-black/80 rounded-xl p-4 border border-emittiv-dark hover:border-emittiv-splash/30 transition-all cursor-pointer">
                  <div class="flex items-start justify-between gap-3">
                    <div class="flex-1 min-w-0">
                      <h3 class="font-medium text-emittiv-white group-hover:text-emittiv-splash transition-colors truncate">
                        {project.name_short}
                      </h3>
                      <p class="text-sm text-emittiv-lighter mt-1">{project.area} • {project.city}</p>
                    </div>
                    <div class="flex items-center gap-2 flex-shrink-0">
                      <span class="px-2 py-1 rounded-lg text-xs font-medium {project.status === 'Active' ? 'bg-green-500/10 text-green-400' : 'bg-emittiv-dark text-emittiv-light'}">
                        {project.status}
                      </span>
                      <span class="text-xs text-emittiv-light font-mono">
                        {project.number?.id}
                      </span>
                    </div>
                  </div>
                </div>
              {/each}
              {#if companyProjects.length > 3}
                <button class="text-sm text-emittiv-splash hover:text-orange-400 transition-colors text-center py-2">
                  View all {companyProjects.length} projects →
                </button>
              {/if}
            </div>
          {/if}
        </section>
        
        <!-- RFPs Section -->
        <section>
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider">Fee Proposals</h2>
            <span class="text-xs text-emittiv-light bg-emittiv-darker px-2 py-1 rounded-full">
              {companyRfps.length} total
            </span>
          </div>
          
          {#if companyRfps.length === 0}
            <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
              <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <p class="text-emittiv-light text-sm">No proposals yet</p>
            </div>
          {:else}
            <div class="grid gap-3">
              {#each companyRfps.slice(0, 3) as rfp}
                <div class="group bg-emittiv-black hover:bg-emittiv-black/80 rounded-xl p-4 border border-emittiv-dark hover:border-emittiv-splash/30 transition-all cursor-pointer">
                  <div class="flex items-start justify-between gap-3">
                    <div class="flex-1 min-w-0">
                      <h3 class="font-medium text-emittiv-white group-hover:text-emittiv-splash transition-colors">
                        {rfp.number}
                      </h3>
                      <p class="text-sm text-emittiv-lighter mt-1 truncate">{rfp.name}</p>
                      {#if rfp.activity}
                        <p class="text-xs text-emittiv-light mt-2">{rfp.activity}</p>
                      {/if}
                    </div>
                    <div class="flex flex-col items-end gap-2">
                      <span class="px-2 py-1 rounded-lg text-xs font-medium {rfp.status === 'Awarded' ? 'bg-green-500/10 text-green-400' : rfp.status === 'Lost' ? 'bg-red-500/10 text-red-400' : rfp.status === 'Sent' ? 'bg-blue-500/10 text-blue-400' : 'bg-emittiv-dark text-emittiv-light'}">
                        {rfp.status}
                      </span>
                      <span class="text-xs text-emittiv-light">Rev {rfp.rev}</span>
                    </div>
                  </div>
                  <div class="flex items-center justify-between mt-3 pt-3 border-t border-emittiv-dark">
                    <span class="text-xs text-emittiv-light">
                      {new Date(rfp.issue_date.length === 6 ? `20${rfp.issue_date.substring(0,2)}-${rfp.issue_date.substring(2,4)}-${rfp.issue_date.substring(4,6)}` : rfp.issue_date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}
                    </span>
                    {#if rfp.staff_name}
                      <span class="text-xs text-emittiv-light">{rfp.staff_name}</span>
                    {/if}
                  </div>
                </div>
              {/each}
              {#if companyRfps.length > 3}
                <button class="text-sm text-emittiv-splash hover:text-orange-400 transition-colors text-center py-2">
                  View all {companyRfps.length} proposals →
                </button>
              {/if}
            </div>
          {/if}
        </section>
        
        <!-- Contacts Section -->
        <section>
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider">Contacts</h2>
            <span class="text-xs text-emittiv-light bg-emittiv-darker px-2 py-1 rounded-full">
              {companyContacts.length} total
            </span>
          </div>
          
          {#if companyContacts.length === 0}
            <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
              <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
              </svg>
              <p class="text-emittiv-light text-sm">No contacts yet</p>
            </div>
          {:else}
            <div class="grid gap-3">
              {#each companyContacts as contact}
                <div class="group bg-emittiv-black hover:bg-emittiv-black/80 rounded-xl p-4 border border-emittiv-dark hover:border-emittiv-splash/30 transition-all">
                  <div class="flex items-center justify-between gap-3">
                    <div class="flex items-center gap-3 flex-1 min-w-0">
                      <!-- Contact Avatar -->
                      <div class="w-10 h-10 rounded-full bg-gradient-to-br from-purple-500/20 to-purple-500/10 border border-purple-500/30 flex items-center justify-center flex-shrink-0">
                        <span class="text-sm font-medium text-purple-400">
                          {contact.first_name.charAt(0)}{contact.last_name.charAt(0)}
                        </span>
                      </div>
                      <div class="flex-1 min-w-0">
                        <h3 class="font-medium text-emittiv-white">{contact.full_name}</h3>
                        <p class="text-sm text-emittiv-lighter">{contact.position}</p>
                      </div>
                    </div>
                    <div class="flex items-center gap-2">
                      <a 
                        href="mailto:{contact.email}" 
                        class="p-2 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-all"
                        aria-label="Email {contact.full_name}"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 7.89a2 2 0 002.83 0L21 9M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                        </svg>
                      </a>
                      <a 
                        href="tel:{contact.phone}" 
                        class="p-2 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-all"
                        aria-label="Call {contact.full_name}"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z" />
                        </svg>
                      </a>
                    </div>
                  </div>
                  <div class="flex items-center gap-4 mt-3 text-xs text-emittiv-light">
                    <span class="flex items-center gap-1">
                      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 7.89a2 2 0 002.83 0L21 9M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                      </svg>
                      {contact.email}
                    </span>
                    <span class="flex items-center gap-1">
                      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z" />
                      </svg>
                      {contact.phone}
                    </span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </section>
        
        <!-- Metadata Section -->
        <section class="border-t border-emittiv-dark pt-8">
          <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider mb-4">Information</h2>
          <div class="bg-emittiv-black/50 rounded-xl p-5 border border-emittiv-dark/50">
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div>
                <div class="text-xs text-emittiv-light mb-1">Created</div>
                <div class="text-emittiv-lighter">
                  {new Date(company.time.created_at).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })}
                </div>
              </div>
              <div>
                <div class="text-xs text-emittiv-light mb-1">Last Updated</div>
                <div class="text-emittiv-lighter">
                  {new Date(company.time.updated_at).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })}
                </div>
              </div>
              <div class="col-span-2">
                <div class="text-xs text-emittiv-light mb-1">Record ID</div>
                <div class="text-emittiv-light font-mono text-xs opacity-50">{company.id}</div>
              </div>
            </div>
          </div>
        </section>
        
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
  
  /* Custom scrollbar for dark theme */
  .overflow-y-auto::-webkit-scrollbar {
    width: 8px;
  }
  
  .overflow-y-auto::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
  }
  
  .overflow-y-auto::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
  }
  
  .overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }
</style>