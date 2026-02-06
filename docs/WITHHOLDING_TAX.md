# Withholding Tax — Reference & Implementation Notes

## Overview

Withholding tax is a tax deducted at source by the client when paying invoices. Unlike VAT (which is added on top), withholding tax is subtracted from the invoice amount before payment. This is common in Saudi Arabia (KSA) at a rate of 5% for foreign consultants.

## How It Works (Real-World Example)

Reference invoice: `WADG INV - 22041 SB5 PTG 2207.xlsx`
(Project 22-96603, RUA SB5, P&T Group Arabia, SAR)

### The Problem

If we quote a fee of 5,250 SAR and invoice that amount, the client withholds 5%:
- Invoice: 5,250 SAR
- Withholding (5%): -262.50 SAR
- **We receive: 4,987.50 SAR** (less than our quoted fee)

### The Solution: Gross-Up

We inflate the invoiced line item so that after withholding, the net receivable equals our intended (quoted) fee.

**Formula:**
```
invoiced_amount = quoted_amount + (rate / (1 - rate)) * quoted_amount
invoiced_amount = quoted_amount / (1 - rate)
```

**Example at 5%:**
```
invoiced_amount = 5,250 / (1 - 0.05) = 5,250 / 0.95 = 5,526.32 SAR

Verification:
  Invoice total:     5,526.32 SAR
  Withholding (5%): -  276.32 SAR
  Net receivable:    5,250.00 SAR  ✓ (equals our quoted fee)
```

In the Excel invoice, this appears as:
```
E16 = I16 + (5%/95% * I16)     -- gross-up formula per line item
G45 = SUM(G14:G34)             -- subtotal (grossed-up amounts)
G48 = G45 * 0%                 -- UAE VAT (0% for KSA client)
G49 = G45 + G48                -- total invoice amount
G50 = G49 * 5%                 -- withholding tax deduction
G51 = G49 - G50                -- receivable amount (= our quoted fee)
```

## Invoice Layout

```
Subtotal:              5,526.32 SAR   (grossed-up line items)
UAE VAT (0%):              0.00 SAR
Total Invoice Amt:     5,526.32 SAR
Withholding Tax (5%):  - 276.32 SAR
Receivable Amount:     5,250.00 SAR   (= quoted fee)
```

## Proposal vs Invoice

| Aspect | Fee Proposal | Tax Invoice |
|--------|-------------|-------------|
| Fee amounts | Quoted fee (net, what we want to receive) | Grossed-up amounts |
| Withholding | Mentioned as a note/comment only | Calculated and deducted |
| Grand total | = Subtotal (no tax adjustment) | = Grossed-up total |
| What client sees | Our fee | Larger amount (but they keep the tax portion) |

## Current Implementation (Proposal Module)

In the pricing calculator (`src/types/database.ts:calculatePricingTotals`):

- When `tax_type === 'vat'`: VAT is calculated and added to the subtotal → `grand_total = subtotal + vat`
- When `tax_type === 'withholding'`: No tax is added to totals. `vat_amount = 0`, `grand_total = subtotal`. A text note indicates that withholding applies and invoices will be grossed up.
- When `tax_type === 'none'`: No tax calculation.

The `vat_percent` config field stores the tax rate for both VAT and withholding scenarios (e.g., 5% VAT in UAE, 5% withholding in KSA).

### UI Display

- **Calculator panel**: Tax row only shown for VAT. Withholding has no visible tax line in the matrix.
- **Summary panel**: VAT shows as a line item added to subtotal. Withholding shows as a footnote: "Withholding tax (X%) applies — invoices will be grossed up so net receivable equals the quoted fee"
- **Summary bar**: Only VAT amount shown in the bar. Withholding does not inflate the displayed total.

## Future: Invoice Module

When an invoice module is implemented, the withholding gross-up calculation will need to be applied:

```typescript
// Gross-up a quoted amount for withholding tax
function grossUpForWithholding(quotedAmount: number, rate: number): number {
  return quotedAmount / (1 - rate);
}

// Calculate withholding deduction from a grossed-up invoice amount
function calculateWithholding(invoiceAmount: number, rate: number): number {
  return invoiceAmount * rate;
}

// Example usage:
const quoted = 5250;
const rate = 0.05;
const invoiced = grossUpForWithholding(quoted, rate);  // 5,526.32
const withheld = calculateWithholding(invoiced, rate);  // 276.32
const received = invoiced - withheld;                    // 5,250.00
```

### Key Considerations for Invoice Implementation

1. **Per-line-item gross-up**: Each invoice line item is grossed up individually (not just the total), as shown in the reference invoice formula `E16 = I16 + (5%/95% * I16)`.
2. **VAT interaction**: When both VAT and withholding apply, withholding is calculated on the total invoice amount (after VAT). In the KSA example, VAT was 0% since the client is in Saudi Arabia.
3. **Currency**: Withholding is most common for KSA (SAR) projects but may apply to other jurisdictions.
4. **Rate**: Currently 5% for KSA. Store the rate in config (reuses `vat_percent` field) so it can be adjusted per proposal.

---

*Last updated: 2026-02-06*
*Reference: Project 22-96603 RUA SB5 invoice (July 2022)*
