# Multi-Currency Display Completion — Design

**Date**: 2026-03-02
**Status**: Approved
**Scope**: Frontend display only — no backend changes

## Context

Multi-currency infrastructure is 70% built:
- Exchange rate service (Frankfurter/ECB) with caching
- `PricingConfig` fields: `client_currency`, `exchange_rate`, `quote_currency`, `rate_locked_at`
- Bidirectional quoting logic in `PricingSummaryBar` (`isQuotingInClient`, primary/secondary swap)
- `convertToClientCurrency()` utility
- Rate lock/unlock mechanism
- Currency formatting via `formatCurrency()`

**What's missing**: The `quote_currency` toggle only affects the summary bar. Summary panel, cards, dashboard, and modal all still hardcode AED.

## Design

### New Component: `CurrencyAmount.svelte`

Reusable wrapper that respects the fee's currency config:

```svelte
<CurrencyAmount amount={105000} config={fee.pricing?.config} level="summary" />
```

**Props:**
- `amount: number` — the base currency (AED) amount
- `config: PricingConfig | undefined` — from fee.pricing.config
- `level: 'summary' | 'line'` — summary amounts convert when quoting in client currency; line items always show base

**Behavior:**
- When `config.quote_currency === config.client_currency` AND level is `summary`:
  - Displays converted amount in client currency
  - Styled tooltip on hover shows AED equivalent + exchange rate
- When no client currency or level is `line`:
  - Displays base currency amount, no tooltip
- When client currency set but quoting in base:
  - Displays base currency amount
  - Tooltip shows client currency equivalent

### Styled Tooltip: `.emittiv-currency-tooltip`

CSS-only tooltip using `data-tooltip` attribute + `::after` pseudo-element:
- Dark background (`var(--emittiv-black)`), border (`var(--emittiv-dark)`)
- Positioned below the amount
- Shows: "AED 105,000 @ 1.0025 SAR/AED"
- 300ms fade-in transition matching emittiv design system
- No JS tooltip library

### Changes by Surface

#### 1. PricingSummaryPanel
**Current**: All amounts hardcoded to `formatCurrency(amount, currency)`. Single "Converted" row at bottom.
**After**: Summary rows (design phase total, post-contract total, costs total, subtotal, VAT, grand total, mobilisation) use `<CurrencyAmount level="summary">`. Line items (individual disciplines, individual post-contract items, individual costs) use `<CurrencyAmount level="line">`. Remove the standalone "Converted" row at bottom — conversion is now inline.

#### 2. PricingSummaryBar
**Current**: Custom primary/secondary swap logic inline.
**After**: Refactor total display to use `<CurrencyAmount>` for consistency. Keep the inline secondary display in parens for the compact bar format.

#### 3. PricingCalculatorPanel
**Current**: "Converted to X" row at bottom.
**After**: When quoting in client currency, the quoted fee display shows client currency with AED tooltip. Grid cells always show base currency. The "Converted to X" section becomes redundant when quoting in client — hide it.

#### 4. ProposalCard
**Current**: No fee amounts displayed.
**After**: Add quoted fee in the extra slot using `<CurrencyAmount>`. Shows "SAR 105,263" or "AED 105,000" depending on quote_currency. Falls back gracefully when fee has no pricing data.

#### 5. PendingProposals (dashboard)
**Current**: No amounts.
**After**: Add fee amount per proposal item. Uses `<CurrencyAmount>` with the fee's pricing config.

#### 6. ProposalModal
**Current**: Shows "Pricing configured: AED X" as plain text.
**After**: Use `<CurrencyAmount>` so it respects quote_currency.

### What Always Stays in Base Currency

- Discipline x Stage grid cells (internal calculation detail)
- Target fee / buffer (internal, not client-facing)
- Per-unit rates on post-contract items
- Payment schedule entries (invoiced in base currency per contract)

### Data Flow

1. User sets `client_currency` in pricing calculator (existing)
2. User sets `exchange_rate` manually or clicks "Use Live" (existing)
3. User toggles `quote_currency` dropdown: base vs client (existing)
4. All `<CurrencyAmount level="summary">` components reactively display in the selected quote currency
5. Hover reveals the other currency via styled tooltip

### No Backend Changes

All fields (`quote_currency`, `client_currency`, `exchange_rate`, `rate_locked_at`) already exist in `PricingConfig` and are persisted to SurrealDB. This is purely frontend display logic.

### Files to Create

| File | Purpose |
|------|---------|
| `src/lib/components/CurrencyAmount.svelte` | Reusable currency display component |

### Files to Modify

| File | Change |
|------|--------|
| `src/styles/app.css` | Add `.emittiv-currency-tooltip` styles |
| `src/lib/components/pricing/PricingSummaryPanel.svelte` | Replace `formatCurrency()` calls with `<CurrencyAmount>` on summary rows |
| `src/lib/components/pricing/PricingCalculatorPanel.svelte` | Use `<CurrencyAmount>` for quoted fee display, hide redundant converted row |
| `src/lib/components/ProposalCard.svelte` | Add quoted fee amount display |
| `src/lib/components/dashboard/PendingProposals.svelte` | Add fee amount per item |
| `src/lib/components/ProposalModal.svelte` | Replace plain text pricing display |

### Estimated Scope

~200-300 lines of new/changed code. One new component, one new CSS class, five file modifications.
