<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { importScanDirectory, importExecute } from '$lib/api/import';
  import type { ImportPreview, ImportResult } from '$lib/api/import';
  import { selectFolder } from '$lib/api';

  let {
    isOpen = $bindable(false),
    onclose,
    onimported
  }: {
    isOpen?: boolean;
    onclose?: () => void;
    onimported?: (result: ImportResult | null) => void;
  } = $props();

  // Wizard state
  let step: 'select' | 'preview' | 'importing' | 'result' = $state('select');
  let directory = $state('');
  let preview: ImportPreview | null = $state(null);
  let result: ImportResult | null = $state(null);
  let error = $state('');
  let scanLoading = $state(false);

  // Import options
  let importProposals = $state(true);
  let importCompanies = $state(true);

  function closeModal() {
    step = 'select';
    directory = '';
    preview = null;
    result = null;
    error = '';
    scanLoading = false;
    onclose?.();
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget && step !== 'importing') {
      closeModal();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && step !== 'importing') {
      closeModal();
    }
  }

  async function handleBrowse() {
    try {
      const selected = await selectFolder();
      if (selected) {
        directory = selected;
      }
    } catch (err) {
      error = 'Failed to open folder selector';
    }
  }

  async function handleScan() {
    if (!directory.trim()) {
      error = 'Please select a directory';
      return;
    }

    error = '';
    scanLoading = true;

    try {
      preview = await importScanDirectory(directory);
      step = 'preview';
    } catch (err: unknown) {
      error = typeof err === 'string' ? err : (err as Error)?.message || 'Failed to scan directory';
    } finally {
      scanLoading = false;
    }
  }

  async function handleImport() {
    if (!preview) return;

    step = 'importing';
    error = '';

    try {
      result = await importExecute(directory, importProposals, importCompanies);
      step = 'result';
    } catch (err: unknown) {
      error = typeof err === 'string' ? err : (err as Error)?.message || 'Import failed';
      step = 'preview';
    }
  }

  function handleDone() {
    onimported?.(result);
    closeModal();
  }

  function formatCurrency(amount: number, currency: string): string {
    return `${currency} ${amount.toLocaleString()}`;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div
    class="emittiv-backdrop"
    style="z-index: 100;"
    on:click={handleBackdropClick}
    on:keydown={() => {}}
    role="button"
    tabindex="-1"
    aria-label="Close modal"
    in:fade={{ duration: 200 }}
    out:fade={{ duration: 200 }}
  ></div>

  <div
    class="fixed inset-0 flex items-center justify-center p-4 pointer-events-none"
    style="z-index: 101;"
    role="dialog"
    aria-modal="true"
    aria-labelledby="import-wizard-title"
  >
    <div
      class="emittiv-modal w-full pointer-events-auto"
      style="max-width: 680px;"
      on:click={e => e.stopPropagation()}
      in:scale={{ duration: 250, start: 0.95, easing: cubicOut }}
      out:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
    >
      <!-- Header -->
      <div class="emittiv-modal__header">
        <h2 id="import-wizard-title" class="emittiv-modal__title">
          {#if step === 'select'}Import RFPs Data{/if}
          {#if step === 'preview'}Import Preview{/if}
          {#if step === 'importing'}Importing...{/if}
          {#if step === 'result'}Import Complete{/if}
        </h2>
        {#if step !== 'importing'}
          <button on:click={closeModal} class="emittiv-modal__close" aria-label="Close modal">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        {/if}
      </div>

      <!-- Step indicator -->
      <div class="import-wizard-steps">
        <div
          class="import-wizard-step"
          class:active={step === 'select'}
          class:done={step !== 'select'}
        >
          <span class="import-wizard-step__number">1</span>
          <span class="import-wizard-step__label">Select</span>
        </div>
        <div class="import-wizard-step__line" class:done={step !== 'select'}></div>
        <div
          class="import-wizard-step"
          class:active={step === 'preview'}
          class:done={step === 'importing' || step === 'result'}
        >
          <span class="import-wizard-step__number">2</span>
          <span class="import-wizard-step__label">Preview</span>
        </div>
        <div
          class="import-wizard-step__line"
          class:done={step === 'importing' || step === 'result'}
        ></div>
        <div
          class="import-wizard-step"
          class:active={step === 'importing' || step === 'result'}
          class:done={step === 'result'}
        >
          <span class="import-wizard-step__number">3</span>
          <span class="import-wizard-step__label">Import</span>
        </div>
      </div>

      <!-- Content -->
      <div class="import-wizard-content">
        <!-- Step 1: Select Directory -->
        {#if step === 'select'}
          <p class="import-wizard-description">
            Select the directory containing RFPs JSON export files (staff-rates.json, proposal
            files, etc.)
          </p>

          <div class="import-wizard-directory-select">
            <input
              type="text"
              class="emittiv-input"
              placeholder="Select export directory..."
              bind:value={directory}
              readonly
            />
            <button class="emittiv-btn emittiv-btn--secondary" on:click={handleBrowse}>
              Browse
            </button>
          </div>

          {#if error}
            <div class="import-wizard-error">{error}</div>
          {/if}

          <div class="import-wizard-actions">
            <button class="emittiv-btn emittiv-btn--ghost" on:click={closeModal}>Cancel</button>
            <button
              class="emittiv-btn emittiv-btn--primary"
              on:click={handleScan}
              disabled={!directory.trim() || scanLoading}
            >
              {#if scanLoading}Scanning...{:else}Scan Directory{/if}
            </button>
          </div>

          <!-- Step 2: Preview -->
        {:else if step === 'preview' && preview}
          <div class="import-wizard-preview">
            <!-- Proposals -->
            {#if preview.proposals.length > 0}
              <div class="import-wizard-section">
                <div class="import-wizard-section__header">
                  <label class="import-wizard-checkbox">
                    <input type="checkbox" bind:checked={importProposals} />
                    <span>Proposals ({preview.proposals.length})</span>
                  </label>
                </div>
                {#if importProposals}
                  <div class="import-wizard-table">
                    <div class="import-wizard-table__header">
                      <span class="import-wizard-table__col import-wizard-table__col--name"
                        >Project</span
                      >
                      <span class="import-wizard-table__col import-wizard-table__col--num">#</span>
                      <span class="import-wizard-table__col import-wizard-table__col--fee">Fee</span
                      >
                      <span class="import-wizard-table__col import-wizard-table__col--type"
                        >Type</span
                      >
                    </div>
                    {#each preview.proposals as proposal}
                      <div class="import-wizard-table__row">
                        <span
                          class="import-wizard-table__col import-wizard-table__col--name"
                          title={proposal.project_name}
                        >
                          {proposal.project_name}
                        </span>
                        <span class="import-wizard-table__col import-wizard-table__col--num"
                          >{proposal.project_number}</span
                        >
                        <span class="import-wizard-table__col import-wizard-table__col--fee">
                          {formatCurrency(proposal.total_fee, proposal.currency)}
                        </span>
                        <span class="import-wizard-table__col import-wizard-table__col--type">
                          {#if proposal.stage_type === 'custom_milestones'}
                            {proposal.stage_count} milestones
                          {:else}
                            {proposal.phase_count} phases
                          {/if}
                        </span>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}

            <!-- Companies -->
            {#if preview.clients.length > 0}
              <div class="import-wizard-section">
                <div class="import-wizard-section__header">
                  <label class="import-wizard-checkbox">
                    <input type="checkbox" bind:checked={importCompanies} />
                    <span>Companies ({preview.clients.length})</span>
                  </label>
                </div>
                {#if importCompanies}
                  <div class="import-wizard-table">
                    <div class="import-wizard-table__header">
                      <span class="import-wizard-table__col import-wizard-table__col--name"
                        >Name</span
                      >
                      <span class="import-wizard-table__col import-wizard-table__col--type"
                        >Type</span
                      >
                      <span class="import-wizard-table__col import-wizard-table__col--num"
                        >Projects</span
                      >
                    </div>
                    {#each preview.clients as client}
                      <div class="import-wizard-table__row">
                        <span class="import-wizard-table__col import-wizard-table__col--name"
                          >{client.name}</span
                        >
                        <span class="import-wizard-table__col import-wizard-table__col--type"
                          >{client.client_type}</span
                        >
                        <span class="import-wizard-table__col import-wizard-table__col--num"
                          >{client.project_count}</span
                        >
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}

            <!-- Methodology -->
            {#if preview.methodology}
              <div class="import-wizard-section">
                <div class="import-wizard-section__header">
                  <span class="import-wizard-section__title">Methodology</span>
                </div>
                <div class="import-wizard-info-grid">
                  <span class="import-wizard-info-label">WHT Rate</span>
                  <span class="import-wizard-info-value"
                    >{(preview.methodology.wht_rate * 100).toFixed(0)}%</span
                  >
                  <span class="import-wizard-info-label">AED to SAR</span>
                  <span class="import-wizard-info-value">{preview.methodology.aed_to_sar}</span>
                  <span class="import-wizard-info-label">Overhead</span>
                  <span class="import-wizard-info-value"
                    >{(preview.methodology.overhead_rate * 100).toFixed(0)}%</span
                  >
                  <span class="import-wizard-info-label">Margin</span>
                  <span class="import-wizard-info-value"
                    >{(preview.methodology.margin_rate * 100).toFixed(0)}%</span
                  >
                </div>
              </div>
            {/if}

            <!-- Staff Rates -->
            {#if preview.staff_rates}
              <div class="import-wizard-section">
                <div class="import-wizard-section__header">
                  <span class="import-wizard-section__title">
                    Staff Rates ({preview.staff_rates.count} roles in {preview.staff_rates
                      .currency})
                  </span>
                </div>
              </div>
            {/if}

            <!-- Warnings -->
            {#if preview.warnings.length > 0}
              <div class="import-wizard-warnings">
                {#each preview.warnings as warning}
                  <div class="import-wizard-warning">{warning}</div>
                {/each}
              </div>
            {/if}
          </div>

          {#if error}
            <div class="import-wizard-error">{error}</div>
          {/if}

          <div class="import-wizard-actions">
            <button
              class="emittiv-btn emittiv-btn--ghost"
              on:click={() => {
                step = 'select';
              }}>Back</button
            >
            <button
              class="emittiv-btn emittiv-btn--primary"
              on:click={handleImport}
              disabled={!importProposals && !importCompanies}
            >
              Import Selected
            </button>
          </div>

          <!-- Step 3: Importing -->
        {:else if step === 'importing'}
          <div class="import-wizard-loading">
            <div class="import-wizard-spinner"></div>
            <p>Importing data into E-Fees...</p>
            <p class="import-wizard-loading__sub">This may take a few seconds.</p>
          </div>

          <!-- Step 4: Result -->
        {:else if step === 'result' && result}
          <div class="import-wizard-result">
            <div
              class="import-wizard-result__icon"
              class:success={result.success}
              class:error={!result.success}
            >
              {#if result.success}
                <svg class="w-10 h-10" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              {:else}
                <svg class="w-10 h-10" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"
                  />
                </svg>
              {/if}
            </div>

            <div class="import-wizard-result__stats">
              {#if result.projects_created > 0}
                <div class="import-wizard-stat">
                  <span class="import-wizard-stat__value">{result.projects_created}</span>
                  <span class="import-wizard-stat__label">Projects Created</span>
                </div>
              {/if}
              {#if result.fees_created > 0}
                <div class="import-wizard-stat">
                  <span class="import-wizard-stat__value">{result.fees_created}</span>
                  <span class="import-wizard-stat__label">Fees Created</span>
                </div>
              {/if}
              {#if result.companies_created > 0}
                <div class="import-wizard-stat">
                  <span class="import-wizard-stat__value">{result.companies_created}</span>
                  <span class="import-wizard-stat__label">Companies Created</span>
                </div>
              {/if}
              {#if result.companies_skipped > 0}
                <div class="import-wizard-stat import-wizard-stat--muted">
                  <span class="import-wizard-stat__value">{result.companies_skipped}</span>
                  <span class="import-wizard-stat__label">Companies Skipped</span>
                </div>
              {/if}
            </div>

            {#if result.warnings.length > 0}
              <div class="import-wizard-warnings">
                {#each result.warnings as warning}
                  <div class="import-wizard-warning">{warning}</div>
                {/each}
              </div>
            {/if}

            {#if result.errors.length > 0}
              <div class="import-wizard-errors">
                {#each result.errors as err}
                  <div class="import-wizard-error">{err}</div>
                {/each}
              </div>
            {/if}
          </div>

          <div class="import-wizard-actions">
            <button class="emittiv-btn emittiv-btn--primary" on:click={handleDone}>Done</button>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .import-wizard-steps {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0;
    margin-bottom: 24px;
    padding: 0 20px;
  }

  .import-wizard-step {
    display: flex;
    align-items: center;
    gap: 6px;
    opacity: 0.4;
    transition: opacity 0.2s ease;
  }

  .import-wizard-step.active,
  .import-wizard-step.done {
    opacity: 1;
  }

  .import-wizard-step__number {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background-color: var(--emittiv-dark);
    color: var(--emittiv-lighter);
    font-size: 11px;
    font-weight: 600;
  }

  .import-wizard-step.active .import-wizard-step__number {
    background-color: var(--emittiv-splash);
    color: var(--emittiv-black);
  }

  .import-wizard-step.done .import-wizard-step__number {
    background-color: var(--emittiv-splash);
    color: var(--emittiv-black);
  }

  .import-wizard-step__label {
    font-size: 12px;
    color: var(--emittiv-lighter);
  }

  .import-wizard-step__line {
    width: 40px;
    height: 2px;
    background-color: var(--emittiv-dark);
    margin: 0 8px;
    transition: background-color 0.2s ease;
  }

  .import-wizard-step__line.done {
    background-color: var(--emittiv-splash);
  }

  .import-wizard-content {
    min-height: 200px;
  }

  .import-wizard-description {
    font-size: 13px;
    color: var(--emittiv-light);
    margin-bottom: 16px;
    line-height: 1.5;
  }

  .import-wizard-directory-select {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }

  .import-wizard-directory-select input {
    flex: 1;
  }

  .import-wizard-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--emittiv-dark);
  }

  .import-wizard-error {
    color: var(--color-error);
    font-size: 12px;
    padding: 8px 12px;
    background-color: color-mix(in srgb, var(--color-error) 10%, transparent);
    border-radius: 4px;
    margin-top: 8px;
  }

  /* Preview section */
  .import-wizard-preview {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-height: 400px;
    overflow-y: auto;
  }

  .import-wizard-section {
    border: 1px solid var(--emittiv-dark);
    border-radius: 6px;
    padding: 12px;
  }

  .import-wizard-section__header {
    margin-bottom: 8px;
  }

  .import-wizard-section__title {
    font-size: 13px;
    font-weight: 600;
    color: var(--emittiv-lighter);
  }

  .import-wizard-checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    color: var(--emittiv-white);
  }

  .import-wizard-checkbox input[type='checkbox'] {
    accent-color: var(--emittiv-splash);
    width: 16px;
    height: 16px;
  }

  /* Table */
  .import-wizard-table {
    font-size: 12px;
  }

  .import-wizard-table__header {
    display: flex;
    gap: 8px;
    padding: 4px 0;
    border-bottom: 1px solid var(--emittiv-dark);
    color: var(--emittiv-light);
    font-weight: 500;
  }

  .import-wizard-table__row {
    display: flex;
    gap: 8px;
    padding: 6px 0;
    border-bottom: 1px solid rgba(102, 102, 102, 0.3);
    color: var(--emittiv-lighter);
  }

  .import-wizard-table__col--name {
    flex: 2;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .import-wizard-table__col--num {
    width: 50px;
    text-align: center;
  }

  .import-wizard-table__col--fee {
    width: 120px;
    text-align: right;
    font-family: 'Montserrat', monospace;
  }

  .import-wizard-table__col--type {
    width: 100px;
    text-align: right;
  }

  /* Info grid */
  .import-wizard-info-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 4px 16px;
    font-size: 12px;
  }

  .import-wizard-info-label {
    color: var(--emittiv-light);
  }

  .import-wizard-info-value {
    color: var(--emittiv-lighter);
    font-weight: 500;
  }

  /* Warnings */
  .import-wizard-warnings {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .import-wizard-warning {
    color: var(--color-warning);
    font-size: 12px;
    padding: 6px 10px;
    background-color: color-mix(in srgb, var(--color-warning) 10%, transparent);
    border-radius: 4px;
  }

  .import-wizard-errors {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  /* Loading */
  .import-wizard-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 0;
    color: var(--emittiv-lighter);
    font-size: 14px;
  }

  .import-wizard-loading__sub {
    color: var(--emittiv-light);
    font-size: 12px;
    margin-top: 4px;
  }

  .import-wizard-spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--emittiv-dark);
    border-top-color: var(--emittiv-splash);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 16px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Result */
  .import-wizard-result {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 16px 0;
  }

  .import-wizard-result__icon {
    margin-bottom: 16px;
  }

  .import-wizard-result__icon.success {
    color: var(--color-success);
  }

  .import-wizard-result__icon.error {
    color: var(--color-error);
  }

  .import-wizard-result__stats {
    display: flex;
    gap: 24px;
    margin-bottom: 16px;
  }

  .import-wizard-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .import-wizard-stat__value {
    font-size: 24px;
    font-weight: 700;
    color: var(--emittiv-splash);
    font-family: var(--font-heading);
  }

  .import-wizard-stat--muted .import-wizard-stat__value {
    color: var(--emittiv-light);
  }

  .import-wizard-stat__label {
    font-size: 11px;
    color: var(--emittiv-light);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
</style>
