<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import { getWinRatioReport } from '$lib/api/system';
  import { logApiError } from '$lib/services/logger';
  import { onMount } from 'svelte';
  import type { CurrencyAmount, WinRatioReport } from '../types';

  let report = $state<WinRatioReport | null>(null);
  let isLoading = $state(true);
  // Distinguishes "genuinely no data" from "failed to load" - same pattern
  // as Dashboard's statsError / Companies' loadError.
  let loadError = $state('');

  type SortKey = 'decided_count' | 'win_ratio' | 'company_name';
  let sortKey = $state<SortKey>('decided_count');
  let sortDesc = $state(true);

  async function load() {
    isLoading = true;
    loadError = '';
    try {
      report = await getWinRatioReport();
    } catch (error) {
      logApiError('load win-ratio report', error as Error);
      loadError = 'Failed to load the win-ratio report - the figures below may be stale or wrong.';
    } finally {
      isLoading = false;
    }
  }

  onMount(load);

  const sortedClients = $derived.by(() => {
    const clients = report?.clients ?? [];
    const sorted = [...clients].sort((a, b) => {
      let cmp: number;
      if (sortKey === 'company_name') {
        cmp = a.company_name.localeCompare(b.company_name);
      } else if (sortKey === 'win_ratio') {
        // Clients with no decided proposals yet (null ratio) sort last
        // regardless of direction - there is nothing to rank them on.
        if (a.win_ratio === null && b.win_ratio === null) cmp = 0;
        else if (a.win_ratio === null) return 1;
        else if (b.win_ratio === null) return -1;
        else cmp = a.win_ratio - b.win_ratio;
      } else {
        cmp = a.decided_count - b.decided_count;
      }
      return sortDesc ? -cmp : cmp;
    });
    return sorted;
  });

  function setSort(key: SortKey) {
    if (sortKey === key) {
      sortDesc = !sortDesc;
    } else {
      sortKey = key;
      // Numeric columns (decided count, win ratio) start high-to-low - the
      // biggest number is usually what you want first. The name column
      // starts A-Z, matching how every other list in this app sorts text.
      sortDesc = key !== 'company_name';
    }
  }

  // Overall figures across every client, for the summary row.
  const totals = $derived.by(() => {
    const clients = report?.clients ?? [];
    const won = clients.reduce((sum, c) => sum + c.won_count, 0);
    const lost = clients.reduce((sum, c) => sum + c.lost_count, 0);
    const noResponse = clients.reduce((sum, c) => sum + c.no_response_count, 0);
    const decided = won + lost + noResponse;
    const winRatio = decided > 0 ? won / decided : null;
    const wonValue = mergeCurrencyAmounts(clients.flatMap(c => c.won_value));
    return { won, decided, winRatio, wonValue };
  });

  function mergeCurrencyAmounts(amounts: CurrencyAmount[]): CurrencyAmount[] {
    const byCurrency = new Map<string, number>();
    for (const { currency, amount } of amounts) {
      byCurrency.set(currency, (byCurrency.get(currency) ?? 0) + amount);
    }
    return [...byCurrency.entries()]
      .map(([currency, amount]) => ({ currency, amount }))
      .sort((a, b) => a.currency.localeCompare(b.currency));
  }

  function formatRatio(ratio: number | null): string {
    return ratio === null ? '—' : `${Math.round(ratio * 100)}%`;
  }

  function formatMoney(amounts: CurrencyAmount[]): string {
    if (amounts.length === 0) return '—';
    // Deliberately not Intl.NumberFormat's `style: 'currency'` - it inserts
    // a locale-dependent non-breaking space between code and amount, and
    // rejects currency codes it doesn't recognise. Grouped digits plus a
    // plain ISO code is unambiguous for every value this report can produce
    // (`pricing.config.currency` is free text, not validated against a
    // fixed list - see PricingConfig).
    return amounts
      .map(({ currency, amount }) => `${currency} ${Math.round(amount).toLocaleString('en-US')}`)
      .join(' · ');
  }

  function sortIndicator(key: SortKey): string {
    if (sortKey !== key) return '';
    return sortDesc ? '▼' : '▲';
  }
</script>

<div class="p-8 win-ratio-page">
  <div class="win-ratio-header">
    <h1 class="win-ratio-title">Client Win Ratio</h1>
    <p class="win-ratio-subtitle">
      How often a proposal to each client turns into a won project, and what that's worth.
    </p>
  </div>

  {#if loadError}
    <div class="emittiv-alert emittiv-alert--error" style="margin-bottom: 12px;">
      {loadError}
      <button type="button" class="emittiv-link" onclick={load} style="margin-left: 8px;">
        Retry
      </button>
    </div>
  {:else if isLoading}
    <div class="flex flex-col items-center justify-center py-12">
      <div class="emittiv-spinner emittiv-spinner--page"></div>
      <p class="text-emittiv-light text-sm">Loading win-ratio report...</p>
    </div>
  {:else if !report || report.clients.length === 0}
    <div class="text-center py-12">
      <h3 class="text-lg font-medium text-emittiv-light mb-2">No decided proposals yet</h3>
      <p class="text-emittiv-light opacity-60">
        Win ratios appear once a proposal's outcome (Won, Lost, or No Response) is recorded on its
        project.
      </p>
    </div>
  {:else}
    <div class="summary-grid">
      <Card padding="p-5">
        <div class="summary-label">Overall win ratio</div>
        <div class="summary-value">{formatRatio(totals.winRatio)}</div>
        <div class="summary-detail">{totals.won} won of {totals.decided} decided</div>
      </Card>
      <Card padding="p-5">
        <div class="summary-label">Total won value</div>
        <div class="summary-value">{formatMoney(totals.wonValue)}</div>
        <div class="summary-detail">Sum of quoted fee on won projects, by currency</div>
      </Card>
      <Card padding="p-5">
        <div class="summary-label">Clients tracked</div>
        <div class="summary-value">{report.clients.length}</div>
        <div class="summary-detail">
          {#if report.unattributed_decided_count > 0}
            {report.unattributed_decided_count} decided project{report.unattributed_decided_count ===
            1
              ? ''
              : 's'} not linked to a fee - excluded above
          {:else}
            Every decided project links back to a client
          {/if}
        </div>
      </Card>
    </div>

    <div class="win-ratio-table-wrapper">
      <table class="win-ratio-table">
        <thead>
          <tr>
            <th>
              <button class="sort-header" onclick={() => setSort('company_name')}>
                Client {sortIndicator('company_name')}
              </button>
            </th>
            <th class="num">Won</th>
            <th class="num">Lost</th>
            <th class="num">No Response</th>
            <th class="num">Cancelled</th>
            <th class="num">Pending</th>
            <th class="num">
              <button class="sort-header" onclick={() => setSort('decided_count')}>
                Decided {sortIndicator('decided_count')}
              </button>
            </th>
            <th class="num">
              <button class="sort-header" onclick={() => setSort('win_ratio')}>
                Win Ratio {sortIndicator('win_ratio')}
              </button>
            </th>
            <th class="num">Won Value</th>
            <th class="num">Lost Value</th>
          </tr>
        </thead>
        <tbody>
          {#each sortedClients as client (client.company_id)}
            <tr>
              <td class="client-name">{client.company_name}</td>
              <td class="num">{client.won_count}</td>
              <td class="num">{client.lost_count}</td>
              <td class="num">{client.no_response_count}</td>
              <td class="num">{client.cancelled_count}</td>
              <td class="num">{client.pending_count}</td>
              <td class="num">{client.decided_count}</td>
              <td class="num">
                {#if client.win_ratio !== null}
                  <span
                    class="emittiv-badge {client.win_ratio >= 0.5
                      ? 'emittiv-badge--green'
                      : 'emittiv-badge--gray'}"
                  >
                    {formatRatio(client.win_ratio)}
                  </span>
                {:else}
                  —
                {/if}
              </td>
              <td class="num">{formatMoney(client.won_value)}</td>
              <td class="num">{formatMoney(client.lost_value)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .win-ratio-header {
    margin-bottom: 24px;
  }

  .win-ratio-title {
    font-family: 'Ubuntu', sans-serif;
    font-size: 24px;
    font-weight: 600;
    color: var(--emittiv-white);
    margin: 0 0 4px 0;
  }

  .win-ratio-subtitle {
    color: var(--emittiv-light);
    font-size: 14px;
    margin: 0;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
  }

  .summary-label {
    color: var(--emittiv-light);
    font-size: 13px;
    margin-bottom: 6px;
  }

  .summary-value {
    color: var(--emittiv-white);
    font-size: 26px;
    font-weight: 600;
    font-family: 'Ubuntu', sans-serif;
  }

  .summary-detail {
    color: var(--emittiv-light);
    font-size: 12px;
    margin-top: 6px;
    opacity: 0.8;
  }

  .win-ratio-table-wrapper {
    overflow-x: auto;
    border: 1px solid var(--emittiv-dark);
    border-radius: 8px;
  }

  .win-ratio-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .win-ratio-table th,
  .win-ratio-table td {
    padding: 10px 14px;
    text-align: left;
    white-space: nowrap;
  }

  .win-ratio-table thead th {
    background-color: var(--emittiv-darker);
    color: var(--emittiv-lighter);
    font-weight: 500;
    font-size: 12px;
    border-bottom: 1px solid var(--emittiv-dark);
  }

  .win-ratio-table tbody tr {
    border-bottom: 1px solid var(--emittiv-darker);
    transition: background-color 300ms cubic-bezier(0.4, 0, 0.2, 1);
  }

  .win-ratio-table tbody tr:hover {
    background-color: var(--emittiv-darker);
  }

  .win-ratio-table tbody tr:last-child {
    border-bottom: none;
  }

  .win-ratio-table .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
    color: var(--emittiv-lighter);
  }

  .client-name {
    color: var(--emittiv-white);
    font-weight: 500;
  }

  .sort-header {
    background: none;
    border: none;
    color: inherit;
    font: inherit;
    cursor: pointer;
    padding: 0;
  }

  .sort-header:hover {
    color: var(--emittiv-splash);
  }
</style>
