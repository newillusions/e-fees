<script lang="ts">
  import type { PaymentScheduleEntry, PaymentSchedule, Stage, PricingConfig, PricingCell } from '../../../types/database';
  import { generatePricingId } from '../../../types/database';
  import { formatNumber, formatPercent, formatDate } from '$lib/utils/format';
  import { roundWithConfig, whtTooltip as whtTooltipFn } from '$lib/utils/pricingUtils';
  import { formattedNumber } from '$lib/actions/formattedNumber';
  import IconButton from '../IconButton.svelte';
  import PanelCard from '../PanelCard.svelte';

  interface Props {
    schedule: PaymentSchedule;
    stages: Stage[];
    cells: PricingCell[];
    config: PricingConfig;
    designTotal: number;
    onUpdate: (schedule: PaymentSchedule) => void;
    readonly?: boolean;
  }

  let {
    schedule = $bindable(),
    stages,
    cells,
    config,
    designTotal: rawDesignTotal,
    onUpdate,
    readonly = false
  }: Props = $props();

  // Design stages for milestone generation
  const designStages = $derived(stages.filter(s => !s.is_post_contract).sort((a, b) => a.order - b.order));

  // Compute design total from cells + stages (same logic as generateFromPricing)
  // This is more reliable than the prop which depends on Svelte 4 $: reactivity
  const designTotal = $derived(
    designStages.reduce((sum, stage) => sum + getRoundedStageTotal(stage.id), 0)
  );

  // Calculate rounded stage total from cells (same logic as PricingCalculatorPanel)
  function getRoundedStageTotal(stageId: string): number {
    const rawTotal = cells
      .filter(c => c.stage_id === stageId)
      .reduce((sum, c) => sum + (c.override_amount ?? c.amount), 0);
    return roundWithConfig(rawTotal, config);
  }

  // Status colors
  const statusColors = {
    pending: 'payment-pending',
    invoiced: 'payment-invoiced',
    paid: 'payment-paid',
  };

  const statusIcons = {
    pending: '○',
    invoiced: '◐',
    paid: '●',
  };

  // Calculate totals
  function recalculateTotals(entries: PaymentScheduleEntry[]): PaymentSchedule {
    const totalInvoiced = entries
      .filter(e => e.status === 'invoiced' || e.status === 'paid')
      .reduce((sum, e) => sum + e.amount, 0);
    const totalPaid = entries
      .filter(e => e.status === 'paid')
      .reduce((sum, e) => sum + e.amount, 0);
    const totalOutstanding = entries.reduce((sum, e) => sum + e.amount, 0) - totalPaid;

    return {
      entries,
      total_invoiced: totalInvoiced,
      total_paid: totalPaid,
      total_outstanding: totalOutstanding,
    };
  }

  // Generate schedule from pricing using actual rounded stage totals
  function generateFromPricing() {
    const entries: PaymentScheduleEntry[] = [];

    // Calculate design subtotal from actual rounded stage totals
    const designSubtotal = designStages.reduce((sum, stage) => sum + getRoundedStageTotal(stage.id), 0);

    // Mobilisation based on design subtotal
    const mobilisationAmount = designSubtotal * (config.mobilisation_percent / 100);
    const mobilisationPercent = designSubtotal > 0 ? (mobilisationAmount / designSubtotal) * 100 : 0;
    entries.push({
      id: generatePricingId('pay'),
      type: 'mobilisation',
      description: `Mobilisation (${config.mobilisation_percent}%)`,
      amount: mobilisationAmount,
      percentage_of_total: mobilisationPercent,
      status: 'pending',
    });

    // Milestone payments using actual rounded stage totals
    // Each stage payment = quoted stage value minus its share of mobilisation
    const remainingAmount = designSubtotal - mobilisationAmount;

    for (const stage of designStages) {
      const stageTotal = getRoundedStageTotal(stage.id);
      // Payment = stage's proportion of remaining (after mobilisation deduction)
      const stageAmount = designSubtotal > 0 ? remainingAmount * (stageTotal / designSubtotal) : 0;
      const stagePercent = designSubtotal > 0 ? (stageAmount / designSubtotal) * 100 : 0;

      entries.push({
        id: generatePricingId('pay'),
        type: 'milestone',
        description: `${stage.name} Submittal`,
        stage_id: stage.id,
        stage_percentage: 100,
        amount: stageAmount,
        quoted_stage_amount: stageTotal,
        percentage_of_total: stagePercent,
        status: 'pending',
      });
    }

    const newSchedule = recalculateTotals(entries);
    schedule = newSchedule;
    onUpdate(newSchedule);
  }

  function addPayment() {
    const newEntry: PaymentScheduleEntry = {
      id: generatePricingId('pay'),
      type: 'milestone',
      description: 'New Payment',
      amount: 0,
      percentage_of_total: 0,
      status: 'pending',
    };
    const entries = [...schedule.entries, newEntry];
    const newSchedule = recalculateTotals(entries);
    schedule = newSchedule;
    onUpdate(newSchedule);
  }

  function removePayment(id: string) {
    const entries = schedule.entries.filter(e => e.id !== id);
    const newSchedule = recalculateTotals(entries);
    schedule = newSchedule;
    onUpdate(newSchedule);
  }

  function updatePayment(id: string, field: keyof PaymentScheduleEntry, value: string | number) {
    const entries = schedule.entries.map(entry => {
      if (entry.id !== id) return entry;
      const updated = { ...entry, [field]: value };
      // Recalculate percentage when amount changes
      if (field === 'amount') {
        updated.percentage_of_total = designTotal > 0 ? ((value as number) / designTotal) * 100 : 0;
      }
      // Recalculate amount when percentage changes
      if (field === 'percentage_of_total') {
        updated.amount = designTotal * ((value as number) / 100);
      }
      return updated;
    });
    const newSchedule = recalculateTotals(entries);
    schedule = newSchedule;
    onUpdate(newSchedule);
  }

  function cycleStatus(id: string) {
    const entry = schedule.entries.find(e => e.id === id);
    if (!entry) return;

    const statusOrder: PaymentScheduleEntry['status'][] = ['pending', 'invoiced', 'paid'];
    const currentIndex = statusOrder.indexOf(entry.status);
    const nextStatus = statusOrder[(currentIndex + 1) % statusOrder.length];

    updatePayment(id, 'status', nextStatus);
  }

  function getStageName(stageId?: string): string {
    if (!stageId) return '';
    const stage = stages.find(s => s.id === stageId);
    return stage?.name || '';
  }

  // Split a payment into sub-milestones (e.g., 50%/100% DD)
  function splitPayment(id: string) {
    const entry = schedule.entries.find(e => e.id === id);
    if (!entry || entry.type !== 'milestone' || !entry.stage_id) return;

    const halfAmount = entry.amount / 2;
    const halfPercent = entry.percentage_of_total / 2;
    const stageName = getStageName(entry.stage_id);

    const entries = schedule.entries.flatMap(e => {
      if (e.id !== id) return [e];
      return [
        {
          ...e,
          id: generatePricingId('pay'),
          description: `50% ${stageName}`,
          stage_percentage: 50,
          amount: halfAmount,
          percentage_of_total: halfPercent,
        },
        {
          ...e,
          id: generatePricingId('pay'),
          description: `100% ${stageName}`,
          stage_percentage: 100,
          amount: halfAmount,
          percentage_of_total: halfPercent,
        },
      ];
    });

    const newSchedule = recalculateTotals(entries);
    schedule = newSchedule;
    onUpdate(newSchedule);
  }

  // Withholding tax gross-up for payment tooltips
  function whtTooltip(amount: number): string {
    return whtTooltipFn(amount, config, formatNumber);
  }

  // Total of all payments
  const scheduledTotal = $derived(schedule.entries.reduce((sum, e) => sum + e.amount, 0));
  const scheduleDifference = $derived(scheduledTotal - designTotal);
  const scheduleValid = $derived(Math.abs(scheduleDifference) < 1);
</script>

<PanelCard title="Payment Schedule">
  {#snippet headerActions()}
    {#if !readonly}
      <div class="flex items-center gap-2">
        <button type="button" class="emittiv-text-btn" onclick={generateFromPricing}>
          Generate from Pricing
        </button>
        <IconButton icon="plus" label="Add" variant="primary" size="md" onclick={addPayment} />
      </div>
    {/if}
  {/snippet}

  <!-- Payments Table -->
  {#if schedule.entries.length === 0}
    <div class="p-6 text-center">
      <p class="text-emittiv-light text-sm mb-3">No payment schedule defined.</p>
      {#if !readonly}
        <button type="button" class="emittiv-btn emittiv-btn--primary emittiv-btn--lg" onclick={generateFromPricing}>
          Generate from Pricing
        </button>
      {/if}
    </div>
  {:else}
    <!-- Header row -->
    <div class="emittiv-sortable-header emittiv-sortable-header--compact">
      <div class="emittiv-sortable-col--grow">Payment</div>
      <div class="emittiv-sortable-col--number">Quoted</div>
      <div class="emittiv-sortable-col--number">Payment</div>
      <div class="emittiv-sortable-col--status">Status</div>
      {#if !readonly}<div class="emittiv-sortable-col--action"></div>{/if}
    </div>

    <!-- Data rows -->
    {#each schedule.entries as entry (entry.id)}
      <div class="emittiv-sortable-row emittiv-sortable-row--static emittiv-sortable-row--compact">
        <!-- Description -->
        <div class="emittiv-sortable-col--grow">
          {#if !readonly}
            <input
              type="text"
              class="emittiv-table-input emittiv-table-input--left"
              value={entry.description}
              onchange={(e) => updatePayment(entry.id, 'description', e.currentTarget.value)}
            />
          {:else}
            <span class="text-emittiv-white">{entry.description}</span>
          {/if}
        </div>

        <!-- Quoted stage value / mobilisation % -->
        <div class="emittiv-sortable-col--number">
          {#if entry.type === 'mobilisation'}
            <span class="text-emittiv-light">{formatPercent(config.mobilisation_percent)}</span>
          {:else if entry.quoted_stage_amount !== undefined}
            <span class="text-emittiv-light">{formatNumber(entry.quoted_stage_amount)}</span>
          {:else}
            <span class="text-emittiv-dark">—</span>
          {/if}
        </div>

        <!-- Payment amount -->
        <div class="emittiv-sortable-col--number" title={whtTooltip(entry.amount)}>
          {#if !readonly}
            <input
              type="text"
              inputmode="numeric"
              class="emittiv-table-input emittiv-table-input--lg"
              use:formattedNumber={{ value: Math.round(entry.amount), onChange: (v) => updatePayment(entry.id, 'amount', v), min: 0 }}
            />
          {:else}
            <span class="text-emittiv-splash font-medium">{formatNumber(entry.amount)}</span>
          {/if}
        </div>

        <!-- Status -->
        <div class="emittiv-sortable-col--status">
          {#if !readonly}
            <button
              type="button"
              class="emittiv-status-btn {statusColors[entry.status]}"
              onclick={() => cycleStatus(entry.id)}
              title="Click to change status"
            >
              {statusIcons[entry.status]} {entry.status}
            </button>
          {:else}
            <span class="{statusColors[entry.status]} text-xs">
              {statusIcons[entry.status]} {entry.status}
            </span>
          {/if}
        </div>

        <!-- Actions -->
        {#if !readonly}
          <div class="emittiv-sortable-col--action">
            <div class="flex items-center gap-0.5">
              {#if entry.type === 'milestone' && entry.stage_id && !entry.description.includes('%')}
                <IconButton
                  icon="split"
                  variant="secondary"
                  size="sm"
                  title="Split into 50%/100%"
                  onclick={() => splitPayment(entry.id)}
                />
              {/if}
              <IconButton icon="trash" variant="danger" size="sm" title="Remove" onclick={() => removePayment(entry.id)} />
            </div>
          </div>
        {/if}
      </div>
    {/each}

    <!-- Footer/totals row -->
    <div class="emittiv-sortable-footer emittiv-sortable-footer--compact">
      <div class="emittiv-sortable-col--grow">TOTAL</div>
      <div class="emittiv-sortable-col--number"></div>
      <div class="emittiv-sortable-col--number">
        <span class:text-emittiv-splash={scheduleValid} class:schedule-invalid={!scheduleValid} class="font-bold">
          {formatNumber(scheduledTotal)}
        </span>
      </div>
      <div class="emittiv-sortable-col--status"></div>
      {#if !readonly}<div class="emittiv-sortable-col--action"></div>{/if}
    </div>

    <!-- Validation warning -->
    {#if !scheduleValid}
      <div class="emittiv-schedule-diff">
        <span class="text-emittiv-light">Schedule difference:</span>
        <span class={scheduleDifference > 0 ? 'schedule-surplus' : 'schedule-invalid'} class:font-medium={true}>
          {scheduleDifference > 0 ? '+' : ''}{formatNumber(Math.round(scheduleDifference))}
        </span>
        <span class="text-emittiv-light">from target</span>
      </div>
    {/if}
  {/if}

  {#snippet footerContent()}
    <div class="flex items-center gap-3 text-xs">
      <div class="flex items-center gap-1">
        <span class="text-emittiv-light">Invoiced:</span>
        <span class="payment-invoiced font-medium">{formatNumber(schedule.total_invoiced)}</span>
      </div>
      <div class="flex items-center gap-1">
        <span class="text-emittiv-light">Paid:</span>
        <span class="payment-paid font-medium">{formatNumber(schedule.total_paid)}</span>
      </div>
      <div class="flex items-center gap-1">
        <span class="text-emittiv-light">Outstanding:</span>
        <span class="text-emittiv-splash font-medium">{formatNumber(schedule.total_outstanding)}</span>
      </div>
    </div>
  {/snippet}
</PanelCard>

<style>
  .payment-pending  { color: var(--color-status-pending); }
  .payment-invoiced { color: var(--color-status-invoiced); }
  .payment-paid     { color: var(--color-status-paid); }
  .schedule-invalid { color: var(--color-error); }
  .schedule-surplus { color: var(--color-success); }
</style>
