/**
 * Scope module type definitions.
 *
 * Mirrors the Rust models in e-fees-scope/src/models.rs.
 */

// ── Layer enum ──────────────────────────────────────────────────────

export type DeliverableLayer = 'generic' | 'discipline' | 'conditional';

// ── Deliverable ─────────────────────────────────────────────────────

export interface Deliverable {
  id: string;
  title: string;
  short_name: string;
  body: string;
  stage: string;
  layer: DeliverableLayer;
  discipline?: string;
  condition?: Record<string, unknown>;
  replaces?: string;
  sort_order: number;
  source_proposals?: string[];
  usage_history?: Record<string, unknown>[];
  tags?: string[];
  status: string;
  version: number;
  created_at: string;
  updated_at: string;
}

export interface NewDeliverable {
  title: string;
  short_name: string;
  body: string;
  stage: string;
  layer: string;
  discipline?: string;
  condition?: Record<string, unknown>;
  replaces?: string;
  sort_order: number;
  source_proposals?: string[];
  tags?: string[];
}

export interface UpdateDeliverable {
  title?: string;
  short_name?: string;
  body?: string;
  stage?: string;
  layer?: string;
  discipline?: string;
  condition?: Record<string, unknown>;
  replaces?: string;
  sort_order?: number;
  source_proposals?: string[];
  tags?: string[];
}

// ── Stage Config ────────────────────────────────────────────────────

export interface StageConfig {
  id?: string;
  canonical_name: string;
  default_label: string;
  aliases?: string[];
  sort_order: number;
  intro_text?: string;
  status: string;
}

export interface UpdateStageConfig {
  default_label?: string;
  aliases?: string[];
  sort_order?: number;
  intro_text?: string;
}

export interface StageDisplay {
  canonical_name: string;
  label: string;
  sort_order: number;
}

// ── Scope Assembly ──────────────────────────────────────────────────

export interface ScopeDeliverableEntry {
  deliverable_id: string;
  stage: string;
  sort_order: number;
  wording_override: string | null;
}

export interface ManualDeliverableEntry {
  title: string;
  short_name: string;
  body: string;
  stage: string;
  sort_order: number;
}

export interface SaveScopeBuilderRequest {
  fee_id: string;
  deliverables: ScopeDeliverableEntry[];
  manual_items?: ManualDeliverableEntry[];
  stage_labels?: Record<string, string>;
  polish?: boolean;
}

export interface AssembleRequest {
  fee_id: string;
  disciplines: string[];
  conditions?: Record<string, unknown>;
  stages?: string[];
  stage_labels?: Record<string, string>;
  polish?: boolean;
}

export interface AssembledStage {
  canonical_name: string;
  label: string;
  deliverables: Deliverable[];
}

export interface AssembleResponse {
  stages: AssembledStage[];
}

// ── Scope Clause Structure (from auto_number_clauses) ──────────────

export interface ScopeSection {
  number: string; // e.g. "1.0"
  title: string; // Category name (e.g. "Administrative")
  clauses: ScopeClauseItem[];
}

export interface ScopeClauseItem {
  number: string; // e.g. "1.1", "1.2"
  clause_id: string; // SurrealDB key
  title: string; // Clause title
  body: string; // Clause body text
}

export interface ScopeAssembly {
  id?: string;
  fee_id: string;
  clauses: ScopeSection[];
  generated_text: string;
  numbering?: Record<string, string>;
  llm_model?: string;
  llm_polished: boolean;
  current_revision?: number;
  stages_snapshot?: string[];
  created_at: string;
  updated_at: string;
}

export interface StageInput {
  name: string;
  code: string;
  is_post_contract: boolean;
  order: number;
}

export interface GenerateScopeRequest {
  fee_id: string;
  polish?: boolean;
  stages?: StageInput[];
}

export interface UpdateScopeRequest {
  generated_text?: string;
  clauses?: ScopeSection[];
}

// ── Clause Library (Stage 1) ────────────────────────────────────────

/** A clause record from the library (e-fees-scope /clauses endpoint). */
export interface Clause {
  id: string;
  category: string;
  subcategory?: string;
  title: string;
  body: string;
  sort_order: number;
  is_default: boolean;
  status: string;
  version: number;
  tags?: string[];
  created_at: string;
  updated_at: string;
}

/**
 * A single clause selection entry persisted on the scope_assembly.
 *
 * `override_body` is structural — once set it is never silently removed.
 * `null` means "use the library body as-is".
 */
export interface ClauseSelectionEntry {
  clause_id: string;
  included: boolean;
  override_body: string | null;
}

/** Payload for POST /scope/clause-selection. */
export interface SaveClauseSelectionRequest {
  fee_id: string;
  selections: ClauseSelectionEntry[];
}

/**
 * Response from GET /scope/{fee_id}/clause-selection.
 *
 * Each entry is a library clause merged with the saved selection state.
 * `has_custom_selection` is false when no selection has been saved yet
 * (all clauses are implicitly included).
 */
export interface ClauseSelectionItem extends ClauseSelectionEntry {
  title: string;
  category: string;
  body: string;
  sort_order: number;
  is_default: boolean;
}

export interface ClauseSelectionResponse {
  fee_id: string;
  has_custom_selection: boolean;
  selections: ClauseSelectionItem[];
}

/**
 * A ranked clause suggestion from GET /scope/{fee_id}/clause-suggestions
 * (Stage 3). Mined from historical FP corpus usage frequency - see
 * docs/plans/2026-07-10-clause-selection-stage3-design.md. Never filtered
 * by discipline (the clause library has no discipline field); always an
 * explicit opt-in add, never auto-included.
 */
export interface ClauseSuggestion {
  clause_id: string;
  title: string;
  category: string;
  usage_count: number;
  sample_project_numbers: string[];
  classified_at: string;
}

export interface ClauseSuggestionsResponse {
  fee_id: string;
  suggestions: ClauseSuggestion[];
}

// ── API response wrappers ───────────────────────────────────────────

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
}

export interface DeliverableAnalytics {
  deliverable_id: string;
  title: string;
  usage_count: number;
}
