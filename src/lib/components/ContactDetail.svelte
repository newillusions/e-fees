<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { feesStore, feesActions, companiesStore, companiesActions } from '$lib/stores';
  import { onMount } from 'svelte';
  import { extractId } from '$lib/utils';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import DetailPanel from './DetailPanel.svelte';
  import DetailHeader from './DetailHeader.svelte';
  import InfoCard from './InfoCard.svelte';
  import ListCard from './ListCard.svelte';
  import StatusBadge from './StatusBadge.svelte';
  import ActionButton from './ActionButton.svelte';
  import type { Contact, Fee, Company } from '../../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let contact: Contact | null = null;
  
  // Create optimized company lookup
  $: companyLookup = createCompanyLookup($companiesStore);
  
  // Find related company
  $: relatedCompany = contact ? companyLookup.getCompany(contact.company) : null;
  
  // Filter Fees where this contact is involved
  $: contactFees = contact ? $feesStore.filter(fee => {
    if (!fee.contact_id || !contact?.id) return false;
    
    let feeContactId = '';
    if (typeof fee.contact_id === 'string') {
      feeContactId = fee.contact_id;
    } else if (fee.contact_id && typeof fee.contact_id === 'object') {
      if ((fee.contact_id as any).tb && (fee.contact_id as any).id) {
        if (typeof (fee.contact_id as any).id === 'string') {
          feeContactId = `${(fee.contact_id as any).tb}:${(fee.contact_id as any).id}`;
        } else if ((fee.contact_id as any).id.String) {
          feeContactId = `${(fee.contact_id as any).tb}:${(fee.contact_id as any).id.String}`;
        }
      }
    }
    
    let contactIdStr = '';
    if (typeof contact.id === 'string') {
      contactIdStr = contact.id;
    } else if (contact.id && typeof contact.id === 'object') {
      if ((contact.id as any).tb && (contact.id as any).id) {
        if (typeof (contact.id as any).id === 'string') {
          contactIdStr = `${(contact.id as any).tb}:${(contact.id as any).id}`;
        } else if ((contact.id as any).id.String) {
          contactIdStr = `${(contact.id as any).tb}:${(contact.id as any).id.String}`;
        }
      }
    }
    
    const id1 = feeContactId.replace('contacts:', '');
    const id2 = contactIdStr.replace('contacts:', '');
    return id1 === id2 || feeContactId === contactIdStr;
  }).sort((a, b) => {
    // Parse issue_date for proper sorting
    const parseIssueDate = (dateStr: string) => {
      if (dateStr.length === 6) {
        return new Date(`20${dateStr.substring(0,2)}-${dateStr.substring(2,4)}-${dateStr.substring(4,6)}`);
      }
      return new Date(dateStr);
    };
    
    return parseIssueDate(b.issue_date).getTime() - parseIssueDate(a.issue_date).getTime();
  }) : [];

  // Load related data when component mounts
  onMount(() => {
    feesActions.load();
    companiesActions.load();
  });
  
  function handleEdit() {
    dispatch('edit', contact);
  }
  
  function handleClose() {
    dispatch('close');
  }
  
  // Generate initials for avatar
  $: initials = contact ? contact.full_name.split(' ').map(n => n[0]).join('').substring(0, 2).toUpperCase() : '';
</script>

<DetailPanel 
  {isOpen}
  show={!!contact}
  title="contact"
  on:edit={handleEdit}
  on:close={handleClose}
>
  <svelte:fragment slot="header">
    {#if contact}
      <DetailHeader 
        name={contact.full_name}
        subtitle="{contact.position || 'Contact'}{relatedCompany ? ` • ${relatedCompany.name}` : ''}"
        location="{relatedCompany?.city || 'Unknown'}, {relatedCompany?.country || 'Unknown'}"
        stats={[
          { label: 'Proposals', value: contactFees.length },
          { label: 'Awarded', value: contactFees.filter(fee => fee.status === 'Awarded').length },
          { label: 'Pending', value: contactFees.filter(fee => fee.status === 'Sent').length },
          { label: 'Lost', value: contactFees.filter(fee => fee.status === 'Lost').length }
        ]}
      />
    {/if}
  </svelte:fragment>
  
  <svelte:fragment slot="content">
    {#if contact}
      <!-- Contact Information Section -->
      <InfoCard 
        title="Contact Information" 
        columns={3}
        fields={[
          { label: 'Full Name', value: contact.full_name },
          { label: 'Position', value: contact.position || '—' },
          { label: 'Email', value: contact.email },
          { label: 'First Name', value: contact.first_name || '—' },
          { label: 'Last Name', value: contact.last_name || '—' },
          { label: 'Phone', value: contact.phone || '—' },
          { label: 'Created', value: contact.time.created_at, type: 'date' },
          { label: 'Last Updated', value: contact.time.updated_at, type: 'date' },
          { label: 'Record ID', value: extractId(contact.id), type: 'id' }
        ]}
      />
    
    <!-- Company Information Section -->
    {#if relatedCompany}
      <InfoCard 
        title="Company Information" 
        columns={3}
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
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider mb-2">Company Information</h2>
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
          </svg>
          <p class="text-emittiv-light text-sm">Company not found</p>
        </div>
      </section>
    {/if}
    
    <!-- Fee Proposals Section -->
    <section>
      <div class="flex items-center justify-between mb-2">
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider">Fee Proposals</h2>
        <span class="text-xs text-emittiv-light px-2 py-1 rounded-lg" style="background-color: #111;">
          {contactFees.length} total
        </span>
      </div>
      
      {#if contactFees.length === 0}
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-emittiv-light text-sm">No proposals yet</p>
        </div>
      {:else}
        <div class="grid gap-2">
          {#each contactFees as fee}
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
                  <div class="mt-2 space-y-1">
                    {#if fee.staff_name}
                      <div>
                        <h3 class="text-xs font-medium text-emittiv-light">Staff:</h3>
                        <p class="text-sm text-emittiv-white">{fee.staff_name}</p>
                      </div>
                    {/if}
                    <div class="flex items-center gap-4 text-xs text-emittiv-light">
                      <span>Rev: {fee.rev}</span>
                      <span>
                        {new Date(fee.issue_date.length === 6 ? `20${fee.issue_date.substring(0,2)}-${fee.issue_date.substring(2,4)}-${fee.issue_date.substring(4,6)}` : fee.issue_date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}
                      </span>
                    </div>
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
    {/if}
  </svelte:fragment>
</DetailPanel>