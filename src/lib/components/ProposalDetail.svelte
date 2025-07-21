<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { projectsStore, projectsActions, companiesStore, companiesActions, contactsStore, contactsActions } from '$lib/stores';
  import { onMount } from 'svelte';
  import { extractId } from '$lib/utils';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import DetailPanel from './DetailPanel.svelte';
  import DetailHeader from './DetailHeader.svelte';
  import InfoCard from './InfoCard.svelte';
  import type { FeeProposal, Project, Company, Contact } from '../../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let proposal: FeeProposal | null = null;
  
  // Create optimized company lookup
  $: companyLookup = createCompanyLookup($companiesStore);
  
  // Find related project
  $: relatedProject = proposal ? $projectsStore.find(p => {
    if (!proposal.project_id || !p?.id) return false;
    
    let projectIdStr = '';
    if (typeof p.id === 'string') {
      projectIdStr = p.id;
    } else if (p.id && typeof p.id === 'object') {
      if ((p.id as any).tb && (p.id as any).id) {
        if (typeof (p.id as any).id === 'string') {
          projectIdStr = `${(p.id as any).tb}:${(p.id as any).id}`;
        } else if ((p.id as any).id.String) {
          projectIdStr = `${(p.id as any).tb}:${(p.id as any).id.String}`;
        }
      }
    }
    
    let rfpProjectId = '';
    if (typeof proposal.project_id === 'string') {
      rfpProjectId = proposal.project_id;
    } else if (proposal.project_id && typeof proposal.project_id === 'object') {
      if ((proposal.project_id as any).tb && (proposal.project_id as any).id) {
        if (typeof (proposal.project_id as any).id === 'string') {
          rfpProjectId = `${(proposal.project_id as any).tb}:${(proposal.project_id as any).id}`;
        } else if ((proposal.project_id as any).id.String) {
          rfpProjectId = `${(proposal.project_id as any).tb}:${(proposal.project_id as any).id.String}`;
        }
      }
    }
    
    const id1 = rfpProjectId.replace('projects:', '');
    const id2 = projectIdStr.replace('projects:', '');
    return id1 === id2 || rfpProjectId === projectIdStr;
  }) : null;
  
  // Find related company
  $: relatedCompany = proposal ? companyLookup.getCompany(proposal.company_id) : null;
  
  // Find related contact
  $: relatedContact = proposal ? $contactsStore.find(c => {
    if (!proposal.contact_id || !c?.id) return false;
    
    let contactIdStr = '';
    if (typeof c.id === 'string') {
      contactIdStr = c.id;
    } else if (c.id && typeof c.id === 'object') {
      if ((c.id as any).tb && (c.id as any).id) {
        if (typeof (c.id as any).id === 'string') {
          contactIdStr = `${(c.id as any).tb}:${(c.id as any).id}`;
        } else if ((c.id as any).id.String) {
          contactIdStr = `${(c.id as any).tb}:${(c.id as any).id.String}`;
        }
      }
    }
    
    let rfpContactId = '';
    if (typeof proposal.contact_id === 'string') {
      rfpContactId = proposal.contact_id;
    } else if (proposal.contact_id && typeof proposal.contact_id === 'object') {
      if ((proposal.contact_id as any).tb && (proposal.contact_id as any).id) {
        if (typeof (proposal.contact_id as any).id === 'string') {
          rfpContactId = `${(proposal.contact_id as any).tb}:${(proposal.contact_id as any).id}`;
        } else if ((proposal.contact_id as any).id.String) {
          rfpContactId = `${(proposal.contact_id as any).tb}:${(proposal.contact_id as any).id.String}`;
        }
      }
    }
    
    const id1 = rfpContactId.replace('contacts:', '');
    const id2 = contactIdStr.replace('contacts:', '');
    return id1 === id2 || rfpContactId === contactIdStr;
  }) : null;

  // Load related data when component mounts
  onMount(() => {
    projectsActions.load();
    companiesActions.load();
    contactsActions.load();
  });
  
  function handleEdit() {
    dispatch('edit', proposal);
  }
  
  function handleClose() {
    dispatch('close');
  }
  
  function formatIssueDate(dateStr: string): string {
    if (dateStr.length === 6) {
      return new Date(`20${dateStr.substring(0,2)}-${dateStr.substring(2,4)}-${dateStr.substring(4,6)}`).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
    }
    return new Date(dateStr).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }
</script>

{#if proposal}
<DetailPanel 
  bind:isOpen 
  title="proposal"
  on:edit={handleEdit}
  on:close={handleClose}
>
  <svelte:fragment slot="header">
    <DetailHeader 
      name="{proposal.number} - {proposal.name}"
      subtitle="{relatedProject?.name || 'Unknown Project'}{proposal.package ? ` • ${proposal.package}` : ''}"
      location="{relatedCompany?.city || 'Unknown'}, {relatedCompany?.country || 'Unknown'}"
      stats={[
        { label: 'Revision', value: proposal.rev || 0 },
        { label: 'Days Active', value: Math.floor((new Date().getTime() - new Date(proposal.time.created_at).getTime()) / (1000 * 60 * 60 * 24)) }
      ]}
    />
  </svelte:fragment>
  
  <svelte:fragment slot="content">
    <!-- Proposal Information Section -->
    <InfoCard 
      title="Proposal Information" 
      columns={3}
      fields={[
        { label: 'FP Number', value: proposal.number },
        { label: 'Status', value: proposal.status },
        { label: 'Revision', value: proposal.rev.toString() },
        { label: 'Issue Date', value: formatIssueDate(proposal.issue_date) },
        { label: 'Staff Member', value: proposal.staff_name || '—' },
        { label: 'Created', value: proposal.time.created_at, type: 'date' },
        { label: 'Last Updated', value: proposal.time.updated_at, type: 'date' },
        { label: 'Record ID', value: extractId(proposal.id), type: 'id' }
      ]}
    />
    
    <!-- Project Information Section -->
    {#if relatedProject}
      <InfoCard 
        title="Related Project" 
        columns={2}
        fields={[
          { label: 'Project Number', value: relatedProject.number?.id || '—' },
          { label: 'Project Name', value: relatedProject.name },
          { label: 'Short Name', value: relatedProject.name_short },
          { label: 'Status', value: relatedProject.status },
          { label: 'Area', value: relatedProject.area },
          { label: 'Location', value: `${relatedProject.city}, ${relatedProject.country}` }
        ]}
      />
    {:else}
      <section>
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider mb-2">Related Project</h2>
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <p class="text-emittiv-light text-sm">Project not found</p>
        </div>
      </section>
    {/if}
    
    <!-- Company Information Section -->
    {#if relatedCompany}
      <InfoCard 
        title="Client Company" 
        columns={2}
        fields={[
          { label: 'Company Name', value: relatedCompany.name },
          { label: 'Short Name', value: relatedCompany.name_short },
          { label: 'Abbreviation', value: relatedCompany.abbreviation },
          { label: 'Location', value: `${relatedCompany.city}, ${relatedCompany.country}` },
          { label: 'Registration', value: relatedCompany.reg_no || '—' },
          { label: 'Tax Number', value: relatedCompany.tax_no || '—' }
        ]}
      />
    {:else}
      <section>
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider mb-2">Client Company</h2>
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
          </svg>
          <p class="text-emittiv-light text-sm">Company not found</p>
        </div>
      </section>
    {/if}
    
    <!-- Contact Information Section -->
    {#if relatedContact}
      <InfoCard 
        title="Primary Contact" 
        columns={2}
        fields={[
          { label: 'Full Name', value: relatedContact.full_name },
          { label: 'Position', value: relatedContact.position || '—' },
          { label: 'Email', value: relatedContact.email },
          { label: 'Phone', value: relatedContact.phone || '—' }
        ]}
      />
    {:else}
      <section>
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider mb-2">Primary Contact</h2>
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
          </svg>
          <p class="text-emittiv-light text-sm">Contact not found</p>
        </div>
      </section>
    {/if}
  </svelte:fragment>
</DetailPanel>
{/if}