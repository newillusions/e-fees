# Clickable Filter Badges — Design Spec

**Date:** 2026-03-23
**Status:** Approved
**Scope:** Frontend UI — badge click-to-filter across Projects, Proposals, Contacts pages

---

## Problem

Status badges on cards (e.g. "Awarded", "Draft") are display-only `<span>` elements. Users expect clicking them to filter the list to that status. Company name on contact cards should similarly filter by company. There is no shared contract ensuring all filter state is consistently cleared.

## Solution

Make card badges clickable shortcuts to existing filter mechanisms. No new filter state or components — badges call into the same `statusSelected` SvelteSet and dropdown filters already used by each page.

## Changes

### 1. StatusBadge Component

**File:** `src/lib/components/StatusBadge.svelte`

Add an optional `onclick` callback prop. When provided:
- Render as a `<button>` element (not `<span>`) for proper keyboard accessibility
- `event.stopPropagation()` to prevent card click from also firing
- Call `onclick(status)` on click
- When not provided, render as inert `<span>` (backward compatible)

```typescript
let { status, type = 'general', onclick }: {
  status: string;
  type?: 'project' | 'proposal' | 'general';
  onclick?: (status: string) => void;
} = $props();
```

When `onclick` is provided, render `<button>` with `.emittiv-badge .emittiv-badge--clickable .emittiv-badge--{color}`. When not, render `<span>` as before. The `<button>` element provides keyboard accessibility (Enter/Space) for free without manual `onkeydown` handling.

### 2. Card Components — Pass onclick Through

**ProjectCard.svelte:** Add `onstatusclick` prop, pass to `<StatusBadge onclick={onstatusclick}>`.

**ProposalCard.svelte:** Same pattern — `onstatusclick` prop forwarded to StatusBadge.

**ContactCard.svelte:** Add `oncompanyclick` prop. Make the company name subtitle text clickable — wrap in a `<button>` with `.emittiv-badge--clickable` styling when `oncompanyclick` is provided. Pass the company **short name** (not abbreviation) since that's what `filters.company` matches against. Position badge stays inert.

**CompanyCard.svelte:** No changes — abbreviation badge on the Companies page remains inert.

### 3. Route Pages — Wire Badge Clicks to Filters

**Projects.svelte:**
- Pass `onstatusclick` to each `<ProjectCard>`:
  ```typescript
  onstatusclick={(status) => {
    const next = new SvelteSet(statusSelected);
    if (next.has(status)) next.delete(status);
    else next.add(status);
    statusSelected = next;
  }}
  ```
- Uses the same reassign-SvelteSet pattern as StatusChips to trigger `$derived` reactivity.

**Proposals.svelte:**
- Same pattern — `onstatusclick` creates new SvelteSet and reassigns.

**Contacts.svelte:**
- Company name click sets `filters.company` dropdown to match using the **short name** value (same key used by `createContactFilterConfig` and the dropdown options):
  ```typescript
  oncompanyclick={(shortName) => {
    filters.company = filters.company === shortName ? '' : shortName;
  }}
  ```

**Companies.svelte:** No badge click wiring.

### 4. clearFilters() Audit & Fix

Each page's `clearFilters()` must reset ALL filter state. Use `new SvelteSet()` (not `new Set()`) for status reset to maintain reactivity.

| Page | search | dropdowns | statusSelected | dateFrom/dateTo |
|------|--------|-----------|---------------|-----------------|
| Projects | `searchQuery = ''` | `filters.country = ''`, `filters.city = ''` | `statusSelected = new SvelteSet()` | `dateFrom = ''`, `dateTo = ''` |
| Proposals | `searchQuery = ''` | `filters.company = ''`, `filters.project = ''`, `filters.staff = ''` | `statusSelected = new SvelteSet()` | `dateFrom = ''`, `dateTo = ''` |
| Companies | `searchQuery = ''` | `filters.country = ''`, `filters.city = ''` | N/A | `dateFrom = ''`, `dateTo = ''` |
| Contacts | `searchQuery = ''` | `filters.company = ''`, `filters.country = ''`, `filters.position = ''` | N/A | `dateFrom = ''`, `dateTo = ''` |

**Known bug to fix:** Projects.svelte `clearFilters()` currently uses `new Set()` — must change to `new SvelteSet()` to maintain reactivity for subsequent filter operations.

### 5. CSS

**File:** `src/app.css`

Add `.emittiv-badge--clickable`:
```css
.emittiv-badge--clickable {
  cursor: pointer;
  transition: opacity 300ms cubic-bezier(0.4, 0, 0.2, 1);
  border: none;
  background: inherit;
  padding: inherit;
  font: inherit;
}
.emittiv-badge--clickable:hover {
  opacity: 0.8;
}
```

Button reset styles ensure the `<button>` element looks identical to the `<span>` badge.

## Non-Goals

- No new filter dimensions (e.g. filtering by project number, area)
- Position badges on contacts remain inert
- Company abbreviation badges on the Companies page remain inert
- No changes to filter bar layout or ResultsCounter
- No changes to `filters.ts` shared utilities
- No active/selected visual state on badges — the StatusChips bar already shows which filters are active, and adding a second indicator would create sync complexity

## Testing

- Verify clicking a status badge on a project card toggles that status in StatusChips
- Verify clicking the same badge again deselects it
- Verify badge click does NOT also open the detail panel (stopPropagation)
- Verify clicking company name on a contact card filters to that company
- Verify clearFilters() on each page resets all filter state including badge-triggered filters
- Verify badges without onclick remain inert (no cursor change, no click handler)
- Verify keyboard accessibility: Tab to badge, Enter/Space triggers filter
