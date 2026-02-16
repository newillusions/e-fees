<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { getActivityLogs, type ActivityLog } from '$lib/api';
  import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
  import { push } from 'svelte-spa-router';

  let { isLoading = false }: { isLoading?: boolean } = $props();

  const dispatch = createEventDispatcher();

  // Emit event to open full activity log modal
  function openFullActivityLog() {
    dispatch('viewAll');
  }

  // Activity log state
  let activities: ActivityLog[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);

  // Pagination state
  let currentPage = $state(0);
  let hasMore = $state(true);
  let loadingMore = $state(false);
  const PAGE_SIZE = 15;

  // Filter states
  let entityFilter: 'all' | 'project' | 'fee' | 'company' | 'contact' = $state('all');

  // Scroll container reference
  let scrollContainer: HTMLDivElement;

  // Load activity logs (initial or reset)
  async function loadActivities(reset = true) {
    if (reset) {
      loading = true;
      currentPage = 0;
      hasMore = true;
      activities = [];
    }
    error = null;

    try {
      const entityType = entityFilter === 'all' ? undefined : entityFilter;
      const offset = currentPage * PAGE_SIZE;
      const newActivities = await getActivityLogs(PAGE_SIZE, entityType, offset);

      if (reset) {
        activities = newActivities;
      } else {
        activities = [...activities, ...newActivities];
      }

      // Check if there are more
      hasMore = newActivities.length === PAGE_SIZE;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load activities';
      if (reset) activities = [];
    } finally {
      loading = false;
      loadingMore = false;
    }
  }

  // Load more activities
  async function loadMore() {
    if (loadingMore || !hasMore) return;
    loadingMore = true;
    currentPage++;
    await loadActivities(false);
  }

  // Handle scroll for infinite loading
  function handleScroll(event: Event) {
    const target = event.target as HTMLDivElement;
    const scrollBottom = target.scrollHeight - target.scrollTop - target.clientHeight;

    // Load more when within 100px of bottom
    if (scrollBottom < 100 && hasMore && !loadingMore && !loading) {
      loadMore();
    }
  }

  // Load on mount and reload when filter changes
  $effect(() => {
    entityFilter; // track dependency
    loadActivities(true);
  });

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
      case 'create': return 'emittiv-badge--green';
      case 'update': return 'emittiv-badge--blue';
      case 'delete': return 'emittiv-badge--red';
      case 'status_change': return 'emittiv-badge--orange';
      default: return 'emittiv-badge--gray';
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

    switch (entityType) {
      case 'project':
        push(`/projects/${entityId}`);
        break;
      case 'fee':
        push(`/proposals/${entityId}`);
        break;
      case 'company':
        push(`/companies/${entityId}`);
        break;
      case 'contact':
        push(`/contacts/${entityId}`);
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

</script>

<div class="activity-feed">
  <div class="feed-header">
    <h2 class="section-title">Recent Activity</h2>

    <div class="header-actions">
      <select bind:value={entityFilter} class="filter-select">
        {#each entityOptions as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>

      <button class="view-all-button" on:click={openFullActivityLog}>
        View All
        <svg class="view-all-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
      </button>
    </div>
  </div>

  <div class="feed-content">
    {#if isLoading || loading}
      <LoadingSkeleton rows={4} />
    {:else if error}
      <div class="error-state">
        <svg class="error-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <p class="error-message">{error}</p>
        <button class="retry-button" on:click={() => loadActivities()}>Retry</button>
      </div>
    {:else if activities.length > 0}
      <div class="activity-list" bind:this={scrollContainer} on:scroll={handleScroll}>
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
              <div class="activity-time">{formatRelativeTime(activity.timestamp)}</div>
              <div class="activity-user">{activity.user}</div>
            </div>
          </div>
        {/each}

        {#if loadingMore}
          <div class="loading-more">
            <div class="spinner"></div>
            <span>Loading more...</span>
          </div>
        {:else if !hasMore && activities.length > PAGE_SIZE}
          <div class="end-of-list">
            <span>No more activities</span>
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

  .header-actions {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .filter-controls {
    display: flex;
    gap: 12px;
    align-items: flex-end;
    min-height: 32px;
  }

  .view-all-button {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    background: transparent;
    border: 1px solid var(--emittiv-dark);
    border-radius: 8px;
    color: var(--emittiv-light);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
  }

  .view-all-button:hover {
    border-color: var(--emittiv-splash);
    color: var(--emittiv-splash);
    background: rgba(255, 153, 0, 0.08);
  }

  .view-all-icon {
    width: 14px;
    height: 14px;
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
    min-width: 100px;
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
    border-left: 3px solid;
    border-radius: 12px;
    transition: all 150ms ease;
    cursor: pointer;
  }

  .activity-item:hover {
    background: rgba(255, 153, 0, 0.06);
    border-color: rgba(255, 153, 0, 0.3);
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .activity-item:focus {
    outline: none;
    background: rgba(255, 153, 0, 0.06);
    border-color: var(--emittiv-splash);
  }

  .activity-icon {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .activity-icon .icon {
    width: 18px;
    height: 18px;
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
  }

  .entity-name {
    font-weight: 600;
    color: var(--emittiv-white);
    font-size: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .entity-type {
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--emittiv-dark);
    color: var(--emittiv-light);
    text-transform: capitalize;
  }

  .activity-description {
    font-size: 13px;
    color: var(--emittiv-lighter);
    margin-bottom: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .activity-change {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }

  .old-value {
    color: var(--emittiv-light);
    text-decoration: line-through;
  }

  .arrow-icon {
    width: 14px;
    height: 14px;
    color: var(--emittiv-light);
  }

  .new-value {
    color: var(--emittiv-splash);
    font-weight: 500;
  }

  .activity-meta {
    text-align: right;
    min-width: 70px;
  }

  .activity-time {
    font-size: 12px;
    color: var(--emittiv-light);
    font-weight: 500;
    margin-bottom: 2px;
  }

  .activity-user {
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
    margin: 0 0 8px 0;
  }

  .empty-hint {
    color: var(--emittiv-lighter);
    font-size: 12px;
    margin: 0;
  }

  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    text-align: center;
  }

  .error-icon {
    width: 48px;
    height: 48px;
    color: #ef4444;
    margin-bottom: 16px;
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

  .loading-more,
  .end-of-list {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 16px;
    color: var(--emittiv-light);
    font-size: 12px;
  }

  .loading-more .spinner {
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
    opacity: 0.6;
    font-style: italic;
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
      margin-left: 0;
    }

    .activity-item {
      grid-template-columns: auto 1fr;
    }

    .activity-meta {
      grid-column: 1 / -1;
      text-align: left;
      display: flex;
      gap: 12px;
      margin-top: 8px;
    }
  }
</style>
