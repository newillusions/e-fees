<script lang="ts">
  import { push } from 'svelte-spa-router';
  import CompanyModal from '$lib/components/CompanyModal.svelte';
  import ContactModal from '$lib/components/ContactModal.svelte';
  import NewProjectModal from '$lib/components/NewProjectModal.svelte';
  import DisciplinesPanel from '$lib/components/pricing/DisciplinesPanel.svelte';
  import StagesPanel from '$lib/components/pricing/StagesPanel.svelte';
  import CostsPanel from '$lib/components/pricing/CostsPanel.svelte';
  import PricingCalculatorPanel from '$lib/components/pricing/PricingCalculatorPanel.svelte';
  import PaymentSchedulePanel from '$lib/components/pricing/PaymentSchedulePanel.svelte';
  import PricingSummaryBar from '$lib/components/pricing/PricingSummaryBar.svelte';
  import type {
    Discipline,
    Stage,
    PricingCell,
    PostContractItem,
    ReimbursableCost,
    PaymentSchedule,
    PricingConfig,
    PricingBreakdown
  } from '../types/database';
  import {
    createDefaultDisciplines,
    createDefaultDesignStages,
    createDefaultPostContractStages,
    calculatePricingTotals,
    DEFAULT_PRICING_CONFIG
  } from '../types/database';

  // Form modal states
  let activeForm = $state<string | null>(null);
  let showCompanyForm = $state(false);
  let showContactForm = $state(false);
  let showProjectForm = $state(false);

  // Scope builder navigation
  let scopeFeeId = $state('');
  function navigateToScope() {
    if (scopeFeeId.trim()) push(`/scope/${scopeFeeId.trim()}`);
  }

  // ============================================================================
  // PRICING MODULE STATE
  // ============================================================================
  let activeTab = $state<'disciplines' | 'stages' | 'costs' | 'calculator' | 'payments'>(
    'disciplines'
  );

  // Initialize with defaults
  let config = $state<PricingConfig>({
    ...DEFAULT_PRICING_CONFIG,
    target_fee: 250000,
    quoted_fee: 263158
  });
  let disciplines = $state<Discipline[]>(createDefaultDisciplines());
  let stages = $state<Stage[]>([
    ...createDefaultDesignStages(),
    ...createDefaultPostContractStages()
  ]);
  let cells = $state<PricingCell[]>([]);
  let postContractItems = $state<PostContractItem[]>([]);
  let reimbursableCosts = $state<ReimbursableCost[]>([]);
  let paymentSchedule = $state<PaymentSchedule>({
    entries: [],
    total_invoiced: 0,
    total_paid: 0,
    total_outstanding: 0
  });

  // Calculate totals reactively
  const totals = $derived(
    calculatePricingTotals(cells, postContractItems, reimbursableCosts, config, stages)
  );

  // Build pricing breakdown for summary bar
  const pricingBreakdown = $derived<PricingBreakdown>({
    config,
    disciplines,
    stages,
    cells,
    costs: reimbursableCosts,
    ...totals
  });

  // Update handlers for pricing panels
  function handleConfigUpdate(newConfig: PricingConfig) {
    config = newConfig;
  }

  function handleCellsUpdate(newCells: PricingCell[]) {
    cells = newCells;
  }

  function handleDisciplinesUpdate(newDisciplines: Discipline[]) {
    disciplines = newDisciplines;
  }

  function handleStagesUpdate(newStages: Stage[]) {
    stages = newStages;
  }

  function handlePostContractUpdate(newItems: PostContractItem[]) {
    postContractItems = newItems;
  }

  function handleCostsUpdate(newCosts: ReimbursableCost[]) {
    reimbursableCosts = newCosts;
  }

  function handlePaymentScheduleUpdate(newSchedule: PaymentSchedule) {
    paymentSchedule = newSchedule;
  }

  function resetPricingData() {
    config = { ...DEFAULT_PRICING_CONFIG, target_fee: 250000, quoted_fee: 263158 };
    disciplines = createDefaultDisciplines();
    stages = [...createDefaultDesignStages(), ...createDefaultPostContractStages()];
    cells = [];
    postContractItems = [];
    reimbursableCosts = [];
    paymentSchedule = { entries: [], total_invoiced: 0, total_paid: 0, total_outstanding: 0 };
    activeTab = 'disciplines';
  }

  // Tab config
  const tabs = [
    {
      id: 'disciplines' as const,
      label: 'Disciplines',
      icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10'
    },
    {
      id: 'stages' as const,
      label: 'Stages',
      icon: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01'
    },
    {
      id: 'costs' as const,
      label: 'Costs',
      icon: 'M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z'
    },
    {
      id: 'calculator' as const,
      label: 'Calculator',
      icon: 'M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z'
    },
    {
      id: 'payments' as const,
      label: 'Payments',
      icon: 'M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z'
    }
  ];

  // Form modal handlers
  function handleFormClose() {
    showCompanyForm = false;
    showContactForm = false;
    showProjectForm = false;
    activeForm = null;
  }

  function openForm(formType: string) {
    handleFormClose();
    activeForm = formType;
    switch (formType) {
      case 'company':
        showCompanyForm = true;
        break;
      case 'contact':
        showContactForm = true;
        break;
      case 'project':
        showProjectForm = true;
        break;
    }
  }

  function handleCompanySubmit() {
    handleFormClose();
  }

  function handleContactSubmit() {
    handleFormClose();
  }

  function handleProjectSubmit() {
    handleFormClose();
  }
</script>

<div class="dev-container">
  <!-- ============================================================================
       PRICING MODULE - INLINE TESTING
       Full pricing workflow embedded for direct testing without modal.
       ============================================================================ -->
  <section class="dev-section pricing-section">
    <div class="pricing-header">
      <h2 class="section-title">Fee Pricing Module</h2>
      <button class="reset-btn" onclick={resetPricingData}>
        <svg
          style="width: 14px; height: 14px;"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
          />
        </svg>
        Reset
      </button>
    </div>

    <!-- Summary Bar -->
    <div class="mb-3">
      <PricingSummaryBar pricing={pricingBreakdown} />
    </div>

    <!-- Tab Navigation -->
    <div class="flex gap-0.5 mb-3 border-b border-emittiv-dark pb-1">
      {#each tabs as tab}
        <button
          type="button"
          class="emittiv-tab"
          class:emittiv-tab--active={activeTab === tab.id}
          onclick={() => (activeTab = tab.id)}
        >
          <svg class="emittiv-tab__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={tab.icon} />
          </svg>
          {tab.label}
        </button>
      {/each}
    </div>

    <!-- Tab Content -->
    <div class="pricing-content">
      {#if activeTab === 'disciplines'}
        <DisciplinesPanel bind:disciplines onUpdate={handleDisciplinesUpdate} />
      {:else if activeTab === 'stages'}
        <StagesPanel bind:stages onUpdateStages={handleStagesUpdate} />
      {:else if activeTab === 'costs'}
        <CostsPanel bind:costs={reimbursableCosts} {stages} onUpdate={handleCostsUpdate} />
      {:else if activeTab === 'calculator'}
        <PricingCalculatorPanel
          bind:config
          {disciplines}
          {stages}
          bind:cells
          bind:postContractItems
          onUpdateConfig={handleConfigUpdate}
          onUpdateCells={handleCellsUpdate}
          onUpdatePostContract={handlePostContractUpdate}
        />
      {:else if activeTab === 'payments'}
        <PaymentSchedulePanel
          bind:schedule={paymentSchedule}
          {stages}
          {cells}
          {config}
          designTotal={totals.grand_total}
          onUpdate={handlePaymentScheduleUpdate}
        />
      {/if}
    </div>
  </section>

  <!-- Form Selection Cards -->
  <section class="dev-section">
    <h2 class="section-title">CRUD Form Testing</h2>
    <div class="form-cards-grid">
      <button
        class="form-card"
        class:active={activeForm === 'company'}
        onclick={() => openForm('company')}
      >
        <div class="form-card-content">
          <div class="form-icon">
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"
              />
            </svg>
          </div>
          <div class="form-info">
            <div class="form-title">Company</div>
            <div class="form-desc">Create and edit companies</div>
          </div>
        </div>
      </button>

      <button
        class="form-card"
        class:active={activeForm === 'contact'}
        onclick={() => openForm('contact')}
      >
        <div class="form-card-content">
          <div class="form-icon">
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"
              />
            </svg>
          </div>
          <div class="form-info">
            <div class="form-title">Contact</div>
            <div class="form-desc">Create and edit contacts</div>
          </div>
        </div>
      </button>

      <button
        class="form-card"
        class:active={activeForm === 'project'}
        onclick={() => openForm('project')}
      >
        <div class="form-card-content">
          <div class="form-icon">
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
              />
            </svg>
          </div>
          <div class="form-info">
            <div class="form-title">Project</div>
            <div class="form-desc">Create and edit projects</div>
          </div>
        </div>
      </button>
    </div>
  </section>

  <!-- Scope Builder -->
  <section class="dev-section">
    <h2 class="section-title">Scope Builder</h2>
    <div class="dev-tool-card">
      <div class="dev-tool-header">
        <div class="dev-tool-info">
          <div class="dev-tool-title">Scope Builder</div>
          <div class="dev-tool-desc">
            Open the scope builder for a fee proposal by ID (e.g. <code class="dev-code"
              >24_96606_1</code
            >)
          </div>
        </div>
      </div>
      <div class="dev-tool-body">
        <div class="dev-tool-row">
          <input
            class="emittiv-input dev-scope-input"
            type="text"
            placeholder="Fee ID (e.g. 24_96606_1)"
            bind:value={scopeFeeId}
            onkeydown={e => {
              if (e.key === 'Enter' && scopeFeeId.trim()) navigateToScope();
            }}
          />
          <button
            class="emittiv-btn emittiv-btn--primary"
            disabled={!scopeFeeId.trim()}
            onclick={navigateToScope}
          >
            Open Scope Builder
          </button>
        </div>
        <p class="dev-tool-note">
          Note: Scope service connectivity from AI server is still WIP — use for UI/design
          validation only.
        </p>
      </div>
    </div>
  </section>

  <!-- Design System: Color Test Page -->
  <section class="dev-section">
    <h2 class="section-title">Design System — Color Palette</h2>
    <div class="color-test-grid">
      <!-- Core brand tokens -->
      <div class="color-group">
        <h3 class="color-group-title">Core Brand Tokens</h3>
        <div class="color-swatches">
          <div class="color-swatch">
            <div class="swatch-block" style="background: var(--emittiv-black);"></div>
            <div class="swatch-label">--emittiv-black</div>
            <div class="swatch-value">#11111b</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: var(--emittiv-darker);"></div>
            <div class="swatch-label">--emittiv-darker</div>
            <div class="swatch-value">#1e1e2e</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: var(--emittiv-dark);"></div>
            <div class="swatch-label">--emittiv-dark</div>
            <div class="swatch-value">#666666</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: var(--emittiv-light);"></div>
            <div class="swatch-label">--emittiv-light</div>
            <div class="swatch-value">#999999</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: var(--emittiv-lighter);"></div>
            <div class="swatch-label">--emittiv-lighter</div>
            <div class="swatch-value">#cccccc</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: var(--emittiv-white);"></div>
            <div class="swatch-label">--emittiv-white</div>
            <div class="swatch-value">#ffffff</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: var(--emittiv-splash);"></div>
            <div class="swatch-label">--emittiv-splash</div>
            <div class="swatch-value">#ff9900</div>
          </div>
        </div>
      </div>

      <!-- Catppuccin Mocha — full accent palette -->
      <div class="color-group">
        <h3 class="color-group-title">Catppuccin Mocha — Accent Colours</h3>
        <p class="color-group-note">
          The full Mocha accent palette. All 14 named colours available as <code class="dev-code"
            >--ctp-mocha-*</code
          > tokens alongside the emittiv splash. The dark surface/overlay shades are listed separately
          below.
        </p>
        <div class="color-swatches">
          <div class="color-swatch">
            <div class="swatch-block" style="background: #f5e0dc;"></div>
            <div class="swatch-label">Rosewater</div>
            <div class="swatch-value">#f5e0dc</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #f2cdcd;"></div>
            <div class="swatch-label">Flamingo</div>
            <div class="swatch-value">#f2cdcd</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #f5c2e7;"></div>
            <div class="swatch-label">Pink</div>
            <div class="swatch-value">#f5c2e7</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #cba6f7;"></div>
            <div class="swatch-label">Mauve</div>
            <div class="swatch-value">#cba6f7</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #f38ba8;"></div>
            <div class="swatch-label">Red</div>
            <div class="swatch-value">#f38ba8</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #eba0ac;"></div>
            <div class="swatch-label">Maroon</div>
            <div class="swatch-value">#eba0ac</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #fab387;"></div>
            <div class="swatch-label">Peach</div>
            <div class="swatch-value">#fab387</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #f9e2af;"></div>
            <div class="swatch-label">Yellow</div>
            <div class="swatch-value">#f9e2af</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #a6e3a1;"></div>
            <div class="swatch-label">Green</div>
            <div class="swatch-value">#a6e3a1</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #94e2d5;"></div>
            <div class="swatch-label">Teal</div>
            <div class="swatch-value">#94e2d5</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #89dceb;"></div>
            <div class="swatch-label">Sky</div>
            <div class="swatch-value">#89dceb</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #74c7ec;"></div>
            <div class="swatch-label">Sapphire</div>
            <div class="swatch-value">#74c7ec</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #89b4fa;"></div>
            <div class="swatch-label">Blue</div>
            <div class="swatch-value">#89b4fa</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #b4befe;"></div>
            <div class="swatch-label">Lavender</div>
            <div class="swatch-value">#b4befe</div>
          </div>
        </div>
      </div>

      <!-- Catppuccin Mocha — surface / text shades -->
      <div class="color-group">
        <h3 class="color-group-title">Catppuccin Mocha — Surface &amp; Text</h3>
        <p class="color-group-note">
          The Mocha dark surfaces and text ramp. Note how the Base/Mantle/Crust sit in the same
          territory as the existing emittiv dark palette — these could align or be used as
          alternatives.
        </p>
        <div class="color-swatches">
          <div class="color-swatch">
            <div class="swatch-block" style="background: #cdd6f4;"></div>
            <div class="swatch-label">Text</div>
            <div class="swatch-value">#cdd6f4</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #bac2de;"></div>
            <div class="swatch-label">Subtext 1</div>
            <div class="swatch-value">#bac2de</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #a6adc8;"></div>
            <div class="swatch-label">Subtext 0</div>
            <div class="swatch-value">#a6adc8</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #9399b2;"></div>
            <div class="swatch-label">Overlay 2</div>
            <div class="swatch-value">#9399b2</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #7f849c;"></div>
            <div class="swatch-label">Overlay 1</div>
            <div class="swatch-value">#7f849c</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #6c7086;"></div>
            <div class="swatch-label">Overlay 0</div>
            <div class="swatch-value">#6c7086</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #585b70;"></div>
            <div class="swatch-label">Surface 2</div>
            <div class="swatch-value">#585b70</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #45475a;"></div>
            <div class="swatch-label">Surface 1</div>
            <div class="swatch-value">#45475a</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #313244;"></div>
            <div class="swatch-label">Surface 0</div>
            <div class="swatch-value">#313244</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #1e1e2e;"></div>
            <div class="swatch-label">Base</div>
            <div class="swatch-value">#1e1e2e</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #181825;"></div>
            <div class="swatch-label">Mantle</div>
            <div class="swatch-value">#181825</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #11111b;"></div>
            <div class="swatch-label">Crust</div>
            <div class="swatch-value">#11111b</div>
          </div>
        </div>
      </div>

      <!-- Proposed semantic mappings -->
      <div class="color-group">
        <h3 class="color-group-title">
          Proposed Semantic Mappings <span class="color-group-badge">Pending approval</span>
        </h3>
        <p class="color-group-note">
          Suggested Mocha colours for each semantic role in the app. Keeping <code class="dev-code"
            >--emittiv-splash</code
          > (#ff9900) as the primary brand accent.
        </p>
        <div class="color-swatches">
          <div class="color-swatch">
            <div class="swatch-block" style="background: #a6e3a1;"></div>
            <div class="swatch-label">Success / Connected</div>
            <div class="swatch-value">Green #a6e3a1</div>
            <div class="swatch-usage">DB connected, success states</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #f9e2af;"></div>
            <div class="swatch-label">Warning</div>
            <div class="swatch-value">Yellow #f9e2af</div>
            <div class="swatch-usage">Warning states</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #f38ba8;"></div>
            <div class="swatch-label">Danger / Error</div>
            <div class="swatch-value">Red #f38ba8</div>
            <div class="swatch-usage">Delete, error states</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #89b4fa;"></div>
            <div class="swatch-label">Stat: Projects</div>
            <div class="swatch-value">Blue #89b4fa</div>
            <div class="swatch-usage">Dashboard stat icon</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #94e2d5;"></div>
            <div class="swatch-label">Stat: Companies</div>
            <div class="swatch-value">Teal #94e2d5</div>
            <div class="swatch-usage">Dashboard stat icon</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #cba6f7;"></div>
            <div class="swatch-label">Stat: Contacts</div>
            <div class="swatch-value">Mauve #cba6f7</div>
            <div class="swatch-usage">Dashboard stat icon</div>
          </div>
        </div>
      </div>

      <!-- Status badge colours -->
      <div class="color-group">
        <h3 class="color-group-title">
          Proposed Status Colours <span class="color-group-badge">Pending approval</span>
        </h3>
        <p class="color-group-note">Project and proposal status badges mapped to Mocha accents.</p>
        <div class="color-swatches">
          <div class="color-swatch">
            <div class="swatch-block" style="background: #89b4fa;"></div>
            <div class="swatch-label">Lead / Draft</div>
            <div class="swatch-value">Blue #89b4fa</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #fab387;"></div>
            <div class="swatch-label">RFP / Sent</div>
            <div class="swatch-value">Peach #fab387</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #cba6f7;"></div>
            <div class="swatch-label">Negotiation</div>
            <div class="swatch-value">Mauve #cba6f7</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #a6e3a1;"></div>
            <div class="swatch-label">Awarded / Accepted</div>
            <div class="swatch-value">Green #a6e3a1</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #f38ba8;"></div>
            <div class="swatch-label">Lost / Rejected</div>
            <div class="swatch-value">Red #f38ba8</div>
          </div>
          <div class="color-swatch">
            <div class="swatch-block" style="background: #6c7086;"></div>
            <div class="swatch-label">No Response</div>
            <div class="swatch-value">Overlay 0 #6c7086</div>
          </div>
        </div>
      </div>
    </div>
  </section>

  <!-- Instructions Panel -->
  <section class="dev-section">
    <div class="info-panel">
      <h3 class="info-title">Testing Guidelines</h3>
      <ul class="info-list">
        <li>Pricing module uses test data (AED 250,000 target fee) — no database connection</li>
        <li>Use Reset button to restore default pricing state</li>
        <li>Click a CRUD card above to open the corresponding modal (connects to live DB)</li>
        <li>Use "DELETE ME" prefix for test data names</li>
      </ul>
    </div>
  </section>
</div>

<CompanyModal isOpen={showCompanyForm} onclose={handleFormClose} />
<ContactModal isOpen={showContactForm} onclose={handleFormClose} />
<NewProjectModal isOpen={showProjectForm} onclose={handleFormClose} />

<style>
  .dev-container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 24px;
    height: calc(100vh - 80px);
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--emittiv-dark) transparent;
  }

  .dev-section {
    margin-bottom: 32px;
  }

  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--emittiv-lighter);
    margin-bottom: 16px;
  }

  /* Pricing section */
  .pricing-section {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 8px;
    padding: 16px;
  }

  .pricing-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .pricing-header .section-title {
    margin-bottom: 0;
  }

  .reset-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    font-size: 12px;
    color: var(--emittiv-light);
    background: transparent;
    border: 1px solid var(--emittiv-dark);
    border-radius: 4px;
    cursor: pointer;
    transition: all 200ms ease;
  }

  .reset-btn:hover {
    color: var(--emittiv-splash);
    border-color: var(--emittiv-splash);
  }

  .pricing-content {
    min-height: 300px;
  }

  /* Form cards */
  .form-cards-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 20px;
  }

  .form-card {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 12px;
    padding: 20px;
    transition: all 300ms cubic-bezier(0.4, 0, 0.2, 1);
    cursor: pointer;
    text-align: left;
    min-height: 88px;
  }

  .form-card:hover {
    border-color: var(--emittiv-light);
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(255, 153, 0, 0.1);
  }

  .form-card:focus {
    outline: none;
    border-color: var(--emittiv-splash);
    box-shadow: 0 0 0 2px rgba(255, 153, 0, 0.2);
  }

  .form-card.active {
    border-color: var(--emittiv-splash);
    background: rgba(255, 153, 0, 0.05);
  }

  .form-card-content {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .form-icon {
    width: 48px;
    height: 48px;
    padding: 12px;
    background: rgba(255, 153, 0, 0.1);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .form-icon svg {
    width: 24px;
    height: 24px;
    color: var(--emittiv-splash);
  }

  .form-info {
    flex: 1;
  }

  .form-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--emittiv-white);
    line-height: 1.2;
    margin-bottom: 4px;
  }

  .form-desc {
    font-size: 14px;
    color: var(--emittiv-lighter);
    line-height: 1.2;
  }

  .info-panel {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 12px;
    padding: 20px;
  }

  .info-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--emittiv-white);
    margin-bottom: 12px;
  }

  .info-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .info-list li {
    font-size: 14px;
    color: var(--emittiv-light);
    padding: 4px 0;
    padding-left: 16px;
    position: relative;
  }

  .info-list li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 4px;
    height: 4px;
    background: var(--emittiv-splash);
    border-radius: 50%;
  }

  /* Dev tool cards (scope builder, etc.) */
  .dev-tool-card {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 12px;
    overflow: hidden;
  }

  .dev-tool-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--emittiv-dark);
  }

  .dev-tool-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--emittiv-white);
    margin-bottom: 4px;
  }

  .dev-tool-desc {
    font-size: 13px;
    color: var(--emittiv-light);
  }

  .dev-tool-body {
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .dev-tool-row {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .dev-scope-input {
    flex: 1;
    max-width: 320px;
  }

  .dev-tool-note {
    font-size: 12px;
    color: var(--emittiv-dark);
    margin: 0;
  }

  .dev-code {
    font-family: monospace;
    font-size: 12px;
    background: rgba(255, 255, 255, 0.08);
    padding: 1px 5px;
    border-radius: 3px;
    color: var(--emittiv-lighter);
  }

  /* Color test page */
  .color-test-grid {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .color-group {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 12px;
    padding: 20px;
  }

  .color-group-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--emittiv-white);
    margin-bottom: 8px;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .color-group-badge {
    font-size: 11px;
    font-weight: 500;
    color: var(--emittiv-splash);
    background: rgba(255, 153, 0, 0.12);
    border: 1px solid rgba(255, 153, 0, 0.3);
    border-radius: 10px;
    padding: 2px 8px;
  }

  .color-group-note {
    font-size: 13px;
    color: var(--emittiv-light);
    margin: 0 0 16px;
    line-height: 1.5;
  }

  .color-swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
  }

  .color-swatch {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 100px;
  }

  .swatch-block {
    width: 100px;
    height: 60px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .swatch-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--emittiv-lighter);
    line-height: 1.2;
  }

  .swatch-value {
    font-size: 11px;
    color: var(--emittiv-light);
    font-family: monospace;
  }

  .swatch-usage {
    font-size: 11px;
    color: var(--emittiv-dark);
    font-style: italic;
  }

  @media (max-width: 768px) {
    .dev-container {
      padding: 16px;
    }

    .form-cards-grid {
      grid-template-columns: 1fr;
      gap: 16px;
    }

    .dev-tool-row {
      flex-direction: column;
      align-items: stretch;
    }

    .dev-scope-input {
      max-width: none;
    }
  }
</style>
