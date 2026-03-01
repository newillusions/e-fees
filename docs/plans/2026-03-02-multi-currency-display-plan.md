# Multi-Currency Display Completion — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the existing `quote_currency` toggle affect all monetary displays across the app — summary panel, cards, dashboard, and modal — with styled tooltips showing the alternate currency on hover.

**Architecture:** Create a reusable `CurrencyAmount.svelte` component that encapsulates the existing `isQuotingInClient` pattern from `PricingSummaryBar`. Add a CSS-only tooltip class. Integrate the component into 5 existing surfaces. No backend changes.

**Tech Stack:** Svelte 5 (runes), TypeScript, CSS (emittiv design system), existing `convertToClientCurrency()` utility

---

### Task 1: Add `.emittiv-currency-tooltip` CSS Class

**Files:**
- Modify: `src/styles/app.css` (append to end of file)

**Step 1: Add the tooltip CSS**

Append this block to the end of `src/styles/app.css`:

```css
/* Currency Tooltip — CSS-only hover tooltip for multi-currency display */
.emittiv-currency-tooltip {
  position: relative;
  cursor: default;
  border-bottom: 1px dotted var(--emittiv-dark);
}

.emittiv-currency-tooltip::after {
  content: attr(data-tooltip);
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-top: 4px;
  padding: 4px 8px;
  background: var(--emittiv-black);
  border: 1px solid var(--emittiv-dark);
  border-radius: 4px;
  color: var(--emittiv-lighter);
  font-size: 11px;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity 300ms cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 50;
}

.emittiv-currency-tooltip:hover::after {
  opacity: 1;
}
```

**Step 2: Verify the styles load**

Run: `npm run dev` (start dev server if not running)
Expected: App loads without CSS errors. The class exists but isn't used yet.

**Step 3: Commit**

```bash
git add src/styles/app.css
git commit -m "style: add .emittiv-currency-tooltip CSS class for multi-currency hover"
```

---

### Task 2: Create `CurrencyAmount.svelte` Component

**Files:**
- Create: `src/lib/components/CurrencyAmount.svelte`

**Step 1: Create the component**

Create `src/lib/components/CurrencyAmount.svelte` with this content:

```svelte
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
```

**Step 2: Verify it compiles**

Run: `npm run check`
Expected: No type errors. The component is created but not imported anywhere yet.

**Step 3: Commit**

```bash
git add src/lib/components/CurrencyAmount.svelte
git commit -m "feat: add CurrencyAmount component for multi-currency display"
```

---

### Task 3: Update `PricingSummaryPanel` to Use `CurrencyAmount`

**Files:**
- Modify: `src/lib/components/pricing/PricingSummaryPanel.svelte`

This is the most impactful change. Summary-level amounts (design phase total, post-contract total, costs total, subtotal, VAT, grand total, mobilisation) will convert when quoting in client currency. Line items (individual disciplines, individual post-contract items, individual costs) always show base currency. Payment schedule entries always stay in base currency (contractual).

**Step 1: Add the import**

In `src/lib/components/pricing/PricingSummaryPanel.svelte`, add the import after line 11 (after the `import PanelCard` line):

```typescript
  import CurrencyAmount from '../CurrencyAmount.svelte';
```

**Step 2: Replace summary-level `formatCurrency` calls with `<CurrencyAmount>`**

Replace these specific lines (summary-level amounts only):

**Line 80** — TARGET FEE (summary):
```svelte
<!-- Before -->
            {formatCurrency(pricing.config.target_fee, currency)}
<!-- After -->
            <CurrencyAmount amount={pricing.config.target_fee} config={pricing.config} />
```

**Line 86** — Buffer amount (sub-line, keep base — it's an internal calculation detail):
No change. Buffer stays as `formatCurrency(...)`.

**Line 92** — QUOTED FEE (summary, prominent):
```svelte
<!-- Before -->
            {formatCurrency(pricing.config.quoted_fee, currency)}
<!-- After -->
            <CurrencyAmount amount={pricing.config.quoted_fee} config={pricing.config} />
```

**Line 106** — Design Phase total (summary):
```svelte
<!-- Before -->
              {formatCurrency(pricing.design_phase_total, currency)}
<!-- After -->
              <CurrencyAmount amount={pricing.design_phase_total} config={pricing.config} />
```

**Lines 113** — Individual discipline amounts (line items — keep base):
No change. These stay as `formatCurrency(getDisciplineTotal(disc.id), currency)`.

**Line 125** — Post-Contract Services total (summary):
```svelte
<!-- Before -->
                {formatCurrency(pricing.post_contract_total, currency)}
<!-- After -->
                <CurrencyAmount amount={pricing.post_contract_total} config={pricing.config} />
```

**Line 134** — Individual post-contract items (line items — keep base):
No change.

**Line 147** — Reimbursable Costs total (summary):
```svelte
<!-- Before -->
                {formatCurrency(pricing.costs_total, currency)}
<!-- After -->
                <CurrencyAmount amount={pricing.costs_total} config={pricing.config} />
```

**Line 156** — Individual cost items (line items — keep base):
No change.

**Line 169** — SUBTOTAL (summary):
```svelte
<!-- Before -->
            {formatCurrency(pricing.subtotal, currency)}
<!-- After -->
            <CurrencyAmount amount={pricing.subtotal} config={pricing.config} />
```

**Line 176** — VAT amount (summary):
```svelte
<!-- Before -->
              {formatCurrency(pricing.vat_amount, currency)}
<!-- After -->
              <CurrencyAmount amount={pricing.vat_amount} config={pricing.config} />
```

**Line 185** — GRAND TOTAL / TOTAL (summary):
```svelte
<!-- Before -->
            {formatCurrency(pricing.config.tax_type === 'vat' && pricing.config.show_tax_in_summary ? pricing.grand_total : pricing.subtotal, currency)}
<!-- After -->
            <CurrencyAmount amount={pricing.config.tax_type === 'vat' && pricing.config.show_tax_in_summary ? pricing.grand_total : pricing.subtotal} config={pricing.config} />
```

**Lines 188-204** — Remove the standalone "Converted" row at the bottom. This entire block becomes redundant because conversion is now inline via tooltips. Delete lines 188-204 (the `{#if hasConversion}...{/if}` block).

**Line 224** — Mobilisation (summary):
```svelte
<!-- Before -->
            {formatCurrency(pricing.subtotal * (pricing.config.mobilisation_percent / 100), currency)}
<!-- After -->
            <CurrencyAmount amount={pricing.subtotal * (pricing.config.mobilisation_percent / 100)} config={pricing.config} />
```

**Payment Schedule section (lines 237-254)** — Keep as `formatCurrency(...)`. Payment schedule amounts are in base currency per contract, not converted.

**Step 3: Remove now-unused helpers**

Remove the `conversionTitle()` function (lines 46-51) and its usages on lines 103 and 166 (the `title={conversionTitle(...)}` attributes). The tooltip is now built into `<CurrencyAmount>`.

Remove the `convertAmount()` function (lines 40-43) — no longer used.

Remove the `isRateLocked` derived (line 37) — only used in the deleted "Converted" row.

Keep `hasConversion` if used elsewhere, but check — after removing the "Converted" block, if `hasConversion` has no remaining references, remove it too (lines 34-36).

**Step 4: Verify it compiles and displays correctly**

Run: `npm run check`
Expected: No type errors.

Manually verify in app: Open a fee with multi-currency configured. Summary panel should show amounts in the quote currency. Hover should show tooltip with alternate currency and exchange rate.

**Step 5: Commit**

```bash
git add src/lib/components/pricing/PricingSummaryPanel.svelte
git commit -m "feat(pricing): use CurrencyAmount in summary panel for multi-currency display"
```

---

### Task 4: Update `PricingCalculatorPanel` Quoted Fee Display

**Files:**
- Modify: `src/lib/components/pricing/PricingCalculatorPanel.svelte`

The calculator has two changes: (1) the "Calculated Fee" display (line 415) should respect quote_currency, and (2) the "Converted Total" row (lines 618-626) should hide when quoting in client currency (it's redundant).

**Step 1: Add the import**

Add after the existing imports at the top of the file:

```typescript
  import CurrencyAmount from '../CurrencyAmount.svelte';
```

**Step 2: Replace the Calculated Fee display**

On line 415, replace:
```svelte
<!-- Before -->
        <span class="emittiv-calc-quoted">{formatCurrency(config.quoted_fee, config.currency)}</span>
<!-- After -->
        <span class="emittiv-calc-quoted"><CurrencyAmount amount={config.quoted_fee} config={config} /></span>
```

**Step 3: Conditionally hide the Converted Total row**

The "Converted Total" row (lines 618-626) is redundant when `quote_currency === client_currency` because the main display already shows the client currency. Add a condition:

```svelte
<!-- Before (line 619) -->
    {#if convertedTotal !== undefined && config.client_currency}
<!-- After -->
    {#if convertedTotal !== undefined && config.client_currency && (config.quote_currency ?? config.currency) !== config.client_currency}
```

This means: show the "Converted Total" row only when quoting in base currency (so the user can see the client currency equivalent). When quoting in client currency, the main display already shows it.

**Step 4: Verify**

Run: `npm run check`
Expected: No type errors.

Manually verify: Toggle the "Quote in" dropdown between AED and client currency. The Calculated Fee should swap currencies. The Converted Total row should appear/disappear based on the toggle.

**Step 5: Commit**

```bash
git add src/lib/components/pricing/PricingCalculatorPanel.svelte
git commit -m "feat(pricing): add multi-currency display to calculator quoted fee"
```

---

### Task 5: Add Quoted Fee Amount to `ProposalCard`

**Files:**
- Modify: `src/lib/components/ProposalCard.svelte`

Currently ProposalCard shows no amounts. Add the quoted fee amount in the extra slot, using `CurrencyAmount` so it respects the quote_currency.

**Step 1: Add the import**

After line 6 (`import type { Fee } from '../../types';`), add:

```typescript
  import CurrencyAmount from './CurrencyAmount.svelte';
```

**Step 2: Add the fee amount display**

In the `slot="extra"` section (lines 75-89), add the amount display before the existing package/metadata section. Insert after line 76 (`<div class="space-y-1">`):

```svelte
      {#if proposal.pricing?.grand_total}
        <div class="text-emittiv-splash font-bold text-sm">
          <CurrencyAmount
            amount={proposal.pricing.grand_total}
            config={proposal.pricing.config}
          />
        </div>
      {/if}
```

**Step 3: Verify**

Run: `npm run check`
Expected: No type errors.

Manually verify: Navigate to Proposals page. Cards with pricing data should show the fee amount in the correct quote currency. Hover should show the tooltip when multi-currency is configured.

**Step 4: Commit**

```bash
git add src/lib/components/ProposalCard.svelte
git commit -m "feat(proposals): display quoted fee amount on proposal cards"
```

---

### Task 6: Add Fee Amount to `PendingProposals` Dashboard

**Files:**
- Modify: `src/lib/components/dashboard/PendingProposals.svelte`

The dashboard's PendingProposals widget currently shows no amounts — only status, number, project, company, and timing. Add a fee amount column.

**Step 1: Add the import**

After line 6 (`import { push } from 'svelte-spa-router';`), add:

```typescript
  import CurrencyAmount from '$lib/components/CurrencyAmount.svelte';
```

**Step 2: Remove the hardcoded `formatCurrency` function**

Delete the local `formatCurrency` function on lines 24-31. It's hardcoded to AED and won't be used anymore. (If other parts of the same file still use it, keep it — but check first.)

**Step 3: Add the fee amount to each proposal item**

In the proposal item template, after the `proposal-details` div (line 105, after the closing `</div>` of `proposal-details`), add a fee amount section:

```svelte
            {#if fee.pricing?.grand_total}
              <div class="proposal-amount">
                <CurrencyAmount
                  amount={fee.pricing.grand_total}
                  config={fee.pricing.config}
                />
              </div>
            {/if}
```

**Step 4: Add minimal styling for the amount**

The `proposal-amount` class needs positioning within the grid. Check the existing CSS in the file's `<style>` block. Add within the component's `<style>` block:

```css
  .proposal-amount {
    font-weight: 600;
    color: var(--emittiv-splash);
    font-size: 12px;
    white-space: nowrap;
    display: flex;
    align-items: center;
  }
```

**Step 5: Verify**

Run: `npm run check`
Expected: No type errors.

Manually verify: Dashboard should show fee amounts on pending proposals. Multi-currency fees should display in the quote currency with hover tooltip.

**Step 6: Commit**

```bash
git add src/lib/components/dashboard/PendingProposals.svelte
git commit -m "feat(dashboard): display fee amounts on pending proposals"
```

---

### Task 7: Update `ProposalModal` Pricing Display

**Files:**
- Modify: `src/lib/components/ProposalModal.svelte`

The modal currently shows `Pricing configured: AED 105,000` as plain text (line 1146). Replace with `CurrencyAmount`.

**Step 1: Add the import**

Add after the existing component imports in the `<script>` block:

```typescript
  import CurrencyAmount from './CurrencyAmount.svelte';
```

**Step 2: Replace the plain text pricing display**

On line 1146, replace:

```svelte
<!-- Before -->
              Pricing configured: {proposal.pricing.config?.currency || 'AED'} {(proposal.pricing.grand_total || 0).toLocaleString()}
<!-- After -->
              Pricing configured: <CurrencyAmount amount={proposal.pricing.grand_total || 0} config={proposal.pricing.config} />
```

**Step 3: Verify**

Run: `npm run check`
Expected: No type errors.

Manually verify: Open a proposal in edit mode. The "Fee Pricing" section should show the amount in the correct quote currency with hover tooltip.

**Step 4: Commit**

```bash
git add src/lib/components/ProposalModal.svelte
git commit -m "feat(proposals): use CurrencyAmount in proposal modal pricing display"
```

---

### Task 8: Final Verification and Cleanup

**Step 1: Run full type check**

Run: `npm run check`
Expected: Zero errors.

**Step 2: Run Rust tests**

Run: `cd src-tauri && cargo test 2>&1 | tail -10`
Expected: All tests pass (no backend changes, but verify nothing broke).

**Step 3: Manual smoke test**

Test these scenarios in the running app:

1. **Fee without client_currency**: All amounts show in AED, no tooltips.
2. **Fee with client_currency + quoting in AED**: All summary amounts show AED, tooltips show client currency equivalent.
3. **Fee with client_currency + quoting in client currency**: All summary amounts show client currency, tooltips show AED equivalent and exchange rate.
4. **Toggle quote_currency dropdown**: Calculator, summary panel, cards, dashboard, and modal all update reactively.
5. **Line items**: Individual disciplines, post-contract items, and costs always show in base currency regardless of toggle.
6. **ProposalCard**: Shows fee amount, respects quote_currency.
7. **PendingProposals dashboard**: Shows fee amounts per item.
8. **ProposalModal**: Shows fee amount with correct currency.

**Step 4: Commit any final adjustments**

If any tweaks needed from smoke testing, commit them as a single fix commit.

**Step 5: Final commit (if all clean)**

```bash
git add -A
git commit -m "feat: complete multi-currency display across all surfaces

Extends the existing quote_currency toggle to affect all monetary
displays: summary panel, calculator, proposal cards, dashboard
pending proposals, and proposal modal. Adds styled CSS-only tooltips
showing the alternate currency on hover."
```
