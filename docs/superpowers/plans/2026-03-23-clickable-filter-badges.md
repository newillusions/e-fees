# Clickable Filter Badges Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make card status badges clickable to toggle filters, add company name click-to-filter on contacts, and fix clearFilters() consistency across all pages.

**Architecture:** Extend existing StatusBadge with optional onclick prop (renders as `<button>` when clickable). Route pages wire badge clicks to their existing filter state. No new state or components.

**Tech Stack:** Svelte 5 (runes), CSS custom properties, SvelteSet

**Spec:** `docs/superpowers/specs/2026-03-23-clickable-filter-badges-design.md`

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `src/lib/components/StatusBadge.svelte` | Modify | Add onclick prop, conditional button/span rendering |
| `src/lib/components/ProjectCard.svelte` | Modify | Add onstatusclick prop, forward to StatusBadge |
| `src/lib/components/ProposalCard.svelte` | Modify | Add onstatusclick prop, forward to StatusBadge |
| `src/lib/components/ContactCard.svelte` | Modify | Add oncompanyclick prop, make company name clickable |
| `src/routes/Projects.svelte` | Modify | Wire onstatusclick, fix clearFilters SvelteSet bug |
| `src/routes/Proposals.svelte` | Modify | Wire onstatusclick, fix clearFilters SvelteSet bug, fix legacy event syntax |
| `src/routes/Companies.svelte` | Modify | Audit clearFilters completeness |
| `src/routes/Contacts.svelte` | Modify | Wire oncompanyclick, audit clearFilters |
| `src/app.css` | Modify | Add .emittiv-badge--clickable styles |

---

### Task 1: CSS — Add clickable badge styles

**Files:**
- Modify: `src/app.css` (after existing `.emittiv-badge` rules, around line 1708)

- [ ] **Step 1: Add .emittiv-badge--clickable CSS**

```css
/* After the existing .emittiv-badge--splash rule */

.emittiv-badge--clickable {
  cursor: pointer;
  border: none;
  background: inherit;
  padding: inherit;
  font: inherit;
  color: inherit;
  line-height: inherit;
  transition: opacity 300ms cubic-bezier(0.4, 0, 0.2, 1);
}
.emittiv-badge--clickable:hover {
  opacity: 0.8;
}
```

- [ ] **Step 2: Verify in browser**

Run: app should already be running via `npm run tauri:dev`
Check: DevTools → no CSS errors

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style(ui): add emittiv-badge--clickable CSS class

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2: StatusBadge — Add onclick prop with button rendering

**Files:**
- Modify: `src/lib/components/StatusBadge.svelte`

- [ ] **Step 1: Update StatusBadge component**

Replace the entire file content with:

```svelte
<script lang="ts">
  let { status, type = 'general', onclick }: {
    status: string;
    type?: 'project' | 'proposal' | 'general';
    onclick?: (status: string) => void;
  } = $props();

  function getStatusClasses(status: string, type: string): string {
    switch (type) {
      case 'project':
        switch (status) {
          case 'Lead':
            return 'emittiv-badge emittiv-badge--gray';
          case 'RFP':
          case 'Submitted':
            return 'emittiv-badge emittiv-badge--blue';
          case 'Awarded':
          case 'Design':
          case 'Construction':
            return 'emittiv-badge emittiv-badge--green';
          case 'Completed':
            return 'emittiv-badge emittiv-badge--purple';
          case 'Lost':
          case 'No Response':
            return 'emittiv-badge emittiv-badge--red';
          case 'Cancelled':
          case 'On Hold':
            return 'emittiv-badge emittiv-badge--gray';
          case 'Superseded':
            return 'emittiv-badge emittiv-badge--orange';
          default:
            return 'emittiv-badge emittiv-badge--gray';
        }

      case 'proposal':
        switch (status) {
          case 'Draft':
            return 'emittiv-badge emittiv-badge--gray';
          case 'Sent':
            return 'emittiv-badge emittiv-badge--blue';
          case 'Negotiation':
            return 'emittiv-badge emittiv-badge--yellow';
          case 'Accepted':
            return 'emittiv-badge emittiv-badge--green';
          case 'Rejected':
          case 'No Response':
            return 'emittiv-badge emittiv-badge--red';
          case 'Superseded':
            return 'emittiv-badge emittiv-badge--orange';
          default:
            return 'emittiv-badge emittiv-badge--gray';
        }

      default:
        return 'emittiv-badge emittiv-badge--splash';
    }
  }

  function handleClick(event: Event) {
    event.stopPropagation();
    onclick?.(status);
  }
</script>

{#if onclick}
  <button
    type="button"
    class="{getStatusClasses(status, type)} emittiv-badge--clickable"
    onclick={handleClick}
  >
    {status}
  </button>
{:else}
  <span class={getStatusClasses(status, type)}>
    {status}
  </span>
{/if}
```

- [ ] **Step 2: Verify existing badges still render correctly**

Take a screenshot of the Projects page — badges should look identical (no onclick passed yet).

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/StatusBadge.svelte
git commit -m "feat(ui): add onclick prop to StatusBadge component

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3: ProjectCard — Forward onstatusclick to StatusBadge

**Files:**
- Modify: `src/lib/components/ProjectCard.svelte:7` (props) and `~line 67` (StatusBadge usage)

- [ ] **Step 1: Add onstatusclick prop**

In the props destructure (line 7), add `onstatusclick`:

```typescript
let { project, clickable = true, showFolderLink = true, onFolderClick = undefined, selectable = false, selected = false, onedit, onview, onselect, onstatusclick }: {
    project: Project;
    clickable?: boolean;
    showFolderLink?: boolean;
    onFolderClick?: ((project: Project) => void) | undefined;
    selectable?: boolean;
    selected?: boolean;
    onedit?: (project: Project) => void;
    onview?: (project: Project) => void;
    onselect?: (selected: boolean) => void;
    onstatusclick?: (status: string) => void;
  } = $props();
```

- [ ] **Step 2: Pass onclick to StatusBadge**

Find the `<StatusBadge>` usage and add the onclick prop:

```svelte
<StatusBadge status={project.status} type="project" onclick={onstatusclick} />
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ProjectCard.svelte
git commit -m "feat(ui): add onstatusclick prop to ProjectCard

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4: ProposalCard — Forward onstatusclick to StatusBadge

**Files:**
- Modify: `src/lib/components/ProposalCard.svelte` (props and StatusBadge usage)

- [ ] **Step 1: Add onstatusclick prop**

In the props destructure, add `onstatusclick`:

```typescript
onstatusclick?: (status: string) => void;
```

- [ ] **Step 2: Pass onclick to StatusBadge**

Update the StatusBadge usage (around line 58):

```svelte
<StatusBadge status={proposal.status} type="proposal" onclick={onstatusclick} />
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ProposalCard.svelte
git commit -m "feat(ui): add onstatusclick prop to ProposalCard

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5: ContactCard — Add clickable company name

**Files:**
- Modify: `src/lib/components/ContactCard.svelte:6` (props) and `~line 47` (company name display)

- [ ] **Step 1: Add oncompanyclick prop**

```typescript
let { contact, clickable = true, companyName = '', selectable = false, selected = false, onedit, onview, onselect, oncompanyclick }: {
    contact: Contact;
    clickable?: boolean;
    companyName?: string;
    selectable?: boolean;
    selected?: boolean;
    onedit?: (contact: Contact) => void;
    onview?: (contact: Contact) => void;
    onselect?: (selected: boolean) => void;
    oncompanyclick?: (companyName: string) => void;
  } = $props();
```

- [ ] **Step 2: Make company name clickable**

Replace the company name display (around line 47):

```svelte
<p class="text-sm text-emittiv-lighter">
  {#if companyName && oncompanyclick}
    <button
      type="button"
      class="emittiv-badge--clickable text-sm text-emittiv-lighter"
      onclick={(e) => { e.stopPropagation(); oncompanyclick?.(companyName); }}
      style="text-decoration: underline; text-decoration-style: dotted; text-underline-offset: 2px;"
    >
      {companyName}
    </button>
  {:else if companyName}
    {companyName}
  {/if}
</p>
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ContactCard.svelte
git commit -m "feat(ui): add oncompanyclick prop to ContactCard

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 6: Projects.svelte — Wire badge click + fix clearFilters

**Files:**
- Modify: `src/routes/Projects.svelte:194-199` (clearFilters) and `~line 411` (ProjectCard usage)

- [ ] **Step 1: Fix clearFilters — use SvelteSet**

Change line 196 from `statusSelected = new Set();` to:

```typescript
statusSelected = new SvelteSet();
```

Ensure `SvelteSet` is imported (check existing imports — it should already be imported since `statusSelected` is initialized with it on line 46).

- [ ] **Step 2: Wire onstatusclick to ProjectCard**

Find the `<ProjectCard>` usage (around line 411) and add:

```svelte
<ProjectCard
  {project}
  selectable={selectMode}
  selected={selectedIds.has(extractIdFromRelation(project.id || ''))}
  onFolderClick={openProjectFolder}
  onedit={(project) => handleEditProject(project)}
  onview={(project) => handleViewProject(project)}
  onselect={() => toggleSelect(extractIdFromRelation(project.id || ''))}
  onstatusclick={(status) => {
    const next = new SvelteSet(statusSelected);
    if (next.has(status)) next.delete(status);
    else next.add(status);
    statusSelected = next;
  }}
/>
```

- [ ] **Step 3: Verify in running app**

Navigate to Projects page. Click a status badge (e.g. "Awarded"). Verify:
1. The StatusChips bar shows "Awarded" as active
2. The list filters to only Awarded projects
3. Click the badge again — filter clears
4. Click "Clear all filters" — resets everything

- [ ] **Step 4: Commit**

```bash
git add src/routes/Projects.svelte
git commit -m "feat(ui): wire status badge click-to-filter on Projects page

Fixes clearFilters() to use SvelteSet instead of Set for reactivity.

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 7: Proposals.svelte — Wire badge click + fix clearFilters + fix legacy event

**Files:**
- Modify: `src/routes/Proposals.svelte:258-263` (clearFilters), `~line 332` (ResultsCounter event), and ProposalCard usage

- [ ] **Step 1: Fix clearFilters — use SvelteSet**

Change line 260 from `statusSelected = new Set();` to:

```typescript
statusSelected = new SvelteSet();
```

- [ ] **Step 2: Fix legacy ResultsCounter event syntax**

Change line 332 from:
```svelte
on:clear-filters={clearFilters}
```
to:
```svelte
onclearfilters={clearFilters}
```

- [ ] **Step 3: Wire onstatusclick to ProposalCard**

Find the ProposalCard usage and add:

```svelte
onstatusclick={(status) => {
  const next = new SvelteSet(statusSelected);
  if (next.has(status)) next.delete(status);
  else next.add(status);
  statusSelected = next;
}}
```

- [ ] **Step 4: Verify in running app**

Navigate to Proposals page. Click a status badge. Verify filter toggles correctly.

- [ ] **Step 5: Commit**

```bash
git add src/routes/Proposals.svelte
git commit -m "feat(ui): wire status badge click-to-filter on Proposals page

Fixes clearFilters() SvelteSet bug and legacy on:clear-filters syntax.

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 8: Contacts.svelte — Wire company name click + audit clearFilters

**Files:**
- Modify: `src/routes/Contacts.svelte` (ContactCard usage)

- [ ] **Step 1: Audit clearFilters**

Read Contacts.svelte clearFilters function. Verify it resets: `searchQuery`, `filters.company`, `filters.country`, `filters.position`, `dateFrom`, `dateTo`. Fix any missing resets.

- [ ] **Step 2: Wire oncompanyclick to ContactCard**

Find the ContactCard usage and add:

```svelte
oncompanyclick={(name) => {
  filters.company = filters.company === name ? '' : name;
}}
```

- [ ] **Step 3: Verify in running app**

Navigate to Contacts page. Click a company name on a contact card. Verify:
1. The company dropdown filter updates to that company
2. The list filters to contacts from that company
3. Click the same company name again — filter clears
4. "Clear all filters" resets everything

- [ ] **Step 4: Commit**

```bash
git add src/routes/Contacts.svelte
git commit -m "feat(ui): wire company name click-to-filter on Contacts page

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 9: Companies.svelte — Audit clearFilters

**Files:**
- Modify: `src/routes/Companies.svelte` (only if clearFilters is incomplete)

- [ ] **Step 1: Audit clearFilters**

Read Companies.svelte clearFilters function. Verify it resets: `searchQuery`, `filters.country`, `filters.city`, `dateFrom`, `dateTo`. Fix any missing resets.

- [ ] **Step 2: Commit (only if changes made)**

```bash
git add src/routes/Companies.svelte
git commit -m "fix(ui): ensure Companies clearFilters resets all state

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 10: End-to-end verification

- [ ] **Step 1: Full visual verification**

Take screenshots of each page showing:
1. Projects — click "Awarded" badge → list filters, chip activates
2. Proposals — click "Draft" badge → list filters
3. Contacts — click company name → dropdown updates, list filters
4. All pages — "Clear all filters" resets everything

- [ ] **Step 2: Run smoke test**

Run the smoke test checks to verify nothing regressed — especially navigation, search filter, and dropdown filter checks.

- [ ] **Step 3: Squash commit (optional)**

If there are many small commits, consider squashing into a single feature commit before shipping.
