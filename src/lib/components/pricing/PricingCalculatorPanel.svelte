<script lang="ts">
  import type { PricingConfig, Discipline, Stage, PricingCell, PostContractItem } from '../../../types/database';
  import {
    calculateQuotedFee,
    calculateCellAmount,
    convertToClientCurrency,
    generatePricingId,
    DEFAULT_PRICING_CONFIG
  } from '../../../types/database';
  import { formatCurrency, formatNumber, formatPercent, roundToIncrement } from '$lib/utils/format';
  import { formattedNumber } from '$lib/actions/formattedNumber';
  import IconButton from '../IconButton.svelte';
  import PanelCard from '../PanelCard.svelte';
  import {
    exchangeRatesStore,
    exchangeRatesLoading,
    exchangeRatesActions,
    SUPPORTED_CURRENCIES,
    type CurrencyCode
  } from '$lib/stores/exchangeRates';
  import { getRate } from '$lib/services/exchangeRates';

  interface Props {
    config: PricingConfig;
    disciplines: Discipline[];
    stages: Stage[];
    cells: PricingCell[];
    postContractItems: PostContractItem[];
    onUpdateConfig: (config: PricingConfig) => void;
    onUpdateCells: (cells: PricingCell[]) => void;
    onUpdatePostContract: (items: PostContractItem[]) => void;
    readonly?: boolean;
  }

  let {
    config = $bindable(),
    disciplines,
    stages,
    cells = $bindable([]),
    postContractItems = $bindable([]),
    onUpdateConfig,
    onUpdateCells,
    onUpdatePostContract,
    readonly = false
  }: Props = $props();

  // Design stages only (not post-contract)
  const designStages = $derived(stages.filter(s => !s.is_post_contract).sort((a, b) => a.order - b.order));
  const postContractStages = $derived(stages.filter(s => s.is_post_contract).sort((a, b) => a.order - b.order));
  const sortedDisciplines = $derived([...disciplines].sort((a, b) => a.order - b.order));

  // Stage total overrides (manual edits to rounded stage totals)
  let stageTotalOverrides = $state<Map<string, number>>(new Map());

  // Post-contract totals
  const postContractSubtotal = $derived(postContractItems.reduce((sum, i) => sum + i.amount, 0));
  const postContractVat = $derived(postContractSubtotal * (config?.vat_percent || 0) / 100);
  const postContractTotal = $derived(postContractSubtotal + postContractVat);

  // Recalculate quoted fee when target or buffer changes
  // Note: only mutate config locally — bind:config propagates to parent.
  // Do NOT call onUpdateConfig here to avoid double-update race with cells effect.
  $effect(() => {
    if (config && config.target_fee > 0) {
      const newQuoted = calculateQuotedFee(config.target_fee, config.buffer_percent);
      if (Math.abs(newQuoted - config.quoted_fee) > 0.01) {
        config = { ...config, quoted_fee: newQuoted };
      }
    }
  });

  // Auto-generate cells when disciplines or stages change
  $effect(() => {
    if (disciplines.length > 0 && designStages.length > 0) {
      const newCells: PricingCell[] = [];
      for (const disc of disciplines) {
        for (const stage of designStages) {
          const existing = cells.find(c => c.discipline_id === disc.id && c.stage_id === stage.id);
          const amount = calculateCellAmount(config.quoted_fee, disc.percentage, stage.percentage);
          newCells.push({
            discipline_id: disc.id,
            stage_id: stage.id,
            amount,
            override_amount: existing?.override_amount,
          });
        }
      }
      // Only update if cells actually changed — compare by ID, not index position
      const cellsChanged = newCells.length !== cells.length ||
        newCells.some(c => {
          const old = cells.find(oc => oc.discipline_id === c.discipline_id && oc.stage_id === c.stage_id);
          return !old || Math.abs(c.amount - old.amount) > 0.01;
        });
      if (cellsChanged) {
        cells = newCells;
        onUpdateCells(newCells);
      }
    }
  });

  // Get cell value (with override support)
  function getCellValue(disciplineId: string, stageId: string): number {
    const cell = cells.find(c => c.discipline_id === disciplineId && c.stage_id === stageId);
    return cell?.override_amount ?? cell?.amount ?? 0;
  }

  // Check if cell has override (only if it actually differs from calculated)
  function hasOverride(disciplineId: string, stageId: string): boolean {
    const cell = cells.find(c => c.discipline_id === disciplineId && c.stage_id === stageId);
    if (cell?.override_amount === undefined) return false;
    // Only show override indicator if value actually differs from calculated
    return Math.round(cell.override_amount) !== Math.round(cell.amount);
  }

  // Set cell override (only if value differs from calculated)
  function setCellOverride(disciplineId: string, stageId: string, value: number | null) {
    const cell = cells.find(c => c.discipline_id === disciplineId && c.stage_id === stageId);
    const calculatedAmount = cell?.amount ?? 0;

    // Only set override if value actually differs from calculated
    // Use Math.round for comparison since we display rounded values
    const newOverride = (value !== null && Math.round(value) !== Math.round(calculatedAmount))
      ? value
      : undefined;

    const updated = cells.map(c => {
      if (c.discipline_id === disciplineId && c.stage_id === stageId) {
        return { ...c, override_amount: newOverride };
      }
      return c;
    });
    cells = updated;
    onUpdateCells(updated);
  }

  // Clear override
  function clearOverride(disciplineId: string, stageId: string) {
    setCellOverride(disciplineId, stageId, null);
  }

  // Calculate totals
  function getStageTotal(stageId: string): number {
    return sortedDisciplines.reduce((sum, disc) => sum + getCellValue(disc.id, stageId), 0);
  }

  // Get rounded stage total for display (client-facing)
  function getRoundedStageTotal(stageId: string): number {
    // Check for manual override first
    if (stageTotalOverrides.has(stageId)) {
      return stageTotalOverrides.get(stageId)!;
    }
    const raw = getStageTotal(stageId);
    const increment = config?.rounding_increment ?? 50;
    const mode = config?.rounding_mode ?? 'ceiling';
    return roundToIncrement(raw, increment, mode);
  }

  // Check if stage total has override (only if it actually differs from calculated)
  function hasStageTotalOverride(stageId: string): boolean {
    if (!stageTotalOverrides.has(stageId)) return false;
    const override = stageTotalOverrides.get(stageId)!;
    const raw = getStageTotal(stageId);
    const increment = config?.rounding_increment ?? 50;
    const mode = config?.rounding_mode ?? 'ceiling';
    const calculatedRounded = roundToIncrement(raw, increment, mode);
    return Math.round(override) !== Math.round(calculatedRounded);
  }

  // Set stage total override (only if value differs from calculated rounded total)
  function setStageTotalOverride(stageId: string, value: number) {
    const raw = getStageTotal(stageId);
    const increment = config?.rounding_increment ?? 50;
    const mode = config?.rounding_mode ?? 'ceiling';
    const calculatedRounded = roundToIncrement(raw, increment, mode);

    // Only set override if value actually differs from calculated rounded value
    if (Math.round(value) !== Math.round(calculatedRounded)) {
      stageTotalOverrides = new Map(stageTotalOverrides).set(stageId, value);
    } else {
      // Clear any existing override if value matches calculated
      const newMap = new Map(stageTotalOverrides);
      newMap.delete(stageId);
      stageTotalOverrides = newMap;
    }
  }

  // Clear stage total override
  function clearStageTotalOverride(stageId: string) {
    const newMap = new Map(stageTotalOverrides);
    newMap.delete(stageId);
    stageTotalOverrides = newMap;
  }

  function getDisciplineTotal(disciplineId: string): number {
    return designStages.reduce((sum, stage) => sum + getCellValue(disciplineId, stage.id), 0);
  }

  // Design subtotal uses rounded stage totals for consistency
  const designSubtotal = $derived(
    designStages.reduce((sum, stage) => sum + getRoundedStageTotal(stage.id), 0)
  );
  // VAT is added on top; withholding is NOT added to proposal totals (applied at invoice time only)
  const designVat = $derived(config?.tax_type === 'vat' ? designSubtotal * (config?.vat_percent || 0) / 100 : 0);
  const grandTotal = $derived(designSubtotal + designVat);

  // Multi-currency conversion
  const hasClientCurrency = $derived(
    !!config?.client_currency && config.client_currency !== config.currency && !!config?.exchange_rate
  );
  const convertedTotal = $derived(
    hasClientCurrency ? convertToClientCurrency(
      config.tax_type === 'vat' && config.show_tax_in_summary ? grandTotal : designSubtotal,
      config
    ) : undefined
  );

  // Live exchange rate from store
  const liveRate = $derived.by(() => {
    // Subscribe to store to trigger reactivity when rates are fetched
    const rates = $exchangeRatesStore;
    if (!rates || !config?.client_currency || config.client_currency === config.currency) return null;
    return getRate(config.currency as CurrencyCode, config.client_currency as CurrencyCode);
  });
  const liveRateDate = $derived($exchangeRatesStore?.date ?? null);
  const isRateLocked = $derived(!!config?.rate_locked_at);

  function useLiveRate() {
    if (liveRate !== null) {
      updateConfig('exchange_rate', Math.round(liveRate * 10000) / 10000);
    }
  }

  function lockRate() {
    updateConfig('rate_locked_at', new Date().toISOString());
  }

  function unlockRate() {
    updateConfig('rate_locked_at', undefined);
  }

  // Update config field
  function updateConfig(field: keyof PricingConfig, value: number | string | boolean | undefined) {
    const updated = { ...config, [field]: value };
    // Clean up undefined values
    if (value === undefined) {
      delete (updated as Record<string, unknown>)[field];
    }
    config = updated;
    onUpdateConfig(config);
  }

  // Auto-sync post-contract items to stages (one item per stage)
  $effect(() => {
    if (postContractStages.length === 0) {
      if (postContractItems.length > 0) {
        postContractItems = [];
        onUpdatePostContract([]);
      }
      return;
    }
    const stageIds = new Set(postContractStages.map(s => s.id));
    const existingStageIds = new Set(postContractItems.map(i => i.stage_id));

    // Check if sync needed
    const needsAdd = postContractStages.some(s => !existingStageIds.has(s.id));
    const needsRemove = postContractItems.some(i => !stageIds.has(i.stage_id));
    if (!needsAdd && !needsRemove) return;

    // Keep existing items for stages that still exist, add new ones
    const kept = postContractItems.filter(i => stageIds.has(i.stage_id));
    const newItems = postContractStages
      .filter(s => !existingStageIds.has(s.id))
      .map(s => ({
        id: generatePricingId('pcitem'),
        stage_id: s.id,
        description: s.name,
        quantity: 1,
        unit: 'visit',
        rate: 0,
        amount: 0,
      }));
    const updated = [...kept, ...newItems];
    postContractItems = updated;
    onUpdatePostContract(updated);
  });

  function updatePostContractItem(id: string, field: keyof PostContractItem, value: string | number) {
    const updated = postContractItems.map(item => {
      if (item.id !== id) return item;
      const newItem = { ...item, [field]: value };
      if (field === 'quantity' || field === 'rate') {
        newItem.amount = newItem.quantity * newItem.rate;
      }
      return newItem;
    });
    postContractItems = updated;
    onUpdatePostContract(updated);
  }

  // Get item for a stage
  function getItemForStage(stageId: string): PostContractItem | undefined {
    return postContractItems.find(i => i.stage_id === stageId);
  }
</script>

<div class="space-y-3">
  <!-- Fee Configuration -->
  <PanelCard title="Fee Calculator">
    <div class="emittiv-calc-row">
      <div class="emittiv-calc-field emittiv-calc-field--fee">
        <label class="emittiv-label">Target Fee</label>
        {#if !readonly}
          <input
            type="text"
            inputmode="numeric"
            class="emittiv-table-input emittiv-table-input--left"
            use:formattedNumber={{ value: config.target_fee, onChange: (v) => updateConfig('target_fee', v), min: 0 }}
          />
        {:else}
          <span class="text-emittiv-white">{formatNumber(config.target_fee)}</span>
        {/if}
      </div>
      <div class="emittiv-calc-field emittiv-calc-field--currency">
        <label class="emittiv-label">Currency</label>
        {#if !readonly}
          <select
            class="emittiv-select"
            value={config.currency}
            onchange={(e) => updateConfig('currency', e.currentTarget.value)}
          >
            <option value="AED">AED</option>
            <option value="USD">USD</option>
            <option value="EUR">EUR</option>
            <option value="GBP">GBP</option>
            <option value="SAR">SAR</option>
          </select>
        {:else}
          <span class="text-emittiv-white">{config.currency}</span>
        {/if}
      </div>
      <div class="emittiv-calc-field emittiv-calc-field--pct">
        <label class="emittiv-label">Buffer</label>
        {#if !readonly}
          <div class="emittiv-field-suffix">
            <input
              type="number"
              min="0"
              max="50"
              step="1"
              class="emittiv-table-input emittiv-table-input--lg"
              value={config.buffer_percent}
              onchange={(e) => updateConfig('buffer_percent', parseFloat(e.currentTarget.value) || 0)}
            />
            <span class="emittiv-field-suffix__unit">%</span>
          </div>
        {:else}
          <span class="text-emittiv-white">{config.buffer_percent}%</span>
        {/if}
      </div>
      <div class="emittiv-calc-field emittiv-calc-field--pct">
        <label class="emittiv-label">VAT</label>
        {#if !readonly}
          <div class="emittiv-field-suffix">
            <input
              type="number"
              min="0"
              max="30"
              step="0.5"
              class="emittiv-table-input emittiv-table-input--lg"
              value={config.vat_percent}
              onchange={(e) => updateConfig('vat_percent', parseFloat(e.currentTarget.value) || 0)}
            />
            <span class="emittiv-field-suffix__unit">%</span>
          </div>
        {:else}
          <span class="text-emittiv-white">{config.vat_percent}%</span>
        {/if}
      </div>
      <div class="emittiv-calc-field emittiv-calc-field--pct">
        <label class="emittiv-label">Mobilisation</label>
        {#if !readonly}
          <div class="emittiv-field-suffix">
            <input
              type="number"
              min="0"
              max="100"
              step="5"
              class="emittiv-table-input emittiv-table-input--lg"
              value={config.mobilisation_percent}
              onchange={(e) => updateConfig('mobilisation_percent', parseFloat(e.currentTarget.value) || 0)}
            />
            <span class="emittiv-field-suffix__unit">%</span>
          </div>
        {:else}
          <span class="text-emittiv-white">{config.mobilisation_percent}%</span>
        {/if}
      </div>
      <div class="emittiv-calc-field emittiv-calc-field--quoted">
        <label class="emittiv-label">Calculated Fee</label>
        <span class="emittiv-calc-quoted">{formatCurrency(config.quoted_fee, config.currency)}</span>
      </div>
    </div>

    <!-- Rounding Configuration Row -->
    <div class="emittiv-calc-row mt-2">
      <div class="emittiv-calc-field emittiv-calc-field--currency">
        <label class="emittiv-label">Rounding</label>
        {#if !readonly}
          <select
            class="emittiv-select"
            value={config.rounding_increment ?? 50}
            onchange={(e) => updateConfig('rounding_increment', parseInt(e.currentTarget.value))}
          >
            <option value={50}>50</option>
            <option value={100}>100</option>
            <option value={250}>250</option>
            <option value={500}>500</option>
            <option value={1000}>1,000</option>
          </select>
        {:else}
          <span class="text-emittiv-white">{config.rounding_increment ?? 50}</span>
        {/if}
      </div>
      <div class="emittiv-calc-field emittiv-calc-field--currency">
        <label class="emittiv-label">Mode</label>
        {#if !readonly}
          <select
            class="emittiv-select"
            value={config.rounding_mode ?? 'ceiling'}
            onchange={(e) => updateConfig('rounding_mode', e.currentTarget.value)}
          >
            <option value="ceiling">Ceiling (up)</option>
            <option value="nearest">Nearest</option>
          </select>
        {:else}
          <span class="text-emittiv-white">{config.rounding_mode === 'nearest' ? 'Nearest' : 'Ceiling'}</span>
        {/if}
      </div>
      <div class="emittiv-calc-field emittiv-calc-field--currency">
        <label class="emittiv-label">Tax Type</label>
        {#if !readonly}
          <select
            class="emittiv-select"
            value={config.tax_type ?? 'vat'}
            onchange={(e) => updateConfig('tax_type', e.currentTarget.value)}
          >
            <option value="vat">VAT</option>
            <option value="withholding">Withholding</option>
            <option value="none">None</option>
          </select>
        {:else}
          <span class="text-emittiv-white">{config.tax_type === 'none' ? 'None' : config.tax_type === 'withholding' ? 'Withholding' : 'VAT'}</span>
        {/if}
      </div>
      <div class="emittiv-calc-field emittiv-calc-field--pct">
        <label class="emittiv-label">Show Tax</label>
        {#if !readonly}
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              class="emittiv-checkbox"
              checked={config.show_tax_in_summary ?? false}
              onchange={(e) => updateConfig('show_tax_in_summary', e.currentTarget.checked)}
            />
            <span class="text-emittiv-light text-xs">In summary</span>
          </label>
        {:else}
          <span class="text-emittiv-white">{config.show_tax_in_summary ? 'Yes' : 'No'}</span>
        {/if}
      </div>
      <div class="emittiv-calc-field emittiv-calc-field--quoted">
        <!-- Spacer to align with row above -->
      </div>
    </div>

    <!-- Exchange Rate Row -->
    <div class="emittiv-calc-row mt-2">
      <div class="emittiv-calc-field emittiv-calc-field--currency">
        <label class="emittiv-label">Client Currency</label>
        {#if !readonly}
          <select
            class="emittiv-select"
            value={config.client_currency ?? ''}
            onchange={(e) => {
              const val = e.currentTarget.value;
              updateConfig('client_currency', val || undefined);
              if (!val || val === config.currency) {
                updateConfig('exchange_rate', undefined);
                updateConfig('rate_locked_at', undefined);
              }
            }}
          >
            <option value="">Same as base</option>
            {#each SUPPORTED_CURRENCIES as cur}
              {#if cur !== config.currency}
                <option value={cur}>{cur}</option>
              {/if}
            {/each}
          </select>
        {:else}
          <span class="text-emittiv-white">{config.client_currency || 'Same as base'}</span>
        {/if}
      </div>
      {#if config.client_currency && config.client_currency !== config.currency}
        <div class="emittiv-calc-field emittiv-calc-field--currency">
          <label class="emittiv-label">Quote in</label>
          {#if !readonly}
            <select
              class="emittiv-select"
              value={config.quote_currency ?? config.currency}
              onchange={(e) => updateConfig('quote_currency', e.currentTarget.value)}
            >
              <option value={config.currency}>{config.currency} (base)</option>
              <option value={config.client_currency}>{config.client_currency} (client)</option>
            </select>
          {:else}
            <span class="text-emittiv-white">{config.quote_currency ?? config.currency}</span>
          {/if}
        </div>
        <div class="emittiv-calc-field emittiv-calc-field--fee">
          <label class="emittiv-label">Exchange Rate (1 {config.currency} =)</label>
          {#if !readonly}
            <div class="emittiv-exchange-rate-input">
              <input
                type="number"
                min="0"
                step="0.0001"
                class="emittiv-input"
                value={config.exchange_rate ?? ''}
                disabled={isRateLocked}
                onchange={(e) => {
                  const val = parseFloat(e.currentTarget.value);
                  updateConfig('exchange_rate', isNaN(val) || val <= 0 ? undefined : val);
                }}
                placeholder="e.g. 0.9799"
              />
              {#if liveRate !== null && !isRateLocked}
                <button
                  type="button"
                  class="emittiv-btn-inline"
                  onclick={useLiveRate}
                  title="Use live rate: {liveRate.toFixed(4)}"
                >
                  Use Live
                </button>
              {/if}
            </div>
          {:else}
            <span class="text-emittiv-white">{config.exchange_rate ?? 'Not set'}</span>
          {/if}
        </div>

        <!-- Live Rate Info -->
        <div class="emittiv-calc-field emittiv-calc-field--fee">
          {#if $exchangeRatesLoading}
            <label class="emittiv-label">Live Rate</label>
            <span class="text-emittiv-dark text-xs">Fetching...</span>
          {:else if liveRate !== null && liveRateDate}
            <label class="emittiv-label">Live Rate</label>
            <span class="text-emittiv-light text-xs">
              1 {config.currency} = {liveRate.toFixed(4)} {config.client_currency}
              <span class="text-emittiv-dark">(ECB {liveRateDate})</span>
            </span>
          {:else}
            <label class="emittiv-label">Live Rate</label>
            <span class="text-emittiv-dark text-xs">Unavailable</span>
          {/if}
        </div>

        <!-- Rate Lock & Converted Total -->
        <div class="emittiv-calc-field emittiv-calc-field--quoted">
          {#if isRateLocked}
            <label class="emittiv-label">Rate Locked</label>
            <div class="emittiv-rate-lock">
              <span class="text-emittiv-light text-xs">
                Locked {new Date(config.rate_locked_at!).toLocaleDateString('en-GB', { day: '2-digit', month: 'short', year: 'numeric' })}
              </span>
              {#if !readonly}
                <button
                  type="button"
                  class="emittiv-btn-inline emittiv-btn-inline--warn"
                  onclick={unlockRate}
                >
                  Unlock
                </button>
              {/if}
            </div>
          {:else if config.exchange_rate && !readonly}
            <label class="emittiv-label">&nbsp;</label>
            <button
              type="button"
              class="emittiv-btn-inline"
              onclick={lockRate}
              title="Lock this rate so it won't change when you save"
            >
              Lock Rate
            </button>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Converted Total Row (shown when exchange rate is active) -->
    {#if convertedTotal !== undefined && config.client_currency}
      <div class="emittiv-calc-row mt-1">
        <div class="emittiv-calc-field emittiv-calc-field--quoted" style="margin-left: auto;">
          <label class="emittiv-label">Converted Total</label>
          <span class="emittiv-calc-quoted">{formatCurrency(convertedTotal, config.client_currency)}</span>
        </div>
      </div>
    {/if}
  </PanelCard>

  <!-- Design Phase Pricing Matrix -->
  {#if designStages.length > 0 && sortedDisciplines.length > 0}
    <PanelCard title="Design Phase Breakdown">
      <!-- Header row -->
      <div class="emittiv-sortable-header emittiv-sortable-header--compact">
        <div class="emittiv-sortable-col--grow">Stage</div>
        {#each sortedDisciplines as disc}
          <div class="emittiv-sortable-col--number">
            {disc.name} <span class="text-emittiv-lighter">{formatPercent(disc.percentage)}</span>
          </div>
        {/each}
        <div class="emittiv-sortable-col--accent">Total</div>
      </div>

      <!-- Data rows -->
      {#each designStages as stage}
        <div class="emittiv-sortable-row emittiv-sortable-row--static emittiv-sortable-row--compact">
          <div class="emittiv-sortable-col--grow flex items-center justify-between text-emittiv-white">
            <span>{stage.name}</span>
            <span class="text-emittiv-light text-xs flex">
              <span class="w-8 text-center">[{stage.code}]</span>
              <span class="w-10 text-right">[{formatPercent(stage.percentage)}]</span>&nbsp;&nbsp;
            </span>
          </div>
          {#each sortedDisciplines as disc}
            {@const cellValue = getCellValue(disc.id, stage.id)}
            {@const isOverridden = hasOverride(disc.id, stage.id)}
            <div class="emittiv-sortable-col--number">
              {#if !readonly}
                <div class="emittiv-sortable-cell-group">
                  {#if isOverridden}
                    <button
                      type="button"
                      class="emittiv-override-reset"
                      onclick={() => clearOverride(disc.id, stage.id)}
                      title="Reset to calculated value"
                    >
                      <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                      </svg>
                    </button>
                  {/if}
                  <input
                    type="text"
                    inputmode="numeric"
                    class="emittiv-table-input emittiv-table-input--md"
                    class:emittiv-text--override={isOverridden}
                    class:emittiv-table-input--has-override={isOverridden}
                    use:formattedNumber={{ value: Math.round(cellValue), onChange: (v) => setCellOverride(disc.id, stage.id, v), min: 0 }}
                  />
                </div>
              {:else}
                <span class:emittiv-text--override={isOverridden}>
                  {formatNumber(cellValue)}
                </span>
              {/if}
            </div>
          {/each}
          <div class="emittiv-sortable-col--accent" title="Calculated: {formatNumber(getStageTotal(stage.id))} → Rounded: {formatNumber(roundToIncrement(getStageTotal(stage.id), config?.rounding_increment ?? 50, config?.rounding_mode ?? 'ceiling'))}">
            {#if !readonly}
              {@const isOverridden = hasStageTotalOverride(stage.id)}
              <div class="emittiv-sortable-cell-group">
                {#if isOverridden}
                  <button
                    type="button"
                    class="emittiv-override-reset"
                    onclick={() => clearStageTotalOverride(stage.id)}
                    title="Reset to calculated value"
                  >
                    <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                  </button>
                {/if}
                <input
                  type="text"
                  inputmode="numeric"
                  class="emittiv-table-input emittiv-table-input--md emittiv-table-input--accent"
                  class:emittiv-text--override={isOverridden}
                  class:emittiv-table-input--has-override={isOverridden}
                  use:formattedNumber={{ value: getRoundedStageTotal(stage.id), onChange: (v) => setStageTotalOverride(stage.id, v), min: 0 }}
                />
              </div>
            {:else}
              {formatNumber(getRoundedStageTotal(stage.id))}
            {/if}
          </div>
        </div>
      {/each}

      <!-- Subtotal row -->
      <div class="emittiv-sortable-footer emittiv-sortable-footer--compact">
        <div class="emittiv-sortable-col--grow">SUBTOTAL</div>
        {#each sortedDisciplines as disc}
          <div class="emittiv-sortable-col--number">
            {formatNumber(getDisciplineTotal(disc.id))}
          </div>
        {/each}
        <div class="emittiv-sortable-col--accent">
          {formatNumber(designSubtotal)}
        </div>
      </div>
      <!-- Tax row (only shown for VAT when show_tax_in_summary is true) -->
      {#if config.tax_type === 'vat' && config.show_tax_in_summary}
        <div class="emittiv-sortable-footer emittiv-sortable-footer--compact">
          <div class="emittiv-sortable-col--grow">VAT ({config.vat_percent}%)</div>
          {#each sortedDisciplines as disc}
            <div class="emittiv-sortable-col--number"></div>
          {/each}
          <div class="emittiv-sortable-col--accent">
            {formatNumber(designVat)}
          </div>
        </div>
      {/if}
      <!-- Total row -->
      <div class="emittiv-sortable-footer emittiv-sortable-footer--compact emittiv-sortable-footer--total">
        <div class="emittiv-sortable-col--grow">{config.tax_type === 'vat' && config.show_tax_in_summary ? 'TOTAL (incl. VAT)' : 'TOTAL'}</div>
        {#each sortedDisciplines as disc}
          <div class="emittiv-sortable-col--number"></div>
        {/each}
        <div class="emittiv-sortable-col--accent">
          {formatNumber(config.tax_type === 'vat' && config.show_tax_in_summary ? grandTotal : designSubtotal)}
        </div>
      </div>

      <!-- Matrix validation - only warn if difference exceeds rounding threshold -->
      {#if Math.abs(designSubtotal - config.quoted_fee) > (config.rounding_increment ?? 50) * designStages.length}
        <div class="emittiv-matrix-warning">
          Note: Matrix subtotal ({formatNumber(designSubtotal)}) differs significantly from quoted fee ({formatNumber(config.quoted_fee)}) - check for manual overrides
        </div>
      {/if}
    </PanelCard>
  {:else}
    <PanelCard title="Design Phase Breakdown">
      <div class="p-4 text-center">
        <p class="text-emittiv-light text-sm">
          Configure disciplines and stages to see the pricing matrix
        </p>
      </div>
    </PanelCard>
  {/if}

  <!-- Post-Contract Pricing -->
  {#if postContractStages.length > 0}
    <PanelCard title="Post-Contract Pricing">
      <!-- Header row -->
      <div class="emittiv-sortable-header emittiv-sortable-header--compact">
        <div class="emittiv-sortable-col--grow">Service</div>
        <div class="emittiv-sortable-col--number">Qty</div>
        <div class="emittiv-sortable-col--number">Rate</div>
        <div class="emittiv-sortable-col--accent">Total</div>
      </div>

      <!-- One row per post-contract stage -->
      {#each postContractStages as stage (stage.id)}
        {@const item = getItemForStage(stage.id)}
        {#if item}
          <div class="emittiv-sortable-row emittiv-sortable-row--static emittiv-sortable-row--compact">
            <div class="emittiv-sortable-col--grow text-emittiv-white">
              {stage.name}
            </div>
            <div class="emittiv-sortable-col--number">
              {#if !readonly}
                <input
                  type="text"
                  inputmode="numeric"
                  class="emittiv-table-input emittiv-table-input--md"
                  use:formattedNumber={{ value: item.quantity, onChange: (v) => updatePostContractItem(item.id, 'quantity', v), min: 0 }}
                />
              {:else}
                <span class="text-emittiv-white">{item.quantity}</span>
              {/if}
            </div>
            <div class="emittiv-sortable-col--number">
              {#if !readonly}
                <input
                  type="text"
                  inputmode="numeric"
                  class="emittiv-table-input emittiv-table-input--md"
                  use:formattedNumber={{ value: item.rate, onChange: (v) => updatePostContractItem(item.id, 'rate', v), min: 0 }}
                />
              {:else}
                <span class="text-emittiv-white">{formatNumber(item.rate)}</span>
              {/if}
            </div>
            <div class="emittiv-sortable-col--accent">
              {formatNumber(item.amount)}
            </div>
          </div>
        {/if}
      {/each}

      <!-- Subtotal row -->
      <div class="emittiv-sortable-footer emittiv-sortable-footer--compact">
        <div class="emittiv-sortable-col--grow">SUBTOTAL</div>
        <div class="emittiv-sortable-col--number"></div>
        <div class="emittiv-sortable-col--number"></div>
        <div class="emittiv-sortable-col--accent">
          {formatNumber(postContractSubtotal)}
        </div>
      </div>
      <!-- VAT row -->
      <div class="emittiv-sortable-footer emittiv-sortable-footer--compact">
        <div class="emittiv-sortable-col--grow">VAT ({config.vat_percent}%)</div>
        <div class="emittiv-sortable-col--number"></div>
        <div class="emittiv-sortable-col--number"></div>
        <div class="emittiv-sortable-col--accent">
          {formatNumber(postContractVat)}
        </div>
      </div>
      <!-- Total row -->
      <div class="emittiv-sortable-footer emittiv-sortable-footer--compact emittiv-sortable-footer--total">
        <div class="emittiv-sortable-col--grow">TOTAL</div>
        <div class="emittiv-sortable-col--number"></div>
        <div class="emittiv-sortable-col--number"></div>
        <div class="emittiv-sortable-col--accent">
          {formatNumber(postContractTotal)}
        </div>
      </div>
    </PanelCard>
  {/if}
</div>
