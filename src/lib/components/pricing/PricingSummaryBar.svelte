<script lang="ts">
  import type { PricingBreakdown } from '../../../types/database';
  import { formatNumber } from '$lib/utils/format';

  interface Props {
    pricing: PricingBreakdown | null;
    onExpand?: () => void;
  }

  let { pricing, onExpand }: Props = $props();

  // Computed values with fallbacks
  const targetFee = $derived(pricing?.config?.target_fee ?? 0);
  const quotedFee = $derived(pricing?.config?.quoted_fee ?? 0);
  const designTotal = $derived(pricing?.design_phase_total ?? 0);
  const postContractTotal = $derived(pricing?.post_contract_total ?? 0);
  const costsTotal = $derived(pricing?.costs_total ?? 0);
  const vatAmount = $derived(pricing?.vat_amount ?? 0);
  const grandTotal = $derived(pricing?.grand_total ?? 0);
  const currency = $derived(pricing?.config?.currency ?? 'AED');

  // Check if we have any pricing data
  const hasPricing = $derived(pricing !== null && targetFee > 0);
</script>

{#if hasPricing}
  <button
    type="button"
    class="emittiv-summary-bar"
    onclick={onExpand}
    title="Click to view full breakdown"
  >
    <div class="emittiv-summary-bar__items">
      <span class="emittiv-summary-bar__currency">{currency}</span>

      <div class="emittiv-summary-bar__item">
        <span class="emittiv-summary-bar__label">Target</span>
        <span class="emittiv-summary-bar__value">{formatNumber(targetFee)}</span>
      </div>

      <div class="emittiv-summary-bar__sep"></div>

      <div class="emittiv-summary-bar__item">
        <span class="emittiv-summary-bar__label">Quoted</span>
        <span class="emittiv-summary-bar__value emittiv-summary-bar__value--accent">{formatNumber(quotedFee)}</span>
      </div>

      {#if designTotal > 0}
        <div class="emittiv-summary-bar__sep"></div>
        <div class="emittiv-summary-bar__item">
          <span class="emittiv-summary-bar__label">Design</span>
          <span class="emittiv-summary-bar__value">{formatNumber(Math.round(designTotal))}</span>
        </div>
      {/if}

      {#if postContractTotal > 0}
        <div class="emittiv-summary-bar__sep"></div>
        <div class="emittiv-summary-bar__item">
          <span class="emittiv-summary-bar__label">Post-Contract</span>
          <span class="emittiv-summary-bar__value">{formatNumber(Math.round(postContractTotal))}</span>
        </div>
      {/if}

      {#if costsTotal > 0}
        <div class="emittiv-summary-bar__sep"></div>
        <div class="emittiv-summary-bar__item">
          <span class="emittiv-summary-bar__label">Costs</span>
          <span class="emittiv-summary-bar__value">{formatNumber(Math.round(costsTotal))}</span>
        </div>
      {/if}

      {#if vatAmount > 0}
        <div class="emittiv-summary-bar__sep"></div>
        <div class="emittiv-summary-bar__item">
          <span class="emittiv-summary-bar__label">VAT</span>
          <span class="emittiv-summary-bar__value">{formatNumber(Math.round(vatAmount))}</span>
        </div>
      {/if}
    </div>

    <div class="emittiv-summary-bar__total">
      <span class="emittiv-summary-bar__label">TOTAL</span>
      <span class="emittiv-summary-bar__value emittiv-summary-bar__value--accent emittiv-summary-bar__value--bold">{formatNumber(Math.round(grandTotal))} {currency}</span>
    </div>
  </button>
{:else}
  <div class="emittiv-summary-bar emittiv-summary-bar--empty">
    <span class="text-emittiv-light text-sm">No pricing data - configure target fee to begin</span>
  </div>
{/if}
