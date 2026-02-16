// SurrealDB Type Definitions
// Auto-generated from database schema

// Enums
export type ProjectActivity = 
  | 'Design and Consultancy'
  | 'Management'
  | 'Construction'
  | 'Maintenance'
  | 'Research'
  | 'Other';

export type ProjectStatus =
  | 'Draft'
  | 'RFP'
  | 'Active'
  | 'Awarded'
  | 'Revised'
  | 'On Hold'
  | 'Completed'
  | 'Lost'
  | 'Cancelled';

export type ProjectStage = 
  | 'Concept'
  | 'Design Development'
  | 'Documentation'
  | 'Tender'
  | 'Construction'
  | 'Handover';

export type FeeStatus =
  | 'Draft'
  | 'Active'
  | 'Sent'
  | 'Negotiation'
  | 'Awarded'
  | 'Revised'
  | 'Completed'
  | 'On Hold'
  | 'Lost'
  | 'Cancelled';

export type FeeStage = 
  | 'Draft'
  | 'Prepared'
  | 'Sent'
  | 'Under Review'
  | 'Clarification'
  | 'Negotiation'
  | 'Awarded'
  | 'Lost';

// Base types
export interface TimeInfo {
  created_at: string; // datetime
  updated_at: string; // datetime
}

export interface ProjectNumber {
  year: number; // 20-50
  country: number; // dial code
  seq: number; // 1-999
  id: string; // computed: YY-CCCNN
}

export interface Revision {
  revision_number: number;
  revision_date: string; // datetime
  author_email: string; // valid email
  author_name: string;
  notes: string;
}

// Main entities
export interface Project {
  id?: string; // projects:YY_CCCNN
  name: string;
  name_short?: string;
  status: ProjectStatus;
  area?: string;
  city: string;
  country: string;
  folder?: string;
  number?: ProjectNumber;
  project_number?: string;
  client_company?: string;
  time?: TimeInfo;
}

export interface Fee {
  id?: string; // fee:YY_CCCNN_R
  name: string;
  number: string;
  project_id: string; // record<projects>
  company_id: string; // record<company>
  contact_id: string; // record<contacts>
  status: FeeStatus;
  stage?: FeeStage;
  issue_date: string; // YYMMDD format
  activity?: string;
  package?: string;
  strap_line?: string;
  staff_name?: string;
  staff_email?: string;
  staff_phone?: string;
  staff_position?: string;
  rev?: number; // auto-computed
  revisions: Revision[];
  time?: TimeInfo;

  // Pricing fields (optional for backward compatibility)
  pricing?: PricingBreakdown;
  post_contract_items?: PostContractItem[];
  reimbursable_costs?: ReimbursableCost[];
  payment_schedule?: PaymentSchedule;
  pricing_revisions?: PricingRevision[];
  current_revision_number?: number;
  current_release_number?: number;
  import_source?: {
    file_path: string;
    file_modified: string;
    file_size: number;
    imported_at: string;
    import_version: string;
    checksum: string;
    project_folder_status: string;
    filename_pattern: string;
  };
}

export interface Company {
  id?: string; // company:ABBREVIATION
  name: string;
  name_short?: string;
  abbreviation?: string;
  city: string;
  country: string;
  address?: string;
  phone?: string;
  email?: string;
  website?: string;
  reg_no?: string;
  tax_no?: string;
  time?: TimeInfo;
}

export interface Contact {
  id?: string; // contacts:random_id
  first_name: string;
  last_name: string;
  email: string; // unique
  phone: string; // must contain '+'
  position: string;
  company: string; // record<company>
  full_name?: string; // auto-computed
  time?: TimeInfo;
}

export interface Country {
  id?: string; // country:CODE
  name: string;
  name_formal: string;
  name_official: string;
  code: string;
  code_alt: string;
  dial_code: number;
  currency_code?: string; // record<currency>
}

export interface Currency {
  id?: string; // currency:CODE
  code: string;
  name: string;
}

// ============================================================================
// PRICING TYPES
// ============================================================================

/** Discipline allocation for fee breakdown */
export interface Discipline {
  id: string;
  name: string;           // "Lighting Design", "Audio Visual"
  code: string;           // Short abbreviation for column headers ("LD", "AV")
  percentage: number;     // Allocation percentage (0-100)
  order: number;          // Display order
}

/** Design stage definition */
export interface Stage {
  id: string;
  name: string;           // "Schematic Design", "Design Development"
  code: string;           // "SD", "DD", "CD"
  percentage: number;     // Fee percentage for this stage
  order: number;          // Display order
  is_post_contract: boolean;  // Design phase vs post-contract
}

/** Tax type for pricing */
export type TaxType = 'vat' | 'withholding' | 'none';

/** Rounding mode for stage totals */
export type RoundingMode = 'ceiling' | 'nearest';

/** Pricing configuration */
export interface PricingConfig {
  target_fee: number;           // What we want to achieve
  buffer_percent: number;       // Negotiation margin (typically 0-10%)
  quoted_fee: number;           // Calculated: target / (1 - buffer/100)
  currency: string;             // "AED", "USD", etc.
  vat_percent: number;          // VAT rate (typically 5% UAE)
  vat_included: boolean;        // Whether VAT is included in quoted_fee
  mobilisation_percent: number; // Advance payment percentage (typically 30%)
  // Rounding settings for client-friendly stage totals
  rounding_increment: number;   // 50, 100, 250, 500, 1000 (default: 50)
  rounding_mode: RoundingMode;  // 'ceiling' or 'nearest' (default: 'ceiling')
  tax_type: TaxType;            // 'vat', 'withholding', or 'none' (default: 'vat')
  show_tax_in_summary: boolean; // Show tax in summary panels (default: false)
  // Multi-currency support (e.g., AED→SAR for Saudi projects)
  exchange_rate?: number;       // Conversion rate (1 base currency = X client currency)
  client_currency?: string;     // Target currency code (e.g., "SAR")
  quote_currency?: string;      // Currency amounts are quoted in (default: same as currency)
  rate_locked_at?: string;      // ISO datetime when the rate was locked (set on save)
}

/** Discipline × Stage pricing cell */
export interface PricingCell {
  discipline_id: string;
  stage_id: string;
  amount: number;           // Calculated from percentages
  override_amount?: number; // Manual override if needed
}

/** Reimbursable cost/expense */
export interface ReimbursableCost {
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

/** Post-contract line item (different structure from design stages) */
export interface PostContractItem {
  id: string;
  stage_id: string;           // Links to post-contract stage
  description: string;        // "Site Visit", "As-Built Documentation"
  quantity: number;           // Number of visits/items
  unit: string;               // "visit", "set", "lump sum"
  rate: number;               // Per-unit rate
  amount: number;             // quantity × rate
}

/** Payment schedule entry - multiple entries can link to same stage */
export type PaymentType = 'mobilisation' | 'milestone' | 'final';
export type PaymentStatus = 'pending' | 'invoiced' | 'paid';

export interface PaymentScheduleEntry {
  id: string;
  type: PaymentType;
  description: string;        // "30% Mobilisation", "50% DD", "100% DD"
  stage_id?: string;          // Linked stage (for milestone payments)
  stage_percentage?: number;  // What % of the stage fee this represents (50, 100)
  amount: number;             // Payment amount (after mobilisation deduction)
  quoted_stage_amount?: number; // Original quoted fee for the stage (before deduction)
  percentage_of_total: number; // What % of contract total this represents
  due_date?: string;          // Expected due date
  status: PaymentStatus;
  invoice_number?: string;    // For future Invoice Ninja integration
  invoice_date?: string;
  paid_date?: string;
}

/** Payment tracking summary */
export interface PaymentSchedule {
  entries: PaymentScheduleEntry[];
  total_invoiced: number;
  total_paid: number;
  total_outstanding: number;
}

/** Complete pricing breakdown */
export interface PricingBreakdown {
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

/** Revision tracking (append-only) */
export interface PricingRevision {
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

// ============================================================================
// DEFAULT VALUES
// ============================================================================

export const DEFAULT_DISCIPLINES: Omit<Discipline, 'id'>[] = [
  { name: "Lighting", code: "LX", percentage: 45, order: 1 },
  { name: "Video", code: "VID", percentage: 25, order: 2 },
  { name: "Audio", code: "AUD", percentage: 20, order: 3 },
  { name: "Control", code: "CTL", percentage: 10, order: 4 },
];

export const DEFAULT_DESIGN_STAGES: Omit<Stage, 'id'>[] = [
  { name: "Concept Design", code: "CON", percentage: 25, order: 1, is_post_contract: false },
  { name: "Schematic Design", code: "SD", percentage: 30, order: 2, is_post_contract: false },
  { name: "Detailed Design", code: "DD", percentage: 35, order: 3, is_post_contract: false },
  { name: "Construction Documents", code: "CD", percentage: 10, order: 4, is_post_contract: false },
];

export const DEFAULT_POST_CONTRACT_STAGES: Omit<Stage, 'id'>[] = [
  { name: "Site Meetings", code: "SM", percentage: 0, order: 10, is_post_contract: true },
  { name: "Construction Supervision", code: "CS", percentage: 0, order: 11, is_post_contract: true },
  { name: "Installation Validation", code: "IV", percentage: 0, order: 12, is_post_contract: true },
  { name: "Commissioning Supervision", code: "CMS", percentage: 0, order: 13, is_post_contract: true },
  { name: "Programming Supervision", code: "PS", percentage: 0, order: 14, is_post_contract: true },
];

export const DEFAULT_PRICING_CONFIG: Omit<PricingConfig, 'target_fee' | 'quoted_fee'> = {
  buffer_percent: 5,
  currency: "AED",
  vat_percent: 5,
  vat_included: false,
  mobilisation_percent: 30,
  rounding_increment: 50,
  rounding_mode: 'ceiling',
  tax_type: 'vat',
  show_tax_in_summary: false,
};

export const DEFAULT_COST_MARKUP_PERCENT = 15;

// Create/Update types (without computed fields)
export type ProjectCreate = Omit<Project, 'id' | 'time'>;
export type ProjectUpdate = Partial<ProjectCreate>;

export type FeeCreate = Omit<Fee, 'id' | 'time'>;
export type FeeUpdate = Partial<FeeCreate>;

export type CompanyCreate = Omit<Company, 'id' | 'time'>;
export type CompanyUpdate = Partial<CompanyCreate>;

export type ContactCreate = Omit<Contact, 'id' | 'time' | 'full_name'>;
export type ContactUpdate = Partial<ContactCreate>;

// Validation helpers
export const ValidationRules = {
  project: {
    name: { minLength: 1 },
    number: {
      year: { min: 20, max: 50 },
      seq: { min: 1, max: 999 }
    }
  },
  fee: {
    name: { minLength: 1 },
    issue_date: { pattern: /^\d{6}$/ } // YYMMDD
  },
  company: {
    name: { minLength: 1 }
  },
  contact: {
    email: { pattern: /^[^\s@]+@[^\s@]+\.[^\s@]+$/ },
    phone: { pattern: /\+/, minLength: 1 }
  }
};

// Helper functions
export function generateProjectId(year: number, country: number, seq: number): string {
  const yy = year.toString().padStart(2, '0');
  const ccc = country.toString();
  const nn = seq.toString().padStart(2, '0');
  return `${yy}-${ccc}${nn}`;
}

export function parseProjectId(id: string): ProjectNumber | null {
  const match = id.match(/^(\d{2})-(\d{3})(\d{2})$/);
  if (!match) return null;
  
  return {
    year: parseInt(match[1]),
    country: parseInt(match[2]),
    seq: parseInt(match[3]),
    id
  };
}

export function formatIssueDate(date: Date): string {
  const yy = date.getFullYear().toString().slice(-2);
  const mm = (date.getMonth() + 1).toString().padStart(2, '0');
  const dd = date.getDate().toString().padStart(2, '0');
  return `${yy}${mm}${dd}`;
}

export function parseIssueDate(dateStr: string): Date | null {
  if (!/^\d{6}$/.test(dateStr)) return null;

  const yy = parseInt(dateStr.slice(0, 2));
  const mm = parseInt(dateStr.slice(2, 4)) - 1;
  const dd = parseInt(dateStr.slice(4, 6));

  // Assume 20xx for years 00-50, 19xx for years 51-99
  const yyyy = yy <= 50 ? 2000 + yy : 1900 + yy;

  return new Date(yyyy, mm, dd);
}

// ============================================================================
// PRICING HELPER FUNCTIONS
// ============================================================================

/**
 * Convert an amount to the client currency using the exchange rate.
 * Returns undefined if no exchange rate or client currency is configured.
 */
export function convertToClientCurrency(
  amount: number,
  config: PricingConfig
): number | undefined {
  if (!config.exchange_rate || !config.client_currency || config.client_currency === config.currency) {
    return undefined;
  }
  return amount * config.exchange_rate;
}

/**
 * Calculate quoted fee from target fee using margin-based buffer.
 * Formula: quoted = target / (1 - buffer/100)
 * This allows for discount negotiations without eating into target.
 */
export function calculateQuotedFee(targetFee: number, bufferPercent: number): number {
  if (bufferPercent >= 100) return targetFee; // Avoid division by zero
  return targetFee / (1 - bufferPercent / 100);
}

/**
 * Calculate cost to client with markup.
 * Formula: client_cost = base_cost × (1 + markup/100)
 */
export function calculateCostWithMarkup(baseCost: number, markupPercent: number): number {
  return baseCost * (1 + markupPercent / 100);
}

/**
 * Calculate pricing cell amount from quoted fee, discipline %, and stage %.
 */
export function calculateCellAmount(
  quotedFee: number,
  disciplinePercent: number,
  stagePercent: number
): number {
  return quotedFee * (disciplinePercent / 100) * (stagePercent / 100);
}

/**
 * Generate a unique ID for pricing entities.
 */
export function generatePricingId(prefix: string): string {
  return `${prefix}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Create default disciplines with generated IDs.
 */
export function createDefaultDisciplines(): Discipline[] {
  return DEFAULT_DISCIPLINES.map((d, index) => ({
    ...d,
    id: generatePricingId('disc'),
  }));
}

/**
 * Create default design stages with generated IDs.
 */
export function createDefaultDesignStages(): Stage[] {
  return DEFAULT_DESIGN_STAGES.map((s, index) => ({
    ...s,
    id: generatePricingId('stage'),
  }));
}

/**
 * Create default post-contract stages with generated IDs.
 */
export function createDefaultPostContractStages(): Stage[] {
  return DEFAULT_POST_CONTRACT_STAGES.map((s, index) => ({
    ...s,
    id: generatePricingId('stage'),
  }));
}

/**
 * Round a value to the nearest increment.
 */
function roundToIncrement(
  value: number,
  increment: number,
  mode: RoundingMode = 'ceiling'
): number {
  if (increment <= 0) return value;
  if (mode === 'ceiling') {
    return Math.ceil(value / increment) * increment;
  }
  return Math.round(value / increment) * increment;
}

/**
 * Calculate totals from pricing breakdown with rounding support.
 */
export function calculatePricingTotals(
  cells: PricingCell[],
  postContractItems: PostContractItem[],
  costs: ReimbursableCost[],
  config: Omit<PricingConfig, 'target_fee' | 'quoted_fee'>,
  stages?: Stage[]
): {
  design_phase_total: number;
  post_contract_total: number;
  costs_total: number;
  subtotal: number;
  vat_amount: number;
  grand_total: number;
} {
  const roundingIncrement = config.rounding_increment ?? 50;
  const roundingMode = config.rounding_mode ?? 'ceiling';
  const vatPercent = config.vat_percent ?? 0;
  const taxType = config.tax_type ?? 'vat';
  const showTax = config.show_tax_in_summary ?? false;

  // Calculate design phase total with stage-level rounding
  let design_phase_total: number;

  if (stages && stages.length > 0) {
    // Group cells by stage and round each stage total
    const designStageIds = new Set(
      stages.filter(s => !s.is_post_contract).map(s => s.id)
    );

    const stageTotals = new Map<string, number>();
    for (const cell of cells) {
      if (!designStageIds.has(cell.stage_id)) continue;
      const current = stageTotals.get(cell.stage_id) ?? 0;
      stageTotals.set(cell.stage_id, current + (cell.override_amount ?? cell.amount));
    }

    // Round each stage total and sum
    design_phase_total = 0;
    for (const [, rawTotal] of stageTotals) {
      design_phase_total += roundToIncrement(rawTotal, roundingIncrement, roundingMode);
    }
  } else {
    // Fallback: no stages provided, just sum cells (no per-stage rounding)
    design_phase_total = cells.reduce(
      (sum, cell) => sum + (cell.override_amount ?? cell.amount),
      0
    );
  }

  const post_contract_total = postContractItems.reduce(
    (sum, item) => sum + item.amount,
    0
  );

  const costs_total = costs.reduce(
    (sum, cost) => sum + cost.cost_to_client,
    0
  );

  const subtotal = design_phase_total + post_contract_total + costs_total;

  // Calculate tax based on type
  // VAT: added on top of subtotal (client pays more)
  // Withholding: NOT added to proposal totals — it's only applied at invoice time
  //   (see docs/WITHHOLDING_TAX.md for full explanation)
  const vat_amount = taxType === 'vat' ? subtotal * (vatPercent / 100) : 0;
  const grand_total = subtotal + vat_amount;

  return {
    design_phase_total,
    post_contract_total,
    costs_total,
    subtotal,
    vat_amount,
    grand_total,
  };
}
