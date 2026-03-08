/**
 * Scope Service API client
 *
 * Communicates with the standalone e-fees-scope microservice (axum REST API).
 * Uses fetch() directly — this is NOT a Tauri IPC module.
 */

const SCOPE_API_URL = 'http://10.0.21.81:3201';
const SCOPE_API_KEY = 'efees-scope-2026-s7k2m9xp';

/**
 * Authenticated request helper for the scope service.
 * Attaches the API key header and handles error responses.
 */
async function scopeRequest<T = any>(path: string, options: RequestInit = {}): Promise<T> {
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
    throw new Error(error.message || `Scope API error: ${response.status}`);
  }
  return response.json();
}

// =============================================================================
// STAGE CONFIG
// =============================================================================

/** List all stage configurations. */
export async function getStages() {
  return scopeRequest('/stages');
}

/** Update a stage configuration by canonical name. */
export async function updateStage(canonicalName: string, data: Record<string, unknown>) {
  return scopeRequest(`/stages/${canonicalName}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

// =============================================================================
// DELIVERABLES CRUD
// =============================================================================

/** List deliverables with optional query filters (stage, layer, discipline). */
export async function getDeliverables(params?: Record<string, string>) {
  const qs = params ? '?' + new URLSearchParams(params).toString() : '';
  return scopeRequest(`/deliverables${qs}`);
}

/** Fetch a single deliverable by ID. */
export async function getDeliverable(id: string) {
  return scopeRequest(`/deliverables/${id}`);
}

/** Create a new deliverable. */
export async function createDeliverable(data: Record<string, unknown>) {
  return scopeRequest('/deliverables', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

/** Update an existing deliverable by ID. */
export async function updateDeliverable(id: string, data: Record<string, unknown>) {
  return scopeRequest(`/deliverables/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

/** Delete a deliverable by ID. */
export async function deleteDeliverable(id: string) {
  return scopeRequest(`/deliverables/${id}`, { method: 'DELETE' });
}

/** Get deliverable usage analytics. */
export async function getDeliverableAnalytics() {
  return scopeRequest('/deliverables/analytics');
}

// =============================================================================
// SCOPE ASSEMBLY
// =============================================================================

/** Assemble deliverables into scope text for a fee proposal. */
export async function assembleDeliverables(data: Record<string, unknown>) {
  return scopeRequest('/scope/assemble', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

/** Save scope builder state (selected deliverables per stage). */
export async function saveScopeBuilder(data: Record<string, unknown>) {
  return scopeRequest('/scope/save', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

/** Get saved scope deliverables for a fee proposal. */
export async function getScopeDeliverables(feeId: string) {
  return scopeRequest(`/scope/${feeId}/deliverables`);
}

// =============================================================================
// SCOPE GENERATION & MANAGEMENT
// =============================================================================

/** Generate scope text from clauses (LLM-powered). */
export async function generateScope(data: Record<string, unknown>) {
  return scopeRequest('/scope/generate', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

/** Get scope for a fee proposal. */
export async function getScope(feeId: string) {
  return scopeRequest(`/scope/${feeId}`);
}

/** Update scope for a fee proposal. */
export async function updateScope(feeId: string, data: Record<string, unknown>) {
  return scopeRequest(`/scope/${feeId}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

/** Regenerate scope for a fee proposal. */
export async function regenerateScope(feeId: string) {
  return scopeRequest(`/scope/${feeId}/regenerate`, { method: 'POST' });
}

/** Export scope as formatted text. */
export async function exportScope(feeId: string) {
  return scopeRequest(`/scope/${feeId}/export`);
}
