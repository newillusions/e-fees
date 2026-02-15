<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { push } from 'svelte-spa-router';
  import { projectsStore, companiesStore, contactsStore, feesStore } from '$lib/stores';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import { extractId } from '$lib/utils';
  import {
    createProjectLookup,
    PROJECT_SEARCH_FIELDS,
    COMPANY_SEARCH_FIELDS,
    CONTACT_SEARCH_FIELDS,
    FEE_SEARCH_FIELDS,
    getProjectDisplayInfo,
    getCompanyDisplayInfo,
    getContactDisplayInfo,
    getFeeDisplayInfo,
    normalizedMatch
  } from '$lib/utils/search';
  import type { Project, Company, Contact, Fee } from '../../types';

  let { isOpen = $bindable(false) }: {
    isOpen?: boolean;
  } = $props();

  const dispatch = createEventDispatcher();

  // Search state
  let searchQuery = '';
  let inputElement: HTMLInputElement;
  let selectedIndex = 0;

  // Results
  interface SearchResult {
    type: 'project' | 'company' | 'contact' | 'proposal';
    id: string;
    name: string;
    subtitle: string;
    route: string;
  }

  const MAX_RESULTS_PER_TYPE = 5;

  // Entity icons
  const typeIcons: Record<string, string> = {
    project: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10',
    company: 'M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4',
    contact: 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z',
    proposal: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z'
  };

  const typeColors: Record<string, string> = {
    project: 'text-blue-400 bg-blue-400/20',
    company: 'text-green-400 bg-green-400/20',
    contact: 'text-purple-400 bg-purple-400/20',
    proposal: 'text-orange-400 bg-orange-400/20'
  };

  const typeLabels: Record<string, string> = {
    project: 'Project',
    company: 'Company',
    contact: 'Contact',
    proposal: 'Proposal'
  };

  // Search function - fuzzy match across all entities using unified search utilities
  // Uses normalizedMatch to handle punctuation (e.g., "uae" matches "U.A.E.")
  function performSearch(query: string): SearchResult[] {
    if (!query || query.length < 2) return [];

    const q = query.trim();
    const allResults: SearchResult[] = [];

    // Get data from stores
    const projects = $projectsStore || [];
    const companies = $companiesStore || [];
    const contacts = $contactsStore || [];
    const fees = $feesStore || [];

    // Create lookups for related entity search
    const companyLookup = createCompanyLookup(companies);
    const projectLookup = createProjectLookup(projects);
    const lookups = { companyLookup, projectLookup };

    // Search Projects using unified search fields with normalized matching
    const projectResults = projects
      .filter((p: Project) => {
        const searchFields = PROJECT_SEARCH_FIELDS.map(field => String(p[field] || ''));
        // Also search project_number which isn't in the standard fields
        searchFields.push(String(p.project_number || ''));
        return searchFields.some(f => normalizedMatch(f, q));
      })
      .slice(0, MAX_RESULTS_PER_TYPE)
      .map((p: Project): SearchResult => {
        const displayInfo = getProjectDisplayInfo(p);
        return {
          type: 'project',
          id: extractId(p.id),
          name: displayInfo.name,
          subtitle: displayInfo.subtitle,
          route: `/projects`
        };
      });
    allResults.push(...projectResults);

    // Search Companies using unified search fields with normalized matching
    const companyResults = companies
      .filter((c: Company) => {
        const searchFields = COMPANY_SEARCH_FIELDS.map(field => String(c[field] || ''));
        return searchFields.some(f => normalizedMatch(f, q));
      })
      .slice(0, MAX_RESULTS_PER_TYPE)
      .map((c: Company): SearchResult => {
        const displayInfo = getCompanyDisplayInfo(c);
        return {
          type: 'company',
          id: extractId(c.id),
          name: displayInfo.name,
          subtitle: displayInfo.subtitle,
          route: `/companies`
        };
      });
    allResults.push(...companyResults);

    // Search Contacts using unified search fields + company lookup with normalized matching
    const contactResults = contacts
      .filter((c: Contact) => {
        const searchFields = CONTACT_SEARCH_FIELDS.map(field => String(c[field] || ''));
        // Also search company name, abbreviation, name_short via lookup
        const companySearchText = companyLookup.getCompanySearchText(c.company);
        return searchFields.some(f => normalizedMatch(f, q)) || normalizedMatch(companySearchText, q);
      })
      .slice(0, MAX_RESULTS_PER_TYPE)
      .map((c: Contact): SearchResult => {
        const displayInfo = getContactDisplayInfo(c, lookups);
        return {
          type: 'contact',
          id: extractId(c.id),
          name: displayInfo.name,
          subtitle: displayInfo.subtitle,
          route: `/contacts`
        };
      });
    allResults.push(...contactResults);

    // Search Proposals/Fees using unified search fields + company/project lookups with normalized matching
    const feeResults = [...fees]
      .sort((a, b) => {
        // Sort by updated_at or created_at, newest first (fees use time.updated_at)
        const dateA = a.time?.updated_at || a.time?.created_at || '';
        const dateB = b.time?.updated_at || b.time?.created_at || '';
        return dateB.localeCompare(dateA);
      })
      .filter((f: Fee) => {
        // Search fee fields using unified search fields
        const feeFields = FEE_SEARCH_FIELDS.map(field => String(f[field] || ''));
        // Also search company name, abbreviation, name_short via lookup
        const companySearchText = companyLookup.getCompanySearchText(f.company_id);
        // Also search project name via lookup
        const projectId = extractId(f.project_id);
        const project = projectLookup.get(projectId);
        const projectSearchText = [project?.name, project?.name_short].filter(Boolean).join(' ');
        return feeFields.some(field => normalizedMatch(field, q)) ||
               normalizedMatch(companySearchText, q) ||
               normalizedMatch(projectSearchText, q);
      })
      .slice(0, MAX_RESULTS_PER_TYPE)
      .map((f: Fee): SearchResult => {
        const displayInfo = getFeeDisplayInfo(f, lookups);
        return {
          type: 'proposal',
          id: extractId(f.id),
          name: displayInfo.name,
          subtitle: displayInfo.subtitle,
          route: `/proposals`
        };
      });
    allResults.push(...feeResults);

    return allResults;
  }

  // Reactive search
  const results = $derived(performSearch(searchQuery));
  $effect(() => {
    if (results.length > 0 && selectedIndex >= results.length) {
      selectedIndex = 0;
    }
  });

  // Navigation
  function navigateToResult(result: SearchResult) {
    push(result.route);
    closeModal();
  }

  function closeModal() {
    searchQuery = '';
    selectedIndex = 0;
    isOpen = false;
    dispatch('close');
  }

  // Keyboard navigation
  function handleKeydown(event: KeyboardEvent) {
    if (!isOpen) return;

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        if (results.length > 0) {
          selectedIndex = (selectedIndex + 1) % results.length;
        }
        break;
      case 'ArrowUp':
        event.preventDefault();
        if (results.length > 0) {
          selectedIndex = selectedIndex === 0 ? results.length - 1 : selectedIndex - 1;
        }
        break;
      case 'Enter':
        event.preventDefault();
        if (results.length > 0 && results[selectedIndex]) {
          navigateToResult(results[selectedIndex]);
        }
        break;
      case 'Escape':
        event.preventDefault();
        closeModal();
        break;
    }
  }

  // Focus input when modal opens
  $effect(() => {
    if (isOpen && inputElement) {
      setTimeout(() => inputElement?.focus(), 50);
    }
  });

  // Handle backdrop click
  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      closeModal();
    }
  }
</script>

{#if isOpen}
  <div
    class="modal-backdrop"
    on:click={handleBackdropClick}
    on:keydown={handleKeydown}
    role="dialog"
    aria-modal="true"
    aria-labelledby="search-title"
    tabindex="-1"
  >
    <div class="search-container">
      <!-- Search Input -->
      <div class="search-input-wrapper">
        <svg class="search-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <input
          bind:this={inputElement}
          bind:value={searchQuery}
          type="text"
          class="search-input"
          placeholder="Search projects, companies, contacts, proposals..."
          id="search-title"
          autocomplete="off"
          spellcheck="false"
        />
        <div class="keyboard-hint">
          <kbd>ESC</kbd> to close
        </div>
      </div>

      <!-- Results -->
      {#if searchQuery.length >= 2}
        <div class="results-container">
          {#if results.length > 0}
            <div class="results-list">
              {#each results as result, index}
                <button
                  class="result-item"
                  class:selected={index === selectedIndex}
                  on:click={() => navigateToResult(result)}
                  on:mouseenter={() => selectedIndex = index}
                >
                  <div class="result-icon {typeColors[result.type]}">
                    <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={typeIcons[result.type]} />
                    </svg>
                  </div>
                  <div class="result-content">
                    <div class="result-name">{result.name}</div>
                    <div class="result-subtitle">{result.subtitle}</div>
                  </div>
                  <span class="result-type">{typeLabels[result.type]}</span>
                </button>
              {/each}
            </div>
          {:else}
            <div class="no-results">
              <svg class="no-results-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <p>No results found for "{searchQuery}"</p>
            </div>
          {/if}
        </div>
      {:else if searchQuery.length > 0}
        <div class="results-container">
          <div class="hint-text">Type at least 2 characters to search</div>
        </div>
      {:else}
        <div class="results-container">
          <div class="search-hints">
            <div class="hint-title">Quick Search</div>
            <div class="hint-categories">
              <div class="hint-category">
                <span class="category-icon {typeColors.project}">
                  <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={typeIcons.project} />
                  </svg>
                </span>
                <span>Projects</span>
              </div>
              <div class="hint-category">
                <span class="category-icon {typeColors.company}">
                  <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={typeIcons.company} />
                  </svg>
                </span>
                <span>Companies</span>
              </div>
              <div class="hint-category">
                <span class="category-icon {typeColors.contact}">
                  <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={typeIcons.contact} />
                  </svg>
                </span>
                <span>Contacts</span>
              </div>
              <div class="hint-category">
                <span class="category-icon {typeColors.proposal}">
                  <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={typeIcons.proposal} />
                  </svg>
                </span>
                <span>Proposals</span>
              </div>
            </div>
          </div>
        </div>
      {/if}

      <!-- Footer -->
      <div class="search-footer">
        <div class="footer-hint">
          <kbd>↑</kbd><kbd>↓</kbd> to navigate
          <span class="separator">•</span>
          <kbd>Enter</kbd> to select
          <span class="separator">•</span>
          <kbd>Esc</kbd> to close
        </div>
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
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
    z-index: 1000;
    animation: fadeIn 150ms ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .search-container {
    width: 100%;
    max-width: 640px;
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 16px;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.4);
    overflow: hidden;
    animation: slideDown 200ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-20px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 20px;
    border-bottom: 1px solid var(--emittiv-dark);
  }

  .search-icon {
    width: 24px;
    height: 24px;
    color: var(--emittiv-light);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    font-size: 18px;
    color: var(--emittiv-white);
    font-family: inherit;
  }

  .search-input::placeholder {
    color: var(--emittiv-light);
  }

  .keyboard-hint {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--emittiv-light);
    font-size: 12px;
  }

  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    background: var(--emittiv-dark);
    border: 1px solid var(--emittiv-light);
    border-radius: 4px;
    font-size: 11px;
    font-family: inherit;
    color: var(--emittiv-lighter);
  }

  .results-container {
    max-height: 400px;
    overflow-y: auto;
  }

  .results-list {
    padding: 8px;
  }

  .result-item {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px;
    background: transparent;
    border: none;
    border-radius: 10px;
    cursor: pointer;
    text-align: left;
    transition: background 100ms ease;
  }

  .result-item:hover,
  .result-item.selected {
    background: rgba(255, 153, 0, 0.1);
  }

  .result-item.selected {
    outline: 2px solid rgba(255, 153, 0, 0.5);
    outline-offset: -2px;
  }

  .result-icon {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .result-icon .icon {
    width: 20px;
    height: 20px;
  }

  .result-content {
    flex: 1;
    min-width: 0;
  }

  .result-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--emittiv-white);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .result-subtitle {
    font-size: 12px;
    color: var(--emittiv-light);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .result-type {
    font-size: 11px;
    padding: 4px 8px;
    background: var(--emittiv-dark);
    border-radius: 6px;
    color: var(--emittiv-light);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    flex-shrink: 0;
  }

  .no-results {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    text-align: center;
  }

  .no-results-icon {
    width: 48px;
    height: 48px;
    color: var(--emittiv-light);
    margin-bottom: 12px;
  }

  .no-results p {
    color: var(--emittiv-light);
    font-size: 14px;
    margin: 0;
  }

  .hint-text {
    padding: 24px;
    text-align: center;
    color: var(--emittiv-light);
    font-size: 14px;
  }

  .search-hints {
    padding: 24px;
  }

  .hint-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--emittiv-light);
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-bottom: 16px;
  }

  .hint-categories {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .hint-category {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px;
    background: var(--emittiv-black);
    border: 1px solid var(--emittiv-dark);
    border-radius: 10px;
    color: var(--emittiv-lighter);
    font-size: 13px;
  }

  .category-icon {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .category-icon .icon {
    width: 16px;
    height: 16px;
  }

  .search-footer {
    padding: 12px 20px;
    border-top: 1px solid var(--emittiv-dark);
    background: var(--emittiv-black);
  }

  .footer-hint {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 12px;
    color: var(--emittiv-light);
  }

  .separator {
    opacity: 0.5;
  }
</style>
