/**
 * Scope Service API client
 *
 * Communicates with the standalone e-fees-scope microservice (axum REST API).
 * Uses fetch() directly — this is NOT a Tauri IPC module.
 */

import type {
  StageConfig,
  UpdateStageConfig,
  Deliverable,
  NewDeliverable,
  UpdateDeliverable,
  AssembleRequest,
  AssembleResponse,
  SaveScopeBuilderRequest,
  GenerateScopeRequest,
  UpdateScopeRequest,
  ScopeAssembly,
  ScopeDeliverableEntry,
  PaginatedResponse,
  DeliverableAnalytics,
  Clause,
  SaveClauseSelectionRequest,
  ClauseSelectionResponse,
  ClauseSuggestionsResponse
} from '$lib/types/scope';

const SCOPE_API_URL = import.meta.env.VITE_SCOPE_API_URL || 'http://10.0.21.81:3201';
const SCOPE_API_KEY = import.meta.env.VITE_SCOPE_API_KEY || 'efees-scope-2026-s7k2m9xp';

/**
 * Authenticated request helper for the scope service.
 * Attaches the API key header and handles error responses.
 * Returns the raw response body — use scopeRequestData for endpoints that
 * wrap their payload in a `{ data: ... }` envelope.
 */
async function scopeRequest<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(`${SCOPE_API_URL}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      'X-API-Key': SCOPE_API_KEY,
      ...options.headers
    }
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: response.statusText }));
    const message = error.message || `Scope API error: ${response.status}`;
    const err = new Error(message);
    (err as any).status = response.status;
    throw err;
  }
  return response.json();
}

/**
 * Same as scopeRequest, but unwraps the `{ data: T }` envelope used by most
 * single-item endpoints (`/stages/{n}`, `/deliverables/{id}`, `/scope/...`,
 * etc.). Use scopeRequest directly for paginated list endpoints (which need
 * the `{ data, total }` shape) and for endpoints that return a flat shape
 * like `/scope/assemble` (`{ stages: [...] }`) or `/scope/{id}/export`.
 */
async function scopeRequestData<T>(path: string, options: RequestInit = {}): Promise<T> {
  const envelope = await scopeRequest<{ data: T }>(path, options);
  return envelope.data;
}

// =============================================================================
// STAGE CONFIG
// =============================================================================

/** List all stage configurations. */
export async function getStages(): Promise<PaginatedResponse<StageConfig>> {
  return scopeRequest<PaginatedResponse<StageConfig>>('/stages');
}

/** Update a stage configuration by canonical name. */
export async function updateStage(
  canonicalName: string,
  data: UpdateStageConfig
): Promise<StageConfig> {
  return scopeRequestData<StageConfig>(`/stages/${canonicalName}`, {
    method: 'PUT',
    body: JSON.stringify(data)
  });
}

// =============================================================================
// DELIVERABLES CRUD
// =============================================================================

/** List deliverables with optional query filters (stage, layer, discipline). */
export async function getDeliverables(
  params?: Record<string, string>
): Promise<PaginatedResponse<Deliverable>> {
  const qs = params ? '?' + new URLSearchParams(params).toString() : '';
  return scopeRequest<PaginatedResponse<Deliverable>>(`/deliverables${qs}`);
}

/** Fetch a single deliverable by ID. */
export async function getDeliverable(id: string): Promise<Deliverable> {
  return scopeRequestData<Deliverable>(`/deliverables/${id}`);
}

/** Create a new deliverable. */
export async function createDeliverable(data: NewDeliverable): Promise<Deliverable> {
  return scopeRequestData<Deliverable>('/deliverables', {
    method: 'POST',
    body: JSON.stringify(data)
  });
}

/** Update an existing deliverable by ID. */
export async function updateDeliverable(id: string, data: UpdateDeliverable): Promise<Deliverable> {
  return scopeRequestData<Deliverable>(`/deliverables/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data)
  });
}

/** Delete a deliverable by ID. */
export async function deleteDeliverable(id: string): Promise<void> {
  return scopeRequest<void>(`/deliverables/${id}`, { method: 'DELETE' });
}

/** Get deliverable usage analytics. */
export async function getDeliverableAnalytics(): Promise<DeliverableAnalytics[]> {
  return scopeRequestData<DeliverableAnalytics[]>('/deliverables/analytics');
}

// =============================================================================
// SCOPE ASSEMBLY
// =============================================================================

/** Assemble deliverables into scope text for a fee proposal. */
export async function assembleDeliverables(data: AssembleRequest): Promise<AssembleResponse> {
  return scopeRequest<AssembleResponse>('/scope/assemble', {
    method: 'POST',
    body: JSON.stringify(data)
  });
}

/** Save scope builder state (selected deliverables per stage). */
export async function saveScopeBuilder(data: SaveScopeBuilderRequest): Promise<ScopeAssembly> {
  return scopeRequestData<ScopeAssembly>('/scope/save', {
    method: 'POST',
    body: JSON.stringify(data)
  });
}

/** Get saved scope deliverables for a fee proposal. */
export async function getScopeDeliverables(feeId: string): Promise<ScopeDeliverableEntry[]> {
  return scopeRequest<ScopeDeliverableEntry[]>(`/scope/${feeId}/deliverables`);
}

// =============================================================================
// SCOPE GENERATION & MANAGEMENT
// =============================================================================

/** Generate scope text from clauses (LLM-powered). */
export async function generateScope(
  data: GenerateScopeRequest,
  signal?: AbortSignal
): Promise<ScopeAssembly> {
  return scopeRequestData<ScopeAssembly>('/scope/generate', {
    method: 'POST',
    body: JSON.stringify(data),
    signal
  });
}

/** Get scope for a fee proposal. Returns null if no scope exists (404). */
export async function getScope(feeId: string): Promise<ScopeAssembly | null> {
  try {
    return await scopeRequestData<ScopeAssembly>(`/scope/${feeId}`);
  } catch (err: any) {
    if (err.status === 404) {
      return null;
    }
    throw err;
  }
}

/** Update scope for a fee proposal. */
export async function updateScope(feeId: string, data: UpdateScopeRequest): Promise<ScopeAssembly> {
  return scopeRequestData<ScopeAssembly>(`/scope/${feeId}`, {
    method: 'PUT',
    body: JSON.stringify(data)
  });
}

/** Regenerate scope for a fee proposal. */
export async function regenerateScope(feeId: string): Promise<ScopeAssembly> {
  return scopeRequestData<ScopeAssembly>(`/scope/${feeId}/regenerate`, { method: 'POST' });
}

/** Export scope as formatted text. */
export async function exportScope(feeId: string): Promise<{ text: string }> {
  return scopeRequest<{ text: string }>(`/scope/${feeId}/export`);
}

// =============================================================================
// CLAUSE SELECTION (Stage 1 save/retrieve; Stage 2 is_default + conditions pre-fill)
// =============================================================================

/** List all active library clauses. */
export async function getClauses(params?: Record<string, string>): Promise<PaginatedResponse<Clause>> {
  const qs = params ? '?' + new URLSearchParams(params).toString() : '';
  return scopeRequest<PaginatedResponse<Clause>>(`/clauses${qs}`);
}

/**
 * Save clause selections for a fee proposal.
 *
 * Creates the scope_assembly record if it does not exist yet.
 * Existing records are patch-updated (only selected_clauses is touched).
 */
export async function saveClauseSelection(data: SaveClauseSelectionRequest): Promise<{
  status: string;
  fee_id: string;
  selections_count: number;
  included_count: number;
}> {
  return scopeRequest('/scope/clause-selection', {
    method: 'POST',
    body: JSON.stringify(data)
  });
}

/**
 * Get clause selection for a fee proposal.
 *
 * Returns all active clauses merged with the saved selection.
 *
 * When no selection has been saved yet (Stage 2), each clause defaults to
 * its `is_default` flag, gated by its `conditions` object (if any) against
 * the optional `conditions` param passed here - a project-attributes object
 * subset-matched against each conditional clause's `conditions` (e.g.
 * `{ project_type: 'hospitality' }`). Omit it when the caller has no known
 * project conditions; conditional clauses then default to excluded.
 */
export async function getClauseSelection(
  feeId: string,
  conditions?: Record<string, unknown>
): Promise<ClauseSelectionResponse> {
  const qs =
    conditions && Object.keys(conditions).length > 0
      ? '?' + new URLSearchParams({ conditions: JSON.stringify(conditions) }).toString()
      : '';
  return scopeRequest<ClauseSelectionResponse>(`/scope/${feeId}/clause-selection${qs}`);
}

/**
 * Get ranked clause suggestions for a fee proposal (Stage 3), mined from
 * historical FP corpus usage frequency. Excludes clauses already included
 * in the fee's current selection. Returns an empty list (not an error) when
 * the mining job has not run yet.
 */
export async function getClauseSuggestions(feeId: string): Promise<ClauseSuggestionsResponse> {
  return scopeRequest<ClauseSuggestionsResponse>(`/scope/${feeId}/clause-suggestions`);
}
