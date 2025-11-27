<script lang="ts">
  import { recentFeesStore, projectsStore, companiesStore } from '$lib/stores';
  import { extractId, findEntityById, getEntityDisplayName } from '$lib/utils';
  import Card from '$lib/components/Card.svelte';
  import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
  import { push } from 'svelte-spa-router';
  import type { UnknownSurrealThing, Fee } from '../../../types';

  export let isLoading = false;

  // Filter states
  let activityFilter = 'all';
  let timeFilter = 30; // days

  // Helper functions for activity display
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

  // Filter activity based on selected filters
  $: filteredActivity = $recentFeesStore.filter(fee => {
    // Time filter - skip if no time info
    if (!fee.time) return false;
    const createdDate = new Date(fee.time.created_at);
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - timeFilter);
    
    if (createdDate < cutoffDate) return false;

    // Activity type filter
    if (activityFilter === 'all') return true;
    if (activityFilter === 'sent' && fee.status === 'Sent') return true;
    if (activityFilter === 'awarded' && fee.status === 'Awarded') return true;
    if (activityFilter === 'closed' && (fee.status === 'Lost' || fee.status === 'Cancelled')) return true;
    
    return false;
  })
  .sort((a, b) => {
    // Sort by newest first (most recent created_at or updated_at)
    // Since we filtered out items without time, they should exist
    const dateA = new Date(a.time!.updated_at || a.time!.created_at).getTime();
    const dateB = new Date(b.time!.updated_at || b.time!.created_at).getTime();
    return dateB - dateA; // Newest first
  })
  .slice(0, 10); // Show top 10

  // Activity filter options
  const filterOptions = [
    { value: 'all', label: 'All Activity' },
    { value: 'awarded', label: 'Awarded' },
    { value: 'sent', label: 'Sent' },
    { value: 'closed', label: 'Closed' }
  ];

  const timeOptions = [
    { value: 7, label: 'Last 7 days' },
    { value: 30, label: 'Last 30 days' },
    { value: 90, label: 'Last 90 days' }
  ];

  function handleFeeClick(fee: Fee) {
    const feeId = extractId(fee.id);
    console.log('ActivityFeed: Navigating to proposal detail page for fee:', feeId);
    push(`/proposals/${feeId}`); // Navigate to the specific proposal detail page
  }
</script>

<div class="activity-feed">
  <div class="feed-header">
    <h2 class="section-title">Recent Activity</h2>
    
    <div class="filter-controls">
      <select bind:value={activityFilter} class="filter-select">
        {#each filterOptions as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
      
      <select bind:value={timeFilter} class="filter-select">
        {#each timeOptions as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
  </div>

  <div class="feed-content">
    {#if isLoading}
      <LoadingSkeleton rows={4} />
    {:else if filteredActivity.length > 0}
      <div class="activity-list">
        {#each filteredActivity as fee}
          <div class="activity-item {fee.status.toLowerCase()}" on:click={() => handleFeeClick(fee)} on:keydown={(e) => e.key === 'Enter' && handleFeeClick(fee)} role="button" tabindex="0">
            <div class="activity-status">
              <div class="status-indicator"></div>
              <span class="status-badge">{fee.status}</span>
            </div>
            
            <div class="activity-details">
              <div class="activity-header">
                <span class="fee-number">{fee.number}</span>
              </div>
              
              <div class="activity-project">
                <span class="project-name">{getProjectName(fee.project_id)}</span>
              </div>
              
              <div class="activity-company">
                <span class="company-name">{getCompanyName(fee.company_id)}</span>
              </div>
              
            </div>
            
            <div class="activity-date">
              <div class="primary-date">
                {new Date(fee.time!.created_at).toLocaleDateString('en-US', { 
                  month: 'short', 
                  day: 'numeric'
                })}
              </div>
              {#if fee.time!.updated_at !== fee.time!.created_at}
                <div class="updated-date">
                  Updated {new Date(fee.time!.updated_at).toLocaleDateString('en-US', { 
                    month: 'short', 
                    day: 'numeric'
                  })}
                </div>
              {/if}
            </div>
            
            {#if fee.package}
              <div class="activity-package-full">
                <span class="fee-package-full">{fee.package}</span>
              </div>
            {/if}
          </div>
        {/each}
      </div>
      
    {:else}
      <div class="empty-state">
        <svg class="empty-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
        </svg>
        <p class="empty-message">No activity found for the selected filters</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .activity-feed {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 16px;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .feed-header {
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

  .filter-controls {
    display: flex;
    gap: 12px;
    align-items: flex-end; /* Align with bottom of title */
    min-height: 32px; /* Match the height of the proposals summary */
    margin-left: 20px; /* Add space between title and first dropdown */
  }

  .filter-select {
    padding: 6px 32px 6px 12px;
    background: var(--emittiv-black);
    border: 1px solid var(--emittiv-dark);
    border-radius: 8px;
    color: var(--emittiv-white);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
    height: fit-content;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23999' d='M6 9L2 5h8z'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 10px center;
    min-width: 120px;
  }

  .filter-select:hover {
    border-color: var(--emittiv-splash);
    background-color: rgba(255, 153, 0, 0.05);
  }

  .filter-select:focus {
    outline: none;
    border-color: var(--emittiv-splash);
    box-shadow: 0 0 0 3px rgba(255, 153, 0, 0.1);
  }

  .filter-select option {
    background: var(--emittiv-darker);
    color: var(--emittiv-white);
    padding: 8px;
  }

  .feed-content {
    flex: 1;
    padding: 0 24px 24px 24px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    position: relative;
  }

  .activity-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex: 1;
    overflow-y: auto;
    margin-right: -12px;
    padding-right: 12px;
  }

  .activity-item {
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
  }

  .activity-item.awarded {
    border-left: 3px solid #10b981; /* Green for awarded */
  }

  .activity-item.sent {
    border-left: 3px solid #f59e0b; /* Orange for sent */
  }

  .activity-item.lost,
  .activity-item.cancelled {
    border-left: 3px solid #ef4444; /* Red for lost/cancelled */
  }

  .activity-item.draft {
    border-left: 3px solid #6b7280; /* Gray for draft */
  }

  .activity-item:hover {
    background: rgba(255, 153, 0, 0.06);
    border-color: rgba(255, 153, 0, 0.3);
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .activity-item.awarded:hover {
    border-left-color: #10b981;
  }

  .activity-item.sent:hover {
    border-left-color: #f59e0b;
  }

  .activity-item.lost:hover,
  .activity-item.cancelled:hover {
    border-left-color: #ef4444;
  }

  .activity-item.draft:hover {
    border-left-color: #6b7280;
  }

  .activity-item:focus {
    outline: none;
    background: rgba(255, 153, 0, 0.06);
    border-color: var(--emittiv-splash);
  }

  .activity-item.awarded:focus {
    border-left-color: #10b981;
  }

  .activity-item.sent:focus {
    border-left-color: #f59e0b;
  }

  .activity-item.lost:focus,
  .activity-item.cancelled:focus {
    border-left-color: #ef4444;
  }

  .activity-item.draft:focus {
    border-left-color: #6b7280;
  }

  .activity-status {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    min-width: 60px;
  }

  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--emittiv-light);
  }

  .activity-item.awarded .status-indicator {
    background: #10b981;
  }

  .activity-item.sent .status-indicator {
    background: #f59e0b;
  }

  .activity-item.lost .status-indicator,
  .activity-item.cancelled .status-indicator {
    background: #ef4444;
  }

  .status-badge {
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 500;
    text-align: center;
    background: rgba(156, 163, 175, 0.2);
    color: #9ca3af;
  }

  .activity-item.awarded .status-badge {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
  }

  .activity-item.sent .status-badge {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
  }

  .activity-item.lost .status-badge,
  .activity-item.cancelled .status-badge {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .activity-details {
    flex: 1;
    min-width: 0;
  }

  .activity-header {
    margin-bottom: 4px;
  }

  .fee-number {
    font-weight: 600;
    color: var(--emittiv-white);
    font-size: 15px;
  }

  .activity-project {
    margin-bottom: 2px;
    font-size: 13px;
    min-width: 0;
  }
  
  .activity-company {
    margin-bottom: 4px;
    font-size: 12px;
    min-width: 0;
  }

  .activity-package-full {
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

  .activity-description {
    font-size: 12px;
    color: var(--emittiv-lighter);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .activity-date {
    text-align: right;
    min-width: 80px;
  }

  .primary-date {
    font-size: 13px;
    color: var(--emittiv-light);
    font-weight: 500;
    margin-bottom: 2px;
  }

  .updated-date {
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
    margin: 0;
  }

  @media (max-width: 768px) {
    .feed-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 16px;
    }
    
    .filter-controls {
      width: 100%;
      justify-content: space-between;
    }
    
    .activity-item {
      flex-direction: column;
      gap: 12px;
    }
    
    .activity-status {
      flex-direction: row;
      min-width: auto;
    }
    
    .activity-date {
      text-align: left;
      min-width: auto;
    }
  }
</style>