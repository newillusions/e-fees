<script lang="ts">
  import type { ReimbursableCost, Stage } from '../../../types/database';
  import { generatePricingId, calculateCostWithMarkup, DEFAULT_COST_MARKUP_PERCENT } from '../../../types/database';
  import { formatNumber, formatPercent } from '$lib/utils/format';
  import { formattedNumber } from '$lib/actions/formattedNumber';
  import IconButton from '../IconButton.svelte';
  import PanelCard from '../PanelCard.svelte';

  interface Props {
    costs: ReimbursableCost[];
    stages: Stage[];
    onUpdate: (costs: ReimbursableCost[]) => void;
    defaultMarkup?: number;
    readonly?: boolean;
  }

  let {
    costs = $bindable([]),
    stages,
    onUpdate,
    defaultMarkup = $bindable(DEFAULT_COST_MARKUP_PERCENT),
    readonly = false
  }: Props = $props();

  // All stages for dropdown
  const allStages = $derived([...stages].sort((a, b) => a.order - b.order));

  // Calculate totals
  const totalBaseCost = $derived(costs.reduce((sum, c) => sum + c.base_cost, 0));
  const totalClientCost = $derived(costs.reduce((sum, c) => sum + c.cost_to_client, 0));

  function addCost() {
    const newCost: ReimbursableCost = {
      id: generatePricingId('cost'),
      description: 'New Expense',
      stage_id: allStages[0]?.id || '',
      base_cost: 0,
      markup_percent: defaultMarkup,
      cost_to_client: 0,
      date_incurred: new Date().toISOString().split('T')[0],
    };
    const updated = [...costs, newCost];
    costs = updated;
    onUpdate(updated);
  }

  function removeCost(id: string) {
    const updated = costs.filter(c => c.id !== id);
    costs = updated;
    onUpdate(updated);
  }

  function updateCost(id: string, field: keyof ReimbursableCost, value: string | number) {
    const updated = costs.map(cost => {
      if (cost.id !== id) return cost;
      const newCost = { ...cost, [field]: value };
      // Recalculate cost_to_client when base_cost or markup changes
      if (field === 'base_cost' || field === 'markup_percent') {
        newCost.cost_to_client = calculateCostWithMarkup(
          field === 'base_cost' ? (value as number) : newCost.base_cost,
          field === 'markup_percent' ? (value as number) : newCost.markup_percent
        );
      }
      return newCost;
    });
    costs = updated;
    onUpdate(updated);
  }

  function getStageName(stageId: string): string {
    const stage = allStages.find(s => s.id === stageId);
    return stage?.name || 'Unknown';
  }

  // Apply default markup to all costs
  function applyDefaultMarkup() {
    const updated = costs.map(cost => ({
      ...cost,
      markup_percent: defaultMarkup,
      cost_to_client: calculateCostWithMarkup(cost.base_cost, defaultMarkup),
    }));
    costs = updated;
    onUpdate(updated);
  }
</script>

<PanelCard title="Reimbursable Costs">
  {#snippet headerActions()}
    {#if !readonly}
      <div class="flex items-center gap-2">
        {#if costs.length > 0}
          <button type="button" class="emittiv-text-btn" onclick={applyDefaultMarkup}>
            Apply {defaultMarkup}% to all
          </button>
        {/if}
        <IconButton icon="plus" label="Add Cost" variant="primary" size="md" on:click={addCost} />
      </div>
    {/if}
  {/snippet}

  <!-- Costs List -->
  {#if costs.length === 0}
    <div class="p-4 text-center">
      <p class="text-emittiv-light text-sm mb-1">No reimbursable costs added.</p>
      {#if !readonly}
        <p class="text-emittiv-dark text-xxs">
          Add costs like document purchases, samples, or travel expenses.
        </p>
      {/if}
    </div>
  {:else}
    <!-- Header row -->
    <div class="emittiv-sortable-header">
      <div class="emittiv-sortable-col--grow">Description</div>
      <div class="emittiv-sortable-col--select">Stage</div>
      <div class="emittiv-sortable-col--number">Base Cost</div>
      <div class="emittiv-sortable-col--number">Markup</div>
      <div class="emittiv-sortable-col--accent">Client Cost</div>
      {#if !readonly}<div class="emittiv-sortable-col--action"></div>{/if}
    </div>

    <!-- Data rows -->
    {#each costs as cost (cost.id)}
      <div class="emittiv-sortable-row emittiv-sortable-row--static">
        <!-- Description -->
        <div class="emittiv-sortable-col--grow">
          {#if !readonly}
            <input
              type="text"
              class="emittiv-table-input emittiv-table-input--left"
              value={cost.description}
              onchange={(e) => updateCost(cost.id, 'description', e.currentTarget.value)}
            />
          {:else}
            <span class="text-emittiv-white">{cost.description}</span>
          {/if}
        </div>

        <!-- Stage -->
        <div class="emittiv-sortable-col--select">
          {#if !readonly}
            <select
              class="emittiv-select"
              value={cost.stage_id}
              onchange={(e) => updateCost(cost.id, 'stage_id', e.currentTarget.value)}
            >
              {#each allStages as stage}
                <option value={stage.id}>{stage.name}</option>
              {/each}
            </select>
          {:else}
            <span class="text-emittiv-light">{getStageName(cost.stage_id)}</span>
          {/if}
        </div>

        <!-- Base Cost -->
        <div class="emittiv-sortable-col--number">
          {#if !readonly}
            <input
              type="text"
              inputmode="numeric"
              class="emittiv-table-input emittiv-table-input--lg"
              use:formattedNumber={{ value: cost.base_cost, onChange: (v) => updateCost(cost.id, 'base_cost', v), min: 0 }}
            />
          {:else}
            <span class="text-emittiv-white">{formatNumber(cost.base_cost)}</span>
          {/if}
        </div>

        <!-- Markup -->
        <div class="emittiv-sortable-col--number">
          {#if !readonly}
            <input
              type="number"
              min="0"
              max="100"
              step="5"
              class="emittiv-table-input emittiv-table-input--lg"
              value={cost.markup_percent}
              onchange={(e) => updateCost(cost.id, 'markup_percent', parseFloat(e.currentTarget.value) || 0)}
            />
            <span class="text-emittiv-light">%</span>
          {:else}
            <span class="text-emittiv-white">{formatPercent(cost.markup_percent)}</span>
          {/if}
        </div>

        <!-- Client Cost -->
        <div class="emittiv-sortable-col--accent">
          {formatNumber(cost.cost_to_client)}
        </div>

        <!-- Remove -->
        {#if !readonly}
          <div class="emittiv-sortable-col--action">
            <IconButton icon="trash" variant="danger" size="sm" title="Remove" on:click={() => removeCost(cost.id)} />
          </div>
        {/if}
      </div>
    {/each}

    <!-- Footer/totals row -->
    <div class="emittiv-sortable-footer">
      <div class="emittiv-sortable-col--grow">TOTAL</div>
      <div class="emittiv-sortable-col--select"></div>
      <div class="emittiv-sortable-col--number">
        <span>{formatNumber(totalBaseCost)}</span>
      </div>
      <div class="emittiv-sortable-col--number"></div>
      <div class="emittiv-sortable-col--accent">
        {formatNumber(totalClientCost)}
      </div>
      {#if !readonly}<div class="emittiv-sortable-col--action"></div>{/if}
    </div>
  {/if}

  {#snippet footerContent()}
    {#if !readonly}
      <div class="flex items-center gap-1.5">
        <span class="text-emittiv-light text-xxs">Default Markup:</span>
        <div class="emittiv-field-suffix">
          <input
            type="number"
            min="0"
            max="100"
            step="5"
            class="emittiv-table-input emittiv-table-input--md"
            value={defaultMarkup}
            onchange={(e) => defaultMarkup = parseFloat(e.currentTarget.value) || 0}
          />
          <span class="emittiv-field-suffix__unit">%</span>
        </div>
      </div>
    {/if}
  {/snippet}
</PanelCard>
