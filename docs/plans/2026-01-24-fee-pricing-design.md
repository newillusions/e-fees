# Fee Pricing & Proposal Enhancement Design

**Date**: 2026-01-24
**Status**: Draft
**Author**: Claude + Martin

## Overview

Add comprehensive pricing and proposal generation features to E-Fees:
1. **Pricing Calculator** - Discipline × Stage matrix with margin-based buffer
2. **Enhanced Fee Data Model** - Extended fields for pricing, stages, and revisions
3. **Export Integration** - Excel/CSV output for InDesign integration

## Core Concepts

### Pricing Philosophy

The pricing workflow is primarily **target-based allocation** with **gut-check sanity checking** against hourly rates:

1. Start with a **target fee** the project should achieve
2. Allocate across **disciplines** (percentages totaling 100%)
3. Break down by **design stages** (percentages totaling 100%)
4. Add **buffer margin** for negotiation capacity
5. Sanity check against estimated hours (informal, not binding)

### Buffer Calculation (Margin-Based)

The buffer allows for discount negotiations without eating into the target fee:

```
quoted_fee = target_fee / (1 - buffer_percent/100)
```

**Example**: Target 100,000 AED, Buffer 5%
- Quoted = 100,000 / (1 - 0.05) = 100,000 / 0.95 = **105,263 AED**
- If client negotiates 5% discount: 105,263 × 0.95 = 100,000 ✓

### Two Fee Sections

| Section | Description | Payment Terms |
|---------|-------------|---------------|
| **Design Phase** | Core design stages (SD, DD, CD, etc.) | Per milestone completion |
| **Post-Contract Services** | Site visits, CA, commissioning | Different terms, often optional |

### Payment Schedule & Mobilisation

Payments are tracked separately from fee breakdown:

1. **Mobilisation** - Advance payment (typically 30%) upon contract signing
2. **Milestone payments** - Tied to stage completion
3. **Payment tracking** - For future Invoice Ninja integration

The payment schedule derives from the pricing breakdown but tracks actual billing.

### Reimbursable Costs

Project costs (documents, samples, travel) can be added to the fee:

```
cost_to_client = base_cost × (1 + markup_percent/100)
```

**Example**: Document purchase 1,000 AED with 15% markup
- Cost to client = 1,000 × 1.15 = **1,150 AED**
- Added to the relevant stage fee

Costs are tracked per stage and rolled into the discipline/stage matrix or shown as line items.

## Data Model

### New Types

```typescript
// Discipline allocation
interface Discipline {
  id: string;
  name: string;           // "Lighting Design", "Audio Visual"
  percentage: number;     // Allocation percentage (0-100)
  order: number;          // Display order
}

// Design stage definition
interface Stage {
  id: string;
  name: string;           // "Schematic Design", "Design Development"
  code: string;           // "SD", "DD", "CD"
  percentage: number;     // Fee percentage for this stage
  order: number;          // Display order
  is_post_contract: boolean;  // Design phase vs post-contract
}

// Pricing configuration
interface PricingConfig {
  target_fee: number;           // What we want to achieve
  buffer_percent: number;       // Negotiation margin (typically 0-10%)
  quoted_fee: number;           // Calculated: target / (1 - buffer/100)
  currency: string;             // "AED", "USD", etc.
  vat_percent: number;          // VAT rate (typically 5% UAE)
  vat_included: boolean;        // Whether VAT is included in quoted_fee
  mobilisation_percent: number; // Advance payment percentage (typically 30%)
}

// Discipline × Stage pricing cell
interface PricingCell {
  discipline_id: string;
  stage_id: string;
  amount: number;           // Calculated from percentages
  override_amount?: number; // Manual override if needed
}

// Complete pricing breakdown
interface PricingBreakdown {
  config: PricingConfig;
  disciplines: Discipline[];
  stages: Stage[];
  cells: PricingCell[];        // The matrix values
  costs: ReimbursableCost[];   // Reimbursable expenses
  design_phase_total: number;
  post_contract_total: number;
  costs_total: number;         // Total reimbursable costs to client
  subtotal: number;
  vat_amount: number;
  grand_total: number;
}

// Post-contract line item (different structure)
interface PostContractItem {
  id: string;
  stage_id: string;           // Links to post-contract stage
  description: string;        // "Site Visit", "As-Built Documentation"
  quantity: number;           // Number of visits/items
  unit: string;               // "visit", "set", "lump sum"
  rate: number;               // Per-unit rate
  amount: number;             // quantity × rate
}

// Revision tracking (append-only)
interface PricingRevision {
  id: string;
  fee_id: string;
  revision_number: number;    // Sequential within fee
  created_at: string;         // ISO timestamp
  created_by: string;         // User who made the change
  change_summary: string;     // Brief description of what changed
  pricing_snapshot: PricingBreakdown;  // Full state at this revision
  is_client_release: boolean; // Was this sent to client?
  release_number?: number;    // Client-facing version (01, 02, etc.)
}

// Reimbursable cost/expense
interface ReimbursableCost {
  id: string;
  description: string;        // "CAD Document Purchase", "Sample Materials"
  stage_id: string;           // Which stage this cost belongs to
  discipline_id?: string;     // Optional: specific discipline
  base_cost: number;          // Actual cost incurred
  markup_percent: number;     // Markup percentage (typically 10-20%)
  cost_to_client: number;     // Calculated: base_cost × (1 + markup/100)
  date_incurred: string;      // When the cost was incurred
  notes?: string;             // Optional notes
}

// Payment schedule entry
// NOTE: Multiple payment entries can link to the same stage_id
// e.g., "50% DD" and "100% DD" both link to the DD stage
interface PaymentScheduleEntry {
  id: string;
  type: 'mobilisation' | 'milestone' | 'final';
  description: string;        // "30% Mobilisation", "50% DD", "100% DD"
  stage_id?: string;          // Linked stage (for milestone payments)
  stage_percentage?: number;  // What % of the stage fee this represents (50, 100)
  amount: number;             // Payment amount
  percentage_of_total: number; // What % of contract total this represents
  due_date?: string;          // Expected due date
  status: 'pending' | 'invoiced' | 'paid';
  invoice_number?: string;    // For future Invoice Ninja integration
  invoice_date?: string;
  paid_date?: string;
}

// Payment tracking summary
interface PaymentSchedule {
  entries: PaymentScheduleEntry[];
  total_invoiced: number;
  total_paid: number;
  total_outstanding: number;
}
```

### Extended Fee Type

```typescript
interface Fee {
  // Existing fields...
  id?: string;
  name: string;
  number: string;
  project_id: string;
  company_id: string;
  contact_id: string;
  status: FeeStatus;
  stage: FeeStage;
  issue_date: string;
  // ... other existing fields

  // NEW: Pricing fields
  pricing?: PricingBreakdown;
  post_contract_items?: PostContractItem[];
  reimbursable_costs?: ReimbursableCost[];
  payment_schedule?: PaymentSchedule;
  pricing_revisions?: PricingRevision[];
  current_revision_number: number;
  current_release_number: number;
}
```

### Default Values

```typescript
const DEFAULT_DISCIPLINES: Omit<Discipline, 'id'>[] = [
  { name: "Lighting Design", percentage: 60, order: 1 },
  { name: "Audio Visual", percentage: 40, order: 2 },
];

const DEFAULT_DESIGN_STAGES: Omit<Stage, 'id'>[] = [
  { name: "Schematic Design", code: "SD", percentage: 20, order: 1, is_post_contract: false },
  { name: "Design Development", code: "DD", percentage: 30, order: 2, is_post_contract: false },
  { name: "Construction Documents", code: "CD", percentage: 35, order: 3, is_post_contract: false },
  { name: "Bidding & Negotiation", code: "BN", percentage: 10, order: 4, is_post_contract: false },
  { name: "Construction Administration", code: "CA", percentage: 5, order: 5, is_post_contract: false },
];

const DEFAULT_POST_CONTRACT_STAGES: Omit<Stage, 'id'>[] = [
  { name: "Site Visits", code: "SV", percentage: 0, order: 10, is_post_contract: true },
  { name: "Commissioning Support", code: "CM", percentage: 0, order: 11, is_post_contract: true },
  { name: "As-Built Documentation", code: "AB", percentage: 0, order: 12, is_post_contract: true },
];
```

## UI Design

### Workflow Order

1. **Disciplines Panel** → Define who's involved and allocation
2. **Stages Panel** → Define design phases and breakdown
3. **Pricing Panel** → Calculate fees with buffer and review matrix
4. **Costs Panel** → Add reimbursable expenses with markup
5. **Payment Schedule Panel** → Define billing milestones
6. **Summary Panel** → Review full breakdown and totals

**Persistent Summary Bar** appears on all panels showing running totals.

### Panel Structure

Each panel is a reusable component that works for both:
- Direct manual editing (current workflow)
- Future wizard-style guided setup

#### Persistent Summary Bar

Shows on ALL pricing panels - provides at-a-glance totals while editing:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│ Target: 100,000 │ Quoted: 105,263 │ +Costs: 4,025 │ +VAT: 6,214 │ TOTAL: 130,502 │
└──────────────────────────────────────────────────────────────────────────────────┘
```

Features:
- Always visible when working on pricing
- Updates in real-time as values change
- Compact single-line format
- Click to expand to Summary Panel

#### Summary Panel

Full breakdown view for review and verification:

```
┌─────────────────────────────────────────────────────────────┐
│ Fee Summary                                                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   TARGET FEE                              100,000 AED       │
│   Buffer (5%)                              +5,263 AED       │
│   ───────────────────────────────────────────────           │
│   QUOTED FEE                              105,263 AED       │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│   Fee Breakdown                                             │
│   ───────────────────────────────────────────────           │
│   Design Phase                            105,263 AED       │
│     • Lighting Design (60%)                63,158 AED       │
│     • Audio Visual (40%)                   42,105 AED       │
│                                                             │
│   Post-Contract Services                   15,000 AED       │
│     • Site Visits (4 × 2,500)              10,000 AED       │
│     • Commissioning                         5,000 AED       │
│                                                             │
│   Reimbursable Costs                        4,025 AED       │
│     • CAD Document (1,000 + 15%)            1,150 AED       │
│     • Sample Materials (2,500 + 15%)        2,875 AED       │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│   SUBTOTAL                                124,288 AED       │
│   VAT (5%)                                  6,214 AED       │
│   ───────────────────────────────────────────────           │
│   GRAND TOTAL                             130,502 AED       │
│                                                             │
│   ┌───────────────────────────────────────────────────────┐ │
│   │ MOBILISATION (30%)                     39,151 AED     │ │
│   └───────────────────────────────────────────────────────┘ │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│   Payment Status                                            │
│   ○ Pending    130,502 AED                                  │
│   ○ Invoiced         0 AED                                  │
│   ● Paid             0 AED                                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

Features:
- Complete fee breakdown by category
- Discipline subtotals within design phase
- Line item detail for costs
- Payment status tracking
- Print/export friendly format

#### Disciplines Panel

```
┌─────────────────────────────────────────────────────────────┐
│ Disciplines                                          [+ Add] │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ☰ Lighting Design                              [60%] ✕ │ │
│ └─────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ☰ Audio Visual                                 [40%] ✕ │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│                                    Total: 100% ✓            │
└─────────────────────────────────────────────────────────────┘
```

Features:
- Drag to reorder (☰ handle)
- Inline percentage editing
- Add/remove disciplines
- Validation: must total 100%

#### Stages Panel

```
┌─────────────────────────────────────────────────────────────┐
│ Design Stages                                        [+ Add] │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ☰ Schematic Design (SD)                        [20%] ✕ │ │
│ │ ☰ Design Development (DD)                      [30%] ✕ │ │
│ │ ☰ Construction Documents (CD)                  [35%] ✕ │ │
│ │ ☰ Bidding & Negotiation (BN)                   [10%] ✕ │ │
│ │ ☰ Construction Administration (CA)              [5%] ✕ │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                    Total: 100% ✓            │
├─────────────────────────────────────────────────────────────┤
│ Post-Contract Services                               [+ Add] │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ☰ Site Visits                              [qty × rate] │ │
│ │ ☰ Commissioning Support                    [qty × rate] │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

Features:
- Separate sections for design phase vs post-contract
- Design stages use percentages
- Post-contract items use quantity × rate

#### Pricing Panel

```
┌─────────────────────────────────────────────────────────────┐
│ Fee Calculator                                              │
├─────────────────────────────────────────────────────────────┤
│ Target Fee:     [    100,000 ] AED                          │
│ Buffer:         [        5  ] %                             │
│ Quoted Fee:     [    105,263 ] AED  (calculated)            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Design Phase Breakdown                                      │
│ ┌───────────────┬──────────────┬──────────────┬───────────┐ │
│ │ Stage         │ Lighting 60% │ Audio 40%    │ Total     │ │
│ ├───────────────┼──────────────┼──────────────┼───────────┤ │
│ │ SD (20%)      │ 12,632       │ 8,421        │ 21,053    │ │
│ │ DD (30%)      │ 18,947       │ 12,632       │ 31,579    │ │
│ │ CD (35%)      │ 22,105       │ 14,737       │ 36,842    │ │
│ │ BN (10%)      │ 6,316        │ 4,211        │ 10,526    │ │
│ │ CA (5%)       │ 3,158        │ 2,105        │ 5,263     │ │
│ ├───────────────┼──────────────┼──────────────┼───────────┤ │
│ │ TOTAL         │ 63,158       │ 42,105       │ 105,263   │ │
│ └───────────────┴──────────────┴──────────────┴───────────┘ │
│                                                             │
│ Post-Contract Services                                      │
│ ┌────────────────────────┬─────┬────────┬─────────────────┐ │
│ │ Item                   │ Qty │ Rate   │ Amount          │ │
│ ├────────────────────────┼─────┼────────┼─────────────────┤ │
│ │ Site Visits            │ 4   │ 2,500  │ 10,000          │ │
│ │ Commissioning          │ 1   │ 5,000  │ 5,000           │ │
│ ├────────────────────────┼─────┼────────┼─────────────────┤ │
│ │ TOTAL                  │     │        │ 15,000          │ │
│ └────────────────────────┴─────┴────────┴─────────────────┘ │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Summary                                                     │
│   Design Phase:                         105,263 AED         │
│   Post-Contract:                         15,000 AED         │
│   ─────────────────────────────────────────────────         │
│   Subtotal:                             120,263 AED         │
│   VAT (5%):                               6,013 AED         │
│   ─────────────────────────────────────────────────         │
│   GRAND TOTAL:                          126,276 AED         │
│                                                             │
│   Mobilisation (30%):                    37,883 AED         │
└─────────────────────────────────────────────────────────────┘
```

Features:
- Auto-calculating matrix
- Manual cell override option
- Currency formatting
- VAT calculation
- Mobilisation display

#### Costs Panel

```
┌─────────────────────────────────────────────────────────────┐
│ Reimbursable Costs                                   [+ Add] │
├─────────────────────────────────────────────────────────────┤
│ ┌──────────────────────┬────────┬────────┬────────┬───────┐ │
│ │ Description          │ Stage  │ Cost   │ Markup │ Total │ │
│ ├──────────────────────┼────────┼────────┼────────┼───────┤ │
│ │ CAD Document         │ SD     │ 1,000  │ 15%    │ 1,150 │ │
│ │ Sample Materials     │ DD     │ 2,500  │ 15%    │ 2,875 │ │
│ ├──────────────────────┼────────┼────────┼────────┼───────┤ │
│ │ TOTAL                │        │ 3,500  │        │ 4,025 │ │
│ └──────────────────────┴────────┴────────┴────────┴───────┘ │
│                                                             │
│ Default Markup: [15] %                                      │
└─────────────────────────────────────────────────────────────┘
```

Features:
- Add costs to specific stages
- Configurable markup percentage
- Auto-calculate client cost
- Rolls into pricing summary

#### Payment Schedule Panel

```
┌─────────────────────────────────────────────────────────────┐
│ Payment Schedule                              [Generate from │
│                                                     Pricing] │
├─────────────────────────────────────────────────────────────┤
│ ┌────────────────────────┬──────────┬────────┬────────────┐ │
│ │ Payment                │ Amount   │ %      │ Status     │ │
│ ├────────────────────────┼──────────┼────────┼────────────┤ │
│ │ Mobilisation           │ 37,883   │ 30%    │ ○ Pending  │ │
│ │ SD Completion          │ 21,053   │ 17%    │ ○ Pending  │ │
│ │ DD Completion          │ 31,579   │ 25%    │ ○ Pending  │ │
│ │ CD Completion          │ 36,842   │ 29%    │ ○ Pending  │ │
│ ├────────────────────────┼──────────┼────────┼────────────┤ │
│ │ TOTAL                  │ 126,276  │ 100%   │            │ │
│ └────────────────────────┴──────────┴────────┴────────────┘ │
│                                                             │
│ Invoiced: 0 AED | Paid: 0 AED | Outstanding: 126,276 AED   │
└─────────────────────────────────────────────────────────────┘
```

Features:
- Auto-generate from pricing breakdown
- Manual adjustment of payment amounts
- Track invoice and payment status
- Future Invoice Ninja integration hook

### Revision Tracking UI

```
┌─────────────────────────────────────────────────────────────┐
│ Revision History                              [Save Revision]│
├─────────────────────────────────────────────────────────────┤
│ Rev 3 • 24 Jan 2026 • "Updated SD percentage"               │
│ Rev 2 • 22 Jan 2026 • "Added AV discipline" • Released v02  │
│ Rev 1 • 20 Jan 2026 • "Initial pricing" • Released v01      │
└─────────────────────────────────────────────────────────────┘
```

- Append-only history
- Click to view any revision
- Mark as client release (increments release number)
- Compare revisions (future enhancement)

## Export Format

### Fee Breakdown vs Payment Schedule

**Important distinction**: The export must show both:

1. **Fee Breakdown** - What each stage/discipline is worth (the work value)
2. **Payment Schedule** - When payments are due (the billing plan)

These often differ:
- Fee might show "DD: 31,579 AED" as the stage value
- Payment might show "50% DD: 15,790 AED" and "100% DD: 15,790 AED" as separate billing milestones
- Mobilisation is a payment item, not a fee item

### Pricing Excel/CSV

Maintains compatibility with current InDesign workflow:

**Sheet 1: Fee Breakdown**
```
| Stage | Lighting | Audio | Total |
|-------|----------|-------|-------|
| SD    | 12,632   | 8,421 | 21,053|
| DD    | 18,947   | 12,632| 31,579|
| ...   | ...      | ...   | ...   |
```

**Sheet 2: Payment Schedule**
```
| Payment Milestone      | Amount   | % of Total |
|------------------------|----------|------------|
| Mobilisation (30%)     | 37,883   | 30%        |
| SD Completion          | 21,053   | 17%        |
| 50% DD                 | 15,790   | 12.5%      |
| 100% DD                | 15,790   | 12.5%      |
| ...                    | ...      | ...        |
```

**Sheet 3: Summary**
- Design phase total
- Post-contract total
- Reimbursable costs
- VAT
- Grand total

### Existing var.json

Unchanged - continues to work for InDesign data merge.

## Implementation Phases

### Phase 1: Data Model & Core UI
- [ ] Extend Fee type with pricing fields
- [ ] Create database migration for new fields
- [ ] Build Persistent Summary Bar component
- [ ] Build Disciplines panel component
- [ ] Build Stages panel component
- [ ] Build basic Pricing calculator panel
- [ ] Build Costs panel component (reimbursables with markup)
- [ ] Build Payment Schedule panel component
- [ ] Build Summary panel component (full breakdown)
- [ ] Add revision tracking infrastructure

### Phase 2: Full Calculator & Export
- [ ] Complete pricing matrix with auto-calculation
- [ ] Add manual cell override capability
- [ ] Implement post-contract items editor
- [ ] Add Excel/CSV export
- [ ] Integrate into ProposalModal

### Phase 3: Polish & Integration
- [ ] Revision comparison view
- [ ] Pricing templates (save/load discipline+stage configs)
- [ ] Hourly rate sanity checker (informal)
- [ ] Invoice Ninja integration (future)

## Future Enhancements

Items identified during review that are currently handled manually in InDesign:

- Scope inclusions/exclusions per discipline
- Sub-milestones within stages (50% DD, 100% DD)
- Payment vs Fee distinction tracking
- Optional items with separate pricing
- Variable payment terms (14 vs 30 days)
- Variable daily rates per discipline
- Project areas/spaces list

These will be added as the InDesign workflow migrates more fully to the app.

## Technical Notes

### Database Storage

Pricing data stored as JSON in SurrealDB for flexibility:

```sql
-- Fee table extension
ALTER TABLE fee ADD FIELD pricing TYPE object;
ALTER TABLE fee ADD FIELD post_contract_items TYPE array;
ALTER TABLE fee ADD FIELD reimbursable_costs TYPE array;
ALTER TABLE fee ADD FIELD payment_schedule TYPE object;
ALTER TABLE fee ADD FIELD pricing_revisions TYPE array;
ALTER TABLE fee ADD FIELD current_revision_number TYPE int DEFAULT 0;
ALTER TABLE fee ADD FIELD current_release_number TYPE int DEFAULT 0;
```

### Rust Types

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct PricingConfig {
    pub target_fee: f64,
    pub buffer_percent: f64,
    pub quoted_fee: f64,
    pub currency: String,
    pub vat_percent: f64,
    pub vat_included: bool,
    pub mobilisation_percent: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReimbursableCost {
    pub id: String,
    pub description: String,
    pub stage_id: String,
    pub discipline_id: Option<String>,
    pub base_cost: f64,
    pub markup_percent: f64,
    pub cost_to_client: f64,
    pub date_incurred: String,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PaymentScheduleEntry {
    pub id: String,
    pub payment_type: String,  // "mobilisation", "milestone", "final"
    pub description: String,
    pub stage_id: Option<String>,
    pub amount: f64,
    pub percentage_of_total: f64,
    pub due_date: Option<String>,
    pub status: String,  // "pending", "invoiced", "paid"
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub paid_date: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PaymentSchedule {
    pub entries: Vec<PaymentScheduleEntry>,
    pub total_invoiced: f64,
    pub total_paid: f64,
    pub total_outstanding: f64,
}

// ... additional types matching TypeScript definitions
```

---

**Approved by**: Pending
**Implementation branch**: TBD
**Target version**: 0.11.0
