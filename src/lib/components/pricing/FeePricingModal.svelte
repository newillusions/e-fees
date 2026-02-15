<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Fee } from '../../../types';
  import type { PricingBreakdown, PricingConfig, Discipline, Stage, PricingCell, PostContractItem, ReimbursableCost, PaymentSchedule } from '../../../types/database';
  import { extractSurrealId } from '$lib/utils/surrealdb';
  import {
    createDefaultDisciplines,
    createDefaultDesignStages,
    createDefaultPostContractStages,
    calculatePricingTotals,
    DEFAULT_PRICING_CONFIG
  } from '../../../types/database';
  import { updateFeePricing, type PricingUpdate } from '$lib/api/fees';
  import BaseModal from '../BaseModal.svelte';
  import Button from '../Button.svelte';
  import PricingCalculatorPanel from './PricingCalculatorPanel.svelte';
  import DisciplinesPanel from './DisciplinesPanel.svelte';
  import StagesPanel from './StagesPanel.svelte';
  import CostsPanel from './CostsPanel.svelte';
  import PaymentSchedulePanel from './PaymentSchedulePanel.svelte';
  import PricingSummaryBar from './PricingSummaryBar.svelte';

  const dispatch = createEventDispatcher();

  let { isOpen = $bindable(false), fee = null as Fee | null }: {
    isOpen?: boolean;
    fee?: Fee | null;
  } = $props();

  // UI state
  let activeTab: 'disciplines' | 'stages' | 'costs' | 'calculator' | 'payments' = 'disciplines';
  let saving = false;
  let error = '';
  let message = '';

  // Pricing state - initialized from fee or defaults
  // IMPORTANT: Initialize with empty defaults to prevent undefined errors
  // before initializePricingData() runs
  let config: PricingConfig | null = null;
  let disciplines: Discipline[] = [];
  let stages: Stage[] = [];
  let cells: PricingCell[] = [];
  let postContractItems: PostContractItem[] = [];
  let reimbursableCosts: ReimbursableCost[] = [];
  let paymentSchedule: PaymentSchedule = {
    entries: [],
    total_invoiced: 0,
    total_paid: 0,
    total_outstanding: 0
  };

  // Initialize pricing data when modal opens
  $effect(() => {
    if (isOpen && fee) {
      initializePricingData();
    }
  });

  function initializePricingData() {
    if (fee?.pricing) {
      // Load existing pricing data
      config = { ...fee.pricing.config };
      disciplines = [...fee.pricing.disciplines];
      stages = [...fee.pricing.stages];
      cells = [...fee.pricing.cells];
      reimbursableCosts = fee.pricing.costs ? [...fee.pricing.costs] : [];
    } else {
      // Initialize with defaults
      const defaultDisciplines = createDefaultDisciplines();
      const defaultDesignStages = createDefaultDesignStages();
      const defaultPostContractStages = createDefaultPostContractStages();

      config = {
        ...DEFAULT_PRICING_CONFIG,
        target_fee: 0,
        quoted_fee: 0
      };
      disciplines = defaultDisciplines;
      stages = [...defaultDesignStages, ...defaultPostContractStages];
      cells = [];
      reimbursableCosts = [];
    }

    // Post-contract items
    postContractItems = fee?.post_contract_items ? [...fee.post_contract_items] : [];

    // Payment schedule
    paymentSchedule = fee?.payment_schedule ? { ...fee.payment_schedule } : {
      entries: [],
      total_invoiced: 0,
      total_paid: 0,
      total_outstanding: 0
    };

    // Clear any previous messages
    error = '';
    message = '';
  }

  // Calculate totals reactively
  const totals = $derived(calculatePricingTotals(
    cells,
    postContractItems,
    reimbursableCosts,
    config ?? DEFAULT_PRICING_CONFIG as any,
    stages,
  ));

  // Build pricing breakdown for summary bar
  const pricingBreakdown = $derived(config ? {
    config,
    disciplines,
    stages,
    cells,
    costs: reimbursableCosts,
    ...totals
  } as PricingBreakdown : null);

  // Build pricing breakdown for saving
  function buildPricingBreakdown(): PricingBreakdown {
    if (!config) {
      throw new Error('Cannot build pricing breakdown: config is not initialized');
    }
    return {
      config,
      disciplines,
      stages,
      cells,
      costs: reimbursableCosts,
      ...totals
    };
  }

  // Update handlers for child components
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

  // Save pricing data
  async function handleSave() {
    if (!fee?.id) {
      error = 'Cannot save: No fee ID';
      return;
    }

    saving = true;
    error = '';
    message = '';

    try {
      // Extract the ID (handles both string and SurrealThing types)
      const feeId = extractSurrealId(fee.id);
      if (!feeId) {
        error = 'Cannot save: Invalid fee ID format';
        return;
      }

      const pricingUpdate: PricingUpdate = {
        pricing: buildPricingBreakdown(),
        post_contract_items: postContractItems,
        reimbursable_costs: reimbursableCosts,
        payment_schedule: paymentSchedule
      };

      await updateFeePricing(feeId, pricingUpdate);

      message = 'Pricing saved successfully';

      // Close modal after a brief delay to show success message
      setTimeout(() => {
        dispatch('save');
        closeModal();
      }, 1000);
    } catch (err) {
      console.error('Failed to save pricing:', err);
      error = `Failed to save pricing: ${err}`;
    } finally {
      saving = false;
    }
  }

  function closeModal() {
    isOpen = false;
    dispatch('close');
  }

  // Tab configuration - Order: Disciplines → Stages → Costs → Calculator → Payments
  const tabs = [
    { id: 'disciplines', label: 'Disciplines', icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10' },
    { id: 'stages', label: 'Stages', icon: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01' },
    { id: 'costs', label: 'Costs', icon: 'M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z' },
    { id: 'calculator', label: 'Calculator', icon: 'M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z' },
    { id: 'payments', label: 'Payments', icon: 'M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z' }
  ] as const;
</script>

<BaseModal
  {isOpen}
  title="Fee Pricing: {fee?.number || 'Unknown'}"
  maxWidth="1100px"
  on:close={closeModal}
>
  <div class="flex flex-col h-full">
    <!-- Summary Bar (always visible) -->
    {#if config}
      <div class="mb-2">
        <PricingSummaryBar pricing={pricingBreakdown} />
      </div>
    {/if}

    <!-- Tab Navigation -->
    <div class="flex gap-0.5 mb-2 border-b border-emittiv-dark pb-1">
      {#each tabs as tab}
        <button
          type="button"
          class="emittiv-tab"
          class:emittiv-tab--active={activeTab === tab.id}
          onclick={() => activeTab = tab.id}
        >
          <svg class="emittiv-tab__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={tab.icon} />
          </svg>
          {tab.label}
        </button>
      {/each}
    </div>

    <!-- Tab Content -->
    <div class="flex-1 overflow-y-auto pr-1">
      {#if activeTab === 'calculator' && config}
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
      {:else if activeTab === 'disciplines'}
        <DisciplinesPanel
          bind:disciplines
          onUpdate={handleDisciplinesUpdate}
        />
      {:else if activeTab === 'stages'}
        <StagesPanel
          bind:stages
          onUpdateStages={handleStagesUpdate}
        />
      {:else if activeTab === 'costs'}
        <CostsPanel
          bind:costs={reimbursableCosts}
          {stages}
          onUpdate={handleCostsUpdate}
        />
      {:else if activeTab === 'payments' && config}
        <PaymentSchedulePanel
          bind:schedule={paymentSchedule}
          {stages}
          {cells}
          {config}
          designTotal={totals.design_phase_total}
          onUpdate={handlePaymentScheduleUpdate}
        />
      {/if}
    </div>

    <!-- Error/Success Messages -->
    {#if error}
      <div class="mt-2 text-red-400 text-xs bg-red-900/20 border border-red-500/30 rounded px-2 py-1.5">
        {error}
      </div>
    {/if}

    {#if message}
      <div class="mt-2 text-green-400 text-xs bg-green-900/20 border border-green-500/30 rounded px-2 py-1.5">
        {message}
      </div>
    {/if}

    <!-- Actions -->
    <div class="flex justify-end gap-2 mt-3 pt-3 border-t border-emittiv-dark">
      <Button
        variant="secondary"
        size="md"
        on:click={closeModal}
        disabled={saving}
      >
        Cancel
      </Button>
      <Button
        variant="primary"
        size="md"
        on:click={handleSave}
        disabled={saving}
      >
        {#if saving}
          <div class="emittiv-spinner-sm"></div>
        {/if}
        Save Pricing
      </Button>
    </div>
  </div>
</BaseModal>
