<script lang="ts">
  import type {
    PricingBreakdown,
    Discipline,
    Stage,
    PostContractItem,
    ReimbursableCost,
    PaymentSchedule
  } from '../../../types/database';
  import { formatCurrency, formatNumber, formatPercent } from '$lib/utils/format';
  import IconButton from '../IconButton.svelte';

  interface Props {
    pricing: PricingBreakdown | null;
    postContractItems: PostContractItem[];
    paymentSchedule: PaymentSchedule | null;
    onClose?: () => void;
  }

  let { pricing, postContractItems, paymentSchedule, onClose }: Props = $props();

  // Sorted data
  const disciplines = $derived(
    [...(pricing?.disciplines || [])].sort((a, b) => a.order - b.order)
  );
  const designStages = $derived(
    pricing?.stages?.filter(s => !s.is_post_contract).sort((a, b) => a.order - b.order) || []
  );
  const costs = $derived(pricing?.costs || []);
  const currency = $derived(pricing?.config?.currency || 'AED');

  // Calculate discipline totals
  function getDisciplineTotal(disciplineId: string): number {
    if (!pricing?.cells) return 0;
    return pricing.cells
      .filter(c => c.discipline_id === disciplineId)
      .reduce((sum, c) => sum + (c.override_amount ?? c.amount), 0);
  }
</script>

<div class="bg-emittiv-darker border border-emittiv-dark rounded-lg overflow-hidden">
  <!-- Header -->
  <div class="flex items-center justify-between px-4 py-3 border-b border-emittiv-dark">
    <h3 class="text-emittiv-white font-medium">Fee Summary</h3>
    {#if onClose}
      <IconButton icon="x" variant="secondary" size="md" on:click={onClose} />
    {/if}
  </div>

  {#if !pricing}
    <div class="p-8 text-center">
      <p class="text-emittiv-light text-sm">No pricing data available.</p>
    </div>
  {:else}
    <div class="p-4 space-y-6">
      <!-- Target / Buffer / Quoted -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-emittiv-light">TARGET FEE</span>
          <span class="text-emittiv-white font-medium">
            {formatCurrency(pricing.config.target_fee, currency)}
          </span>
        </div>
        <div class="flex items-center justify-between text-sm">
          <span class="text-emittiv-dark">Buffer ({formatPercent(pricing.config.buffer_percent)})</span>
          <span class="text-emittiv-light">
            +{formatCurrency(pricing.config.quoted_fee - pricing.config.target_fee, currency)}
          </span>
        </div>
        <div class="border-t border-emittiv-dark pt-2 flex items-center justify-between">
          <span class="text-emittiv-white font-medium">QUOTED FEE</span>
          <span class="text-emittiv-splash font-bold text-lg">
            {formatCurrency(pricing.config.quoted_fee, currency)}
          </span>
        </div>
      </div>

      <!-- Fee Breakdown -->
      <div class="border-t border-emittiv-dark pt-4">
        <h4 class="text-emittiv-white font-medium mb-3">Fee Breakdown</h4>

        <!-- Design Phase -->
        <div class="space-y-2 mb-4">
          <div class="flex items-center justify-between">
            <span class="text-emittiv-light">Design Phase</span>
            <span class="text-emittiv-white font-medium">
              {formatCurrency(pricing.design_phase_total, currency)}
            </span>
          </div>
          {#each disciplines as disc}
            <div class="flex items-center justify-between pl-4 text-sm">
              <span class="text-emittiv-dark">• {disc.name} ({formatPercent(disc.percentage)})</span>
              <span class="text-emittiv-light">
                {formatCurrency(getDisciplineTotal(disc.id), currency)}
              </span>
            </div>
          {/each}
        </div>

        <!-- Post-Contract Services -->
        {#if postContractItems.length > 0}
          <div class="space-y-2 mb-4">
            <div class="flex items-center justify-between">
              <span class="text-emittiv-light">Post-Contract Services</span>
              <span class="text-emittiv-white font-medium">
                {formatCurrency(pricing.post_contract_total, currency)}
              </span>
            </div>
            {#each postContractItems as item}
              <div class="flex items-center justify-between pl-4 text-sm">
                <span class="text-emittiv-dark">
                  • {item.description} ({item.quantity} × {formatNumber(item.rate)})
                </span>
                <span class="text-emittiv-light">
                  {formatCurrency(item.amount, currency)}
                </span>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Reimbursable Costs -->
        {#if costs.length > 0}
          <div class="space-y-2 mb-4">
            <div class="flex items-center justify-between">
              <span class="text-emittiv-light">Reimbursable Costs</span>
              <span class="text-emittiv-white font-medium">
                {formatCurrency(pricing.costs_total, currency)}
              </span>
            </div>
            {#each costs as cost}
              <div class="flex items-center justify-between pl-4 text-sm">
                <span class="text-emittiv-dark">
                  • {cost.description} ({formatNumber(cost.base_cost)} + {formatPercent(cost.markup_percent)})
                </span>
                <span class="text-emittiv-light">
                  {formatCurrency(cost.cost_to_client, currency)}
                </span>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Totals -->
      <div class="border-t border-emittiv-dark pt-4 space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-emittiv-light">SUBTOTAL</span>
          <span class="text-emittiv-white font-medium">
            {formatCurrency(pricing.subtotal, currency)}
          </span>
        </div>
        <div class="flex items-center justify-between text-sm">
          <span class="text-emittiv-dark">VAT ({formatPercent(pricing.config.vat_percent)})</span>
          <span class="text-emittiv-light">
            {formatCurrency(pricing.vat_amount, currency)}
          </span>
        </div>
        <div class="border-t border-emittiv-dark pt-2 flex items-center justify-between">
          <span class="text-emittiv-white font-bold text-lg">GRAND TOTAL</span>
          <span class="text-emittiv-splash font-bold text-xl">
            {formatCurrency(pricing.grand_total, currency)}
          </span>
        </div>
      </div>

      <!-- Mobilisation -->
      <div class="bg-emittiv-black/30 rounded-lg p-4">
        <div class="flex items-center justify-between">
          <span class="text-emittiv-light">
            MOBILISATION ({formatPercent(pricing.config.mobilisation_percent)})
          </span>
          <span class="text-emittiv-splash font-bold">
            {formatCurrency(pricing.grand_total * (pricing.config.mobilisation_percent / 100), currency)}
          </span>
        </div>
      </div>

      <!-- Payment Status -->
      {#if paymentSchedule && paymentSchedule.entries.length > 0}
        <div class="border-t border-emittiv-dark pt-4">
          <h4 class="text-emittiv-white font-medium mb-3">Payment Status</h4>
          <div class="grid grid-cols-3 gap-4">
            <div class="text-center">
              <div class="text-emittiv-light text-sm mb-1">Pending</div>
              <div class="text-emittiv-white font-medium">
                {formatCurrency(
                  paymentSchedule.entries
                    .filter(e => e.status === 'pending')
                    .reduce((sum, e) => sum + e.amount, 0),
                  currency
                )}
              </div>
            </div>
            <div class="text-center">
              <div class="text-yellow-500 text-sm mb-1">Invoiced</div>
              <div class="text-yellow-500 font-medium">
                {formatCurrency(paymentSchedule.total_invoiced, currency)}
              </div>
            </div>
            <div class="text-center">
              <div class="text-green-500 text-sm mb-1">Paid</div>
              <div class="text-green-500 font-medium">
                {formatCurrency(paymentSchedule.total_paid, currency)}
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
