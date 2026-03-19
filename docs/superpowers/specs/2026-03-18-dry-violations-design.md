# DRY Violations Fix — Design Spec

**Date:** 2026-03-18
**Status:** Approved (revised after spec review)
**Scope:** Two independent workstreams — GenericDetailPage extraction + pricing utilities module

---

## Problem

Two areas of significant code duplication:

1. **Detail pages**: `ProjectDetailPage.svelte` (198 lines) and `ProposalDetailPage.svelte` (223 lines) are 92% identical. Only ProposalDetailPage has edit modal wiring — ProjectDetailPage should have it too.

2. **Pricing utilities**: `getRoundedStageTotal()`, `whtTooltip()`, WHT gross-up formula, and rounding config extraction duplicated across PricingCalculatorPanel, PaymentSchedulePanel, and PricingSummaryBar with behavioral drift already present.

---

## Workstream A: GenericDetailPage Component

### Design

Create `src/lib/components/GenericDetailPage.svelte` that handles the shared detail page scaffold:
- Slide-in panel animation
- Loading/error states with back navigation
- Backdrop click-to-close
- Entity lookup from store by route param ID
- Edit modal wiring (universal — both entities get edit support)
- Close handler with router navigation (`push()` from svelte-spa-router)

### Props

```typescript
{
  entityType: string;           // "Project" | "Proposal" — for display strings
  store: Writable<T[]>;        // Entity store — accessed via $store syntax inside component
  backRoute: string;            // "/projects" | "/proposals"
  DetailComponent: Component;   // ProjectDetail / ProposalDetail (Svelte 5 Component type)
  ModalComponent: Component;    // ProjectModal / ProposalModal
  entityProps: (entity: T) => Record<string, unknown>;  // Builds props for DetailComponent
  idExtractor: (entity: T) => string;  // Match entity by route param
}
```

**Component rendering:** Uses `<svelte:component this={DetailComponent} {...entityProps(entity)} />` pattern. The `entityProps` function lets each caller map the entity to the correct prop name (`{ project: entity }` vs `{ proposal: entity, onedit: handler }`).

**Router awareness:** GenericDetailPage owns the `push()` import and `window.location`/`window.history` usage directly. It is router-aware by design — not a generic UI component, but a route-level scaffold.

**Dead code cleanup:** `handleProjectUpdate()` in ProjectDetailPage (never wired to anything) and unused `location` import in both pages will be dropped.

### What gets eliminated

- ~200 lines of duplicated scaffold (loading, onMount, handleClose, backdrop, 62-line identical CSS block)
- Both route files become thin wrappers: `<GenericDetailPage>` with entity-specific props
- ProjectDetailPage gains edit capability it was missing

### Files affected

- **Create:** `src/lib/components/GenericDetailPage.svelte`
- **Rewrite:** `src/routes/ProjectDetailPage.svelte` (thin wrapper)
- **Rewrite:** `src/routes/ProposalDetailPage.svelte` (thin wrapper)

---

## Workstream B: Pricing Utilities Module

### Design

Create `src/lib/utils/pricingUtils.ts` with shared pricing functions, each with unit tests.

### Functions

**`getRoundingConfig(config)`**
- Extracts `{ increment, mode }` with defaults (`50`, `'ceiling'`)
- Replaces 6+ inline repetitions of `config?.rounding_increment ?? 50` / `config?.rounding_mode ?? 'ceiling'`

**`roundWithConfig(rawTotal, config)`**
- Applies rounding using extracted config: calls existing `roundToIncrement(rawTotal, increment, mode)`
- Pure function — no override awareness. Override logic stays in the component that owns the override state.
- PricingCalculatorPanel continues to check `stageTotalOverrides` locally before calling this.
- PaymentSchedulePanel uses this for cell-based totals (correct for its purpose — payment amounts come from cells, not display overrides).

**`calcWhtAmounts(amount, whtRate)`**
- Pure function returning **rounded integers**: `{ invoiced: Math.round(amount / (1 - whtRate)), wht: Math.round(invoiced - amount) }`
- Replaces 3 independent implementations: PricingCalculatorPanel's `grossUp()`, PaymentSchedulePanel's inline formula, PricingSummaryBar's template expression
- Returns rounded integers because all 3 current call sites apply `Math.round`

**`whtTooltip(amount, config, formatNumber)`**
- Returns formatted tooltip string when WHT is active (`config.vat_percent > 0`)
- Returns empty string when WHT is off — per user decision, no percentage fallback
- Single source of truth for the tooltip format
- **UX note:** PaymentSchedulePanel currently shows "X.XX% of total" on hover for non-WHT rows. This will become blank. This is a deliberate change — verify via smoke test.

### Override architecture (clarified)

`stageTotalOverrides` is local `$state` inside PricingCalculatorPanel. It is NOT lifted or shared. Each panel's rounding works differently by design:

- **PricingCalculatorPanel:** checks `stageTotalOverrides` → falls back to `roundWithConfig(getStageTotal(stageId), config)`
- **PaymentSchedulePanel:** uses `roundWithConfig(cellBasedTotal, config)` directly — no overrides (payment amounts reflect actual cell values)

The shared utility is the rounding logic, not the override-resolution logic.

### What gets eliminated

- ~40 lines of duplicated rounding/WHT logic across 3 files
- Rounding config extraction deduplicated (6+ inline repetitions → 1 function)
- WHT gross-up formula consolidated (3 implementations → 1)
- `whtTooltip` format string unified (2 diverged implementations → 1)

### Files affected

- **Create:** `src/lib/utils/pricingUtils.ts`
- **Create:** `src/lib/utils/pricingUtils.test.ts`
- **Modify:** `src/lib/components/pricing/PricingCalculatorPanel.svelte` — replace local rounding/WHT functions with imports
- **Modify:** `src/lib/components/pricing/PaymentSchedulePanel.svelte` — replace local rounding/WHT functions with imports
- **Modify:** `src/lib/components/pricing/PricingSummaryBar.svelte` — replace inline WHT calc with import

---

## Testing Strategy

- **Workstream A:** No new unit tests — GenericDetailPage is a UI scaffold with logic identical to existing tested pages. The component passes through to existing DetailComponent/ModalComponent which have their own tests. Smoke test detail page navigation after merge.
- **Workstream B:** Full TDD — tests for `getRoundingConfig`, `roundWithConfig`, `calcWhtAmounts`, `whtTooltip` before implementation

---

## Out of Scope

- Refactoring the pricing panels themselves (only extracting shared utilities)
- Lifting `stageTotalOverrides` to parent scope (overrides stay local to PricingCalculatorPanel)
- Adding edit modals for Companies/Contacts detail pages (they don't have detail pages yet)
- Changing WHT calculation logic (only consolidating existing implementations)
- Optimistic update refactoring in crud.ts (deferred from god function splits)

---

## Decision Log

| Decision | Rationale |
|----------|-----------|
| Edit support on all detail pages | All records should be editable from their detail view |
| Override logic stays local to PricingCalculatorPanel | `stageTotalOverrides` is component-local state — lifting it would expand scope. PaymentSchedulePanel correctly uses cell-based totals for payment amounts. |
| `calcWhtAmounts` returns rounded integers | All 3 current call sites use `Math.round` — embedding it prevents caller-side divergence |
| Empty string for non-WHT tooltip | WHT is rare (1 project), tooltip should only appear when relevant. PaymentSchedulePanel's percentage tooltip is a deliberate UX change — smoke test after merge. |
| Separate workstreams | Independent changes, can be committed/reviewed separately |
| `entityProps` function for component rendering | Solves the prop-name mismatch (`project` vs `proposal`) without string indirection |
