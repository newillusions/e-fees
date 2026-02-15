<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { getActivityLogs, type ActivityLog } from '$lib/api';
  import LoadingSkeleton from './LoadingSkeleton.svelte';
  import { push } from 'svelte-spa-router';

  let { isOpen = $bindable(false) }: { isOpen?: boolean } = $props();

  const dispatch = createEventDispatcher();

  // Activity log state
  let activities: ActivityLog[] = [];
  let loading = true;
  let error: string | null = null;

  // Pagination state
  let currentPage = 1;
  let pageSize = 25;
  let totalLoaded = 0;
  let hasMore = true;
  let loadingMore = false;

  // Filter states
  let entityFilter: 'all' | 'project' | 'fee' | 'company' | 'contact' = 'all';
  let actionFilter: 'all' | 'create' | 'update' | 'delete' | 'status_change' = 'all';

  // Load initial activities
  async function loadActivities(reset = true) {
    if (reset) {
      loading = true;
      activities = [];
      currentPage = 1;
      totalLoaded = 0;
      hasMore = true;
    }
    error = null;

    try {
      const entityType = entityFilter === 'all' ? undefined : entityFilter;
      // Load more than needed to enable pagination
      const limit = pageSize * currentPage;
      let results = await getActivityLogs(limit + 1, entityType);

      // Apply action filter client-side (API doesn't support it yet)
      if (actionFilter !== 'all') {
        results = results.filter(a => a.action === actionFilter);
      }

      hasMore = results.length > limit;
      activities = results.slice(0, limit);
      totalLoaded = activities.length;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load activities';
      activities = [];
    } finally {
      loading = false;
      loadingMore = false;
    }
  }

  // Load more activities
  async function loadMore() {
    if (loadingMore || !hasMore) return;
    loadingMore = true;
    currentPage += 1;
    await loadActivities(false);
  }

  // Handle scroll for infinite loading
  function handleScroll(event: Event) {
    const target = event.target as HTMLElement;
    const scrollBottom = target.scrollHeight - target.scrollTop - target.clientHeight;
    if (scrollBottom < 100 && !loadingMore && hasMore) {
      loadMore();
    }
  }

  // Load when modal opens
  $effect(() => {
    if (isOpen) {
      loadActivities(true);
    }
  });

  // Reload when filters change
  function handleFilterChange() {
    loadActivities(true);
  }

  // Close modal
  function close() {
    isOpen = false;
    dispatch('close');
  }

  // Handle escape key
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      close();
    }
  }

  // Get icon for action type
  function getActionIcon(action: string): string {
    switch (action) {
      case 'create': return 'M12 6v6m0 0v6m0-6h6m-6 0H6';
      case 'update': return 'M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z';
      case 'delete': return 'M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16';
      case 'status_change': return 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z';
      default: return 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z';
    }
  }

  // Get color class for action type
  function getActionColor(action: string): string {
    switch (action) {
      case 'create': return 'text-green-400 bg-green-400/20';
      case 'update': return 'text-blue-400 bg-blue-400/20';
      case 'delete': return 'text-red-400 bg-red-400/20';
      case 'status_change': return 'text-orange-400 bg-orange-400/20';
      default: return 'text-gray-400 bg-gray-400/20';
    }
  }

  // Get border color for action type
  function getActionBorderColor(action: string): string {
    switch (action) {
      case 'create': return '#10b981';
      case 'update': return '#3b82f6';
      case 'delete': return '#ef4444';
      case 'status_change': return '#f59e0b';
      default: return '#6b7280';
    }
  }

  // Format timestamp
  function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  // Format relative time
  function formatRelativeTime(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;

    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  }

  // Handle click to navigate to entity
  function handleActivityClick(activity: ActivityLog) {
    const entityType = activity.entity_type;
    const entityId = activity.entity_id.split(':')[1] || activity.entity_id;

    close();

    switch (entityType) {
      case 'project':
        push(`/projects/${entityId}`);
        break;
      case 'fee':
        push(`/proposals/${entityId}`);
        break;
      case 'company':
        push(`/companies`);
        break;
      case 'contact':
        push(`/contacts`);
        break;
    }
  }

  // Filter options
  const entityOptions = [
    { value: 'all', label: 'All Types' },
    { value: 'project', label: 'Projects' },
    { value: 'fee', label: 'Proposals' },
    { value: 'company', label: 'Companies' },
    { value: 'contact', label: 'Contacts' }
  ];

  const actionOptions = [
    { value: 'all', label: 'All Actions' },
    { value: 'create', label: 'Created' },
    { value: 'update', label: 'Updated' },
    { value: 'delete', label: 'Deleted' },
    { value: 'status_change', label: 'Status Changes' }
  ];
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Backdrop -->
  <div
    class="modal-backdrop"
    on:click={close}
    on:keydown={e => e.key === 'Enter' && close()}
    role="button"
    tabindex="0"
    aria-label="Close modal"
  />

  <!-- Modal -->
  <div class="modal-container">
    <div class="modal-content">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-left">
          <h2 class="modal-title">Activity Log</h2>
          <span class="activity-count">{totalLoaded} activities</span>
        </div>

        <div class="header-right">
          <!-- Filters -->
          <div class="filter-controls">
            <select
              bind:value={entityFilter}
              on:change={handleFilterChange}
              class="filter-select"
            >
              {#each entityOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>

            <select
              bind:value={actionFilter}
              on:change={handleFilterChange}
              class="filter-select"
            >
              {#each actionOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>

          <!-- Close button -->
          <button class="close-button" on:click={close} aria-label="Close">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Body -->
      <div class="modal-body" on:scroll={handleScroll}>
        {#if loading}
          <div class="loading-container">
            <LoadingSkeleton rows={8} />
          </div>
        {:else if error}
          <div class="error-state">
            <svg class="error-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <p class="error-message">{error}</p>
            <button class="retry-button" on:click={() => loadActivities(true)}>Retry</button>
          </div>
        {:else if activities.length > 0}
          <div class="activity-list">
            {#each activities as activity}
              <div
                class="activity-item"
                style="border-left-color: {getActionBorderColor(activity.action)}"
                on:click={() => handleActivityClick(activity)}
                on:keydown={e => e.key === 'Enter' && handleActivityClick(activity)}
                role="button"
                tabindex="0"
              >
                <div class="activity-icon {getActionColor(activity.action)}">
                  <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getActionIcon(activity.action)} />
                  </svg>
                </div>

                <div class="activity-details">
                  <div class="activity-header">
                    <span class="entity-name">{activity.entity_name}</span>
                    <span class="entity-type">{activity.entity_type}</span>
                    <span class="action-badge {getActionColor(activity.action)}">{activity.action.replace('_', ' ')}</span>
                  </div>

                  <div class="activity-description">
                    {activity.description}
                  </div>

                  {#if activity.old_value && activity.new_value}
                    <div class="activity-change">
                      <span class="old-value">{activity.old_value}</span>
                      <svg class="arrow-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
                      </svg>
                      <span class="new-value">{activity.new_value}</span>
                    </div>
                  {/if}
                </div>

                <div class="activity-meta">
                  <div class="activity-time" title={formatTimestamp(activity.timestamp)}>
                    {formatRelativeTime(activity.timestamp)}
                  </div>
                  <div class="activity-user">{activity.user}</div>
                </div>
              </div>
            {/each}

            {#if loadingMore}
              <div class="loading-more">
                <div class="spinner"></div>
                <span>Loading more...</span>
              </div>
            {/if}

            {#if !hasMore && activities.length > 0}
              <div class="end-of-list">
                No more activities to load
              </div>
            {/if}
          </div>
        {:else}
          <div class="empty-state">
            <svg class="empty-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
              />
            </svg>
            <p class="empty-message">No activity logged yet</p>
            <p class="empty-hint">Activities will appear here as you create, update, or change entities</p>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    z-index: 1000;
  }

  .modal-container {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1001;
    padding: 24px;
    pointer-events: none;
  }

  .modal-content {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 16px;
    width: 100%;
    max-width: 800px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    pointer-events: auto;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  }

  .modal-header {
    padding: 20px 24px;
    border-bottom: 1px solid var(--emittiv-dark);
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .modal-title {
    font-family: Ubuntu, sans-serif;
    font-size: 20px;
    font-weight: 600;
    color: var(--emittiv-white);
    margin: 0;
  }

  .activity-count {
    font-size: 13px;
    color: var(--emittiv-light);
    padding: 4px 10px;
    background: var(--emittiv-black);
    border-radius: 20px;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .filter-controls {
    display: flex;
    gap: 8px;
  }

  .filter-select {
    padding: 6px 28px 6px 10px;
    background: var(--emittiv-black);
    border: 1px solid var(--emittiv-dark);
    border-radius: 6px;
    color: var(--emittiv-white);
    font-size: 12px;
    cursor: pointer;
    transition: all 150ms ease;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 12 12'%3E%3Cpath fill='%23999' d='M6 9L2 5h8z'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
  }

  .filter-select:hover {
    border-color: var(--emittiv-splash);
  }

  .filter-select:focus {
    outline: none;
    border-color: var(--emittiv-splash);
  }

  .close-button {
    padding: 8px;
    background: transparent;
    border: none;
    border-radius: 8px;
    color: var(--emittiv-light);
    cursor: pointer;
    transition: all 150ms ease;
  }

  .close-button:hover {
    background: var(--emittiv-dark);
    color: var(--emittiv-white);
  }

  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 16px 24px 24px 24px;
  }

  .activity-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .activity-item {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: flex-start;
    gap: 14px;
    padding: 14px;
    background: var(--emittiv-black);
    border: 1px solid var(--emittiv-dark);
    border-left: 3px solid;
    border-radius: 10px;
    transition: all 150ms ease;
    cursor: pointer;
  }

  .activity-item:hover {
    background: rgba(255, 153, 0, 0.06);
    border-color: rgba(255, 153, 0, 0.3);
    transform: translateY(-1px);
  }

  .activity-icon {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .activity-icon .icon {
    width: 16px;
    height: 16px;
  }

  .activity-details {
    flex: 1;
    min-width: 0;
  }

  .activity-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
    flex-wrap: wrap;
  }

  .entity-name {
    font-weight: 600;
    color: var(--emittiv-white);
    font-size: 13px;
  }

  .entity-type {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--emittiv-dark);
    color: var(--emittiv-light);
    text-transform: capitalize;
  }

  .action-badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    text-transform: capitalize;
  }

  .activity-description {
    font-size: 12px;
    color: var(--emittiv-lighter);
    margin-bottom: 4px;
  }

  .activity-change {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
  }

  .old-value {
    color: var(--emittiv-light);
    text-decoration: line-through;
  }

  .arrow-icon {
    width: 12px;
    height: 12px;
    color: var(--emittiv-light);
  }

  .new-value {
    color: var(--emittiv-splash);
    font-weight: 500;
  }

  .activity-meta {
    text-align: right;
    min-width: 60px;
  }

  .activity-time {
    font-size: 11px;
    color: var(--emittiv-light);
    font-weight: 500;
    margin-bottom: 2px;
  }

  .activity-user {
    font-size: 10px;
    color: var(--emittiv-lighter);
  }

  .loading-container {
    padding: 20px 0;
  }

  .loading-more {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 16px;
    color: var(--emittiv-light);
    font-size: 13px;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--emittiv-dark);
    border-top-color: var(--emittiv-splash);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .end-of-list {
    text-align: center;
    padding: 16px;
    color: var(--emittiv-light);
    font-size: 12px;
    border-top: 1px solid var(--emittiv-dark);
    margin-top: 8px;
  }

  .empty-state,
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    text-align: center;
  }

  .empty-icon,
  .error-icon {
    width: 48px;
    height: 48px;
    margin-bottom: 16px;
  }

  .empty-icon {
    color: var(--emittiv-light);
  }

  .error-icon {
    color: #ef4444;
  }

  .empty-message {
    color: var(--emittiv-light);
    font-size: 14px;
    margin: 0 0 8px 0;
  }

  .empty-hint {
    color: var(--emittiv-lighter);
    font-size: 12px;
    margin: 0;
  }

  .error-message {
    color: #ef4444;
    font-size: 14px;
    margin: 0 0 16px 0;
  }

  .retry-button {
    padding: 8px 16px;
    background: var(--emittiv-dark);
    border: 1px solid var(--emittiv-light);
    border-radius: 6px;
    color: var(--emittiv-white);
    font-size: 13px;
    cursor: pointer;
    transition: all 150ms ease;
  }

  .retry-button:hover {
    background: var(--emittiv-splash);
    border-color: var(--emittiv-splash);
  }
</style>
