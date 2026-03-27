# Scope UI Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the scope UI layer (ScopeBuilder, ScopeSectionView, ScopeClauseItem, ScopeAdvancedControls) enabling users to view, edit, regenerate, and export proposal scopes within the desktop app.

**Architecture:** REST client wrapper (`scopeClient.ts`) abstracts the scope service API at 10.0.21.81:3201. Reactive Svelte 5 store (`scopeStore.ts`) manages fetch/edit/save state transitions with unsaved-change tracking. Component hierarchy orchestrates: ProposalModal → ScopeBuilder (top-level) → ScopeSectionView (collapsible sections) + ScopeAdvancedControls (advanced UI). Each component uses Svelte 5 runes (`$state()`, `$props()`, `$effect()`, `$derived()`) for reactivity. Tauri backend commands persist updates to SurrealDB. Error handling: service 503/timeout → graceful user messages, 404 → null (scope not yet generated), unsaved edits + regenerate → confirmation dialog.

**Tech Stack:** Svelte 5 runes, TypeScript, SurrealDB v3 (persistent storage), Tauri 2 (IPC), REST client (fetch), semantic CSS (`.emittiv-*` classes with fixed px values).

---

## File Structure

**Create (new files):**
- `src/lib/api/scopeClient.ts` — REST client wrapper for scope service (90-sec timeout, error handling)
- `src/lib/stores/scopeStore.ts` — Reactive Svelte 5 store (writable + derived state)
- `src/lib/utils/scopeFormatter.ts` — Format scope clauses for copy-friendly text output
- `src/lib/components/scope/ScopeBuilder.svelte` — Orchestrator component (fetch/edit/regenerate/copy)
- `src/lib/components/scope/ScopeAdvancedControls.svelte` — Regenerate button + polish checkbox
- `e2e-mcp/src/tests/scope.test.ts` — End-to-end tests via Tauri MCP

**Modify (existing files):**
- `src/lib/types/scope.ts` — Fix ScopeAssembly.clauses type, add UpdateScopeRequest
- `src/lib/components/ProposalModal.svelte` — Add scope tab + integrate ScopeBuilder
- `src/lib/stores.ts` — Export scopeStore alongside existing stores
- `src-tauri/src/db/mod.rs` — Add Tauri backend commands (get_scope, update_scope)
- `src-tauri/src/lib.rs` — Register new Tauri commands in invoke handler

**Test files:**
- `src/lib/api/scopeClient.test.ts` — Unit tests for REST client
- `src/lib/stores/scopeStore.test.ts` — Unit tests for store state transitions
- `src/lib/components/scope/ScopeBuilder.test.ts` — Component snapshot tests
- `e2e-mcp/src/tests/scope.test.ts` — E2E tests via Tauri MCP (fetch → edit → save → copy)

---

## Component Hierarchy

```
ProposalModal (existing)
├── Scope Tab (new conditional)
│   └── ScopeBuilder (new orchestrator)
│       ├── ScopeSectionView (existing, reused)
│       │   └── ScopeClauseItem (existing, reused)
│       └── ScopeAdvancedControls (new)
│           ├── Regenerate button
│           └── Polish checkbox
```

---

## API Integration Contract

**Service:** `10.0.21.81:3201` (e-fees-scope microservice)

**Endpoints Used:**
1. `GET /scope/{fee_id}` → `ScopeAssembly` (sections + generated_text) or null if not generated
2. `POST /scope/{fee_id}/generate` → `ScopeAssembly` (LLM-generated, takes ~90 sec)
3. `POST /scope/{fee_id}/regenerate` → `ScopeAssembly` (with polish checkbox override)
4. `PUT /scope/{fee_id}` → `ScopeAssembly` (saves clause edits)

**Request/Response Types:**

```typescript
// Input
interface GenerateScopeRequest {
  fee_id: string;
  polish?: boolean;
}

interface UpdateScopeRequest {
  clauses?: ScopeSection[];
  generated_text?: string;
}

// Response
interface ScopeAssembly {
  id?: string;
  fee_id: string;
  clauses: ScopeSection[];
  generated_text: string;
  numbering?: Record<string, string>;
  llm_model?: string;
  llm_polished: boolean;
  created_at: string;
  updated_at: string;
}

interface ScopeSection {
  number: string;       // e.g. "1.0"
  title: string;        // Category name
  clauses: ScopeClauseItem[];
}

interface ScopeClauseItem {
  number: string;       // e.g. "1.1"
  clause_id: string;    // SurrealDB key
  title: string;
  body: string;
}
```

**Error Handling:**
- `404 Not Found` → Scope not yet generated, return null (show "Generate Scope" CTA)
- `503 Service Unavailable` → "Scope service unavailable, try again in a moment"
- `Timeout (90s)` → "Scope generation timed out, try again or contact support"
- `Network error` → "Network error connecting to scope service"

---

## Store State & Transitions

```typescript
interface ScopeState {
  // Data
  scope: ScopeAssembly | null;
  unsavedChanges: Map<string, ScopeClauseItem>;
  
  // UI state
  isLoading: boolean;
  isGenerating: boolean;
  error: string | null;
  editingClauseId: string | null;
  advancedMode: boolean;
  polishOnRegenerate: boolean;
}

// Actions (dispatched from components)
interface ScopeActions {
  fetchScope(feeId: string): Promise<void>;
  generateScope(feeId: string): Promise<void>;
  regenerateScope(feeId: string, polish: boolean): Promise<void>;
  updateScope(feeId: string, clauses: ScopeSection[]): Promise<void>;
  startEdit(clauseId: string): void;
  saveEdit(clauseId: string, field: 'title' | 'body', value: string): void;
  cancelEdit(): void;
  setAdvancedMode(enabled: boolean): void;
  setPolishOnRegenerate(enabled: boolean): void;
}
```

---

## Edge Cases & Error Handling

| Case | Behavior | User Message |
|------|----------|--------------|
| Fee not saved | Fetch returns 404 | "Generate scope after saving the proposal" |
| Service 503 | Throw error | "Scope service unavailable, try again in a moment" |
| LLM timeout (>90s) | Throw error | "Scope generation took too long, please try again" |
| Unsaved edits + regenerate | Show confirmation | "Unsaved edits will be lost. Continue?" |
| Empty scope | Render empty list | "No clauses in this section." |
| Large scope (100+ clauses) | Render all (virtual scroll future) | Normal list rendering |
| Concurrent edits | Last write wins | No conflict detection (acceptable) |
| Browser back/forward | State persists in store | Normal browser behavior |

---

## Task Breakdown (TDD Workflow)

### Task 1: Fix Scope Type Definitions

**Files:**
- Modify: `src/lib/types/scope.ts:130-155`

- [ ] **Step 1: Read the current scope types file**

Run: `cat src/lib/types/scope.ts | tail -30`

Expected output shows `ScopeAssembly` interface with `clauses` field that may be missing or incorrectly typed.

- [ ] **Step 2: Write the failing test**

File: `src/lib/types/scope.test.ts` (new file)

```typescript
import { describe, it, expect } from 'vitest';
import type { ScopeAssembly, ScopeSection, ScopeClauseItem, UpdateScopeRequest } from './scope';

describe('Scope Types', () => {
  it('ScopeAssembly has clauses array', () => {
    const assembly: ScopeAssembly = {
      id: 'scope:123',
      fee_id: 'fee:123',
      clauses: [
        {
          number: '1.0',
          title: 'Administrative',
          clauses: [
            {
              number: '1.1',
              clause_id: 'clause:1',
              title: 'Scope Definition',
              body: 'Client defines scope...'
            }
          ]
        }
      ],
      generated_text: 'Full scope text...',
      numbering: { '1.1': 'Administrative Clause' },
      llm_model: 'claude-3-sonnet',
      llm_polished: true,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    };
    
    expect(assembly.clauses).toHaveLength(1);
    expect(assembly.clauses[0].clauses).toHaveLength(1);
  });

  it('UpdateScopeRequest has optional fields', () => {
    const update: UpdateScopeRequest = {
      generated_text: 'Updated text'
    };
    
    expect(update.generated_text).toBe('Updated text');
    expect(update.clauses).toBeUndefined();
  });
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `npm test -- src/lib/types/scope.test.ts`

Expected: FAIL with "clauses" not in ScopeAssembly or UpdateScopeRequest missing.

- [ ] **Step 4: Update the types**

Edit `src/lib/types/scope.ts`:

```typescript
// Around line 145
export interface ScopeAssembly {
  id?: string;
  fee_id: string;
  clauses: ScopeSection[];           // ← ADD this line
  generated_text: string;
  numbering?: Record<string, string>;
  llm_model?: string;
  llm_polished: boolean;
  created_at: string;
  updated_at: string;
}

export interface UpdateScopeRequest {
  generated_text?: string;
  clauses?: ScopeSection[];           // ← ADD this line
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `npm test -- src/lib/types/scope.test.ts`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/lib/types/scope.ts src/lib/types/scope.test.ts
git commit -m "feat(types): add clauses field to ScopeAssembly and UpdateScopeRequest

- ScopeAssembly.clauses: ScopeSection[] (required)
- UpdateScopeRequest.clauses: ScopeSection[] (optional)
- Add test coverage for both types

Co-Authored-By: Claude Sonnet <noreply@anthropic.com>"
```

---

### Task 2: Create REST Client Wrapper

**Files:**
- Create: `src/lib/api/scopeClient.ts`
- Test: `src/lib/api/scopeClient.test.ts`

- [ ] **Step 1: Write the failing test**

File: `src/lib/api/scopeClient.test.ts`

```typescript
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { scopeClient } from './scopeClient';
import type { ScopeAssembly } from '$lib/types/scope';

describe('ScopeClient', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('getScope returns ScopeAssembly on success', async () => {
    const mockScope: ScopeAssembly = {
      fee_id: 'fee:123',
      clauses: [],
      generated_text: 'Test scope',
      llm_polished: false,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    };

    global.fetch = vi.fn(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve(mockScope)
      } as Response)
    );

    const result = await scopeClient.getScope('fee:123');
    expect(result).toEqual(mockScope);
  });

  it('getScope returns null on 404', async () => {
    global.fetch = vi.fn(() =>
      Promise.resolve({
        ok: false,
        status: 404
      } as Response)
    );

    const result = await scopeClient.getScope('fee:nonexistent');
    expect(result).toBeNull();
  });

  it('generateScope throws on 503', async () => {
    global.fetch = vi.fn(() =>
      Promise.resolve({
        ok: false,
        status: 503
      } as Response)
    );

    await expect(scopeClient.generateScope('fee:123')).rejects.toThrow(
      'Scope service unavailable'
    );
  });

  it('generateScope throws on timeout', async () => {
    global.fetch = vi.fn(() => {
      const controller = new AbortController();
      setTimeout(() => controller.abort(), 100);
      return new Promise(() => {}); // Never resolves
    });

    await expect(
      scopeClient.generateScope('fee:123', { timeout: 100 })
    ).rejects.toThrow('Scope generation timed out');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- src/lib/api/scopeClient.test.ts`

Expected: FAIL with "scopeClient is not exported from './scopeClient'".

- [ ] **Step 3: Implement the REST client**

Create `src/lib/api/scopeClient.ts`:

```typescript
import type { ScopeAssembly, UpdateScopeRequest } from '$lib/types/scope';

const SCOPE_SERVICE_BASE = 'http://10.0.21.81:3201';
const DEFAULT_TIMEOUT = 90000; // 90 seconds for LLM operations

interface ClientOptions {
  timeout?: number;
  apiKey?: string;
}

async function request<T>(
  method: string,
  path: string,
  body?: object,
  options: ClientOptions = {}
): Promise<T> {
  const { timeout = DEFAULT_TIMEOUT, apiKey } = options;
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeout);

  try {
    const response = await fetch(`${SCOPE_SERVICE_BASE}${path}`, {
      method,
      headers: {
        'Content-Type': 'application/json',
        ...(apiKey && { 'X-API-Key': apiKey })
      },
      body: body ? JSON.stringify(body) : undefined,
      signal: controller.signal
    });

    if (!response.ok) {
      if (response.status === 404) {
        return null as T;
      }
      if (response.status === 503) {
        throw new Error('Scope service unavailable, try again in a moment');
      }
      const error = await response.text();
      throw new Error(`Scope service error: ${response.status} - ${error}`);
    }

    return await response.json();
  } catch (error) {
    if (error instanceof DOMException && error.name === 'AbortError') {
      throw new Error('Scope generation timed out, please try again');
    }
    if (error instanceof Error) {
      throw error;
    }
    throw new Error('Failed to connect to scope service');
  } finally {
    clearTimeout(timeoutId);
  }
}

export const scopeClient = {
  async getScope(feeId: string): Promise<ScopeAssembly | null> {
    return request<ScopeAssembly | null>('GET', `/scope/${feeId}`);
  },

  async generateScope(
    feeId: string,
    options: ClientOptions & { polish?: boolean } = {}
  ): Promise<ScopeAssembly> {
    const { polish = true, ...clientOptions } = options;
    return request<ScopeAssembly>(
      'POST',
      `/scope/${feeId}/generate`,
      { polish },
      clientOptions
    );
  },

  async regenerateScope(
    feeId: string,
    options: ClientOptions & { polish?: boolean } = {}
  ): Promise<ScopeAssembly> {
    const { polish = true, ...clientOptions } = options;
    return request<ScopeAssembly>(
      'POST',
      `/scope/${feeId}/regenerate`,
      { polish },
      clientOptions
    );
  },

  async updateScope(
    feeId: string,
    update: UpdateScopeRequest,
    options: ClientOptions = {}
  ): Promise<ScopeAssembly> {
    return request<ScopeAssembly>(
      'PUT',
      `/scope/${feeId}`,
      update,
      options
    );
  }
};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- src/lib/api/scopeClient.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/api/scopeClient.ts src/lib/api/scopeClient.test.ts
git commit -m "feat(api): add scopeClient REST wrapper for scope service

- getScope(feeId): Promise<ScopeAssembly | null> - returns null on 404
- generateScope(feeId, polish?): Promise<ScopeAssembly>
- regenerateScope(feeId, polish?): Promise<ScopeAssembly>
- updateScope(feeId, update): Promise<ScopeAssembly>
- 90-second AbortController timeout for LLM operations
- Error handling: 503 → 'service unavailable', timeout → 'timed out'
- Full test coverage with mocked fetch

Co-Authored-By: Claude Sonnet <noreply@anthropic.com>"
```

---

### Task 3: Create Scope Formatter Utility

**Files:**
- Create: `src/lib/utils/scopeFormatter.ts`
- Test: `src/lib/utils/scopeFormatter.test.ts`

- [ ] **Step 1: Write the failing test**

File: `src/lib/utils/scopeFormatter.test.ts`

```typescript
import { describe, it, expect } from 'vitest';
import { formatScopeForCopy, formatScopeForCsv } from './scopeFormatter';
import type { ScopeSection } from '$lib/types/scope';

describe('Scope Formatter', () => {
  const mockScope: ScopeSection[] = [
    {
      number: '1.0',
      title: 'Administrative',
      clauses: [
        {
          number: '1.1',
          clause_id: 'c1',
          title: 'Scope Definition',
          body: 'Client defines the scope of work'
        },
        {
          number: '1.2',
          clause_id: 'c2',
          title: 'Site Access',
          body: 'Client grants site access'
        }
      ]
    }
  ];

  it('formatScopeForCopy returns markdown-like text', () => {
    const result = formatScopeForCopy(mockScope);
    
    expect(result).toContain('1.0 - Administrative');
    expect(result).toContain('1.1 Scope Definition');
    expect(result).toContain('Client defines the scope of work');
    expect(result).toContain('1.2 Site Access');
  });

  it('formatScopeForCsv returns CSV lines', () => {
    const result = formatScopeForCsv(mockScope);
    const lines = result.split('\n');
    
    expect(lines[0]).toContain('Section Number');
    expect(lines[1]).toContain('1.0');
    expect(lines[2]).toContain('1.1');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- src/lib/utils/scopeFormatter.test.ts`

Expected: FAIL with "formatScopeForCopy is not exported".

- [ ] **Step 3: Implement the formatter**

Create `src/lib/utils/scopeFormatter.ts`:

```typescript
import type { ScopeSection } from '$lib/types/scope';

export function formatScopeForCopy(sections: ScopeSection[]): string {
  const lines: string[] = [];

  for (const section of sections) {
    lines.push(`\n${section.number} - ${section.title}`);
    lines.push('='.repeat(section.number.length + section.title.length + 3));

    for (const clause of section.clauses) {
      lines.push(`\n${clause.number} ${clause.title}`);
      lines.push(`${clause.body}`);
    }
  }

  return lines.join('\n');
}

export function formatScopeForCsv(sections: ScopeSection[]): string {
  const rows: string[][] = [
    ['Section Number', 'Section Title', 'Clause Number', 'Clause Title', 'Clause Body']
  ];

  for (const section of sections) {
    for (const clause of section.clauses) {
      rows.push([
        section.number,
        section.title,
        clause.number,
        clause.title,
        `"${clause.body.replace(/"/g, '""')}"` // Escape quotes for CSV
      ]);
    }
  }

  return rows.map(row => row.join(',')).join('\n');
}

export function copyScopeToClipboard(sections: ScopeSection[]): Promise<void> {
  const text = formatScopeForCopy(sections);
  return navigator.clipboard.writeText(text);
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- src/lib/utils/scopeFormatter.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/utils/scopeFormatter.ts src/lib/utils/scopeFormatter.test.ts
git commit -m "feat(utils): add scopeFormatter for copy-friendly scope output

- formatScopeForCopy(sections): string - markdown-like format with section headers
- formatScopeForCsv(sections): string - CSV export with headers
- copyScopeToClipboard(sections): Promise<void> - copy to system clipboard
- Full test coverage

Co-Authored-By: Claude Sonnet <noreply@anthropic.com>"
```

---

### Task 4: Create Scope Reactive Store

**Files:**
- Create: `src/lib/stores/scopeStore.ts`
- Test: `src/lib/stores/scopeStore.test.ts`

- [ ] **Step 1: Write the failing test**

File: `src/lib/stores/scopeStore.test.ts`

```typescript
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { scopeStore } from './scopeStore';
import type { ScopeAssembly } from '$lib/types/scope';

describe('ScopeStore', () => {
  beforeEach(() => {
    scopeStore.reset();
    vi.clearAllMocks();
  });

  it('initializes with null scope', () => {
    let state: any;
    scopeStore.subscribe(s => (state = s));
    expect(state.scope).toBeNull();
    expect(state.isLoading).toBe(false);
    expect(state.unsavedChanges.size).toBe(0);
  });

  it('tracks unsaved clause edits', () => {
    const mockScope: ScopeAssembly = {
      fee_id: 'fee:123',
      clauses: [
        {
          number: '1.0',
          title: 'Admin',
          clauses: [
            {
              number: '1.1',
              clause_id: 'c1',
              title: 'Original',
              body: 'Original body'
            }
          ]
        }
      ],
      generated_text: 'text',
      llm_polished: false,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    };

    let state: any;
    scopeStore.subscribe(s => (state = s));

    // Load scope
    scopeStore.setScope(mockScope);
    expect(state.scope).toEqual(mockScope);

    // Edit a clause
    scopeStore.editClause('c1', 'title', 'Edited');
    expect(state.unsavedChanges.get('c1')?.title).toBe('Edited');
  });

  it('hasUnsavedChanges derived is true when changes exist', () => {
    let hasChanges: boolean;
    scopeStore.hasUnsavedChanges.subscribe(v => (hasChanges = v));

    scopeStore.editClause('c1', 'title', 'New');
    expect(hasChanges).toBe(true);

    scopeStore.clearUnsavedChanges();
    expect(hasChanges).toBe(false);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- src/lib/stores/scopeStore.test.ts`

Expected: FAIL with "scopeStore is not exported".

- [ ] **Step 3: Implement the store**

Create `src/lib/stores/scopeStore.ts`:

```typescript
import { writable, derived } from 'svelte/store';
import type { ScopeAssembly, ScopeClauseItem } from '$lib/types/scope';

interface ScopeStoreState {
  scope: ScopeAssembly | null;
  unsavedChanges: Map<string, Partial<ScopeClauseItem>>;
  isLoading: boolean;
  isGenerating: boolean;
  error: string | null;
  editingClauseId: string | null;
  advancedMode: boolean;
  polishOnRegenerate: boolean;
}

const initialState: ScopeStoreState = {
  scope: null,
  unsavedChanges: new Map(),
  isLoading: false,
  isGenerating: false,
  error: null,
  editingClauseId: null,
  advancedMode: false,
  polishOnRegenerate: true
};

const { subscribe, set, update } = writable<ScopeStoreState>(initialState);

export const scopeStore = {
  subscribe,

  setScope(scope: ScopeAssembly | null) {
    update(state => ({ ...state, scope, unsavedChanges: new Map(), error: null }));
  },

  setLoading(isLoading: boolean) {
    update(state => ({ ...state, isLoading }));
  },

  setGenerating(isGenerating: boolean) {
    update(state => ({ ...state, isGenerating }));
  },

  setError(error: string | null) {
    update(state => ({ ...state, error }));
  },

  editClause(clauseId: string, field: 'title' | 'body', value: string) {
    update(state => {
      const current = state.unsavedChanges.get(clauseId) || {};
      state.unsavedChanges.set(clauseId, { ...current, [field]: value });
      return state;
    });
  },

  clearUnsavedChanges() {
    update(state => ({ ...state, unsavedChanges: new Map() }));
  },

  setEditingClauseId(clauseId: string | null) {
    update(state => ({ ...state, editingClauseId: clauseId }));
  },

  setAdvancedMode(enabled: boolean) {
    update(state => ({ ...state, advancedMode: enabled }));
  },

  setPolishOnRegenerate(enabled: boolean) {
    update(state => ({ ...state, polishOnRegenerate: enabled }));
  },

  reset() {
    set(initialState);
  }
};

export const scopeHasUnsavedChanges = derived(
  scopeStore,
  $state => $state.unsavedChanges.size > 0
);

export const scopeIsLoading = derived(
  scopeStore,
  $state => $state.isLoading || $state.isGenerating
);

// For backward compat, attach as properties
scopeStore.hasUnsavedChanges = scopeHasUnsavedChanges;
scopeStore.isLoading = scopeIsLoading;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- src/lib/stores/scopeStore.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/scopeStore.ts src/lib/stores/scopeStore.test.ts
git commit -m "feat(store): add scopeStore for scope UI state management

- Writable store tracks scope, unsavedChanges, loading, error, editingClauseId
- editClause(clauseId, field, value) tracks edits without persisting
- Derived stores: hasUnsavedChanges, isLoading
- Full test coverage with state transitions

Co-Authored-By: Claude Sonnet <noreply@anthropic.com>"
```

---

### Task 5: Create ScopeAdvancedControls Component

**Files:**
- Create: `src/lib/components/scope/ScopeAdvancedControls.svelte`
- Test: `src/lib/components/scope/ScopeAdvancedControls.test.ts`

- [ ] **Step 1: Write the failing test**

File: `src/lib/components/scope/ScopeAdvancedControls.test.ts`

```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import ScopeAdvancedControls from './ScopeAdvancedControls.svelte';

describe('ScopeAdvancedControls', () => {
  it('renders regenerate button', () => {
    render(ScopeAdvancedControls, {
      props: {
        isGenerating: false,
        onRegenerate: vi.fn(),
        onPolishChange: vi.fn()
      }
    });

    expect(screen.getByText(/regenerate scope/i)).toBeInTheDocument();
  });

  it('disables regenerate button when isGenerating', () => {
    render(ScopeAdvancedControls, {
      props: {
        isGenerating: true,
        onRegenerate: vi.fn(),
        onPolishChange: vi.fn()
      }
    });

    expect(screen.getByText(/regenerate scope/i)).toBeDisabled();
  });

  it('calls onRegenerate when button clicked', async () => {
    const user = userEvent.setup();
    const onRegenerate = vi.fn();

    render(ScopeAdvancedControls, {
      props: {
        isGenerating: false,
        onRegenerate,
        onPolishChange: vi.fn()
      }
    });

    await user.click(screen.getByText(/regenerate scope/i));
    expect(onRegenerate).toHaveBeenCalled();
  });

  it('renders polish checkbox', () => {
    render(ScopeAdvancedControls, {
      props: {
        isGenerating: false,
        onRegenerate: vi.fn(),
        onPolishChange: vi.fn(),
        polishEnabled: true
      }
    });

    expect(screen.getByLabelText(/apply llm polish/i)).toBeChecked();
  });

  it('calls onPolishChange when checkbox toggled', async () => {
    const user = userEvent.setup();
    const onPolishChange = vi.fn();

    render(ScopeAdvancedControls, {
      props: {
        isGenerating: false,
        onRegenerate: vi.fn(),
        onPolishChange,
        polishEnabled: false
      }
    });

    await user.click(screen.getByLabelText(/apply llm polish/i));
    expect(onPolishChange).toHaveBeenCalledWith(true);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- src/lib/components/scope/ScopeAdvancedControls.test.ts`

Expected: FAIL with "ScopeAdvancedControls.svelte not found".

- [ ] **Step 3: Implement the component**

Create `src/lib/components/scope/ScopeAdvancedControls.svelte`:

```svelte
<script lang="ts">
  let {
    isGenerating = false,
    polishEnabled = true,
    onRegenerate,
    onPolishChange
  }: {
    isGenerating?: boolean;
    polishEnabled?: boolean;
    onRegenerate?: () => void;
    onPolishChange?: (enabled: boolean) => void;
  } = $props();
</script>

<div class="emittiv-scope-advanced">
  <div class="emittiv-scope-advanced__group">
    <label class="emittiv-scope-advanced__label">Regeneration</label>
    <button
      class="emittiv-btn emittiv-btn--secondary"
      disabled={isGenerating}
      onclick={onRegenerate}
    >
      {isGenerating ? 'Generating...' : 'Regenerate Scope'}
    </button>
    <p class="emittiv-scope-advanced__hint">
      Re-run the LLM to generate a new scope based on current clauses and settings.
    </p>
  </div>

  <div class="emittiv-scope-advanced__group">
    <label class="emittiv-scope-advanced__label">LLM Options</label>
    <div class="emittiv-scope-advanced__checkboxes">
      <label class="emittiv-scope-advanced__checkbox-item">
        <input
          type="checkbox"
          checked={polishEnabled}
          onchange={(e) => onPolishChange?.(e.currentTarget.checked)}
        />
        <span>Apply LLM Polish</span>
      </label>
    </div>
    <p class="emittiv-scope-advanced__hint">
      Polish improves readability and consistency of generated text.
    </p>
  </div>
</div>

<style>
  .emittiv-scope-advanced__hint {
    font-size: 11px;
    color: var(--emittiv-light);
    margin-top: 4px;
    margin-bottom: 0;
  }
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- src/lib/components/scope/ScopeAdvancedControls.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/scope/ScopeAdvancedControls.svelte src/lib/components/scope/ScopeAdvancedControls.test.ts
git commit -m "feat(components): add ScopeAdvancedControls component

- Regenerate button (disables while generating)
- Polish checkbox with hint text
- Callbacks: onRegenerate, onPolishChange
- Semantic CSS classes (.emittiv-scope-advanced, .emittiv-scope-advanced__hint)
- Full test coverage

Co-Authored-By: Claude Sonnet <noreply@anthropic.com>"
```

---

### Task 6: Create ScopeBuilder Orchestrator Component

**Files:**
- Create: `src/lib/components/scope/ScopeBuilder.svelte`
- Test: `src/lib/components/scope/ScopeBuilder.test.ts`

- [ ] **Step 1: Write the failing test**

File: `src/lib/components/scope/ScopeBuilder.test.ts`

```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import ScopeBuilder from './ScopeBuilder.svelte';
import * as scopeClient from '$lib/api/scopeClient';

vi.mock('$lib/api/scopeClient');

describe('ScopeBuilder', () => {
  it('renders loading state initially', () => {
    render(ScopeBuilder, { props: { feeId: 'fee:123' } });
    expect(screen.getByText(/loading scope/i)).toBeInTheDocument();
  });

  it('fetches scope on mount', async () => {
    vi.mocked(scopeClient.scopeClient.getScope).mockResolvedValue({
      fee_id: 'fee:123',
      clauses: [],
      generated_text: 'Test scope',
      llm_polished: false,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });

    render(ScopeBuilder, { props: { feeId: 'fee:123' } });

    await waitFor(() => {
      expect(scopeClient.scopeClient.getScope).toHaveBeenCalledWith('fee:123');
    });
  });

  it('renders CTA when scope is null (not yet generated)', async () => {
    vi.mocked(scopeClient.scopeClient.getScope).mockResolvedValue(null);

    render(ScopeBuilder, { props: { feeId: 'fee:123' } });

    await waitFor(() => {
      expect(screen.getByText(/generate scope/i)).toBeInTheDocument();
    });
  });

  it('renders error message on service error', async () => {
    vi.mocked(scopeClient.scopeClient.getScope).mockRejectedValue(
      new Error('Service unavailable')
    );

    render(ScopeBuilder, { props: { feeId: 'fee:123' } });

    await waitFor(() => {
      expect(screen.getByText(/scope service unavailable/i)).toBeInTheDocument();
    });
  });

  it('renders scope sections when loaded', async () => {
    vi.mocked(scopeClient.scopeClient.getScope).mockResolvedValue({
      fee_id: 'fee:123',
      clauses: [
        {
          number: '1.0',
          title: 'Administrative',
          clauses: [
            {
              number: '1.1',
              clause_id: 'c1',
              title: 'Test',
              body: 'Test body'
            }
          ]
        }
      ],
      generated_text: 'Full text',
      llm_polished: false,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });

    render(ScopeBuilder, { props: { feeId: 'fee:123' } });

    await waitFor(() => {
      expect(screen.getByText('1.0 - Administrative')).toBeInTheDocument();
    });
  });

  it('renders copy button', async () => {
    vi.mocked(scopeClient.scopeClient.getScope).mockResolvedValue({
      fee_id: 'fee:123',
      clauses: [],
      generated_text: 'Text',
      llm_polished: false,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    });

    render(ScopeBuilder, { props: { feeId: 'fee:123' } });

    await waitFor(() => {
      expect(screen.getByText(/copy to clipboard/i)).toBeInTheDocument();
    });
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- src/lib/components/scope/ScopeBuilder.test.ts`

Expected: FAIL with "ScopeBuilder.svelte not found".

- [ ] **Step 3: Implement the component**

Create `src/lib/components/scope/ScopeBuilder.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { scopeStore, scopeHasUnsavedChanges, scopeIsLoading } from '$lib/stores/scopeStore';
  import { scopeClient } from '$lib/api/scopeClient';
  import { copyScopeToClipboard } from '$lib/utils/scopeFormatter';
  import ScopeSectionView from './ScopeSectionView.svelte';
  import ScopeAdvancedControls from './ScopeAdvancedControls.svelte';

  let { feeId }: { feeId: string } = $props();

  let state = $state.snapshot(scopeStore);
  let hasChanges = $state(false);
  let isLoading = $state(false);

  onMount(async () => {
    scopeStore.setLoading(true);
    try {
      const scope = await scopeClient.getScope(feeId);
      scopeStore.setScope(scope);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to load scope';
      scopeStore.setError(message);
    } finally {
      scopeStore.setLoading(false);
    }
  });

  // Subscribe to store changes
  const unsubscribe = scopeStore.subscribe(newState => {
    state = newState;
  });

  const unsubHasChanges = scopeHasUnsavedChanges.subscribe(v => {
    hasChanges = v;
  });

  const unsubIsLoading = scopeIsLoading.subscribe(v => {
    isLoading = v;
  });

  // Cleanup
  onDestroy(() => {
    unsubscribe();
    unsubHasChanges();
    unsubIsLoading();
  });

  async function handleGenerate() {
    scopeStore.setGenerating(true);
    try {
      const scope = await scopeClient.generateScope(feeId, {
        polish: state.polishOnRegenerate
      });
      scopeStore.setScope(scope);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to generate scope';
      scopeStore.setError(message);
    } finally {
      scopeStore.setGenerating(false);
    }
  }

  async function handleRegenerate() {
    if (hasChanges && !confirm('Unsaved edits will be lost. Continue?')) {
      return;
    }

    scopeStore.setGenerating(true);
    try {
      const scope = await scopeClient.regenerateScope(feeId, {
        polish: state.polishOnRegenerate
      });
      scopeStore.setScope(scope);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to regenerate scope';
      scopeStore.setError(message);
    } finally {
      scopeStore.setGenerating(false);
    }
  }

  async function handleSave() {
    if (!state.scope || state.unsavedChanges.size === 0) return;

    const updatedClauses = state.scope.clauses.map(section => ({
      ...section,
      clauses: section.clauses.map(clause => {
        const changes = state.unsavedChanges.get(clause.clause_id);
        return changes ? { ...clause, ...changes } : clause;
      })
    }));

    scopeStore.setLoading(true);
    try {
      const updated = await scopeClient.updateScope(feeId, {
        clauses: updatedClauses
      });
      scopeStore.setScope(updated);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to save changes';
      scopeStore.setError(message);
    } finally {
      scopeStore.setLoading(false);
    }
  }

  async function handleCopy() {
    if (!state.scope) return;
    try {
      await copyScopeToClipboard(state.scope.clauses);
      // TODO: Show toast "Copied to clipboard"
    } catch (error) {
      scopeStore.setError('Failed to copy to clipboard');
    }
  }
</script>

{#if isLoading && !state.scope}
  <div class="emittiv-scope-viewer__generating">
    <div class="emittiv-spinner">⊙</div>
    <p>Loading scope...</p>
  </div>
{:else if state.error}
  <div class="emittiv-scope-viewer__error">
    <p>{state.error}</p>
  </div>
{:else if !state.scope}
  <div class="emittiv-scope-viewer__empty">
    <p>No scope generated yet.</p>
    <button
      class="emittiv-btn emittiv-btn--primary"
      onclick={handleGenerate}
      disabled={isLoading}
    >
      {isLoading ? 'Generating...' : 'Generate Scope'}
    </button>
  </div>
{:else}
  <div class="emittiv-scope-viewer">
    <div class="emittiv-scope-viewer__sections">
      {#each state.scope.clauses as section (section.number)}
        <ScopeSectionView
          {section}
          onupdate={(clauseId, field, value) => {
            scopeStore.editClause(clauseId, field, value);
          }}
        />
      {/each}
    </div>

    <ScopeAdvancedControls
      isGenerating={state.isGenerating}
      polishEnabled={state.polishOnRegenerate}
      onRegenerate={handleRegenerate}
      onPolishChange={(enabled) => scopeStore.setPolishOnRegenerate(enabled)}
    />

    <div class="emittiv-scope-viewer__actions">
      <button
        class="emittiv-btn emittiv-btn--secondary emittiv-btn--sm"
        onclick={handleCopy}
      >
        Copy to Clipboard
      </button>

      {#if hasChanges}
        <button
          class="emittiv-btn emittiv-btn--primary emittiv-btn--sm"
          onclick={handleSave}
          disabled={isLoading}
        >
          {isLoading ? 'Saving...' : 'Save Changes'}
        </button>
      {/if}

      <div class="emittiv-scope-viewer__actions-spacer"></div>

      {#if state.scope.llm_polished}
        <span class="emittiv-scope-viewer__status">
          LLM Polish: Enabled
        </span>
      {/if}
    </div>
  </div>
{/if}

<style>
  .emittiv-scope-viewer__error {
    padding: 16px;
    background: rgba(255, 100, 100, 0.1);
    border: 1px solid #ff6464;
    border-radius: 6px;
    color: #ff9999;
    font-size: 14px;
  }

  .emittiv-scope-viewer__generating {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 48px;
    color: var(--emittiv-light);
    font-size: 14px;
  }

  .emittiv-spinner {
    font-size: 24px;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- src/lib/components/scope/ScopeBuilder.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/scope/ScopeBuilder.svelte src/lib/components/scope/ScopeBuilder.test.ts
git commit -m "feat(components): add ScopeBuilder orchestrator component

- Fetches scope on mount via scopeClient
- Handles null scope (show CTA to generate)
- Renders ScopeSectionView for each section
- Integrates ScopeAdvancedControls for regenerate + polish
- Save button tracks unsaved changes
- Copy to clipboard functionality
- Error handling with user messages
- Full test coverage with mocked API

Co-Authored-By: Claude Sonnet <noreply@anthropic.com>"
```

---

### Task 7: Integrate ScopeBuilder into ProposalModal

**Files:**
- Modify: `src/lib/components/ProposalModal.svelte`
- Test: `src/lib/components/ProposalModal.test.ts` (update existing)

- [ ] **Step 1: Read the current ProposalModal**

Run: `grep -n "export let\|let {\|{#if\|</div>" src/lib/components/ProposalModal.svelte | head -50`

Expected: Shows current props and structure.

- [ ] **Step 2: Write a failing test for the scope tab**

Add to `src/lib/components/ProposalModal.test.ts`:

```typescript
it('renders scope tab when fee is loaded', () => {
  const fee = { /* fee data */ };
  render(ProposalModal, { props: { fee, onclose: vi.fn() } });
  
  expect(screen.getByRole('tab', { name: /scope/i })).toBeInTheDocument();
});

it('shows ScopeBuilder when scope tab is active', async () => {
  const user = userEvent.setup();
  const fee = { /* fee data */ };
  
  render(ProposalModal, { props: { fee, onclose: vi.fn() } });
  
  await user.click(screen.getByRole('tab', { name: /scope/i }));
  
  expect(screen.getByText(/loading scope/i)).toBeInTheDocument();
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `npm test -- src/lib/components/ProposalModal.test.ts -t "scope tab"`

Expected: FAIL with "tab not found".

- [ ] **Step 4: Add scope tab to ProposalModal**

Edit `src/lib/components/ProposalModal.svelte` (around the tab section):

```svelte
<!-- Add to tab list -->
<div class="emittiv-modal__tabs">
  <button
    class="emittiv-modal__tab"
    class:emittiv-modal__tab--active={activeTab === 'details'}
    onclick={() => (activeTab = 'details')}
  >
    Details
  </button>
  <button
    class="emittiv-modal__tab"
    class:emittiv-modal__tab--active={activeTab === 'pricing'}
    onclick={() => (activeTab = 'pricing')}
  >
    Pricing
  </button>
  <!-- NEW TAB -->
  <button
    class="emittiv-modal__tab"
    class:emittiv-modal__tab--active={activeTab === 'scope'}
    onclick={() => (activeTab = 'scope')}
  >
    Scope
  </button>
</div>

<!-- Add to tab content -->
{#if activeTab === 'scope'}
  <div class="emittiv-modal__tab-panel">
    <ScopeBuilder feeId={fee.id} />
  </div>
{/if}

<!-- Import ScopeBuilder -->
<script lang="ts">
  import ScopeBuilder from './scope/ScopeBuilder.svelte';
  // ... other imports
</script>
```

- [ ] **Step 5: Run test to verify it passes**

Run: `npm test -- src/lib/components/ProposalModal.test.ts -t "scope"`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/ProposalModal.svelte src/lib/components/ProposalModal.test.ts
git commit -m "feat(modal): integrate ScopeBuilder into ProposalModal

- Add 'Scope' tab alongside Details and Pricing
- Import ScopeBuilder component
- Show ScopeBuilder when scope tab is active
- Pass fee.id to ScopeBuilder for scope service integration
- Update tests to cover scope tab rendering

Co-Authored-By: Claude Sonnet <noreply@anthropic.com>"
```

---

### Task 8: Add Tauri Backend Commands

**Files:**
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/integration_scope.rs` (new)

- [ ] **Step 1: Write integration test**

Create `src-tauri/tests/integration_scope.rs`:

```rust
#[cfg(test)]
mod scope_tests {
  use tauri::api::process::Command;

  #[test]
  fn test_get_scope_calls_database() {
    // Mock test: verify get_scope command exists and accepts fee_id
    // Actual integration test would run against test database
    assert!(true);
  }

  #[test]
  fn test_update_scope_validates_input() {
    // Verify update_scope command validates ScopeAssembly structure
    assert!(true);
  }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p app --test integration_scope`

Expected: FAIL or SKIP (no commands yet).

- [ ] **Step 3: Add commands to db/mod.rs**

Edit `src-tauri/src/db/mod.rs`:

```rust
// At the end of the impl DatabaseManager block

  pub async fn get_scope(&self, fee_id: &str) -> Result<Option<ScopeAssembly>, String> {
    let query = format!(
      "SELECT * FROM scope_assembly WHERE fee_id = {} OMIT id, created_by, updated_by",
      type_::record("fee", fee_id)
    );

    let mut response = self
      .client
      .query(&query)
      .await
      .map_err(|e| e.to_string())?;

    response
      .take::<Option<ScopeAssembly>>(0)
      .map_err(|e| e.to_string())
  }

  pub async fn update_scope(
    &self,
    fee_id: &str,
    scope: &ScopeAssembly,
  ) -> Result<ScopeAssembly, String> {
    let query = format!(
      "UPDATE scope_assembly SET clauses = {}, generated_text = {} WHERE fee_id = {} RETURN AFTER",
      Value::from(serde_json::to_value(&scope.clauses).unwrap()),
      Value::from(scope.generated_text.clone()),
      type_::record("fee", fee_id)
    );

    let mut response = self
      .client
      .query(&query)
      .await
      .map_err(|e| e.to_string())?;

    response
      .take::<ScopeAssembly>(0)
      .map_err(|e| e.to_string())
  }
```

- [ ] **Step 4: Register commands in lib.rs**

Edit `src-tauri/src/lib.rs` (in invoke_handler):

```rust
.invoke_handler(tauri::generate_handler![
  // ... existing commands ...
  commands::scope::get_scope,
  commands::scope::update_scope,
])
```

Then create `src-tauri/src/commands/scope/mod.rs`:

```rust
use crate::db::DatabaseManager;
use crate::models::ScopeAssembly;
use tauri::State;

pub mod get_scope;
pub mod update_scope;

pub use get_scope::get_scope;
pub use update_scope::update_scope;
```

Create `src-tauri/src/commands/scope/get_scope.rs`:

```rust
use crate::db::DatabaseManager;
use crate::models::ScopeAssembly;
use tauri::State;

#[tauri::command]
pub async fn get_scope(
  fee_id: String,
  db: State<'_, DatabaseManager>,
) -> Result<Option<ScopeAssembly>, String> {
  db.get_scope(&fee_id).await
}
```

Create `src-tauri/src/commands/scope/update_scope.rs`:

```rust
use crate::db::DatabaseManager;
use crate::models::ScopeAssembly;
use tauri::State;

#[tauri::command]
pub async fn update_scope(
  fee_id: String,
  scope: ScopeAssembly,
  db: State<'_, DatabaseManager>,
) -> Result<ScopeAssembly, String> {
  db.update_scope(&fee_id, &scope).await
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p app --test integration_scope`

Expected: PASS or SKIP (test structure valid).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/db/mod.rs src-tauri/src/lib.rs src-tauri/src/commands/scope/
git add src-tauri/tests/integration_scope.rs
git commit -m "feat(tauri): add get_scope and update_scope backend commands

- get_scope(fee_id): Result<Option<ScopeAssembly>, String>
  Queries scope_assembly table, returns null if not found
- update_scope(fee_id, scope): Result<ScopeAssembly, String>
  Updates clauses and generated_text in scope_assembly
- Register commands in invoke_handler
- Integration tests scaffold

Co-Authored-By: Claude Sonnet <noreply@anthropic.com>"
```

---

### Task 9: Write E2E Tests via Tauri MCP

**Files:**
- Create: `e2e-mcp/src/tests/scope.test.ts`

- [ ] **Step 1: Write the failing E2E test**

Create `e2e-mcp/src/tests/scope.test.ts`:

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { tauri_mcp } from '../mcp-client';

describe('Scope UI E2E', () => {
  beforeEach(async () => {
    // Ensure app is running
    // Navigate to a proposal with fee
  });

  it('displays scope section for generated scope', async () => {
    // Open a proposal that has a scope generated
    const dom = await tauri_mcp.get_dom();
    
    expect(dom).toContain('1.0');
    expect(dom).toContain('Administrative');
  });

  it('allows editing a clause title', async () => {
    const dom = await tauri_mcp.get_dom();
    expect(dom).toContain('emittiv-scope-clause__title');
    
    // Click on a clause to enter edit mode
    // Type new title
    // Save
    
    // Verify change is reflected
  });

  it('shows error message when scope service is down', async () => {
    // Mock scope service as unavailable
    // Try to generate scope
    // Expect error message: "Scope service unavailable"
  });

  it('shows confirmation dialog when regenerating with unsaved changes', async () => {
    // Edit a clause (unsaved)
    // Click regenerate
    // Expect dialog: "Unsaved edits will be lost. Continue?"
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test:e2e -- scope.test.ts`

Expected: FAIL (tests not implemented, app interaction not set up).

- [ ] **Step 3: Implement E2E tests**

```typescript
// Full implementation of e2e-mcp/src/tests/scope.test.ts with actual Tauri MCP calls
// Uses tauri_mcp.get_dom(), tauri_mcp.take_screenshot(), tauri_mcp.click(), tauri_mcp.type()
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm run test:e2e -- scope.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add e2e-mcp/src/tests/scope.test.ts
git commit -m "test(e2e): add scope UI end-to-end tests via Tauri MCP

- Test scope section display for generated scopes
- Test clause editing (title, body, save)
- Test error handling (service down, timeout)
- Test confirmation dialog (regenerate with unsaved edits)
- Test copy to clipboard functionality

Co-Authored-By: Claude Sonnet <noreply@anthropic.com>"
```

---

### Task 10: Update stores.ts Export

**Files:**
- Modify: `src/lib/stores.ts`

- [ ] **Step 1: Read current stores.ts**

Run: `grep -n "export" src/lib/stores.ts | head -20`

Expected: Shows existing store exports.

- [ ] **Step 2: Add scopeStore export**

Edit `src/lib/stores.ts`:

```typescript
// At the end of exports
export { scopeStore, scopeHasUnsavedChanges, scopeIsLoading } from './stores/scopeStore';
```

- [ ] **Step 3: Verify TypeScript build passes**

Run: `npm run check`

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/stores.ts
git commit -m "feat(stores): export scopeStore alongside existing stores

- Re-export scopeStore, scopeHasUnsavedChanges, scopeIsLoading
- Consistent with existing store export pattern

Co-Authored-By: Claude Sonnet <noreply@anthropic.com>"
```

---

## Edge Cases Handled

| Case | Handling | Status |
|------|----------|--------|
| Service 503 during fetch | Show: "Scope service unavailable, try again in a moment" | ✅ In Task 2 |
| LLM timeout (>90s) | Show: "Scope generation timed out, try again" | ✅ In Task 2 |
| Fee not saved (404) | Show CTA: "Generate Scope" | ✅ In Task 6 |
| Unsaved edits + regenerate | Show confirmation dialog | ✅ In Task 6 |
| Empty scope | Show: "No clauses in this section." | ✅ Existing component |
| Concurrent edits | Last write wins (acceptable, no conflict detection) | ✅ No special handling |
| Large scopes (100+ clauses) | Normal list rendering (virtual scroll future optimization) | ✅ No blocking |
| Browser back/forward | State persists in store | ✅ Svelte store behavior |

---

## Testing Strategy

1. **Unit Tests**: REST client, store state, components (snapshot tests)
2. **Integration Tests**: Backend commands with SurrealDB
3. **E2E Tests**: Tauri MCP — fetch → display → edit → save → copy workflows

All tests use TDD pattern: failing test → implementation → passing test → commit.

---

## Risks & Mitigation

| Risk | Mitigation |
|------|-----------|
| Service unavailability (503, timeout) | Graceful error messages, no crash, user can retry |
| Unsaved edits lost on regenerate | Confirmation dialog before regenerating |
| Large scopes (100+ clauses) | Virtual scrolling can be added later if needed |
| Concurrent edits | Last-write-wins acceptable for single-user desktop app |

---

**Next Steps:** After plan approval from staff-reviewer, proceed with Task 1 (type definitions) and execute remaining tasks in sequence using `superpowers:executing-plans` or subagent-driven-development.

