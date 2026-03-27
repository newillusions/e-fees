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
} from '$lib/types/scope';

const SCOPE_API_URL = import.meta.env.VITE_SCOPE_API_URL || 'http://10.0.21.81:3201';
const SCOPE_API_KEY = import.meta.env.VITE_SCOPE_API_KEY || 'efees-scope-2026-s7k2m9xp';

/**
 * Authenticated request helper for the scope service.
 * Attaches the API key header and handles error responses.
 */
async function scopeRequest<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(`${SCOPE_API_URL}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      'X-API-Key': SCOPE_API_KEY,
      ...options.headers,
    },
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

// =============================================================================
// STAGE CONFIG
// =============================================================================

/** List all stage configurations. */
export async function getStages(): Promise<PaginatedResponse<StageConfig>> {
  return scopeRequest<PaginatedResponse<StageConfig>>('/stages');
}

/** Update a stage configuration by canonical name. */
export async function updateStage(canonicalName: string, data: UpdateStageConfig): Promise<StageConfig> {
  return scopeRequest<StageConfig>(`/stages/${canonicalName}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

// =============================================================================
// DELIVERABLES CRUD
// =============================================================================

/** List deliverables with optional query filters (stage, layer, discipline). */
export async function getDeliverables(params?: Record<string, string>): Promise<PaginatedResponse<Deliverable>> {
  const qs = params ? '?' + new URLSearchParams(params).toString() : '';
  return scopeRequest<PaginatedResponse<Deliverable>>(`/deliverables${qs}`);
}

/** Fetch a single deliverable by ID. */
export async function getDeliverable(id: string): Promise<Deliverable> {
  return scopeRequest<Deliverable>(`/deliverables/${id}`);
}

/** Create a new deliverable. */
export async function createDeliverable(data: NewDeliverable): Promise<Deliverable> {
  return scopeRequest<Deliverable>('/deliverables', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

/** Update an existing deliverable by ID. */
export async function updateDeliverable(id: string, data: UpdateDeliverable): Promise<Deliverable> {
  return scopeRequest<Deliverable>(`/deliverables/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

/** Delete a deliverable by ID. */
export async function deleteDeliverable(id: string): Promise<void> {
  return scopeRequest<void>(`/deliverables/${id}`, { method: 'DELETE' });
}

/** Get deliverable usage analytics. */
export async function getDeliverableAnalytics(): Promise<DeliverableAnalytics[]> {
  return scopeRequest<DeliverableAnalytics[]>('/deliverables/analytics');
}

// =============================================================================
// SCOPE ASSEMBLY
// =============================================================================

/** Assemble deliverables into scope text for a fee proposal. */
export async function assembleDeliverables(data: AssembleRequest): Promise<AssembleResponse> {
  return scopeRequest<AssembleResponse>('/scope/assemble', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

/** Save scope builder state (selected deliverables per stage). */
export async function saveScopeBuilder(data: SaveScopeBuilderRequest): Promise<ScopeAssembly> {
  return scopeRequest<ScopeAssembly>('/scope/save', {
    method: 'POST',
    body: JSON.stringify(data),
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
export async function generateScope(data: GenerateScopeRequest, signal?: AbortSignal): Promise<ScopeAssembly> {
  return scopeRequest<ScopeAssembly>('/scope/generate', {
    method: 'POST',
    body: JSON.stringify(data),
    signal,
  });
}

/** Get scope for a fee proposal. Returns null if no scope exists (404). */
export async function getScope(feeId: string): Promise<ScopeAssembly | null> {
  try {
    return await scopeRequest<ScopeAssembly>(`/scope/${feeId}`);
  } catch (err: any) {
    if (err.status === 404) {
      return null;
    }
    throw err;
  }
}

/** Update scope for a fee proposal. */
export async function updateScope(feeId: string, data: UpdateScopeRequest): Promise<ScopeAssembly> {
  return scopeRequest<ScopeAssembly>(`/scope/${feeId}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

/** Regenerate scope for a fee proposal. */
export async function regenerateScope(feeId: string): Promise<ScopeAssembly> {
  return scopeRequest<ScopeAssembly>(`/scope/${feeId}/regenerate`, { method: 'POST' });
}

/** Export scope as formatted text. */
export async function exportScope(feeId: string): Promise<{ text: string }> {
  return scopeRequest<{ text: string }>(`/scope/${feeId}/export`);
}
