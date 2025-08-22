<script lang="ts">
  import { feesStore, projectsStore, companiesStore } from '$lib/stores';
  import { extractId, findEntityById, getEntityDisplayName } from '$lib/utils';
  import Card from '$lib/components/Card.svelte';
  import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
  import { push } from 'svelte-spa-router';
  import type { UnknownSurrealThing, Fee } from '../../../types';

  export let isLoading = false;

  // Helper functions
  function getProjectName(projectId: UnknownSurrealThing): string {
    const project = findEntityById($projectsStore, projectId);
    return getEntityDisplayName(project) || 'Unknown Project';
  }

  function getCompanyName(companyId: UnknownSurrealThing): string {
    const company = findEntityById($companiesStore, companyId);
    return getEntityDisplayName(company) || 'Unknown Company';
  }

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('en-AE', {
      style: 'currency',
      currency: 'AED',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(amount);
  }

  function getDaysAgo(date: string): number {
    const sentDate = new Date(date);
    const today = new Date();
    const diffTime = Math.abs(today.getTime() - sentDate.getTime());
    return Math.ceil(diffTime / (1000 * 60 * 60 * 24));
  }

  // Filter for pending proposals (Sent + Draft - not closed)
  $: pendingProposals = $feesStore
    .filter(fee => fee.status === 'Sent' || fee.status === 'Draft')
    .filter(fee => fee.time) // Filter out fees without time info
    .sort((a, b) => new Date(b.time!.updated_at || b.time!.created_at).getTime() - 
                    new Date(a.time!.updated_at || a.time!.created_at).getTime())
    .slice(0, 8); // Show top 8 pending

  // Calculate total pending count
  $: totalPendingCount = pendingProposals.length;

  function handleProposalClick(fee: Fee) {
    const feeId = extractId(fee.id);
    push(`/proposals/${feeId}`); // Navigate to the specific proposal detail page
  }
</script>

<div class="pending-proposals">
  <div class="proposals-header">
    <h2 class="section-title">Pending Proposals</h2>
    
    {#if pendingProposals.length > 0}
      <div class="proposals-summary">
        <div class="summary-label">Pending Work</div>
        <div class="summary-count">
          {totalPendingCount} proposal{totalPendingCount !== 1 ? 's' : ''}
        </div>
      </div>
    {/if}
  </div>

  <div class="proposals-content">
    {#if isLoading}
      <LoadingSkeleton rows={4} />
    {:else if pendingProposals.length > 0}
      <div class="proposals-list">
        {#each pendingProposals as fee}
          {@const daysAgo = getDaysAgo(fee.time!.updated_at || fee.time!.created_at)}
          <div class="proposal-item {fee.status.toLowerCase()}" on:click={() => handleProposalClick(fee)} on:keydown={(e) => e.key === 'Enter' && handleProposalClick(fee)} role="button" tabindex="0">
            <div class="proposal-status">
              <div class="status-indicator {fee.status.toLowerCase()}"></div>
              <span class="status-badge {fee.status.toLowerCase()}">{fee.status}</span>
            </div>
            
            <div class="proposal-details">
              <div class="proposal-header">
                <span class="fee-number">{fee.number}</span>
              </div>
              
              <div class="proposal-project">
                <span class="project-name">{getProjectName(fee.project_id)}</span>
              </div>
              
              <div class="proposal-company">
                <span class="company-name">{getCompanyName(fee.company_id)}</span>
              </div>
              
            </div>
            
            <div class="proposal-timing">
              <div class="days-indicator {
                daysAgo <= 7 ? 'recent' :
                daysAgo <= 30 ? 'moderate' :
                daysAgo <= 60 ? 'old' :
                'stale'
              }">
                {daysAgo}d
              </div>
              <div class="proposal-date">
                {new Date(fee.time.updated_at || fee.time.created_at).toLocaleDateString('en-US', { 
                  month: 'short', 
                  day: 'numeric'
                })}
              </div>
            </div>
            
            {#if fee.package}
              <div class="proposal-package-full">
                <span class="fee-package-full">{fee.package}</span>
              </div>
            {/if}
          </div>
        {/each}
      </div>
      
    {:else}
      <div class="empty-state">
        <svg class="empty-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p class="empty-message">No pending proposals</p>
        <p class="empty-subtitle">All proposals are either active or closed</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .pending-proposals {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 16px;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .proposals-header {
    padding: 24px 24px 0 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
  }

  .section-title {
    font-family: Ubuntu, sans-serif;
    font-size: 20px;
    font-weight: 600;
    color: var(--emittiv-white);
    margin: 0;
    letter-spacing: -0.02em;
  }

  .proposals-summary {
    text-align: right;
    min-width: 0;
    min-height: 32px; /* Match the height of activity filter controls */
    display: flex;
    flex-direction: column;
    justify-content: flex-end; /* Align with bottom like filter controls */
  }

  .summary-label {
    font-size: 12px;
    color: var(--emittiv-lighter);
    margin-bottom: 2px;
  }

  .summary-count {
    font-size: 16px;
    font-weight: 600;
    color: var(--emittiv-splash);
  }

  .proposals-content {
    flex: 1;
    padding: 0 24px 24px 24px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    position: relative;
  }

  .proposals-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex: 1;
    overflow-y: auto;
    margin-right: -12px;
    padding-right: 12px;
  }

  .proposal-item {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: flex-start;
    gap: 16px;
    padding: 16px;
    background: var(--emittiv-black);
    border: 1px solid var(--emittiv-dark);
    border-radius: 12px;
    transition: all 150ms ease;
    cursor: pointer;
    min-width: 0;
  }

  .proposal-item.sent {
    border-left: 3px solid #f59e0b; /* Orange left border for sent */
  }

  .proposal-item.draft {
    border-left: 3px solid #6b7280; /* Gray left border for draft */
  }

  .proposal-item.sent:hover {
    background: rgba(245, 158, 11, 0.06);
    border-color: rgba(245, 158, 11, 0.4);
    border-left-color: #f59e0b;
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .proposal-item.draft:hover {
    background: rgba(107, 114, 128, 0.06);
    border-color: rgba(107, 114, 128, 0.4);
    border-left-color: #6b7280;
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .proposal-item.sent:focus {
    outline: none;
    background: rgba(245, 158, 11, 0.06);
    border-color: #f59e0b;
    border-left-color: #f59e0b;
  }

  .proposal-item.draft:focus {
    outline: none;
    background: rgba(107, 114, 128, 0.06);
    border-color: #6b7280;
    border-left-color: #6b7280;
  }

  .proposal-status {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    min-width: 60px;
    flex-shrink: 0;
  }

  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .status-indicator.sent {
    background: #f59e0b; /* Orange for sent proposals */
  }

  .status-indicator.draft {
    background: #6b7280; /* Gray for draft proposals */
  }

  .status-badge {
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 500;
    text-align: center;
  }

  .status-badge.sent {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
  }

  .status-badge.draft {
    background: rgba(107, 114, 128, 0.2);
    color: #9ca3af;
  }

  .proposal-details {
    flex: 1;
    min-width: 0;
  }

  .proposal-header {
    margin-bottom: 4px;
  }

  .fee-number {
    font-weight: 600;
    color: var(--emittiv-white);
    font-size: 15px;
  }

  .proposal-project {
    margin-bottom: 2px;
    font-size: 13px;
    min-width: 0;
  }
  
  .proposal-company {
    margin-bottom: 4px;
    font-size: 12px;
    min-width: 0;
  }

  .proposal-package-full {
    grid-column: 1 / -1;
    margin-top: 4px;
    width: 100%;
  }

  .fee-package-full {
    font-size: 11px;
    color: var(--emittiv-splash);
    font-weight: 500;
    background: rgba(255, 153, 0, 0.1);
    padding: 2px 6px;
    border-radius: 4px;
    display: inline-block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .project-name {
    color: var(--emittiv-light);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
  }

  .company-name {
    color: var(--emittiv-lighter);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
    font-size: 12px;
  }

  .proposal-description {
    font-size: 12px;
    color: var(--emittiv-lighter);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .proposal-timing {
    text-align: right;
    min-width: 50px;
    flex-shrink: 0;
  }

  .days-indicator {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 4px;
  }

  .days-indicator.recent {
    color: #10b981;
  }

  .days-indicator.moderate {
    color: #f59e0b;
  }

  .days-indicator.old {
    color: #f97316;
  }

  .days-indicator.stale {
    color: #ef4444;
  }

  .proposal-date {
    font-size: 11px;
    color: var(--emittiv-lighter);
  }


  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    text-align: center;
    flex: 1;
  }

  .empty-icon {
    width: 48px;
    height: 48px;
    color: var(--emittiv-light);
    margin-bottom: 16px;
  }

  .empty-message {
    color: var(--emittiv-light);
    font-size: 14px;
    margin: 0 0 4px 0;
  }

  .empty-subtitle {
    color: var(--emittiv-lighter);
    font-size: 12px;
    margin: 0;
  }

  @media (max-width: 768px) {
    .proposals-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }
    
    .proposals-summary {
      text-align: left;
    }
    
    .proposal-item {
      flex-direction: column;
      gap: 12px;
    }
    
    .proposal-status {
      flex-direction: row;
      min-width: auto;
    }
    
    .proposal-timing {
      text-align: left;
      min-width: auto;
    }

    .proposal-meta {
      flex-direction: column;
      align-items: flex-start;
      gap: 4px;
    }
  }
</style>