<script lang="ts">
  import type { PaymentScheduleEntry, PaymentSchedule, Stage, PricingConfig } from '../../../types/database';
  import { generatePricingId } from '../../../types/database';
  import { formatNumber, formatPercent, formatDate } from '$lib/utils/format';
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
      if (field === 'amount' && grandTotal > 0) {
        updated.percentage_of_total = ((value as number) / grandTotal) * 100;
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
        <button
          type="button"
          class="text-xxs text-emittiv-light hover:text-emittiv-white transition-colors"
          onclick={generateFromPricing}
        >
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
        <button
          type="button"
          class="text-emittiv-splash hover:underline text-xs"
          onclick={generateFromPricing}
        >
          Generate from pricing breakdown
        </button>
      {/if}
    </div>
  {:else}
    <div class="overflow-x-auto">
      <table class="w-full text-xs">
        <thead>
          <tr class="border-b border-emittiv-dark bg-emittiv-black/30">
            <th class="px-2 py-1.5 text-left text-emittiv-light font-medium">Payment</th>
            <th class="px-2 py-1.5 text-right text-emittiv-light font-medium">Amount</th>
            <th class="px-2 py-1.5 text-right text-emittiv-light font-medium">%</th>
            <th class="px-2 py-1.5 text-center text-emittiv-light font-medium">Status</th>
            {#if !readonly}
              <th class="px-2 py-1.5 w-16"></th>
            {/if}
          </tr>
        </thead>
        <tbody>
          {#each schedule.entries as entry (entry.id)}
            <tr class="border-b border-emittiv-dark/50 hover:bg-emittiv-black/20">
              <!-- Description -->
              <td class="px-2 py-1">
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
              </td>

              <!-- Amount -->
              <td class="px-2 py-1 text-right">
                {#if !readonly}
                  <input
                    type="number"
                    min="0"
                    step="1000"
                    class="emittiv-table-input emittiv-table-input--lg emittiv-table-input--accent"
                    value={Math.round(entry.amount)}
                    onchange={(e) => updatePayment(entry.id, 'amount', parseFloat(e.currentTarget.value) || 0)}
                  />
                {:else}
                  <span class="text-emittiv-splash font-medium">{formatNumber(entry.amount)}</span>
                {/if}
              </td>

              <!-- Percentage -->
              <td class="px-2 py-1 text-right text-emittiv-light">
                {formatPercent(entry.percentage_of_total, 1)}
              </td>

              <!-- Status -->
              <td class="px-2 py-1 text-center">
                {#if !readonly}
                  <button
                    type="button"
                    class="px-1.5 py-0.5 rounded text-xs font-medium transition-colors {statusColors[entry.status]}"
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
              </td>

              <!-- Actions -->
              {#if !readonly}
                <td class="px-2 py-1 text-right">
                  <div class="flex items-center justify-end gap-0.5">
                    {#if entry.type === 'milestone' && entry.stage_id && !entry.description.includes('%')}
                      <IconButton
                        icon="split"
                        variant="secondary"
                        size="md"
                        title="Split into 50%/100%"
                        on:click={() => splitPayment(entry.id)}
                      />
                    {/if}
                    <IconButton icon="trash" variant="danger" size="md" title="Remove" on:click={() => removePayment(entry.id)} />
                  </div>
                </td>
              {/if}
            </tr>
          {/each}
        </tbody>
        <tfoot>
          <tr class="bg-emittiv-black/30 border-t border-emittiv-dark">
            <td class="px-2 py-1.5 text-emittiv-white font-medium">TOTAL</td>
            <td class="px-2 py-1.5 text-right font-bold" class:text-emittiv-splash={scheduleValid} class:text-red-500={!scheduleValid}>
              {formatNumber(scheduledTotal)}
            </td>
            <td class="px-2 py-1.5 text-right text-emittiv-light">
              {formatPercent(grandTotal > 0 ? (scheduledTotal / grandTotal) * 100 : 0, 1)}
            </td>
            <td colspan={readonly ? 1 : 2}></td>
          </tr>
        </tfoot>
      </table>
    </div>

    <!-- Validation warning -->
    {#if !scheduleValid}
      <div class="px-3 py-1.5 bg-red-500/10 border-t border-red-500/30">
        <span class="text-red-500 text-xxs">
          Schedule total ({formatNumber(scheduledTotal)}) doesn't match grand total ({formatNumber(grandTotal)})
        </span>
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
