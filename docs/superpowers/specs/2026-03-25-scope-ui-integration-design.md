# Scope Service UI Integration — Design Spec

**Date:** 2026-03-25
**Status:** Approved
**Author:** Claude + Martin

## Overview

Integrate the e-fees-scope service into the desktop app UI. The scope service generates structured proposal scope text (clauses grouped by category, with auto-numbering) for fee proposals. The UI allows users to generate, view, edit, save, and copy scope text — all from within the proposal workflow.

## Decisions

- **Entry point:** "Generate Scope" button in ProposalModal (edit mode only, because a saved fee ID is required)
- **Workflow:** Hybrid — one-click generate (default) with collapsible advanced assembly controls
- **Output:** Structured per-category sections with numbered clauses, inline editable
- **Actions:** View, edit, save (to scope service), copy all (to clipboard, formatted locally)
- **Architecture:** Reusable `ScopeBuilder.svelte` component in a modal, portable to a full page later

## Existing Infrastructure

All API integration code already exists:

- **HTTP client:** `src/lib/api/scope.ts` — typed wrappers for all scope service endpoints
- **Types:** `src/lib/types/scope.ts` — mirrors Rust models (needs `clauses` type fix, see below)
- **Config:** `VITE_SCOPE_API_URL` / `VITE_SCOPE_API_KEY` environment variables with defaults
- **Service:** Running at `http://10.0.21.81:3201` with Swagger at `/docs/`

## Data Model — `clauses` Structure

The scope service's `auto_number_clauses()` (`e-fees-scope/src/routes/scope.rs:46`) produces a JSON array of sections:

```typescript
// The actual shape of ScopeAssembly.clauses (currently typed as `unknown`)
interface ScopeSection {
  number: string;       // e.g. "1.0"
  title: string;        // Category name (e.g. "Administrative", "Commercial")
  clauses: ScopeClauseItem[];
}

interface ScopeClauseItem {
  number: string;       // e.g. "1.1", "1.2"
  clause_id: string;    // SurrealDB key
  title: string;        // Clause title
  body: string;         // Clause body text
}
```

**Type fix required:** Update `ScopeAssembly.clauses` from `unknown` to `ScopeSection[]` in `src/lib/types/scope.ts`. Same for `numbering` → `Record<string, string>` (maps clause_id → number).

The `generated_text` field contains the same content as a flat string (for LLM-polished output). The structured `clauses` array is the source of truth for the editable UI.

## Components

### ScopeBuilder.svelte (orchestrator)

**Props:**
- `feeId: string` — the bare fee record key (e.g., `25_97101_1`), extracted via `getEntityId()` at the call site in ProposalModal

**State:**
- `sections: ScopeSection[]` — parsed from `scope.clauses`
- `scope: ScopeAssembly | null` — loaded/generated scope data
- `loading: boolean` — initial load in progress
- `generating: boolean` — scope generation in progress (can take 30-60s)
- `saving: boolean` — save in progress
- `dirty: boolean` — unsaved edits exist
- `showAdvanced: boolean` — advanced controls visible (default: false)
- `error: string | null` — error message
- `stages: StageConfig[]` — fetched on mount for advanced controls
- `disciplines: string[]` — derived from loaded deliverables for advanced controls

**Lifecycle:**
1. On mount: call `getScope(feeId)` wrapped in try/catch
2. **If 404:** set `scope = null`, show "Generate Scope" button (not an error)
3. **If 2xx:** parse `scope.clauses` into `sections`, display structured view
4. **If other error:** show error alert
5. Also fetch `getStages()` in parallel (needed for advanced controls)

**Actions:**
- **Generate:** `POST /scope/generate` with `{ fee_id, polish: true }`. Uses `AbortController` with 90s timeout. On timeout: abort request, show "Generation timed out — the scope service may still be processing. Try again in a minute."
- **Regenerate:** confirm dialog first ("This will replace your current scope text. Continue?"), then `POST /scope/{fee_id}/regenerate`
- **Save:** reconstruct `clauses` array from edited `sections`, then `PUT /scope/{fee_id}` with `{ clauses, generated_text }` where `generated_text` is rebuilt from sections
- **Copy all:** format locally from in-memory `sections` state via `formatSectionsAsText()` → `navigator.clipboard.writeText()`

### ScopeSectionView.svelte

**Props:**
- `section: ScopeSection`
- `expanded: boolean` (default: true for first section, false for rest)
- `onupdate: (clauseIndex: number, field: 'title' | 'body', value: string) => void`

**Renders:**
- Collapsible header with section number + title + clause count badge
- Numbered list of clauses within
- Click header to expand/collapse

### ScopeClauseItem.svelte

**Props:**
- `clause: ScopeClauseItem`
- `onupdate: (field: 'title' | 'body', value: string) => void`

**Renders:**
- Number + title (bold) + body text
- Click to enter inline edit mode (textareas replace text)
- Save/cancel buttons (not keyboard shortcuts, to avoid Escape conflict with parent modal)
- `stopPropagation()` on keydown within edit mode to prevent BaseModal from intercepting Escape

### ScopeAdvancedControls.svelte

**Props:**
- `stages: StageConfig[]` — from parent (fetched on mount)
- `onassemble: (request: AssembleRequest) => void`
- `loading: boolean`

**State (internal):**
- Selected disciplines (checkboxes, derived from available deliverables)
- Selected conditions
- Selected stages

**Renders:**
- Discipline multi-select (checkboxes)
- Condition toggles
- Stage selection
- "Assemble" button
- Collapsed by default behind "Advanced options" toggle

**Data source for disciplines:** Fetched via `getDeliverables()` with `distinct_disciplines` or derived by collecting unique `discipline` values from deliverable results. Cached after first fetch.

## User Flow

```
ProposalModal (edit mode, fee has saved ID)
  └─ "Generate Scope" button (after form fields, before action bar)
      └─ Opens ScopeBuilderModal (BaseModal, size="xl", zIndex=200)
          └─ ScopeBuilder (feeId from getEntityId(proposal))
              ├─ Loading state → spinner
              ├─ No existing scope (404) → "Generate Scope" button + advanced toggle
              ├─ Existing scope → structured view
              │   ├─ Category sections (collapsible)
              │   │   └─ Clause items (inline editable, save/cancel buttons)
              │   ├─ Action bar: Save | Copy All | Regenerate (with confirm)
              │   └─ Advanced toggle (collapsed)
              └─ Error state → alert message
```

## API Calls

| Action | Endpoint | Error Handling |
|--------|----------|---------------|
| Check existing | `GET /scope/{fee_id}` | 404 → `scope = null` (not an error), other → error alert |
| One-click generate | `POST /scope/generate` | AbortController 90s timeout, show timeout message |
| Advanced assembly | `POST /scope/assemble` | Standard error alert |
| Save edits | `PUT /scope/{fee_id}` | Standard error alert, keep dirty state on failure |
| Regenerate | `POST /scope/{fee_id}/regenerate` | Confirm dialog first, then same as generate |
| Fetch stages | `GET /stages` | Fail silently (advanced controls unavailable) |

## Copy Format

"Copy All" formats locally from the in-memory `sections` array. No network call required.

```
1.0 ADMINISTRATIVE

1.1 Title — Body text of the clause
1.2 Title — Body text

2.0 COMMERCIAL

2.1 Title — Body text
2.2 Title — Body text
```

Section titles uppercase, clauses formatted as `{number} {title} — {body}`, blank line between sections. This matches the `auto_number_clauses()` raw text format from the service.

## Save Round-Trip

When the user edits clauses inline and clicks Save:

1. Rebuild `clauses: ScopeSection[]` from the edited `sections` state (direct — it's the same structure)
2. Rebuild `generated_text` by running `formatSectionsAsText(sections)` (same function used by Copy All)
3. `PUT /scope/{fee_id}` with `{ clauses, generated_text }`
4. On success: set `dirty = false`
5. On failure: keep `dirty = true`, show error

This ensures both the structured data and the flat text stay in sync.

## Styling

All components use existing `.emittiv-*` classes:

- `.emittiv-form-section` for layout
- `.emittiv-btn` variants for buttons
- `.emittiv-alert` for error/success messages
- `.emittiv-input` / `.emittiv-select` for advanced controls
- New: `.emittiv-scope-section` — collapsible section header with expand/collapse
- New: `.emittiv-scope-clause` — clause item with edit state
- New: `.emittiv-scope-clause--editing` — active edit mode styling
- Fixed `px` values for all sizing (desktop app)
- Emittiv design system colours

## File Structure

```
src/lib/components/
  scope/
    ScopeBuilder.svelte          # Orchestrator
    ScopeSectionView.svelte      # Collapsible category section
    ScopeClauseItem.svelte       # Editable clause item
    ScopeAdvancedControls.svelte # Discipline/condition selectors

src/lib/utils/
  scopeFormatter.ts              # formatSectionsAsText() — shared by Copy All and Save
```

## Type Changes Required

**`src/lib/types/scope.ts`:**
```typescript
// Add new interfaces
export interface ScopeSection {
  number: string;
  title: string;
  clauses: ScopeClauseItem[];
}

export interface ScopeClauseItem {
  number: string;
  clause_id: string;
  title: string;
  body: string;
}

// Update ScopeAssembly
export interface ScopeAssembly {
  id?: string;
  fee_id: string;
  clauses: ScopeSection[];        // was: unknown
  generated_text: string;
  numbering?: Record<string, string>;  // was: unknown
  llm_model?: string;
  llm_polished: boolean;
  created_at: string;
  updated_at: string;
}

// Update UpdateScopeRequest
export interface UpdateScopeRequest {
  generated_text?: string;
  clauses?: ScopeSection[];       // was: unknown
}
```

## API Client Changes Required

**`src/lib/api/scope.ts`:**
- `getScope()`: return `ScopeAssembly | null` — catch 404 and return `null` instead of throwing
- No other changes needed (all other functions already typed correctly)

## Entry Point Changes

**ProposalModal.svelte:**
- Add "Generate Scope" button in edit mode (after form fields, before action bar)
- Button opens `ScopeBuilderModal` (BaseModal with `size="xl"` and `zIndex={200}`)
- Only visible when fee has been saved (`mode === 'edit'`)
- Fee ID extracted via `getEntityId(proposal)` before passing to ScopeBuilder

## Edge Cases

- **Scope service unreachable:** Show error alert in ScopeBuilder, don't block ProposalModal
- **Fee not yet saved (create mode):** "Generate Scope" button not shown
- **LLM generation timeout:** AbortController at 90s, show timeout message with retry button
- **Unsaved scope edits + modal close:** Confirm dialog ("Discard unsaved scope changes?")
- **Empty scope result:** Show "No clauses generated" with suggestion to try advanced mode
- **Regenerate with existing edits:** Confirm dialog before overwriting
- **Escape key during inline edit:** `stopPropagation()` prevents parent modal close, save/cancel buttons used instead of keyboard shortcuts

## Out of Scope (Future)

- Dedicated `/scope/:id` full-page route
- Clause library management UI
- Corpus search/browse UI
- Direct InDesign population from scope data
- Scope text in JSON export (var.json) — the export schema will need extending when this is needed

## Testing

- Unit tests for `formatSectionsAsText()` — copy formatting logic
- Unit tests for 404 handling in `getScope()` wrapper
- Integration test for scope API calls (mock responses)
- Smoke test: "Generate Scope" button visible in edit mode, opens modal
