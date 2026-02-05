# Stage Rounding Feature Design

**Date:** 2026-02-05
**Status:** Approved
**Branch:** `feat/fee-pricing-calculator`

## Overview

Implement configurable rounding for stage totals to produce client-friendly "human readable" fee proposals while maintaining precise internal records for payment distribution.

## Requirements

1. **Stage totals** round to configurable increment (50, 100, 250, 500, 1000)
2. **Rounding mode** defaults to ceiling (always up), with option for nearest
3. **Grand total** = sum of rounded stage totals (internally consistent)
4. **Tax** calculated from rounded subtotal, only shown on final pricing page
5. **Tax type** configurable: UAE VAT / KSA Withholding / None
6. **Internal precision** preserved for discipline-level payment distribution

## Data Model Changes

### PricingConfig (src/types/database.ts)

Add new fields:

```typescript
interface PricingConfig {
  // Existing
  target_fee: number;
  buffer_percentage: number;
  vat_percentage: number;
  currency: string;

  // New
  rounding_increment: number;      // 50, 100, 250, 500, 1000 (default: 50)
  rounding_mode: 'ceiling' | 'nearest';  // default: 'ceiling'
  tax_type: 'vat' | 'withholding' | 'none';  // default: 'vat'
  show_tax_in_summary: boolean;    // default: false
}
```

## New Utility Function

### roundToIncrement (src/lib/utils/format.ts)

```typescript
export function roundToIncrement(
  value: number,
  increment: number,
  mode: 'ceiling' | 'nearest' = 'ceiling'
): number {
  if (increment <= 0) return value;
  if (mode === 'ceiling') {
    return Math.ceil(value / increment) * increment;
  }
  return Math.round(value / increment) * increment;
}
```

## Calculation Flow

```
Target Fee
    ↓
Quoted Fee = target / (1 - buffer%)
    ↓
For each Discipline × Stage:
  Cell Amount = quoted × disc% × stage%  [PRECISE - stored]
    ↓
For each Stage:
  Raw Total = sum of cells
  Rounded Total = roundToIncrement(rawTotal, increment, mode)  [DISPLAYED]
    ↓
Design Subtotal = sum of Rounded Stage Totals
Post-Contract Total = sum of items (already round rates)
Costs Total = sum of reimbursables
    ↓
Subtotal = design + post_contract + costs
Tax = subtotal × tax_rate  [only if tax_type !== 'none']
Grand Total = subtotal + tax
```

## UI Changes

### FeeCalculatorPanel
- Add rounding increment dropdown: 50, 100, 250, 500, 1000
- Add rounding mode toggle: Ceiling / Nearest
- Add tax type dropdown: VAT / Withholding / None
- Add checkbox: Show tax in summary

### StagesPanel / PricingCalculatorPanel
- Stage total columns show rounded values
- Tooltip on hover shows precise calculated value
- No visual change to discipline columns (internal)

### PricingSummaryPanel
- Uses rounded stage totals
- Tax line only renders if `show_tax_in_summary === true`
- Grand total always shown

## Files to Modify

| File | Changes |
|------|---------|
| `src/types/database.ts` | Add new PricingConfig fields, defaults |
| `src/lib/utils/format.ts` | Add `roundToIncrement()` function |
| `src/lib/components/pricing/FeeCalculatorPanel.svelte` | Add rounding config UI |
| `src/lib/components/pricing/StagesPanel.svelte` | Apply rounding to display |
| `src/lib/components/pricing/PricingCalculatorPanel.svelte` | Apply rounding to stage totals |
| `src/lib/components/pricing/PricingSummaryPanel.svelte` | Conditional tax, rounded totals |

## Migration

Existing PricingConfig records without new fields will use defaults:
- `rounding_increment: 50`
- `rounding_mode: 'ceiling'`
- `tax_type: 'vat'`
- `show_tax_in_summary: false`

## Reference

Based on actual proposal format from `25-97103-FP-02.pdf`:
- Stage fees rounded to 500 increments for ~550k project
- VAT noted as "added at prevailing rate" - not in fee tables
- Post-contract uses qty × rate (rates already round)
