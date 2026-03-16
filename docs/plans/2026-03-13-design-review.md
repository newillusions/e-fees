# E-Fees Design Review — March 2026

**Conducted:** 2026-03-13
**Reviewers:** UI Visual · UX Interaction · Accessibility · Content/Copy · Design System
**Status:** In progress

---

## How to Use This Document

- `[ ]` = pending | `[x]` = done | `[-]` = won't fix / by design
- Priority: **Critical** → **High** → **Medium** → **Low**
- File:line references are to the state at the time of review (v0.13.7)

---

## CRITICAL

### C-1 · Spacing tokens use `rem` not `px`
**File:** `src/styles/app.css:22–28`
**Detail:** All 7 `--spacing-*` tokens (`--spacing-xs: 0.25rem` through `--spacing-3xl: 4rem`) are in `rem`. On 150% OS-scaled desktops this double-scales. The newer `.emittiv-btn` and form classes already use `px` correctly — tokens just haven't caught up.
**Fix:** Convert to `px` — `--spacing-xs: 4px`, `--spacing-sm: 8px`, `--spacing-md: 16px`, `--spacing-lg: 24px`, `--spacing-xl: 32px`, `--spacing-2xl: 48px`, `--spacing-3xl: 64px`
- [ ] Done

---

### C-2 · `alert()` used for errors — 6 occurrences
**Files:** `Projects.svelte:248,257,261`, `QuickActions.svelte:29`, `ProjectDetail.svelte:112,121,125`
**Detail:** Native OS alert dialog, jarring and inconsistent with `emittiv-alert` component.
**Fix:** Replace with `operationActions.setError()` pattern from `ProposalModal`.
- [ ] Done

---

### C-3 · ProposalModal has 25+ inline `style=` attributes
**File:** `ProposalModal.svelte:919–1352`
**Detail:** Patterns like `style="display: flex; flex-direction: column; gap: 16px;"` repeated 5+ times. The semantic classes `.emittiv-form-section`, `.emittiv-form-section__title`, `.emittiv-form-grid` already exist in `app.css` and cover all these cases.
**Fix:** Pure markup refactor — apply existing CSS classes, no new CSS needed.
- [ ] Done

---

### C-4 · Application name wrong on first-run screen
**File:** `FirstRunSetup.svelte:159, 424`
**Detail:** "Welcome to Fee Proposal Management" → should be "Welcome to E-Fees". Also affects completion message at line 424.
**Fix:** `"Welcome to E-Fees"`, subtitle `"Let's get your workspace set up."`. Completion: `"Your E-Fees workspace is ready."`
- [ ] Done

---

### C-5 · Focus trap missing in all modals
**File:** `BaseModal.svelte` (affects all modal instances)
**Detail:** Tab focus cycles through the entire page behind open dialogs. One implementation in `BaseModal` covers all. Collect focusable descendants on open, cycle Tab within them, return focus to trigger on close.
**Fix:** ~25-line focus trap utility in `BaseModal.svelte`.
- [ ] Done

---

## HIGH

### H-1 · "Active Fees" on Dashboard should be "Active Proposals"
**File:** `Dashboard.svelte:92`
**Detail:** "Fees" is the database table name. Every other surface says "Proposals". Stat card links to `/proposals`.
**Fix:** Change label to `"Active Proposals"`.
- [ ] Done

---

### H-2 · Scope Builder unreachable from main UI
**File:** `Navigation.svelte`, `ProposalDetail.svelte`
**Detail:** Route `/scope/:id` only — no nav item, no link from proposal detail. Feature is invisible.
**Fix (interim):** Link from DevMode page (done — see DevMode section). Final fix: "Build Scope" button in `ProposalDetail`.
- [ ] DevMode link added
- [ ] ProposalDetail button (deferred — scope builder still in development)

---

### H-3 · No unsaved-changes guard on ProposalModal close
**File:** `BaseModal.svelte:27–31`, `ProposalModal.svelte:633–637`
**Detail:** Escape or backdrop click silently discards all edits. `formData` vs initial state comparison already available.
**Fix:** Compare `formData` vs initial on close/Escape. If dirty, show "Discard / Keep Editing" prompt.
- [ ] Done

---

### H-4 · Bulk status change fires immediately on dropdown select
**File:** `BulkActionBar.svelte:19–23`
**Detail:** `handleStatusChange` fires on `onchange`. Misclick applies batch update with no confirm. Can cascade to project status changes.
**Fix:** Add "Apply" button that activates once a status is chosen, or two-step confirm matching delete pattern.
- [ ] Done

---

### H-5 · Backdrop has `role="button"` with no-op keydown handler
**File:** `BaseModal.svelte:38–48`
**Detail:** Screen readers announce a phantom interactive element on every modal open. Keydown handler is `() => {}`.
**Fix:** Remove `role="button"`, `aria-label`, and `tabindex` from backdrop. ESC handler already covers keyboard close.
- [ ] Done

---

### H-6 · Error messages not announced to screen readers
**Files:** `FormInput.svelte:55–57`, `FormSelect.svelte:40–43`, `TypeaheadSelect.svelte:231–233`
**Detail:** No `aria-invalid`, no `aria-describedby`, no `aria-live="polite"` on error divs. Form validation is silent to assistive technology.
**Fix:** Add `aria-invalid={!!error}` + `aria-describedby="{inputId}-error"` to inputs. Add `id="{inputId}-error"` + `aria-live="polite"` to error divs. Apply to all three components.
- [ ] Done

---

### H-7 · Non-unique `id="modal-title"` across modals
**Files:** `BaseModal.svelte:56`, `WarningModal.svelte:67`
**Detail:** When a warning dialog opens over a form modal, `aria-labelledby` points to the wrong heading. IDs must be unique in the document.
**Fix:** Generate a unique ID per instance (same `Math.random()` pattern as `FormInput.svelte:20`).
- [ ] Done

---

### H-8 · GlobalSearchModal `aria-labelledby` points to the input, not a heading
**File:** `GlobalSearchModal.svelte:255,270`
**Detail:** `aria-labelledby="search-title"` targets the `<input>` element. Screen readers read the current input value as the dialog label.
**Fix:** Add visually-hidden `<h2 id="search-title">Quick Search</h2>`. Give input its own id + `aria-label="Search"`.
- [ ] Done

---

### H-9 · Off-brand Tailwind colors throughout UI
**Files:** `Dashboard.svelte:88,95,102,109` · `QuickActions.svelte:38,52,59` · `PricingSummaryPanel.svelte:169–216` · `PaymentSchedulePanel.svelte:51–52,353–381` · `SplashScreen.svelte:115,123,130`
**Detail:** `text-blue-400`, `text-green-400`, `text-purple-400`, `text-yellow-500`, `text-orange-500` (#f97316 ≠ brand #ff9900). Payment status colors (`text-yellow-500`/`text-green-500`) diverge from badge system.
**Fix:** Replace with `--emittiv-splash` or new `--emittiv-accent-*` tokens defined in design system. SplashScreen: `text-orange-500` → `var(--emittiv-splash)`.
**Note:** New accent colors need the color test page first — see Design System section.
- [ ] Color test page created
- [ ] Accent tokens defined in `app.css`
- [ ] Tailwind color classes replaced
- [ ] `SplashScreen.svelte` fixed (quick win — `text-orange-500` → `var(--emittiv-splash)`)

---

### H-10 · Semantic color tokens missing
**File:** `src/styles/app.css` (scattered)
**Detail:** `#ef4444` (error red) 8+ times, `#eab308` (warning yellow) 4+ times, `#e68a00`/`#cc7a00` (hover/active) 3+ times, `#dc2626` (danger) 3+ times.
**Fix:** Define in `:root`:
```css
--color-error: #ef4444;
--color-warning: #eab308;
--color-splash-hover: #e68a00;
--color-splash-active: #cc7a00;
--color-danger: #dc2626;
--color-danger-dark: #b91c1c;
```
Then replace all raw hex occurrences.
- [ ] Tokens defined
- [ ] Raw hex replaced

---

### H-11 · TypeaheadSelect missing ARIA combobox pattern
**File:** `TypeaheadSelect.svelte:158–210`
**Detail:** No `role="combobox"`, `aria-expanded`, `aria-controls`, `aria-activedescendant`. Dropdown is invisible to screen readers. Used on every proposal form for company/contact selection.
**Fix:** Add full ARIA combobox pattern. No behavioral changes needed — attribute additions only.
- [ ] Done

---

### H-12 · LoadingSkeleton has no accessible announcement
**File:** `LoadingSkeleton.svelte:9–13`
**Detail:** Animated divs with no `role` or `aria-*`. Loading states are invisible to screen readers.
**Fix:** Wrap in `<div role="status" aria-label="Loading…" aria-live="polite">`.
- [ ] Done

---

### H-13 · Dead code: duplicate button system
**File:** `app.css:147–193`
**Detail:** `.emittiv-button-primary` / `.emittiv-button-secondary` exist alongside the canonical `.emittiv-btn--primary`/`--secondary`. Different border-radius (8px vs 4px), rem-based padding. Never referenced in any component (confirmed by grep).
**Fix:** Delete the old button class blocks. Verify with `grep -r "emittiv-button-primary\|emittiv-button-secondary" src/` first.
- [ ] Verified unused
- [ ] Deleted from `app.css`

---

## MEDIUM

### M-1 · Issue Date field: no picker/mask, validates only on submit
**File:** `ProposalModal.svelte:1048–1055, validation at 310`
**Detail:** Required YYMMDD format field with no input mask, no date picker. Validates only on submit. Affects 100% of proposals.
**Fix:** Either `type="date"` with YYMMDD conversion layer, or input mask accepting digits only. At minimum, add blur-time validation.
- [ ] Done

---

### M-2 · Quick action verbs inconsistent
**File:** `QuickActions.svelte:33–62`
**Detail:** "New Project / Create Proposal / Add Company / Open Folders" — three verbs for same action class.
**Fix:** "New Project / New Proposal / New Company / Open Folders"
- [ ] Done

---

### M-3 · Keyboard shortcuts have no visible hint
**Files:** `Navigation.svelte:76–93`
**Detail:** Cmd+1–5 are wired globally but invisible. `shortcut` prop exists but is never rendered.
**Fix:** Add `title="Dashboard (⌘1)"` etc. to nav buttons as minimum. Add `<kbd>` badges as ideal (GlobalSearchModal already has `<kbd>` styling).
- [ ] Done

---

### M-4 · Non-functional search button
**File:** `SearchFilterBar.svelte:52–60`
**Detail:** Search button has no `onclick`. Search is already live-reactive. Button creates false affordance.
**Fix:** Remove the button entirely; replace with a clear-input `×` button.
- [ ] Done

---

### M-5 · Pending Proposals excludes "Negotiation" status
**File:** `PendingProposals.svelte:33–41`
**Detail:** Filter is `Sent || Draft` only. "Negotiation" (active client back-and-forth) is excluded from the "Pending" view.
**Fix:** Add `'Negotiation'` to the pending filter. Consider removing `'Draft'` (not yet sent to anyone).
- [ ] Done

---

### M-6 · Nav active-state 2px layout shift
**File:** `app.css:288–293`
**Detail:** `.nav-active` adds `border-right: 2px solid var(--emittiv-splash)`. `.nav-inactive` has no border. Content shifts on route change.
**Fix:** Add `border-right: 2px solid transparent` to `.nav-inactive`.
- [ ] Done

---

### M-7 · rgba() overlay values not tokenised — 22+ occurrences
**File:** `app.css` (scattered)
**Detail:** Five distinct black-overlay alpha levels and two splash-tint values in 30+ declarations.
**Fix:** Define tokens:
```css
--overlay-subtle: rgba(0, 0, 0, 0.15);
--overlay-light: rgba(0, 0, 0, 0.2);
--overlay-medium: rgba(0, 0, 0, 0.3);
--overlay-backdrop: rgba(0, 0, 0, 0.5);
--overlay-dark: rgba(0, 0, 0, 0.6);
--splash-tint: rgba(255, 153, 0, 0.1);
--splash-glow: rgba(255, 153, 0, 0.3);
```
- [ ] Tokens defined
- [ ] Raw rgba values replaced

---

### M-8 · Two parallel transition systems with different easing
**File:** `app.css:31–32` vs component classes
**Detail:** `--transition-smooth: all 0.3s ease-in-out` in tokens, but component classes use `cubic-bezier(0.4, 0, 0.2, 1)`. Inconsistent motion across UI.
**Fix:** Update tokens to `cubic-bezier(0.4, 0, 0.2, 1)`. Audit component classes to use `var(--transition-fast/smooth)`.
- [ ] Done

---

### M-9 · Status sync dialog copy is hard to scan
**File:** `ProposalModal.svelte:1211–1214`
**Detail:** Passive phrasing buries the concrete outcome mid-sentence.
**Fix:**
- Heading: "Also update the project status?"
- Body: "Changing the proposal to [status] would set the project to [ProjectStatus]. Update both, or the proposal only?"
- Buttons: "Update both" / "Proposal only"
- [ ] Done

---

### M-10 · ScopeBuilder shows raw fee ID, no back-link
**File:** `ScopeBuilder.svelte:237–239`
**Detail:** Toolbar shows `Fee: 24_96606_1` (database key). No breadcrumb or back-link to proposal.
**Fix:** Display proposal number (`proposalNumber` fetched with scope data). Add back-link: "← Proposal 25-97105".
- [ ] Done

---

### M-11 · BaseListCard uses layout utilities not semantic classes
**File:** `BaseListCard.svelte:46,50,52,63,65,71,77,85,98,113`
**Detail:** Inner layout built from `flex items-start justify-between gap-3` etc. — Tailwind mental model persisting post-Tailwind.
**Fix:** Extract to `.emittiv-list-card__body`, `.emittiv-list-card__title-row`, `.emittiv-list-card__meta`, `.emittiv-list-card__actions`.
- [ ] Done

---

### M-12 · `BaseModal` inline `style="max-width"` defeats size class variants
**File:** `BaseModal.svelte:60`, `app.css:935–975`
**Detail:** Inline `style="max-width: {maxWidth};"` overrides `emittiv-modal--sm/md/lg` size classes (which use `!important` to fight back). Creates a specificity arms race.
**Fix:** Add a `size` prop (`'sm' | 'md' | 'lg' | 'xl'`), apply `emittiv-modal--{size}` class instead of inline style. Remove `!important` from size classes.
- [ ] Done

---

### M-13 · Status change confirmation needs count in label
**File:** `BulkActionBar.svelte:25–31`
**Detail:** Bulk delete confirm says "Confirm Delete" — no indication of how many items will be deleted.
**Fix:** Show count: "Confirm Delete 7 Projects". Add error display (not just `console.error`) for failed batch operations.
- [ ] Done

---

### M-14 · FormSelect placeholder option disappears after selection
**File:** `FormSelect.svelte:31–33`
**Detail:** `<option value="" disabled>` vanishes once user selects. Cannot return to unset state on optional fields.
**Fix:** For optional fields, use `<option value="">—</option>` (not disabled) as permanent first option. Keep disabled pattern for required fields, controlled by `required` prop.
- [ ] Done

---

### M-15 · TypeaheadSelect label ID generation can collide
**File:** `TypeaheadSelect.svelte:147,161`
**Detail:** ID is `typeahead-{label.toLowerCase()}`. Two "Company" typeaheads on same page share an ID.
**Fix:** Use `Math.random()` or `crypto.randomUUID()` (same as `FormInput.svelte:20`).
- [ ] Done

---

### M-16 · `emittiv-checkbox:focus` uses `:focus` not `:focus-visible`
**File:** `app.css:348–352`
**Detail:** Inconsistent with the global `:focus-visible` pattern used everywhere else.
**Fix:** Change `.emittiv-checkbox:focus` to `.emittiv-checkbox:focus-visible`.
- [ ] Done

---

### M-17 · ProposalCard metadata uses `<br/>` inside flex row
**File:** `ProposalCard.svelte:91–96`
**Detail:** `Rev:<br/>{proposal.rev}` in a flex row creates awkward two-line-per-column layout.
**Fix:** Use `flex-col` cell with label above value, or `dl`/`dt`/`dd` structure.
- [ ] Done

---

## LOW

### L-1 · No `prefers-reduced-motion` handling
**File:** `app.css`, `BaseModal.svelte`, `GlobalSearchModal.svelte`
**Detail:** Modal animations, spinner rotation — nothing respects `prefers-reduced-motion: reduce`.
**Fix:**
```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```
- [ ] Done

---

### L-2 · `EmptyState.svelte` SVG not aria-hidden
**File:** `EmptyState.svelte:13`, `GlobalSearchModal.svelte:306`
**Fix:** Add `aria-hidden="true"` to decorative SVGs.
- [ ] Done

---

### L-3 · `Input.svelte` uses `text-red-500` instead of `.emittiv-error`
**File:** `Input.svelte:46`
**Fix:** `class="emittiv-error"`
- [ ] Done

---

### L-4 · `emittiv-date-input` hardcodes font-family
**File:** `app.css:2494`
**Detail:** `font-family: 'Montserrat', sans-serif` — only class in the file not using `var(--font-body)`.
**Fix:** `font-family: var(--font-body);`
- [ ] Done

---

### L-5 · Scrollbar styles use raw hex instead of tokens
**File:** `app.css:1973–1993`
**Detail:** `#333`, `#999`, `#ccc` — same values as `--emittiv-darker`, `--emittiv-light`, `--emittiv-lighter`.
**Fix:** Replace with token references.
- [ ] Done

---

### L-6 · Tailwind gradient fragments are dead code
**File:** `app.css:2357–2358`
**Detail:** `--tw-gradient-from` / `--tw-gradient-to` — Tailwind internal variable names. Vestigial, non-functional.
**Fix:** Delete these two classes.
- [ ] Done

---

### L-7 · `emittiv-select` background inconsistent with `emittiv-input`
**File:** `app.css:446` vs `app.css:409`
**Detail:** Input uses `var(--emittiv-dark)` (solid), select uses `rgba(0,0,0,0.15)` (near-transparent). Adjacent pairs look mismatched.
**Fix:** Unify to same background. If difference is intentional, define a token for it.
- [ ] Done

---

### L-8 · `group-hover:scale-105` in Navigation — Tailwind JIT dependency
**File:** `Navigation.svelte:84`
**Detail:** `group-hover:scale-105` only works with Tailwind's JIT. Fragile if class is purged.
**Fix:** Remove class; if scale effect is wanted, implement in `.nav-inactive:hover` CSS.
- [ ] Done

---

### L-9 · Button loading state has no screen reader feedback
**File:** `Button.svelte:21–23`
**Detail:** Spinner div is not `aria-hidden`. No `aria-busy` on button during loading.
**Fix:** Add `aria-busy={loading}` to `<button>`, `aria-hidden="true"` to spinner div.
- [ ] Done

---

### L-10 · StatusBadge color contrast may be insufficient
**File:** `app.css` badge variants
**Detail:** `emittiv-badge--gray` (~2.6:1) and some orange variants below WCAG AA for small text (4.5:1 required).
**Fix:** Check each badge variant with contrast checker; increase text color lightness where needed.
- [ ] Audited
- [ ] Fixed failing variants

---

## Design System: Color Test Page

**Status:** Not started
**Priority:** High — blocks H-9 (off-brand colors) fix

The app needs a small set of accent colors beyond the standard brand palette (for status indicators, data categories, payment states). These need to be:
1. Visually distinct on dark backgrounds
2. Harmonious with `--emittiv-splash: #ff9900`
3. Accessible (4.5:1 minimum for text use)

**Plan:**
- Add a "Colors" section to DevMode showing all current `--emittiv-*` tokens + candidate accent colors
- Document final choices in `~/.claude/skills/design-references/design-system.md` as an E-Fees extension
- Define as `--emittiv-accent-blue`, `--emittiv-accent-green`, `--emittiv-accent-purple`, `--emittiv-accent-yellow` tokens in `app.css`

**Current Tailwind colors being replaced (starting points for token values):**
| Usage | Tailwind | Hex | Notes |
|-------|----------|-----|-------|
| Projects icon / search result | `text-blue-400` | `#60a5fa` | Check contrast on `#333` |
| Companies icon | `text-green-400` | `#4ade80` | Check contrast on `#333` |
| Contacts icon | `text-purple-400` | `#c084fc` | Check contrast on `#333` |
| Warning / override | `text-yellow-500` | `#eab308` | Check contrast on `#000` |
| Paid status | `text-green-500` | `#22c55e` | Check contrast on `#333` |
| Invoiced status | `text-yellow-500` | `#eab308` | Same as warning — distinguish? |
| Payment status (active) | `text-blue-400` | `#60a5fa` | Consider own token |

- [ ] Color test section added to DevMode
- [ ] Colors reviewed and approved
- [ ] Tokens defined in `app.css`
- [ ] Referenced in design-system.md

---

## Won't Fix / By Design

### `Release` field label — stays as-is
**Reason:** Company convention. "Release" distinguishes a released document from an internal revision. "Revision 0" is not a concept used internally. Multiple internal revisions may precede each release. The `rev` field name in the data model is an implementation detail; the UI correctly uses the business term.

---

*Last updated: 2026-03-13*
