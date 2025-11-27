<script lang="ts">
  import { onMount } from 'svelte';
  import { feesStore, loadAllData } from '$lib/stores';
  import { extractId, findEntityById } from '$lib/utils';
  import { push, location } from 'svelte-spa-router';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import ProposalDetail from '$lib/components/ProposalDetail.svelte';
  import ProposalModal from '$lib/components/ProposalModal.svelte';
  import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
  import type { Fee } from '../types';

  export let params: { id: string } = { id: '' };
  
  // Track the previous page to return to the correct location
  let previousPage = '/proposals'; // Default fallback
  
  let proposal: Fee | null = null;
  let loading = true;
  let error: string | null = null;
  
  // Modal states
  let showProposalModal = false;
  let proposalModalMode: 'create' | 'edit' = 'edit';
  let selectedProposal: Fee | null = null;
  
  // Debug modal state changes - avoid logging $state objects
  $: if (import.meta.env.DEV) {
    console.log('Modal state changed - showProposalModal:', showProposalModal);
    console.log('Modal mode:', proposalModalMode);
    console.log('Selected proposal id:', selectedProposal?.id);
    console.log('Has selected proposal:', !!selectedProposal);
  }
  
  // Track selectedProposal changes - avoid logging $state objects
  $: if (import.meta.env.DEV && selectedProposal !== null && selectedProposal !== undefined) {
    console.log('ProposalDetailPage: selectedProposal is NOT null/undefined, id:', selectedProposal.id);
  }
  
  $: if (import.meta.env.DEV && selectedProposal === null) {
    console.log('ProposalDetailPage: WARNING - selectedProposal is NULL');
  }

  // Extract the proposal ID from params
  $: proposalId = params.id;

  // Find the proposal when the store updates or ID changes
  $: {
    if ($feesStore.length > 0 && proposalId) {
      const foundProposal = findEntityById($feesStore, proposalId);
      if (foundProposal) {
        proposal = foundProposal as Fee;
        loading = false;
        error = null;
      } else {
        proposal = null;
        loading = false;
        error = `Proposal with ID "${proposalId}" not found`;
      }
    }
  }

  onMount(async () => {
    // Check if we came from the dashboard
    const referrer = document.referrer;
    const urlParams = new URLSearchParams(window.location.search);
    const fromParam = urlParams.get('from');
    
    if (fromParam === 'dashboard' || window.history.length > 1) {
      // If there's a 'from' parameter or browser history, check where we came from
      if (fromParam === 'dashboard') {
        previousPage = '/';
      } else {
        // Try to determine from browser history or default to dashboard if uncertain
        previousPage = window.history.length > 1 ? '/' : '/proposals';
      }
    }

    // If stores are empty, load data
    if ($feesStore.length === 0) {
      try {
        await loadAllData();
      } catch (err) {
        console.error('Failed to load proposals:', err);
        error = 'Failed to load proposal data';
        loading = false;
      }
    } else {
      // Data already loaded, check if proposal exists
      const foundProposal = findEntityById($feesStore, proposalId);
      if (foundProposal) {
        proposal = foundProposal as Fee;
      } else {
        error = `Proposal with ID "${proposalId}" not found`;
      }
      loading = false;
    }
  });

  function handleClose() {
    // Use browser history to go back if possible, otherwise use previousPage
    if (window.history.length > 1) {
      window.history.back();
    } else {
      push(previousPage);
    }
  }

  function handleProposalUpdate() {
    // Reload data to get the updated proposal
    loadAllData();
  }

  function handleEditFromDetail(event: CustomEvent) {
    // Open edit modal with the proposal from the detail panel
    if (import.meta.env.DEV) {
      console.log('ProposalDetailPage: handleEditFromDetail called');
      console.log('Event detail id:', event.detail?.id);
    }
    selectedProposal = event.detail;
    proposalModalMode = 'edit';
    showProposalModal = true;
    if (import.meta.env.DEV) {
      console.log('Modal state set - showProposalModal:', showProposalModal);
      console.log('ProposalDetailPage: selectedProposal.id:', selectedProposal?.id);
    }
  }
</script>

<!-- Backdrop -->
<div 
  class="fixed inset-0 bg-black bg-opacity-50 backdrop-blur-sm z-40"
  on:click={handleClose}
  on:keydown={(e) => e.key === 'Escape' && handleClose()}
  role="button"
  tabindex="-1"
  aria-label="Close detail view"
  in:fade={{ duration: 200 }}
  out:fade={{ duration: 200 }}
></div>

<!-- Sliding Panel -->
<div 
  class="fixed top-0 right-0 h-full bg-emittiv-black z-50 overflow-hidden shadow-2xl flex flex-col proposal-detail-page" 
  style="width: calc(100vw - 240px); left: 240px;"
  in:fly={{ x: '100%', duration: 300, easing: cubicOut }}
  out:fly={{ x: '100%', duration: 250, easing: cubicOut }}
>
  {#if loading}
    <div class="loading-container">
      <LoadingSkeleton rows={8} />
    </div>
  {:else if error}
    <div class="error-container">
      <div class="error-card">
        <h2>Proposal Not Found</h2>
        <p>{error}</p>
        <button class="back-button" on:click={handleClose}>
          ← Back to Proposals
        </button>
      </div>
    </div>
  {:else if proposal}
    <ProposalDetail 
      {proposal} 
      isOpen={true} 
      on:close={handleClose}
      on:edit={handleEditFromDetail}
      on:proposalUpdated={handleProposalUpdate}
    />
  {/if}
</div>

<!-- Proposal Modal for editing -->
{#if showProposalModal}
  <ProposalModal 
    isOpen={true}
    proposal={selectedProposal}
    mode={proposalModalMode}
    on:close={() => showProposalModal = false}
    on:proposalUpdated={handleProposalUpdate}
  />
{/if}

<style>
  .proposal-detail-page {
    min-height: 100vh;
    background: var(--emittiv-black);
    color: var(--emittiv-white);
  }

  .loading-container {
    padding: 48px 24px;
    max-width: 800px;
    margin: 0 auto;
  }

  .error-container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 50vh;
    padding: 24px;
  }

  .error-card {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 16px;
    padding: 48px;
    text-align: center;
    max-width: 400px;
  }

  .error-card h2 {
    color: var(--emittiv-white);
    margin: 0 0 16px 0;
    font-size: 24px;
    font-weight: 600;
  }

  .error-card p {
    color: var(--emittiv-lighter);
    margin: 0 0 32px 0;
    font-size: 16px;
    line-height: 1.5;
  }

  .back-button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 12px 24px;
    background: var(--emittiv-splash);
    color: var(--emittiv-black);
    border: none;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: all 200ms ease;
  }

  .back-button:hover {
    background: rgba(255, 153, 0, 0.9);
    transform: translateY(-1px);
  }
</style>