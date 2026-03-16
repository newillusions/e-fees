# Design Review Fixes Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix all 40+ findings from the March 2026 design review (`docs/plans/2026-03-13-design-review.md`)

**Architecture:** Batched by type — CSS tokens first (unblocks colour replacements), then quick markup fixes, accessibility, component refactors, and finally off-brand colour replacements. Each batch produces a commit.

**Tech Stack:** Svelte 5, CSS custom properties, semantic `.emittiv-*` classes

**Path corrections from exploration (review assumed flat `src/lib/components/`):**
| Review Path | Actual Path |
|---|---|
| `QuickActions.svelte` | `dashboard/QuickActions.svelte` |
| `PendingProposals.svelte` | `dashboard/PendingProposals.svelte` |
| `ScopeBuilder.svelte` | `scope/ScopeBuilder.svelte` |
| `PricingSummaryPanel.svelte` | `pricing/PricingSummaryPanel.svelte` |
| `PaymentSchedulePanel.svelte` | `pricing/PaymentSchedulePanel.svelte` |
| `ProjectDetail.svelte` | `src/lib/components/ProjectDetail.svelte` (not `src/routes/`) |

---

## Chunk 1: CSS Token Foundation

**Unblocks:** Chunks 4 and 5. Must land first.

### Task 1.1: Convert spacing tokens from rem to px (C-1)

**Files:**
- Modify: `src/styles/app.css:22-28`

- [ ] **Step 1: Update spacing tokens**

Replace in `app.css` `:root`:
```css
/* OLD */
--spacing-xs: 0.25rem;
--spacing-sm: 0.5rem;
--spacing-md: 1rem;
--spacing-lg: 1.5rem;
--spacing-xl: 2rem;
--spacing-2xl: 3rem;
--spacing-3xl: 4rem;

/* NEW */
--spacing-xs: 4px;
--spacing-sm: 8px;
--spacing-md: 16px;
--spacing-lg: 24px;
--spacing-xl: 32px;
--spacing-2xl: 48px;
--spacing-3xl: 64px;
```

- [ ] **Step 2: Grep for any component using spacing tokens directly**

Run: `grep -rn 'var(--spacing-' src/`
Verify no component relies on rem-specific behavior (e.g., calc with rem units).

- [ ] **Step 3: Visual check**

Run: `npm run tauri:dev` and verify spacing looks correct across Dashboard, Projects, Proposals pages.

---

### Task 1.2: Define semantic colour tokens (H-10)

**Files:**
- Modify: `src/styles/app.css` `:root` block

- [ ] **Step 1: Add semantic colour tokens after the brand palette**

```css
/* Semantic colours — Catppuccin Mocha accents */
--color-success: #a6e3a1;       /* Mocha Green */
--color-warning: #f9e2af;       /* Mocha Yellow */
--color-error: #f38ba8;         /* Mocha Red */
--color-danger: #f38ba8;        /* alias for error */
--color-info: #89b4fa;          /* Mocha Blue */

/* Splash hover/active states */
--color-splash-hover: #e68a00;
--color-splash-active: #cc7a00;

/* Stat icon accents */
--color-stat-projects: #89b4fa;  /* Blue */
--color-stat-proposals: var(--emittiv-splash); /* Orange — brand */
--color-stat-companies: #94e2d5; /* Teal */
--color-stat-contacts: #cba6f7;  /* Mauve */

/* Status badge accents */
--color-status-lead: #89b4fa;       /* Blue */
--color-status-draft: #89b4fa;      /* Blue */
--color-status-rfp: #fab387;        /* Peach */
--color-status-sent: #fab387;       /* Peach */
--color-status-negotiation: #cba6f7; /* Mauve */
--color-status-awarded: #a6e3a1;    /* Green */
--color-status-accepted: #a6e3a1;   /* Green */
--color-status-lost: #f38ba8;       /* Red */
--color-status-rejected: #f38ba8;   /* Red */
--color-status-no-response: #6c7086; /* Overlay 0 */

/* Payment status */
--color-status-invoiced: #f9e2af;  /* Yellow */
--color-status-paid: #a6e3a1;      /* Green */
--color-status-pending: var(--emittiv-light);
```

- [ ] **Step 2: Replace raw hex for error/warning/danger throughout app.css**

Run: `grep -n '#ef4444\|#eab308\|#e68a00\|#cc7a00\|#dc2626\|#b91c1c' src/styles/app.css`

Replace each occurrence with the appropriate `var(--color-*)` token.

---

### Task 1.3: Define overlay tokens (M-7)

**Files:**
- Modify: `src/styles/app.css` `:root` block

- [ ] **Step 1: Add overlay tokens**

```css
/* Overlay values */
--overlay-subtle: rgba(0, 0, 0, 0.15);
--overlay-light: rgba(0, 0, 0, 0.2);
--overlay-medium: rgba(0, 0, 0, 0.3);
--overlay-backdrop: rgba(0, 0, 0, 0.5);
--overlay-dark: rgba(0, 0, 0, 0.6);
--splash-tint: rgba(255, 153, 0, 0.1);
--splash-glow: rgba(255, 153, 0, 0.3);
```

- [ ] **Step 2: Replace raw rgba values throughout app.css**

Search for `rgba(0, 0, 0,` and `rgba(255, 153, 0,` patterns. Replace with nearest token. Only replace exact matches — don't force-fit values that don't match.

---

### Task 1.4: Unify transition easing (M-8)

**Files:**
- Modify: `src/styles/app.css:30-31`

- [ ] **Step 1: Update transition tokens**

```css
/* OLD */
--transition-smooth: all 0.3s ease-in-out;
--transition-fast: all 0.15s ease-in-out;

/* NEW */
--transition-smooth: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
--transition-fast: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
```

---

### Task 1.5: Commit Chunk 1

- [ ] **Commit**

```bash
git add src/styles/app.css
git commit -m "refactor(css): add semantic colour tokens, fix spacing to px, unify transitions

- C-1: spacing tokens rem→px
- H-10: semantic colour tokens (Mocha accents)
- M-7: overlay tokens
- M-8: transition easing unified to cubic-bezier"
```

---

## Chunk 2: Quick Markup & CSS Fixes

Single-line or few-line changes. No behavioral changes.

### Task 2.1: App name fix (C-4)

**Files:**
- Modify: `src/lib/components/FirstRunSetup.svelte:159,424`

- [ ] **Step 1:** Change "Welcome to Fee Proposal Management" → "Welcome to E-Fees" and subtitle → "Let's get your workspace set up."
- [ ] **Step 2:** Change completion message → "Your E-Fees workspace is ready."

---

### Task 2.2: "Active Fees" → "Active Proposals" (H-1)

**Files:**
- Modify: `src/routes/Dashboard.svelte:92`

- [ ] **Step 1:** Change label from `"Active Fees"` to `"Active Proposals"`

---

### Task 2.3: Quick action verb consistency (M-2)

**Files:**
- Modify: `src/lib/components/dashboard/QuickActions.svelte:34-56`

- [ ] **Step 1:** Change `title` values:
  - `'New Project'` → keep
  - `'Create Proposal'` → `'New Proposal'`
  - `'Add Company'` → `'New Company'`
  - `'Open Folders'` → keep (different action class)

---

### Task 2.4: Nav layout shift fix (M-6)

**Files:**
- Modify: `src/styles/app.css` (`.nav-inactive` block, around line 316)

- [ ] **Step 1:** Add `border-right: 2px solid transparent;` to `.nav-inactive`

---

### Task 2.5: Delete dead button classes (H-13)

**Files:**
- Modify: `src/styles/app.css:149-195`

- [ ] **Step 1: Verify unused**

Run: `grep -rn 'emittiv-button-primary\|emittiv-button-secondary' src/`
Expected: 0 results outside app.css

- [ ] **Step 2:** Delete `.emittiv-button-primary` and `.emittiv-button-secondary` blocks

---

### Task 2.6: Delete dead Tailwind gradient classes (L-6)

**Files:**
- Modify: `src/styles/app.css:2363-2364`

- [ ] **Step 1: Verify unused**

Run: `grep -rn 'from-emittiv-darker\|to-emittiv-black' src/`
Expected: 0 results outside app.css

- [ ] **Step 2:** Delete the two `.from-emittiv-darker` and `.to-emittiv-black` class blocks

---

### Task 2.7: Scrollbar tokens (L-5)

**Files:**
- Modify: `src/styles/app.css:1973-1993`

- [ ] **Step 1:** Replace `#333` → `var(--emittiv-darker)`, `#999` → `var(--emittiv-light)`, `#ccc` → `var(--emittiv-lighter)`

---

### Task 2.8: Date-input font (L-4)

**Files:**
- Modify: `src/styles/app.css` (`.emittiv-date-input`)

- [ ] **Step 1:** Replace `font-family: 'Montserrat', sans-serif;` → `font-family: var(--font-body);`

---

### Task 2.9: Select/input bg consistency (L-7)

**Files:**
- Modify: `src/styles/app.css` (`.emittiv-select` background)

- [ ] **Step 1:** Change `.emittiv-select` background from `rgba(0,0,0,0.15)` → `var(--emittiv-dark)` to match `.emittiv-input`

---

### Task 2.10: Checkbox focus-visible (M-16)

**Files:**
- Modify: `src/styles/app.css:354`

- [ ] **Step 1:** Change `.emittiv-checkbox:focus` → `.emittiv-checkbox:focus-visible`

---

### Task 2.11: Nav hover scale (L-8)

**Files:**
- Modify: `src/lib/components/Navigation.svelte:84` (approximate)

- [ ] **Step 1:** Remove `group-hover:scale-105` class. If scale effect wanted, add to `.nav-inactive:hover` in CSS.

---

### Task 2.12: Input.svelte error class (L-3)

**Files:**
- Modify: `src/lib/components/Input.svelte:46` (approx line 36-46)

- [ ] **Step 1:** Replace `text-red-500` in error message with `emittiv-error` class

---

### Task 2.13: Pending Proposals filter fix (M-5)

**Files:**
- Modify: `src/lib/components/dashboard/PendingProposals.svelte:34`

- [ ] **Step 1:** Add `'Negotiation'` to filter:
```svelte
.filter(fee => fee.status === 'Sent' || fee.status === 'Draft' || fee.status === 'Negotiation')
```

---

### Task 2.14: Status sync dialog copy (M-9)

**Files:**
- Modify: `src/lib/components/ProposalModal.svelte:1208-1214`

- [ ] **Step 1:** Update heading → "Also update the project status?"
- [ ] **Step 2:** Update body → "Changing the proposal to {status} would set the project to {projectStatus}. Update both, or the proposal only?"
- [ ] **Step 3:** Update buttons → "Update both" / "Proposal only"

---

### Task 2.15: ProposalCard metadata layout (M-17)

**Files:**
- Modify: `src/lib/components/ProposalCard.svelte:91-96`

- [ ] **Step 1:** Replace `<br/>` in metadata spans with flex-col structure:
```svelte
<div class="emittiv-card-meta">
  <span class="emittiv-card-meta__item"><span class="emittiv-card-meta__label">Rev</span>{proposal.rev}</span>
  ...
</div>
```
Add `.emittiv-card-meta__item` and `.emittiv-card-meta__label` to app.css (flex-col, label above value).

---

### Task 2.16: Reduced motion (L-1)

**Files:**
- Modify: `src/styles/app.css` (end of file)

- [ ] **Step 1:** Add media query:
```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

### Task 2.17: Commit Chunk 2

- [ ] **Commit**

```bash
git add -A
git commit -m "fix(ui): quick markup and CSS fixes from design review

- C-4: app name → E-Fees
- H-1: Active Fees → Active Proposals
- H-13: dead button classes removed
- M-2: verb consistency
- M-5: pending filter includes Negotiation
- M-6: nav layout shift
- M-9: status sync copy
- M-16: checkbox focus-visible
- M-17: card meta layout
- L-1 through L-8: token/class cleanup"
```

---

## Chunk 3: Accessibility Fixes

### Task 3.1: Focus trap for BaseModal (C-5)

**Files:**
- Modify: `src/lib/components/BaseModal.svelte`

- [ ] **Step 1: Add focus trap utility**

In `<script>`, add after `handleKeydown`:
```typescript
let modalElement: HTMLElement;

function trapFocus(node: HTMLElement) {
  const focusable = node.querySelectorAll<HTMLElement>(
    'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
  );
  const first = focusable[0];
  const last = focusable[focusable.length - 1];

  function handleTab(e: KeyboardEvent) {
    if (e.key !== 'Tab') return;
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last?.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first?.focus();
    }
  }

  node.addEventListener('keydown', handleTab);
  // Focus first focusable on open
  requestAnimationFrame(() => first?.focus());

  return {
    destroy() { node.removeEventListener('keydown', handleTab); }
  };
}
```

- [ ] **Step 2:** Add `use:trapFocus` to the modal content div (the `emittiv-modal` element).

---

### Task 3.2: Backdrop cleanup (H-5)

**Files:**
- Modify: `src/lib/components/BaseModal.svelte:38-48`

- [ ] **Step 1:** Remove `role="button"`, `aria-label`, `tabindex` from backdrop div. Keep the `onclick={closeModal}` for mouse close.

---

### Task 3.3: Form ARIA attributes (H-6)

**Files:**
- Modify: `src/lib/components/FormInput.svelte`
- Modify: `src/lib/components/FormSelect.svelte`
- Modify: `src/lib/components/TypeaheadSelect.svelte`

- [ ] **Step 1: FormInput** — Add `aria-invalid={!!error}` and `aria-describedby="{inputId}-error"` to `<input>`. Add `id="{inputId}-error"` and `aria-live="polite"` to error div.
- [ ] **Step 2: FormSelect** — Same pattern on `<select>` and error div.
- [ ] **Step 3: TypeaheadSelect** — Same pattern on the combobox input and error div.

---

### Task 3.4: Unique modal IDs (H-7)

**Files:**
- Modify: `src/lib/components/BaseModal.svelte`
- Modify: `src/lib/components/WarningModal.svelte`

- [ ] **Step 1: BaseModal** — Generate unique ID: `const modalId = \`modal-\${Math.random().toString(36).substr(2, 9)}\``. Use `id="{modalId}-title"` on heading, `aria-labelledby="{modalId}-title"` on dialog.
- [ ] **Step 2: WarningModal** — Same pattern.

---

### Task 3.5: GlobalSearchModal ARIA fix (H-8)

**Files:**
- Modify: `src/lib/components/GlobalSearchModal.svelte:255,270`

- [ ] **Step 1:** Add visually-hidden heading: `<h2 id="search-title" class="sr-only">Quick Search</h2>`
- [ ] **Step 2:** Give input its own id + `aria-label="Search"`. Remove `id="search-title"` from input.

---

### Task 3.6: TypeaheadSelect combobox pattern (H-11)

**Files:**
- Modify: `src/lib/components/TypeaheadSelect.svelte:158-210`

- [ ] **Step 1:** Add to input: `role="combobox"`, `aria-expanded={showDropdown}`, `aria-controls="{inputId}-listbox"`, `aria-activedescendant={activeIndex >= 0 ? \`\${inputId}-option-\${activeIndex}\` : undefined}`
- [ ] **Step 2:** Add to dropdown: `role="listbox"`, `id="{inputId}-listbox"`
- [ ] **Step 3:** Add to each option: `role="option"`, `id="{inputId}-option-{i}"`, `aria-selected={i === activeIndex}`

---

### Task 3.7: TypeaheadSelect ID collision (M-15)

**Files:**
- Modify: `src/lib/components/TypeaheadSelect.svelte:147`

- [ ] **Step 1:** Change ID generation from `typeahead-{label.toLowerCase()}` to `typeahead-${Math.random().toString(36).substr(2, 9)}`

---

### Task 3.8: LoadingSkeleton accessibility (H-12)

**Files:**
- Modify: `src/lib/components/LoadingSkeleton.svelte:9`

- [ ] **Step 1:** Wrap in: `<div role="status" aria-label="Loading..." aria-live="polite">`

---

### Task 3.9: EmptyState SVG aria-hidden (L-2)

**Files:**
- Modify: `src/lib/components/EmptyState.svelte:13`

- [ ] **Step 1:** Add `aria-hidden="true"` to `<svg>`

---

### Task 3.10: Button loading ARIA (L-9)

**Files:**
- Modify: `src/lib/components/Button.svelte:21-23`

- [ ] **Step 1:** Add `aria-busy={loading}` to `<button>`. Add `aria-hidden="true"` to spinner div.

---

### Task 3.11: Commit Chunk 3

- [ ] **Commit**

```bash
git add -A
git commit -m "fix(a11y): accessibility fixes from design review

- C-5: focus trap in BaseModal
- H-5: backdrop role cleanup
- H-6: form ARIA (FormInput, FormSelect, TypeaheadSelect)
- H-7: unique modal IDs
- H-8: GlobalSearchModal ARIA
- H-11: TypeaheadSelect combobox pattern
- H-12: LoadingSkeleton role=status
- M-15: TypeaheadSelect ID collision
- L-2: decorative SVG aria-hidden
- L-9: button loading aria-busy"
```

---

## Chunk 4: Component Refactors

### Task 4.1: Replace alert() with emittiv-alert (C-2)

**Files:**
- Modify: `src/routes/Projects.svelte:249,258,262`
- Modify: `src/lib/components/ProjectDetail.svelte:112,121,125`
- Modify: `src/lib/components/dashboard/QuickActions.svelte:29`

- [ ] **Step 1:** In Projects.svelte — add a reactive error state (`let folderError = $state('')`). Replace each `alert(msg)` with `folderError = msg`. Add `{#if folderError}<div class="emittiv-alert emittiv-alert--error">{folderError}</div>{/if}` near the relevant UI area. Add auto-dismiss after 5s with `setTimeout`.
- [ ] **Step 2:** Same pattern in ProjectDetail.svelte.
- [ ] **Step 3:** Same pattern in QuickActions.svelte.

---

### Task 4.2: Unsaved changes guard (H-3)

**Files:**
- Modify: `src/lib/components/ProposalModal.svelte:633-637`
- Modify: `src/lib/components/BaseModal.svelte`

- [ ] **Step 1: ProposalModal** — Add `isDirty` derived state comparing `formData` vs initial snapshot. In `closeModal()`, if dirty, set `showDiscardConfirm = true` instead of closing.
- [ ] **Step 2:** Add discard confirmation UI (two buttons: "Discard" / "Keep Editing").
- [ ] **Step 3: BaseModal** — Pass `onBeforeClose` callback prop. If provided, call it before closing. If it returns false, don't close.

---

### Task 4.3: Bulk status confirmation (H-4)

**Files:**
- Modify: `src/lib/components/BulkActionBar.svelte:19-23`

- [ ] **Step 1:** Replace immediate `onchange` dispatch with two-step: dropdown sets `selectedStatus`, then "Apply" button dispatches. Mirror the existing delete confirmation pattern.

---

### Task 4.4: Bulk delete count in label (M-13)

**Files:**
- Modify: `src/lib/components/BulkActionBar.svelte:25-31`

- [ ] **Step 1:** Change confirm button text from `"Confirm Delete"` to `"Confirm Delete {selectedCount} {entityLabel}"`

---

### Task 4.5: Search button → clear button (M-4)

**Files:**
- Modify: `src/lib/components/SearchFilterBar.svelte:52-60`

- [ ] **Step 1:** Remove the non-functional search button. Replace with a clear `x` button that appears when `searchQuery` is non-empty.

---

### Task 4.6: BaseModal size prop (M-12)

**Files:**
- Modify: `src/lib/components/BaseModal.svelte`

- [ ] **Step 1:** Add `size` prop (`'sm' | 'md' | 'lg' | 'xl'`, default `'md'`). Apply `emittiv-modal--{size}` class. Remove inline `style="max-width: {maxWidth};"`.
- [ ] **Step 2:** Remove `!important` from `.emittiv-modal--*` size classes in app.css.
- [ ] **Step 3:** Update all BaseModal callers to use `size` instead of `maxWidth`.

---

### Task 4.7: FormSelect placeholder (M-14)

**Files:**
- Modify: `src/lib/components/FormSelect.svelte:31-33`

- [ ] **Step 1:** For optional fields (`!required`), use `<option value="">—</option>` (not disabled). Keep `disabled` for required.

---

### Task 4.8: ProposalModal inline styles → classes (C-3)

**Files:**
- Modify: `src/lib/components/ProposalModal.svelte:919-1352`

- [ ] **Step 1:** Identify repeated `style="display: flex; flex-direction: column; gap: Npx;"` patterns (5+ occurrences).
- [ ] **Step 2:** Replace with existing `.emittiv-form-section`, `.emittiv-form-grid`, `.emittiv-form-section__title` classes from app.css.
- [ ] **Step 3:** For any gap values not covered by existing classes, add `.emittiv-form-group` or similar in app.css.

---

### Task 4.9: Keyboard shortcut hints (M-3)

**Files:**
- Modify: `src/lib/components/Navigation.svelte`

- [ ] **Step 1:** Add `title` attribute to nav buttons: `title="{item.label} ({isMac ? '⌘' : 'Ctrl+'}${item.shortcut})"`

---

### Task 4.10: ScopeBuilder breadcrumb (M-10)

**Files:**
- Modify: `src/lib/components/scope/ScopeBuilder.svelte:237-239`

- [ ] **Step 1:** Replace raw fee ID display with proposal number. Add back-link: `← Proposal {number}`.

---

### Task 4.11: Commit Chunk 4

- [ ] **Commit**

```bash
git add -A
git commit -m "refactor(ui): component refactors from design review

- C-2: alert() → emittiv-alert
- C-3: ProposalModal inline styles → CSS classes
- H-3: unsaved changes guard
- H-4: bulk status confirmation
- M-3: keyboard shortcut hints
- M-4: search button → clear button
- M-10: ScopeBuilder breadcrumb
- M-12: BaseModal size prop
- M-13: bulk delete count
- M-14: FormSelect placeholder"
```

---

## Chunk 5: Off-Brand Colour Replacements (H-9)

**Depends on:** Chunk 1 (semantic tokens defined)

### Task 5.1: Dashboard stat icon colours

**Files:**
- Modify: `src/routes/Dashboard.svelte:88,102,109`

- [ ] **Step 1:** Replace:
  - `'text-blue-400'` → CSS class using `color: var(--color-stat-projects)`
  - `'text-green-400'` → CSS class using `color: var(--color-stat-companies)`
  - `'text-purple-400'` → CSS class using `color: var(--color-stat-contacts)`

Add `.emittiv-stat--projects`, `.emittiv-stat--companies`, `.emittiv-stat--contacts` classes in app.css referencing the tokens.

---

### Task 5.2: SplashScreen colours

**Files:**
- Modify: `src/lib/components/SplashScreen.svelte:115-130`

- [ ] **Step 1:** Replace all Tailwind colour classes with `var(--emittiv-*)` tokens:
  - `text-orange-500` → `color: var(--emittiv-splash)`
  - `text-white` → `color: var(--emittiv-white)`
  - `text-gray-400/500` → `color: var(--emittiv-light)` / `var(--emittiv-dark)`
  - `bg-gray-700` → `background: var(--emittiv-darker)`
  - `border-gray-600` → `border-color: var(--emittiv-dark)`

Extract to `.emittiv-splash-screen__*` classes or use inline `style` with tokens.

---

### Task 5.3: PricingSummaryPanel colours

**Files:**
- Modify: `src/lib/components/pricing/PricingSummaryPanel.svelte:209,215`

- [ ] **Step 1:** Replace `text-yellow-500` → `color: var(--color-status-invoiced)`, `text-green-500` → `color: var(--color-status-paid)`

---

### Task 5.4: PaymentSchedulePanel colours

**Files:**
- Modify: `src/lib/components/pricing/PaymentSchedulePanel.svelte:49-53,353-388`

- [ ] **Step 1:** Replace status colour map:
```typescript
const statusColors = {
  pending: 'var(--color-status-pending)',
  invoiced: 'var(--color-status-invoiced)',
  paid: 'var(--color-status-paid)',
};
```
Switch from Tailwind class application to inline `style="color: {statusColors[status]}"` or define `.emittiv-payment--pending/invoiced/paid` classes.

- [ ] **Step 2:** Replace `text-red-500` → `color: var(--color-error)`, `text-green-500` → `color: var(--color-success)` in footer totals.

---

### Task 5.5: StatusBadge contrast audit (L-10)

**Files:**
- Modify: `src/styles/app.css` badge variants

- [ ] **Step 1:** Check each `emittiv-badge--*` variant against WCAG AA (4.5:1 for text). Use the Mocha accent values from the tokens.
- [ ] **Step 2:** Increase text lightness for any failing variant.

---

### Task 5.6: Commit Chunk 5

- [ ] **Commit**

```bash
git add -A
git commit -m "fix(ui): replace off-brand Tailwind colours with Mocha accent tokens

- H-9: Dashboard, SplashScreen, PricingSummary, PaymentSchedule
- L-10: badge contrast audit
- All colours now reference --color-* or --emittiv-* tokens"
```

---

## Chunk 6: Remaining Medium Items

### Task 6.1: Issue Date input mask (M-1)

**Files:**
- Modify: `src/lib/components/ProposalModal.svelte:1048-1055`

- [ ] **Step 1:** Add `inputmode="numeric"` and `pattern="[0-9]*"` to the issue date field.
- [ ] **Step 2:** Add blur-time validation: on blur, validate YYYYMM format and show error immediately (not just on submit).

---

### Task 6.2: BaseListCard semantic classes (M-11)

**Files:**
- Modify: `src/lib/components/BaseListCard.svelte`
- Modify: `src/styles/app.css`

- [ ] **Step 1:** Add CSS classes:
```css
.emittiv-list-card__body { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; }
.emittiv-list-card__content { flex: 1; min-width: 0; }
.emittiv-list-card__title-row { display: flex; align-items: center; gap: 8px; }
.emittiv-list-card__meta { display: flex; gap: 12px; margin-top: 4px; }
.emittiv-list-card__actions { flex-shrink: 0; }
```
- [ ] **Step 2:** Replace Tailwind layout utilities in BaseListCard with these classes.

---

### Task 6.3: Commit Chunk 6

- [ ] **Commit**

```bash
git add -A
git commit -m "fix(ui): remaining medium design review items

- M-1: issue date blur validation
- M-11: BaseListCard semantic classes"
```

---

## Final Verification

- [ ] **Run Rust tests:** `cargo test -p app --lib`
- [ ] **Run frontend tests:** `npm test`
- [ ] **Run TypeScript check:** `npm run check`
- [ ] **Visual smoke test:** `npm run tauri:dev` — check Dashboard, Projects, Proposals, Companies, Contacts
- [ ] **Update design review doc:** Mark completed items with `[x]`

---

*Plan created: 2026-03-16*
*Estimated: 6 commits, ~40 files touched*
