<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { rfpsStore, rfpsActions, companiesStore, companiesActions } from '$lib/stores';
  import { onMount } from 'svelte';
  import { extractId } from '$lib/utils';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import DetailPanel from './DetailPanel.svelte';
  import DetailHeader from './DetailHeader.svelte';
  import InfoCard from './InfoCard.svelte';
  import ListCard from './ListCard.svelte';
  import StatusBadge from './StatusBadge.svelte';
  import ActionButton from './ActionButton.svelte';
  import type { Contact, Rfp, Company } from '../../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let contact: Contact | null = null;
  
  // Create optimized company lookup
  $: companyLookup = createCompanyLookup($companiesStore);
  
  // Find related company
  $: relatedCompany = contact ? companyLookup.getCompany(contact.company) : null;
  
  // Filter RFPs where this contact is involved
  $: contactRfps = contact ? $rfpsStore.filter(rfp => {
    if (!rfp.contact_id || !contact?.id) return false;
    
    let rfpContactId = '';
    if (typeof rfp.contact_id === 'string') {
      rfpContactId = rfp.contact_id;
    } else if (rfp.contact_id && typeof rfp.contact_id === 'object') {
      if ((rfp.contact_id as any).tb && (rfp.contact_id as any).id) {
        if (typeof (rfp.contact_id as any).id === 'string') {
          rfpContactId = `${(rfp.contact_id as any).tb}:${(rfp.contact_id as any).id}`;
        } else if ((rfp.contact_id as any).id.String) {
          rfpContactId = `${(rfp.contact_id as any).tb}:${(rfp.contact_id as any).id.String}`;
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
    
    const id1 = rfpContactId.replace('contacts:', '');
    const id2 = contactIdStr.replace('contacts:', '');
    return id1 === id2 || rfpContactId === contactIdStr;
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
    rfpsActions.load();
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

{#if contact}
<DetailPanel 
  bind:isOpen 
  title="contact"
  on:edit={handleEdit}
  on:close={handleClose}
>
  <svelte:fragment slot="header">
    <DetailHeader 
      name={contact.full_name}
      subtitle="{contact.position || 'Contact'}{relatedCompany ? ` • ${relatedCompany.name}` : ''}"
      location="{relatedCompany?.city || 'Unknown'}, {relatedCompany?.country || 'Unknown'}"
      stats={[
        { label: 'Proposals', value: contactRfps.length },
        { label: 'Awarded', value: contactRfps.filter(rfp => rfp.status === 'Awarded').length },
        { label: 'Pending', value: contactRfps.filter(rfp => rfp.status === 'Sent').length },
        { label: 'Lost', value: contactRfps.filter(rfp => rfp.status === 'Lost').length }
      ]}
    />
  </svelte:fragment>
  
  <svelte:fragment slot="content">
    <!-- Contact Information Section -->
    <section>
      <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider mb-2">Contact Information</h2>
      <div class="bg-emittiv-black rounded-xl px-3 py-2 border border-emittiv-dark">
        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <div class="text-xs text-emittiv-light mb-1">Full Name</div>
            <div class="text-sm text-emittiv-white">{contact.full_name}</div>
          </div>
          <div>
            <div class="text-xs text-emittiv-light mb-1">Position</div>
            <div class="text-sm text-emittiv-white">{contact.position || '—'}</div>
          </div>
          <div>
            <div class="text-xs text-emittiv-light mb-1">First Name</div>
            <div class="text-sm text-emittiv-white">{contact.first_name || '—'}</div>
          </div>
          <div>
            <div class="text-xs text-emittiv-light mb-1">Last Name</div>
            <div class="text-sm text-emittiv-white">{contact.last_name || '—'}</div>
          </div>
          <div>
            <div class="text-xs text-emittiv-light mb-1 flex items-center gap-1">
              <span>Email</span>
              <ActionButton 
                type="email" 
                href="mailto:{contact.email}" 
                ariaLabel="Email {contact.full_name}"
                size={14}
              />
            </div>
            <div class="text-sm text-emittiv-white">{contact.email}</div>
          </div>
          <div>
            <div class="text-xs text-emittiv-light mb-1 flex items-center gap-1">
              <span>Phone</span>
              {#if contact.phone}
                <ActionButton 
                  type="phone" 
                  href="tel:{contact.phone}" 
                  ariaLabel="Call {contact.full_name}"
                  size={14}
                />
              {/if}
            </div>
            <div class="text-sm text-emittiv-white">
              {contact.phone || '—'}
            </div>
          </div>
          <div>
            <div class="text-xs text-emittiv-light mb-1">Created</div>
            <div class="text-sm text-emittiv-white">{new Date(contact.time.created_at).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}</div>
          </div>
          <div>
            <div class="text-xs text-emittiv-light mb-1">Last Updated</div>
            <div class="text-sm text-emittiv-white">{new Date(contact.time.updated_at).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}</div>
          </div>
          <div>
            <div class="text-xs text-emittiv-light mb-1">Record ID</div>
            <div class="text-sm text-emittiv-white">{extractId(contact.id)}</div>
          </div>
        </div>
      </div>
    </section>
    
    <!-- Company Information Section -->
    {#if relatedCompany}
      <InfoCard 
        title="Company Information" 
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
          {contactRfps.length} total
        </span>
      </div>
      
      {#if contactRfps.length === 0}
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-emittiv-light text-sm">No proposals yet</p>
        </div>
      {:else}
        <div class="grid gap-2">
          {#each contactRfps as rfp}
            <ListCard clickable={false}>
              <div class="flex items-start justify-between gap-3">
                <div class="flex-1 min-w-0">
                  <div>
                    <h3 class="text-xs font-medium text-emittiv-light">RFP Number:</h3>
                    <p class="text-sm text-emittiv-white">{rfp.number}</p>
                  </div>
                  <div class="mt-2">
                    <h3 class="text-xs font-medium text-emittiv-light">Proposal Name:</h3>
                    <p class="text-sm text-emittiv-lighter">{rfp.name}{#if rfp.package} - {rfp.package}{/if}</p>
                  </div>
                  <div class="mt-2 space-y-1">
                    {#if rfp.staff_name}
                      <div>
                        <h3 class="text-xs font-medium text-emittiv-light">Staff:</h3>
                        <p class="text-sm text-emittiv-white">{rfp.staff_name}</p>
                      </div>
                    {/if}
                    <div class="flex items-center gap-4 text-xs text-emittiv-light">
                      <span>Rev: {rfp.rev}</span>
                      <span>Stage: {rfp.stage}</span>
                      <span>
                        {new Date(rfp.issue_date.length === 6 ? `20${rfp.issue_date.substring(0,2)}-${rfp.issue_date.substring(2,4)}-${rfp.issue_date.substring(4,6)}` : rfp.issue_date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}
                      </span>
                    </div>
                  </div>
                </div>
                <div class="flex items-center gap-2 flex-shrink-0">
                  <StatusBadge status={rfp.status} type="proposal" />
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