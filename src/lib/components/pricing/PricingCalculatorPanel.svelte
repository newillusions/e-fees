<script lang="ts">
  import type { PricingConfig, Discipline, Stage, PricingCell, PostContractItem } from '../../../types/database';
  import {
    calculateQuotedFee,
    calculateCellAmount,
    generatePricingId,
    DEFAULT_PRICING_CONFIG
  } from '../../../types/database';
  import { formatCurrency, formatNumber, formatPercent } from '$lib/utils/format';
  import { formattedNumber } from '$lib/actions/formattedNumber';
  import IconButton from '../IconButton.svelte';
  import PanelCard from '../PanelCard.svelte';

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

  // Post-contract totals
  const postContractSubtotal = $derived(postContractItems.reduce((sum, i) => sum + i.amount, 0));
  const postContractVat = $derived(postContractSubtotal * (config?.vat_percent || 0) / 100);
  const postContractTotal = $derived(postContractSubtotal + postContractVat);

  // Recalculate quoted fee when target or buffer changes
  $effect(() => {
    if (config && config.target_fee > 0) {
      const newQuoted = calculateQuotedFee(config.target_fee, config.buffer_percent);
      if (Math.abs(newQuoted - config.quoted_fee) > 0.01) {
        config = { ...config, quoted_fee: newQuoted };
        onUpdateConfig(config);
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
      // Only update if cells actually changed
      const cellsChanged = newCells.length !== cells.length ||
        newCells.some((c, i) => {
          const old = cells[i];
          return !old || c.discipline_id !== old.discipline_id ||
            c.stage_id !== old.stage_id ||
            Math.abs(c.amount - old.amount) > 0.01;
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

  // Check if cell has override
  function hasOverride(disciplineId: string, stageId: string): boolean {
    const cell = cells.find(c => c.discipline_id === disciplineId && c.stage_id === stageId);
    return cell?.override_amount !== undefined;
  }

  // Set cell override
  function setCellOverride(disciplineId: string, stageId: string, value: number | null) {
    const updated = cells.map(c => {
      if (c.discipline_id === disciplineId && c.stage_id === stageId) {
        return { ...c, override_amount: value ?? undefined };
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

  function getDisciplineTotal(disciplineId: string): number {
    return designStages.reduce((sum, stage) => sum + getCellValue(disciplineId, stage.id), 0);
  }

  const designSubtotal = $derived(
    sortedDisciplines.reduce((sum, disc) => sum + getDisciplineTotal(disc.id), 0)
  );
  const designVat = $derived(designSubtotal * (config?.vat_percent || 0) / 100);
  const grandTotal = $derived(designSubtotal + designVat);

  // Update config field
  function updateConfig(field: keyof PricingConfig, value: number | string | boolean) {
    config = { ...config, [field]: value };
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
        <label class="emittiv-label">Quoted Fee</label>
        <span class="emittiv-calc-quoted">{formatCurrency(config.quoted_fee, config.currency)}</span>
      </div>
    </div>
  </PanelCard>

  <!-- Design Phase Pricing Matrix -->
  {#if designStages.length > 0 && sortedDisciplines.length > 0}
    <PanelCard title="Design Phase Breakdown">
      <!-- Header row -->
      <div class="emittiv-sortable-header emittiv-sortable-header--compact">
        <div class="emittiv-sortable-col--grow">Stage</div>
        {#each sortedDisciplines as disc}
          <div class="emittiv-sortable-col--number">
            {disc.name}
            <span class="emittiv-sortable-col--subtitle">{formatPercent(disc.percentage)}</span>
          </div>
        {/each}
        <div class="emittiv-sortable-col--accent">Total</div>
      </div>

      <!-- Data rows -->
      {#each designStages as stage}
        <div class="emittiv-sortable-row emittiv-sortable-row--static emittiv-sortable-row--compact">
          <div class="emittiv-sortable-col--grow text-emittiv-white">
            {stage.name}
            <span class="emittiv-sortable-col--subtitle">({formatPercent(stage.percentage)})</span>
          </div>
          {#each sortedDisciplines as disc}
            {@const cellValue = getCellValue(disc.id, stage.id)}
            {@const isOverridden = hasOverride(disc.id, stage.id)}
            <div class="emittiv-sortable-col--number">
              {#if !readonly}
                <div class="emittiv-sortable-cell-group">
                  <input
                    type="text"
                    inputmode="numeric"
                    class="emittiv-table-input emittiv-table-input--md"
                    class:emittiv-text--override={isOverridden}
                    use:formattedNumber={{ value: Math.round(cellValue), onChange: (v) => setCellOverride(disc.id, stage.id, v), min: 0 }}
                  />
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
                </div>
              {:else}
                <span class:emittiv-text--override={isOverridden}>
                  {formatNumber(cellValue)}
                </span>
              {/if}
            </div>
          {/each}
          <div class="emittiv-sortable-col--accent">
            {formatNumber(getStageTotal(stage.id))}
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
      <!-- VAT row -->
      <div class="emittiv-sortable-footer emittiv-sortable-footer--compact">
        <div class="emittiv-sortable-col--grow">VAT ({config.vat_percent}%)</div>
        {#each sortedDisciplines as disc}
          <div class="emittiv-sortable-col--number"></div>
        {/each}
        <div class="emittiv-sortable-col--accent">
          {formatNumber(designVat)}
        </div>
      </div>
      <!-- Total row -->
      <div class="emittiv-sortable-footer emittiv-sortable-footer--compact emittiv-sortable-footer--total">
        <div class="emittiv-sortable-col--grow">TOTAL</div>
        {#each sortedDisciplines as disc}
          <div class="emittiv-sortable-col--number"></div>
        {/each}
        <div class="emittiv-sortable-col--accent">
          {formatNumber(grandTotal)}
        </div>
      </div>

      <!-- Matrix validation -->
      {#if Math.abs(designSubtotal - config.quoted_fee) > 1}
        <div class="emittiv-matrix-warning">
          Note: Matrix subtotal ({formatNumber(designSubtotal)}) differs from quoted fee ({formatNumber(config.quoted_fee)}) due to manual overrides
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
