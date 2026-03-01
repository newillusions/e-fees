<script lang="ts">
  import type { PricingConfig } from '../../types/database';
  import { convertToClientCurrency } from '../../types/database';
  import { formatCurrency } from '$lib/utils/format';

  interface Props {
    amount: number;
    config: PricingConfig | undefined;
    level?: 'summary' | 'line';
  }

  let { amount, config, level = 'summary' }: Props = $props();

  // Mirror the pattern from PricingSummaryBar.svelte:36-49
  const currency = $derived(config?.currency ?? 'AED');
  const clientCurrency = $derived(config?.client_currency);
  const quoteCurrency = $derived(config?.quote_currency ?? currency);
  const isQuotingInClient = $derived(
    !!clientCurrency && clientCurrency !== currency && quoteCurrency === clientCurrency
  );

  // Convert the amount
  const convertedAmount = $derived(
    config ? convertToClientCurrency(amount, config) : undefined
  );

  // Determine what to display based on level and quoting direction
  const showConverted = $derived(level === 'summary' && isQuotingInClient && convertedAmount !== undefined);
  const displayAmount = $derived(showConverted ? convertedAmount! : amount);
  const displayCurrency = $derived(showConverted ? clientCurrency! : currency);

  // Tooltip: show the "other" currency
  const hasTooltip = $derived(
    !!clientCurrency && clientCurrency !== currency && convertedAmount !== undefined
  );
  const tooltipText = $derived(() => {
    if (!hasTooltip || !config?.exchange_rate) return '';
    if (showConverted) {
      // Showing client currency — tooltip shows base
      return `${formatCurrency(amount, currency)} @ ${config.exchange_rate} ${clientCurrency}/${currency}`;
    } else {
      // Showing base currency — tooltip shows client equivalent
      return `${formatCurrency(convertedAmount!, clientCurrency!)} @ ${config.exchange_rate} ${clientCurrency}/${currency}`;
    }
  });
</script>

{#if hasTooltip}
  <span class="emittiv-currency-tooltip" data-tooltip={tooltipText()}>
    {formatCurrency(displayAmount, displayCurrency)}
  </span>
{:else}
  <span>{formatCurrency(displayAmount, displayCurrency)}</span>
{/if}
