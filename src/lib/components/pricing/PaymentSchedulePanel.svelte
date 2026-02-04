<script lang="ts">
  import type { PaymentScheduleEntry, PaymentSchedule, Stage, PricingConfig } from '../../../types/database';
  import { generatePricingId } from '../../../types/database';
  import { formatNumber, formatPercent, formatDate } from '$lib/utils/format';
  import { formattedNumber } from '$lib/actions/formattedNumber';
  import IconButton from '../IconButton.svelte';
  import PanelCard from '../PanelCard.svelte';

  interface Props {
    schedule: PaymentSchedule;
    stages: Stage[];
    config: PricingConfig;
    grandTotal: number;
    onUpdate: (schedule: PaymentSchedule) => void;
    readonly?: boolean;
  }

  let {
    schedule = $bindable(),
    stages,
    config,
    grandTotal,
    onUpdate,
    readonly = false
  }: Props = $props();

  // Design stages for milestone generation
  const designStages = $derived(stages.filter(s => !s.is_post_contract).sort((a, b) => a.order - b.order));

  // Status colors
  const statusColors = {
    pending: 'text-emittiv-light',
    invoiced: 'text-yellow-500',
    paid: 'text-green-500',
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

  // Generate schedule from pricing
  function generateFromPricing() {
    const entries: PaymentScheduleEntry[] = [];

    // Mobilisation
    const mobilisationAmount = grandTotal * (config.mobilisation_percent / 100);
    entries.push({
      id: generatePricingId('pay'),
      type: 'mobilisation',
      description: `Mobilisation (${config.mobilisation_percent}%)`,
      amount: mobilisationAmount,
      percentage_of_total: config.mobilisation_percent,
      status: 'pending',
    });

    // Milestone payments for each design stage
    const remainingPercent = 100 - config.mobilisation_percent;
    const remainingAmount = grandTotal - mobilisationAmount;

    for (const stage of designStages) {
      const stagePercent = stage.percentage * (remainingPercent / 100);
      const stageAmount = remainingAmount * (stage.percentage / 100);

      entries.push({
        id: generatePricingId('pay'),
        type: 'milestone',
        description: `${stage.name} Completion`,
        stage_id: stage.id,
        stage_percentage: 100,
        amount: stageAmount,
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
        updated.percentage_of_total = grandTotal > 0 ? ((value as number) / grandTotal) * 100 : 0;
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

  // Total of all payments
  const scheduledTotal = $derived(schedule.entries.reduce((sum, e) => sum + e.amount, 0));
  const scheduleValid = $derived(Math.abs(scheduledTotal - grandTotal) < 1);
</script>

<PanelCard title="Payment Schedule">
  {#snippet headerActions()}
    {#if !readonly}
      <div class="flex items-center gap-2">
        <button type="button" class="emittiv-text-btn" onclick={generateFromPricing}>
          Generate from Pricing
        </button>
        <IconButton icon="plus" label="Add" variant="primary" size="md" on:click={addPayment} />
      </div>
    {/if}
  {/snippet}

  <!-- Payments Table -->
  {#if schedule.entries.length === 0}
    <div class="p-4 text-center">
      <p class="text-emittiv-light text-sm mb-1">No payment schedule defined.</p>
      {#if !readonly}
        <button type="button" class="emittiv-text-btn emittiv-text-btn--primary" onclick={generateFromPricing}>
          Generate from pricing breakdown
        </button>
      {/if}
    </div>
  {:else}
    <!-- Header row -->
    <div class="emittiv-sortable-header">
      <div class="emittiv-sortable-col--grow">Payment</div>
      <div class="emittiv-sortable-col--number">Amount</div>
      <div class="emittiv-sortable-col--pct">%</div>
      <div class="emittiv-sortable-col--status">Status</div>
      {#if !readonly}<div class="emittiv-sortable-col--action"></div>{/if}
    </div>

    <!-- Data rows -->
    {#each schedule.entries as entry (entry.id)}
      <div class="emittiv-sortable-row emittiv-sortable-row--static">
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

        <!-- Amount -->
        <div class="emittiv-sortable-col--number">
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

        <!-- Percentage -->
        <div class="emittiv-sortable-col--pct">
          <span class="text-emittiv-light">{formatPercent(entry.percentage_of_total, 1)}</span>
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
                  on:click={() => splitPayment(entry.id)}
                />
              {/if}
              <IconButton icon="trash" variant="danger" size="sm" title="Remove" on:click={() => removePayment(entry.id)} />
            </div>
          </div>
        {/if}
      </div>
    {/each}

    <!-- Footer/totals row -->
    <div class="emittiv-sortable-footer">
      <div class="emittiv-sortable-col--grow">TOTAL</div>
      <div class="emittiv-sortable-col--number">
        <span class:text-emittiv-splash={scheduleValid} class:text-red-500={!scheduleValid} class="font-bold">
          {formatNumber(scheduledTotal)}
        </span>
      </div>
      <div class="emittiv-sortable-col--pct">
        <span class="text-emittiv-light">{formatPercent(grandTotal > 0 ? (scheduledTotal / grandTotal) * 100 : 0, 1)}</span>
      </div>
      <div class="emittiv-sortable-col--status"></div>
      {#if !readonly}<div class="emittiv-sortable-col--action"></div>{/if}
    </div>

    <!-- Validation warning -->
    {#if !scheduleValid}
      <div class="emittiv-matrix-warning">
        Schedule total ({formatNumber(scheduledTotal)}) doesn't match grand total ({formatNumber(grandTotal)})
      </div>
    {/if}
  {/if}

  {#snippet footerContent()}
    <div class="flex items-center gap-3 text-xs">
      <div class="flex items-center gap-1">
        <span class="text-emittiv-light">Invoiced:</span>
        <span class="text-yellow-500 font-medium">{formatNumber(schedule.total_invoiced)}</span>
      </div>
      <div class="flex items-center gap-1">
        <span class="text-emittiv-light">Paid:</span>
        <span class="text-green-500 font-medium">{formatNumber(schedule.total_paid)}</span>
      </div>
      <div class="flex items-center gap-1">
        <span class="text-emittiv-light">Outstanding:</span>
        <span class="text-emittiv-splash font-medium">{formatNumber(schedule.total_outstanding)}</span>
      </div>
    </div>
  {/snippet}
</PanelCard>
