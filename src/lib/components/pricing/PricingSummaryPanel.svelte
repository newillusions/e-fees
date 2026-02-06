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
  import PanelCard from '../PanelCard.svelte';

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

<PanelCard title="Fee Summary">
  {#snippet headerActions()}
    {#if onClose}
      <IconButton icon="x" variant="secondary" size="sm" on:click={onClose} />
    {/if}
  {/snippet}

  {#if !pricing}
    <div class="p-8 text-center">
      <p class="text-emittiv-light text-sm">No pricing data available.</p>
    </div>
  {:else}
    <div class="p-4 space-y-6">
      <!-- Target / Buffer / Quoted -->
      <div class="space-y-2">
        <div class="emittiv-summary-line">
          <span class="text-emittiv-light">TARGET FEE</span>
          <span class="text-emittiv-white font-medium">
            {formatCurrency(pricing.config.target_fee, currency)}
          </span>
        </div>
        <div class="emittiv-summary-line emittiv-summary-line--sub">
          <span class="text-emittiv-dark">Buffer ({formatPercent(pricing.config.buffer_percent)})</span>
          <span class="text-emittiv-light">
            +{formatCurrency(pricing.config.quoted_fee - pricing.config.target_fee, currency)}
          </span>
        </div>
        <div class="emittiv-summary-line emittiv-summary-line--total">
          <span class="text-emittiv-white font-medium">QUOTED FEE</span>
          <span class="text-emittiv-splash font-bold text-lg">
            {formatCurrency(pricing.config.quoted_fee, currency)}
          </span>
        </div>
      </div>

      <!-- Fee Breakdown -->
      <div class="emittiv-summary-section">
        <h4 class="text-emittiv-white font-medium mb-3">Fee Breakdown</h4>

        <!-- Design Phase -->
        <div class="space-y-2 mb-4">
          <div class="emittiv-summary-line">
            <span class="text-emittiv-light">Design Phase</span>
            <span class="text-emittiv-white font-medium">
              {formatCurrency(pricing.design_phase_total, currency)}
            </span>
          </div>
          {#each disciplines as disc}
            <div class="emittiv-summary-line emittiv-summary-line--sub">
              <span class="text-emittiv-dark">{disc.name} ({formatPercent(disc.percentage)})</span>
              <span class="text-emittiv-light">
                {formatCurrency(getDisciplineTotal(disc.id), currency)}
              </span>
            </div>
          {/each}
        </div>

        <!-- Post-Contract Services -->
        {#if postContractItems.length > 0}
          <div class="space-y-2 mb-4">
            <div class="emittiv-summary-line">
              <span class="text-emittiv-light">Post-Contract Services</span>
              <span class="text-emittiv-white font-medium">
                {formatCurrency(pricing.post_contract_total, currency)}
              </span>
            </div>
            {#each postContractItems as item}
              <div class="emittiv-summary-line emittiv-summary-line--sub">
                <span class="text-emittiv-dark">
                  {item.description} ({item.quantity} x {formatNumber(item.rate)})
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
            <div class="emittiv-summary-line">
              <span class="text-emittiv-light">Reimbursable Costs</span>
              <span class="text-emittiv-white font-medium">
                {formatCurrency(pricing.costs_total, currency)}
              </span>
            </div>
            {#each costs as cost}
              <div class="emittiv-summary-line emittiv-summary-line--sub">
                <span class="text-emittiv-dark">
                  {cost.description} ({formatNumber(cost.base_cost)} + {formatPercent(cost.markup_percent)})
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
      <div class="emittiv-summary-section space-y-2">
        <div class="emittiv-summary-line">
          <span class="text-emittiv-light">SUBTOTAL</span>
          <span class="text-emittiv-white font-medium">
            {formatCurrency(pricing.subtotal, currency)}
          </span>
        </div>
        {#if pricing.config.tax_type === 'vat' && pricing.config.show_tax_in_summary}
          <div class="emittiv-summary-line emittiv-summary-line--sub">
            <span class="text-emittiv-dark">VAT ({formatPercent(pricing.config.vat_percent)})</span>
            <span class="text-emittiv-light">
              {formatCurrency(pricing.vat_amount, currency)}
            </span>
          </div>
        {/if}
        <div class="emittiv-summary-line emittiv-summary-line--total">
          <span class="text-emittiv-white font-bold text-lg">
            {pricing.config.tax_type === 'vat' && pricing.config.show_tax_in_summary ? 'GRAND TOTAL' : 'TOTAL'}
          </span>
          <span class="text-emittiv-splash font-bold text-xl">
            {formatCurrency(pricing.config.tax_type === 'vat' && pricing.config.show_tax_in_summary ? pricing.grand_total : pricing.subtotal, currency)}
          </span>
        </div>
        {#if pricing.config.tax_type === 'vat' && !pricing.config.show_tax_in_summary}
          <div class="text-emittiv-dark text-xs mt-1">
            VAT will be added at the prevailing rate ({pricing.config.vat_percent}%)
          </div>
        {/if}
        {#if pricing.config.tax_type === 'withholding'}
          <div class="text-emittiv-dark text-xs mt-1">
            Withholding tax ({pricing.config.vat_percent}%) applies — invoices will be grossed up so net receivable equals the quoted fee
          </div>
        {/if}
      </div>

      <!-- Mobilisation (calculated on subtotal, not including tax) -->
      <div class="emittiv-summary-highlight">
        <div class="emittiv-summary-line">
          <span class="text-emittiv-light">
            MOBILISATION ({formatPercent(pricing.config.mobilisation_percent)})
          </span>
          <span class="text-emittiv-splash font-bold">
            {formatCurrency(pricing.subtotal * (pricing.config.mobilisation_percent / 100), currency)}
          </span>
        </div>
      </div>

      <!-- Payment Status -->
      {#if paymentSchedule && paymentSchedule.entries.length > 0}
        <div class="emittiv-summary-section">
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
</PanelCard>
