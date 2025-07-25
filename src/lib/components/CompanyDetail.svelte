<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { contactsStore, contactsActions, feesStore, feesActions, projectsStore, projectsActions } from '$lib/stores';
  import { onMount } from 'svelte';
  import { extractId } from '$lib/utils';
  import DetailPanel from './DetailPanel.svelte';
  import DetailHeader from './DetailHeader.svelte';
  import InfoCard from './InfoCard.svelte';
  import ListCard from './ListCard.svelte';
  import StatusBadge from './StatusBadge.svelte';
  import type { Company, Contact, Fee, Project } from '../../types';
  
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
  
  // Filter Fees for this company
  $: companyFees = company ? $feesStore.filter(fee => {
    if (!fee.company_id || !company?.id) return false;
    
    let feeCompanyId = '';
    if (typeof fee.company_id === 'string') {
      feeCompanyId = fee.company_id;
    } else if (fee.company_id && typeof fee.company_id === 'object') {
      if ((fee.company_id as any).tb && (fee.company_id as any).id) {
        if (typeof (fee.company_id as any).id === 'string') {
          feeCompanyId = `${(fee.company_id as any).tb}:${(fee.company_id as any).id}`;
        } else if ((fee.company_id as any).id.String) {
          feeCompanyId = `${(fee.company_id as any).tb}:${(fee.company_id as any).id.String}`;
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
    
    const id1 = feeCompanyId.replace('company:', '');
    const id2 = companyIdStr.replace('company:', '');
    return id1 === id2 || feeCompanyId === companyIdStr;
  }).sort((a, b) => new Date(b.time.created_at).getTime() - new Date(a.time.created_at).getTime()) : [];

  // Filter projects related to this company (through fees)
  $: companyProjects = company ? $projectsStore.filter(project => {
    return companyFees.some(fee => {
      if (!fee.project_id || !project?.id) return false;
      
      let feeProjectId = '';
      if (typeof fee.project_id === 'string') {
        feeProjectId = fee.project_id;
      } else if (fee.project_id && typeof fee.project_id === 'object') {
        if ((fee.project_id as any).tb && (fee.project_id as any).id) {
          if (typeof (fee.project_id as any).id === 'string') {
            feeProjectId = `${(fee.project_id as any).tb}:${(fee.project_id as any).id}`;
          } else if ((fee.project_id as any).id.String) {
            feeProjectId = `${(fee.project_id as any).tb}:${(fee.project_id as any).id.String}`;
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
      
      const id1 = feeProjectId.replace('projects:', '');
      const id2 = projectIdStr.replace('projects:', '');
      return id1 === id2 || feeProjectId === projectIdStr;
    });
  }).sort((a, b) => new Date(b.time.updated_at).getTime() - new Date(a.time.updated_at).getTime()) : [];

  // Load all related data when component mounts
  onMount(() => {
    contactsActions.load();
    feesActions.load();
    projectsActions.load();
  });
  
  function handleEdit() {
    dispatch('edit', company);
  }
  
  function handleClose() {
    dispatch('close');
  }
</script>

{#if company}
<DetailPanel 
  bind:isOpen 
  title="company"
  on:edit={handleEdit}
  on:close={handleClose}
>
  <svelte:fragment slot="header">
    <DetailHeader 
      name={company.name}
      subtitle="{company.name_short} • {company.abbreviation}"
      location="{company.city}, {company.country}"
      stats={[
        { label: 'Projects', value: companyProjects.length },
        { label: 'Proposals', value: companyFees.length },
        { label: 'Contacts', value: companyContacts.length }
      ]}
    />
  </svelte:fragment>
  
  <svelte:fragment slot="content">
    <!-- Company Information Section -->
    <InfoCard 
      title="Company Information" 
      columns={2}
      fields={[
        { label: 'Name', value: company.name },
        { label: 'Short Name', value: company.name_short },
        { label: 'Abbreviation', value: company.abbreviation },
        { label: 'Location', value: `${company.city}, ${company.country}` },
        { label: 'Registration Number', value: company.reg_no || '—' },
        { label: 'Tax Number', value: company.tax_no || '—' },
        { label: 'Created', value: company.time.created_at, type: 'date' },
        { label: 'Last Updated', value: company.time.updated_at, type: 'date' },
        { label: 'Record ID', value: extractId(company.id), type: 'id' }
      ]}
    />
    
    <!-- Projects Section -->
    <section>
      <div class="flex items-center justify-between mb-2">
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider">Related Projects</h2>
        <span class="text-xs text-emittiv-light px-2 py-1 rounded-lg" style="background-color: #111;">
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
        <div class="grid gap-2">
          {#each companyProjects as project}
            <ListCard clickable={false}>
              <div class="flex items-start justify-between gap-3">
                <div class="flex-1 min-w-0">
                  <h3 class="text-sm font-medium text-emittiv-white truncate">
                    {project.number?.id} - {project.name}
                  </h3>
                  <p class="text-sm text-emittiv-lighter mt-1">
                    {project.area} • {project.city}
                  </p>
                </div>
                <div class="flex items-center gap-2 flex-shrink-0">
                  <StatusBadge status={project.status} type="project" />
                </div>
              </div>
            </ListCard>
          {/each}
        </div>
      {/if}
    </section>
    
    <!-- Fee Proposals Section -->
    <section>
      <div class="flex items-center justify-between mb-2">
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider">Fee Proposals</h2>
        <span class="text-xs text-emittiv-light px-2 py-1 rounded-lg" style="background-color: #111;">
          {companyFees.length} total
        </span>
      </div>
      
      {#if companyFees.length === 0}
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-emittiv-light text-sm">No proposals yet</p>
        </div>
      {:else}
        <div class="grid gap-2">
          {#each companyFees as fee}
            <ListCard clickable={false}>
              <div class="flex items-start justify-between gap-3">
                <div class="flex-1 min-w-0">
                  <div>
                    <h3 class="text-xs font-medium text-emittiv-light">Fee Number:</h3>
                    <p class="text-sm text-emittiv-white">{fee.number}</p>
                  </div>
                  <div class="mt-2">
                    <h3 class="text-xs font-medium text-emittiv-light">Proposal Name:</h3>
                    <p class="text-sm text-emittiv-lighter">{fee.name}{#if fee.package} - {fee.package}{/if}</p>
                  </div>
                  <div class="mt-2 flex items-center gap-4 text-xs text-emittiv-light">
                    <span>Rev: {fee.rev}</span>
                    {#if fee.staff_name}
                      <span>Staff: {fee.staff_name}</span>
                    {/if}
                    <span>
                      {new Date(fee.issue_date.length === 6 ? `20${fee.issue_date.substring(0,2)}-${fee.issue_date.substring(2,4)}-${fee.issue_date.substring(4,6)}` : fee.issue_date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}
                    </span>
                  </div>
                </div>
                <div class="flex items-center gap-2 flex-shrink-0">
                  <StatusBadge status={fee.status} type="proposal" />
                </div>
              </div>
            </ListCard>
          {/each}
        </div>
      {/if}
    </section>
    
    <!-- Contacts Section -->
    <section>
      <div class="flex items-center justify-between mb-2">
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider">Contacts</h2>
        <span class="text-xs text-emittiv-light px-2 py-1 rounded-lg" style="background-color: #111;">
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
        <div class="grid gap-2">
          {#each companyContacts as contact}
            <ListCard clickable={false}>
              <div class="flex items-start justify-between gap-3">
                <div class="flex-1 min-w-0">
                  <h3 class="text-sm font-medium text-emittiv-white">
                    {contact.full_name}
                  </h3>
                  <p class="text-sm text-emittiv-lighter">
                    {contact.position}
                  </p>
                  <div class="flex items-center gap-4 mt-1 text-xs text-emittiv-light">
                    <span>{contact.email}</span>
                    <span>{contact.phone}</span>
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
            </ListCard>
          {/each}
        </div>
      {/if}
    </section>
  </svelte:fragment>
</DetailPanel>
{/if}