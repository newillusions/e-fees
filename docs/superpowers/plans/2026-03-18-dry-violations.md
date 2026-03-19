# DRY Violations Fix — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate duplication between detail pages (extract GenericDetailPage) and pricing panels (extract pricingUtils), adding edit support to ProjectDetailPage and fixing WHT/rounding behavioral drift.

**Architecture:** Two independent workstreams. Workstream A extracts a GenericDetailPage scaffold component from 92%-identical ProjectDetailPage/ProposalDetailPage. Workstream B extracts shared rounding and WHT functions from 3 pricing panels into `pricingUtils.ts`. Both use re-exports for backward compat.

**Tech Stack:** TypeScript, Svelte 5, Vitest

**Spec:** `docs/superpowers/specs/2026-03-18-dry-violations-design.md`

---

## Pre-Flight

- [ ] `npm test` — 678 pass
- [ ] `npm run check` — 5 pre-existing errors only

---

## Task 1: Extract GenericDetailPage component

**Files:**
- Create: `src/lib/components/GenericDetailPage.svelte`
- Modify: `src/routes/ProjectDetailPage.svelte`
- Modify: `src/routes/ProposalDetailPage.svelte`

### Step 1: Create GenericDetailPage

- [ ] Create `src/lib/components/GenericDetailPage.svelte` that consolidates the shared scaffold from both detail pages:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { loadAllData } from '$lib/stores';
  import { findEntityById } from '$lib/utils';
  import { logApiError } from '$lib/services/logger';
  import { push } from 'svelte-spa-router';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import LoadingSkeleton from '$lib/components/LoadingSkeleton.svelte';
  import type { Writable } from 'svelte/store';

  let {
    params = { id: '' },
    entityType,
    store,
    backRoute,
    DetailComponent,
    ModalComponent,
    detailProps,
    modalProps,
  }: {
    params?: { id: string };
    entityType: string;
    store: Writable<any[]>;
    backRoute: string;
    DetailComponent: any;
    ModalComponent: any;
    detailProps: (entity: any, callbacks: { onedit: () => void; onclose: () => void }) => Record<string, unknown>;
    modalProps: (entity: any, callbacks: { onclose: () => void }) => Record<string, unknown>;
  } = $props();

  let previousPage = $state(backRoute);
  let entity: any = $state(null);
  let loading = $state(true);
  let error: string | null = $state(null);

  // Modal state
  let showModal = $state(false);
  let modalMode: 'create' | 'edit' = $state('edit');

  let entityId = $derived(params.id);

  $effect(() => {
    if ($store.length > 0 && entityId) {
      const found = findEntityById($store, entityId);
      if (found) {
        entity = found;
        loading = false;
        error = null;
      } else {
        entity = null;
        loading = false;
        error = `${entityType} with ID "${entityId}" not found`;
      }
    }
  });

  onMount(async () => {
    const urlParams = new URLSearchParams(window.location.search);
    const fromParam = urlParams.get('from');
    if (fromParam === 'dashboard') {
      previousPage = '/';
    } else if (window.history.length > 1) {
      previousPage = '/';
    }

    if ($store.length === 0) {
      try {
        await loadAllData();
      } catch (err) {
        logApiError(`load ${entityType.toLowerCase()}s`, err as Error);
        error = `Failed to load ${entityType.toLowerCase()} data`;
        loading = false;
      }
    } else {
      const found = findEntityById($store, entityId);
      if (found) {
        entity = found;
      } else {
        error = `${entityType} with ID "${entityId}" not found`;
      }
      loading = false;
    }
  });

  function handleClose() {
    if (window.history.length > 1) {
      window.history.back();
    } else {
      push(previousPage);
    }
  }

  function handleEdit() {
    modalMode = 'edit';
    showModal = true;
  }

  function handleModalClose() {
    showModal = false;
    loadAllData();
  }
</script>
```

Template (add after the script block):

```svelte
<!-- Backdrop -->
<div
  class="emittiv-backdrop emittiv-backdrop--blur"
  on:click={handleClose}
  on:keydown={(e) => e.key === 'Escape' && handleClose()}
  role="button"
  tabindex="-1"
  aria-label="Close detail view"
  in:fade={{ duration: 200 }}
  out:fade={{ duration: 200 }}
></div>

<!-- Sliding Panel -->
<div
  class="emittiv-detail-panel generic-detail-page"
  style="width: calc(100vw - 240px); left: 240px;"
  in:fly={{ x: '100%', duration: 300, easing: cubicOut }}
  out:fly={{ x: '100%', duration: 250, easing: cubicOut }}
>
  {#if loading}
    <div class="loading-container">
      <LoadingSkeleton rows={8} />
    </div>
  {:else if error}
    <div class="error-container">
      <div class="error-card">
        <h2>{entityType} Not Found</h2>
        <p>{error}</p>
        <button class="back-button" on:click={handleClose}>
          ← Back to {entityType}s
        </button>
      </div>
    </div>
  {:else if entity}
    <svelte:component
      this={DetailComponent}
      {...detailProps(entity, { onedit: handleEdit, onclose: handleClose })}
    />
  {/if}
</div>

<!-- Edit Modal -->
{#if showModal && entity}
  <svelte:component
    this={ModalComponent}
    {...modalProps(entity, { onclose: handleModalClose })}
  />
{/if}
```

CSS: the shared 62-line block from both pages (loading-container, error-container, error-card, back-button). Use class name `generic-detail-page` instead of entity-specific names.

### Step 2: Rewrite ProjectDetailPage as thin wrapper

- [ ] Rewrite `src/routes/ProjectDetailPage.svelte`:

```svelte
<script lang="ts">
  import GenericDetailPage from '$lib/components/GenericDetailPage.svelte';
  import { projectsStore } from '$lib/stores';
  import ProjectDetail from '$lib/components/ProjectDetail.svelte';
  import ProjectModal from '$lib/components/ProjectModal.svelte';

  let { params = { id: '' } }: { params?: { id: string } } = $props();
</script>

<GenericDetailPage
  {params}
  entityType="Project"
  store={projectsStore}
  backRoute="/projects"
  DetailComponent={ProjectDetail}
  ModalComponent={ProjectModal}
  detailProps={(entity, { onedit, onclose }) => ({ project: entity, isOpen: true, onedit, onclose })}
  modalProps={(entity, { onclose }) => ({ isOpen: true, project: entity, mode: 'edit', onclose })}
/>
```

Note: ProjectDetailPage now gains edit support it previously lacked.

### Step 3: Rewrite ProposalDetailPage as thin wrapper

- [ ] Rewrite `src/routes/ProposalDetailPage.svelte`:

```svelte
<script lang="ts">
  import GenericDetailPage from '$lib/components/GenericDetailPage.svelte';
  import { feesStore } from '$lib/stores';
  import ProposalDetail from '$lib/components/ProposalDetail.svelte';
  import ProposalModal from '$lib/components/ProposalModal.svelte';

  let { params = { id: '' } }: { params?: { id: string } } = $props();
</script>

<GenericDetailPage
  {params}
  entityType="Proposal"
  store={feesStore}
  backRoute="/proposals"
  DetailComponent={ProposalDetail}
  ModalComponent={ProposalModal}
  detailProps={(entity, { onedit, onclose }) => ({ proposal: entity, isOpen: true, onedit, onclose })}
  modalProps={(entity, { onclose }) => ({ isOpen: true, proposal: entity, mode: 'edit', onclose })}
/>
```

### Step 4: Verify and commit

- [ ] `npm test` — all 678+ pass
- [ ] `npm run check` — no new errors
- [ ] Commit: `refactor(ui): extract GenericDetailPage from duplicate detail pages`

---

## Task 2: Write failing tests for pricing utilities (RED)

**Files:**
- Create: `src/lib/utils/pricingUtils.test.ts`

### Step 1: Write tests

- [ ] Create `src/lib/utils/pricingUtils.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import { getRoundingConfig, roundWithConfig, calcWhtAmounts, whtTooltip } from './pricingUtils';

describe('getRoundingConfig', () => {
  it('extracts increment and mode from config', () => {
    const result = getRoundingConfig({ rounding_increment: 100, rounding_mode: 'nearest' });
    expect(result).toEqual({ increment: 100, mode: 'nearest' });
  });

  it('returns defaults when config is undefined', () => {
    const result = getRoundingConfig(undefined);
    expect(result).toEqual({ increment: 50, mode: 'ceiling' });
  });

  it('returns defaults when fields are missing', () => {
    const result = getRoundingConfig({});
    expect(result).toEqual({ increment: 50, mode: 'ceiling' });
  });
});

describe('roundWithConfig', () => {
  it('rounds up to increment with ceiling mode', () => {
    expect(roundWithConfig(12847, { rounding_increment: 50, rounding_mode: 'ceiling' })).toBe(12850);
  });

  it('rounds to nearest increment', () => {
    expect(roundWithConfig(12825, { rounding_increment: 50, rounding_mode: 'nearest' })).toBe(12850);
  });

  it('uses default config when undefined', () => {
    // Default: increment=50, mode=ceiling
    expect(roundWithConfig(12801, undefined)).toBe(12850);
  });

  it('returns 0 for 0 input', () => {
    expect(roundWithConfig(0, { rounding_increment: 50, rounding_mode: 'ceiling' })).toBe(0);
  });
});

describe('calcWhtAmounts', () => {
  it('calculates WHT gross-up with 5% rate', () => {
    const result = calcWhtAmounts(1000, 0.05);
    expect(result.invoiced).toBe(1053); // Math.round(1000 / 0.95) = 1053
    expect(result.wht).toBe(53);
  });

  it('returns amount unchanged when rate is 0', () => {
    const result = calcWhtAmounts(1000, 0);
    expect(result.invoiced).toBe(1000);
    expect(result.wht).toBe(0);
  });

  it('handles non-trivial rounding', () => {
    const result = calcWhtAmounts(5000, 0.05);
    expect(result.invoiced).toBe(5263); // Math.round(5000 / 0.95) = 5263
    expect(result.wht).toBe(263);
  });
});

describe('whtTooltip', () => {
  const mockFormatNumber = (n: number) => n.toLocaleString();

  it('returns tooltip string when WHT is active', () => {
    const config = { vat_percent: 5 };
    const result = whtTooltip(1000, config, mockFormatNumber);
    expect(result).toContain('Invoice:');
    expect(result).toContain('WHT');
    expect(result).toContain('5%');
  });

  it('returns empty string when vat_percent is 0', () => {
    const config = { vat_percent: 0 };
    expect(whtTooltip(1000, config, mockFormatNumber)).toBe('');
  });

  it('returns empty string when config is undefined', () => {
    expect(whtTooltip(1000, undefined, mockFormatNumber)).toBe('');
  });
});
```

### Step 2: Verify tests fail

- [ ] Run: `npx vitest run src/lib/utils/pricingUtils.test.ts`
- [ ] Expected: FAIL (module doesn't exist)

---

## Task 3: Implement pricing utilities (GREEN)

**Files:**
- Create: `src/lib/utils/pricingUtils.ts`

### Step 1: Implement

- [ ] Create `src/lib/utils/pricingUtils.ts`:

```typescript
import { roundToIncrement } from './format';

export function getRoundingConfig(config: any): { increment: number; mode: string } {
  return {
    increment: config?.rounding_increment ?? 50,
    mode: config?.rounding_mode ?? 'ceiling',
  };
}

export function roundWithConfig(rawTotal: number, config: any): number {
  const { increment, mode } = getRoundingConfig(config);
  return roundToIncrement(rawTotal, increment, mode);
}

export function calcWhtAmounts(
  amount: number,
  whtRate: number
): { invoiced: number; wht: number } {
  if (whtRate === 0) return { invoiced: Math.round(amount), wht: 0 };
  const invoiced = Math.round(amount / (1 - whtRate));
  const wht = Math.round(invoiced - amount);
  return { invoiced, wht };
}

export function whtTooltip(
  amount: number,
  config: any,
  formatNumber: (n: number) => string
): string {
  const vat = config?.vat_percent ?? 0;
  if (vat === 0) return '';
  const whtRate = vat / 100;
  const { invoiced, wht } = calcWhtAmounts(amount, whtRate);
  return `Invoice: ${formatNumber(invoiced)} (incl. ${formatNumber(wht)} WHT ${vat}%)`;
}
```

### Step 2: Verify tests pass

- [ ] Run: `npx vitest run src/lib/utils/pricingUtils.test.ts`
- [ ] Expected: all pass

### Step 3: Commit

- [ ] Commit: `refactor(utils): extract pricing utilities with TDD`

---

## Task 4: Migrate PricingCalculatorPanel to use pricingUtils

**Files:**
- Modify: `src/lib/components/pricing/PricingCalculatorPanel.svelte`

### Step 1: Replace local functions

- [ ] Add import: `import { roundWithConfig, calcWhtAmounts, whtTooltip as whtTooltipFn } from '$lib/utils/pricingUtils';`
- [ ] Replace local `getRoundedStageTotal()` (lines 149–158): keep the override check, but use `roundWithConfig()` for the fallback:

```typescript
function getRoundedStageTotal(stageId: string): number {
  if (stageTotalOverrides.has(stageId)) {
    return stageTotalOverrides.get(stageId)!;
  }
  return roundWithConfig(getStageTotal(stageId), config);
}
```

- [ ] Replace `grossUp()` (line ~213) and `whtTooltip()` (lines 216–221) with:

```typescript
function whtTooltip(amount: number): string {
  return whtTooltipFn(amount, config, formatNumber);
}
```

- [ ] Replace inline `config?.rounding_increment ?? 50` / `config?.rounding_mode ?? 'ceiling'` on lines 165–166, 174–175 with `roundWithConfig()` calls
- [ ] Remove now-unused `grossUp()` function

### Step 2: Verify

- [ ] `npm test` — all pass
- [ ] Commit: `refactor(pricing): migrate PricingCalculatorPanel to pricingUtils`

---

## Task 5: Migrate PaymentSchedulePanel to use pricingUtils

**Files:**
- Modify: `src/lib/components/pricing/PaymentSchedulePanel.svelte`

### Step 1: Replace local functions

- [ ] Add import: `import { roundWithConfig, calcWhtAmounts, whtTooltip as whtTooltipFn } from '$lib/utils/pricingUtils';`
- [ ] Replace local `getRoundedStageTotal()` (lines 39–46) with:

```typescript
function getRoundedStageTotal(stageId: string): number {
  const rawTotal = cells
    .filter(c => c.stage_id === stageId)
    .reduce((sum, c) => sum + (c.override_amount ?? c.amount), 0);
  return roundWithConfig(rawTotal, config);
}
```

- [ ] Replace local `whtTooltip()` (lines 223–228) with:

```typescript
function whtTooltip(amount: number): string {
  return whtTooltipFn(amount, config, formatNumber);
}
```

Note: the non-WHT percentage tooltip (`X.XX% of total`) is deliberately removed per user decision.

- [ ] Remove local `isWithholding`, `whtRate`, `grossUp` declarations if now unused
- [ ] Remove `roundToIncrement` from imports if no longer directly used

### Step 2: Verify

- [ ] `npm test` — all pass
- [ ] Commit: `refactor(pricing): migrate PaymentSchedulePanel to pricingUtils`

---

## Task 6: Migrate PricingSummaryBar to use pricingUtils

**Files:**
- Modify: `src/lib/components/pricing/PricingSummaryBar.svelte`

### Step 1: Replace inline WHT calculation

- [ ] Add import: `import { calcWhtAmounts } from '$lib/utils/pricingUtils';`
- [ ] Replace lines 103–104:

```svelte
{@const whtRate = (pricing?.config?.vat_percent ?? 0) / 100}
{@const grossedUp = Math.round(subtotal / (1 - whtRate))}
```

With:

```svelte
{@const { invoiced: grossedUp } = calcWhtAmounts(subtotal, (pricing?.config?.vat_percent ?? 0) / 100)}
```

### Step 2: Verify

- [ ] `npm test` — all pass
- [ ] Commit: `refactor(pricing): migrate PricingSummaryBar to pricingUtils`

---

## Final Verification

- [ ] `npm test` — all tests pass (678 existing + new pricingUtils tests)
- [ ] `npm run check` — 5 pre-existing errors only, no new
- [ ] Verify ProjectDetailPage now renders correctly (has edit support)
- [ ] Verify ProposalDetailPage still works identically
- [ ] Verify pricing panels display correct rounded values
- [ ] No broken imports across codebase
