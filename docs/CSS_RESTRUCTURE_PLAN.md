# CSS Restructure Plan: From Utility Soup to Semantic Design System

**Date**: 2026-02-16
**Status**: Ready for implementation
**Goal**: Replace 661 lines of "Tailwind-without-Tailwind" utility classes with semantic CSS, making app.css clean and markup readable.

---

## The Problem

Tailwind CSS was removed from the build pipeline, but the approach was replicated in pure CSS — 661 lines of utility classes (`.flex`, `.gap-2`, `.text-xs`, `.rounded-lg`, etc.) that get strung together in markup exactly like Tailwind:

```html
<!-- Current: 231-char class string in NewProjectModal.svelte -->
<button class="bg-emittiv-dark border border-emittiv-dark border-l-0 rounded-r
  text-emittiv-light hover:bg-emittiv-light hover:text-emittiv-darker
  disabled:opacity-50 disabled:cursor-not-allowed transition-colors
  flex items-center justify-center">
```

This defeats the purpose. The app has a fixed design system with ~30 distinct element types. Each should be one CSS class, not 15 utilities strung together.

### Current State

| Metric | Count |
|--------|-------|
| app.css total lines | 2,178 |
| `@layer utilities` lines | 661 (30% of file) |
| `@layer components` lines | 1,360 |
| `.emittiv-*` classes defined | 114 |
| Utility classes defined | ~400 |
| Inline `style=""` attributes | 280 across 30 files |
| Class strings > 100 chars | ~39 |
| Components with local `<style>` | 22 of 64 files |

### What Already Works

The `@layer components` section (114 `.emittiv-*` classes) is well-designed:
- **Form system**: `.emittiv-input`, `.emittiv-select`, `.emittiv-textarea`, `.emittiv-label`, `.emittiv-form-section`, `.emittiv-form-grid`
- **Button system**: `.emittiv-btn` + `--sm/--md/--lg` + `--primary/--secondary/--ghost/--danger`
- **Modal system**: `.emittiv-modal` + `--sm/--md/--lg/--xl`, `__header`, `__title`, `__close`
- **Pricing/table system**: `.emittiv-sortable-*`, `.emittiv-table-input`, `.emittiv-summary-bar`
- **Alerts**: `.emittiv-alert` + `--success/--error/--warning/--info`
- **Spinners**: `.emittiv-spinner`, `.emittiv-spinner-sm`

Components that DO use these classes have clean, readable markup. The problem is that **many components don't use them** and instead compose 10-15 utility classes inline.

---

## Target State

### app.css Structure

```
@layer base          ~65 lines  (unchanged — CSS reset, defaults)
@layer components   ~1,400 lines  (existing 114 + ~20 new semantic classes)
@layer utilities      ~80 lines  (REDUCED from 661 — layout primitives only)
scrollbar styles      ~40 lines  (unchanged — outside layers)
─────────────────────────────
Total:              ~1,585 lines  (down from 2,178)
```

### Utility Classes to KEEP (~80 lines)

Only layout primitives that have no semantic meaning — they describe structure, not appearance:

```css
/* Display */
.flex, .inline-flex, .grid, .block, .inline-block, .hidden, .contents

/* Flex direction & alignment */
.flex-col, .flex-row, .flex-wrap, .flex-1, .flex-auto, .flex-none, .flex-shrink-0, .flex-grow
.items-center, .items-start, .items-end, .items-stretch, .items-baseline
.justify-center, .justify-between, .justify-end, .justify-start
.self-start, .self-end, .self-center

/* Grid */
.grid-cols-2, .grid-cols-3, .grid-cols-4, .col-span-2, .col-span-3

/* Position */
.relative, .absolute, .fixed, .sticky
.inset-0

/* Sizing */
.w-full, .h-full, .min-w-0, .min-h-0

/* Gap (replaces margin hacks) */
.gap-1, .gap-2, .gap-3, .gap-4, .gap-6, .gap-8

/* Visibility & overflow */
.overflow-hidden, .overflow-auto, .overflow-y-auto, .overflow-x-hidden
.truncate

/* Cursor & interaction */
.cursor-pointer, .cursor-not-allowed, .pointer-events-none, .select-none
```

That's ~50 classes in ~80 lines. Everything else gets absorbed into semantic classes.

### Utility Classes to DELETE (~580 lines)

Everything that describes **appearance** (colors, typography, spacing, borders, shadows, transitions, animations, transforms, opacity, hover/focus/active states, backgrounds, etc.) — these belong in semantic component classes.

### New Semantic Classes to ADD (~20)

These cover the repeated patterns found in the audit that aren't yet extracted:

| Class | Replaces | Used in | Count |
|-------|----------|---------|-------|
| `.emittiv-icon-btn` | `p-1 rounded text-emittiv-light hover:text-emittiv-splash hover:bg-emittiv-dark transition-all` | Layout, DetailPanel, modals | ~20 |
| `.emittiv-section-title` | `text-sm font-medium text-emittiv-light uppercase tracking-wider mb-2` | All detail views | ~20 |
| `.emittiv-dropdown-item` | `w-full text-left text-emittiv-white hover:bg-emittiv-dark text-xs...` + inline height/padding | TypeaheadSelect, search | ~15 |
| `.emittiv-modal-backdrop` | `fixed inset-0 bg-black/60 backdrop-blur-sm z-40` | All modals, DetailPanel | ~10 |
| `.emittiv-kbd` | `<kbd>` inline styling | Layout, GlobalSearchModal | ~10 |
| `.emittiv-badge` | `px-2 py-1 rounded-lg text-xs font-medium` + color variants | StatusBadge, cards | ~25 |
| `.emittiv-badge--status-*` | Color variants for draft, submitted, approved, etc. | StatusBadge | ~8 |
| `.emittiv-detail-row` | Detail panel label/value pair layout | Detail components | ~30 |
| `.emittiv-detail-label` | `text-xs text-emittiv-light` | Detail components | ~30 |
| `.emittiv-detail-value` | `text-sm text-emittiv-white` | Detail components | ~30 |
| `.emittiv-page-container` | `max-width: 1400px; margin: auto; padding: 24px; overflow-y: auto` | All route pages | ~8 |
| `.emittiv-page-section` | `margin-bottom: 32px` | Route pages | ~12 |
| `.emittiv-confirm-btn` | Status change confirm buttons with inline size styles | ProposalModal, StatusChangeModal | ~6 |
| `.emittiv-link` | `text-emittiv-splash hover:text-orange-400 underline cursor-pointer` | Various | ~8 |
| `.emittiv-divider` | `border-top: 1px solid var(--emittiv-dark)` with spacing | Modal sections | ~15 |
| `.emittiv-input-group` | Input with prefix/suffix (e.g., NewProjectModal country code + number) | NewProjectModal | ~3 |
| `.emittiv-input-group__prefix` | Left side (country code display) | NewProjectModal | ~3 |
| `.emittiv-input-group__suffix` | Right side (action button) | NewProjectModal | ~3 |
| `.emittiv-stat-card` | Dashboard stat card with icon, value, label | Dashboard | ~4 |
| `.emittiv-empty-state` | Empty state with icon + message + action button | List pages | ~5 |

---

## Implementation Plan

### Phase 1: Add New Semantic Classes (app.css only)

**Files changed**: `src/styles/app.css`
**Risk**: Zero — adding classes doesn't break anything

Add the ~20 new `.emittiv-*` classes listed above to the `@layer components` section. Define each with all the properties currently scattered as utility class combinations in markup.

**Verification**: `npm run build` — CSS compiles, no changes visible yet.

### Phase 2: Migrate Simple Components (15 files)

Replace utility class strings with semantic classes. These components have minimal styling complexity.

**Files** (sorted simplest first):
1. `StatusBadge.svelte` — replace base classes with `.emittiv-badge`
2. `ResultsCounter.svelte` — minimal text utilities
3. `ConnectionStatus.svelte` — small indicator
4. `EmptyState.svelte` — replace button + layout classes with `.emittiv-empty-state`
5. `Button.svelte` — already uses `.emittiv-btn`, just cleanup remaining utilities
6. `IconButton.svelte` — replace with `.emittiv-icon-btn`
7. `Card.svelte` — already has `.emittiv-card`, cleanup
8. `Input.svelte` — already uses `.emittiv-input`, remove redundant utilities
9. `FormInput.svelte` — same
10. `FormSelect.svelte` — same
11. `CompanyCard.svelte` — replace badge/layout utilities
12. `ContactCard.svelte` — same pattern
13. `ProjectCard.svelte` — same pattern
14. `ProposalCard.svelte` — same pattern
15. `ListCard.svelte` — already has `.list-card`

**Pattern for each file**:
1. Read file, identify all class attributes
2. Replace composed utility strings with single semantic class
3. Convert inline `style=""` attributes to class properties
4. Keep ONLY layout primitives (flex, gap, items-center) as utility classes
5. Verify appearance hasn't changed

### Phase 3: Migrate Modal Components (10 files)

**Files**:
1. `BaseModal.svelte` — extract backdrop into `.emittiv-modal-backdrop`
2. `WarningModal.svelte` — use `.emittiv-modal--sm` + semantic classes
3. `CompanyModal.svelte` — use existing `.emittiv-form-*` classes
4. `ContactModal.svelte` — same
5. `ProjectModal.svelte` — same
6. `StatusChangeModal.svelte` — extract confirm buttons to `.emittiv-confirm-btn`
7. `FolderSyncModal.svelte` — clean up utility strings
8. `ActivityLogModal.svelte` — already has good local CSS
9. `SettingsModal.svelte` — largest modal, most inline styles (41)
10. `ImportWizard.svelte` — already has good local CSS

### Phase 4: Migrate Complex Components (12 files)

**Files**:
1. `Navigation.svelte` — already has `.nav-active`/`.nav-inactive`
2. `Layout.svelte` — extract icon buttons, search bar utilities
3. `SplashScreen.svelte` — mostly animation, minimal utilities
4. `FirstRunSetup.svelte` — already has good local CSS
5. `GlobalSearchModal.svelte` — already has good local CSS
6. `TypeaheadSelect.svelte` — extract `.emittiv-dropdown-item`
7. `NewProjectModal.svelte` — extract `.emittiv-input-group`
8. `ProposalModal.svelte` — biggest file, most utility strings (35 inline styles)
9. `base/FormField.svelte` — form wrapper
10. `base/CrudModal.svelte` — CRUD modal base
11. `dashboard/ActivityFeed.svelte` — already has 362 lines local CSS
12. `dashboard/PendingProposals.svelte` — already has 330 lines local CSS

### Phase 5: Migrate Detail & Page Components (12 files)

**Files**:
1. `DetailPanel.svelte` — extract `.emittiv-modal-backdrop`, detail layout
2. `DetailHeader.svelte` — already extracted
3. `ProjectDetail.svelte` — use `.emittiv-section-title`, `.emittiv-detail-row`
4. `ProposalDetail.svelte` — same pattern
5. `CompanyDetail.svelte` — same pattern
6. `ContactDetail.svelte` — same pattern
7. `routes/Dashboard.svelte` — extract `.emittiv-stat-card`, `.emittiv-page-container`
8. `routes/Projects.svelte` — use `.emittiv-page-container`
9. `routes/Proposals.svelte` — same
10. `routes/Companies.svelte` — same
11. `routes/Contacts.svelte` — same
12. `App.svelte` — root component, minimal

### Phase 6: Migrate Pricing Module (8 files)

**The heaviest utility user in the app.** Already has good `.emittiv-sortable-*` and `.emittiv-table-input` classes, but many components still have long class strings.

**Files**:
1. `pricing/FeePricingModal.svelte` — main pricing modal wrapper
2. `pricing/DisciplineStageGrid.svelte` — pricing grid (heaviest)
3. `pricing/PostContractPanel.svelte` — post-contract items
4. `pricing/ReimbursablesPanel.svelte` — reimbursable costs
5. `pricing/SummaryPanel.svelte` — pricing summary
6. `pricing/PaymentSchedulePanel.svelte` — payment schedule
7. `pricing/ExchangeRatePanel.svelte` — currency rates
8. `pricing/TaxPanel.svelte` — tax configuration

### Phase 7: Delete Unused Utilities & Polish

1. **Delete the `@layer utilities` section** (lines 1517-2178) entirely
2. **Re-add ONLY the ~50 layout primitives** listed in "Utility Classes to KEEP"
3. **Remove deprecated classes**: `.emittiv-button-primary`, `.emittiv-button-secondary` (replaced by `.emittiv-btn--primary/--secondary`)
4. **Audit for orphaned classes**: grep all `.emittiv-*` classes, remove any not referenced
5. **Convert remaining inline styles**: target zero `style=""` attributes (or close to it — some dynamic styles are fine)
6. **Final verification**: `npm run build`, visual check all routes

---

## Per-File Migration Pattern

For each component file, follow this exact process:

### Step 1: Identify class attributes
Find every `class="..."` in the markup. List them.

### Step 2: Classify each class
- **Keep as utility**: `flex`, `gap-2`, `items-center` (layout primitives)
- **Replace with semantic**: `bg-emittiv-darker border border-emittiv-dark rounded-lg` → `.emittiv-card`
- **Convert from inline style**: `style="padding: 8px; font-size: 11px"` → add to semantic class

### Step 3: Apply semantic classes
Replace the composed utility string with a single (or at most 2-3) class:
```html
<!-- Before -->
<button class="bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded
  font-medium transition-all flex items-center justify-center disabled:opacity-50"
  style="height: 24px; padding: 4px 8px; font-size: 11px;">

<!-- After -->
<button class="emittiv-confirm-btn flex items-center justify-center">
```

### Step 4: Verify
- Visual appearance unchanged
- No broken hover/focus/disabled states
- No missing colors or spacing

---

## Risk Mitigation

- **Each phase is independently committable** — if anything breaks, revert one phase
- **Phase 1 adds only** — zero risk of breaking existing markup
- **Simplest components first** — build confidence before tackling complex files
- **Pricing last** — highest complexity module gets full pattern library
- **Visual verification** — check each route after each phase
- **Build check** — `npm run build` after every phase

## Verification Checklist

After each phase:
- [ ] `npm run build` succeeds
- [ ] `npm run check` passes (no TypeScript/Svelte errors)
- [ ] Navigate all routes — no visual regressions
- [ ] Open each modal type — styling intact
- [ ] Check hover/focus states — all interactive elements work

After Phase 7:
- [ ] `grep -r "class=" --include="*.svelte" src/ | awk -F'class="' '{if(length($2) > 80) print FILENAME": "length($2)" chars"}'` — no class strings > 80 chars
- [ ] `grep -c 'style="' --include="*.svelte" -r src/` — minimal inline styles
- [ ] `wc -l src/styles/app.css` — target ~1,500-1,600 lines
- [ ] No references to deleted utility classes

---

## CSS Custom Properties Reference

All semantic classes should use these tokens (already defined in `:root`):

```css
/* Colors */
--emittiv-black: #000      --emittiv-darker: #333
--emittiv-dark: #666       --emittiv-light: #999
--emittiv-lighter: #ccc    --emittiv-white: #fff
--emittiv-splash: #f90

/* Typography */
--font-heading: 'Ubuntu', system-ui, sans-serif
--font-body: 'Montserrat', sans-serif

/* Transitions */
--transition-smooth: all 0.3s ease-in-out
--transition-fast: all 0.15s ease-in-out
```

**CRITICAL**: All size values must be fixed `px` — NOT `rem`. OS handles DPI scaling for this desktop app.

---

## Expected Outcomes

| Metric | Before | After |
|--------|--------|-------|
| app.css lines | 2,178 | ~1,585 |
| Utility classes | ~400 | ~50 |
| Semantic classes | 114 | ~134 |
| Class strings > 80 chars | ~39 | 0 |
| Inline `style=""` | 280 | <30 |
| Markup readability | Poor (utility soup) | Good (semantic names) |
| DPI scaling issues | Possible (rem/utility mix) | None (all px, all semantic) |
