# Svelte 5 Event Dispatcher Migration — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace all 34 `createEventDispatcher` usages with Svelte 5 callback props (`$props()` pattern), including consumer updates and dead listener cleanup.

**Architecture:** Bottom-up migration in 3 tiers: leaf components first (no downstream dependencies), then shared components (update with all consumers), then foundational components (BaseModal, DetailPanel, BaseListCard, TypeaheadSelect). Each component migration is atomic — the component and ALL its consumers are updated in one pass.

**Tech Stack:** Svelte 5 runes (`$props()`), TypeScript

**Spec:** Research from Context7 `/sveltejs/svelte` migration guide.

---

## Migration Pattern Reference

### Before (Svelte 4)
```svelte
<!-- Child.svelte -->
<script>
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher();
  function handleClick() { dispatch('close'); }
  function handleSave(data) { dispatch('save', data); }
</script>

<!-- Parent.svelte -->
<Child on:close={handleClose} on:save={handleSave} />
```

### After (Svelte 5)
```svelte
<!-- Child.svelte -->
<script>
  let { onclose, onsave } = $props();
  function handleClick() { onclose?.(); }
  function handleSave(data) { onsave?.(data); }
</script>

<!-- Parent.svelte -->
<Child onclose={handleClose} onsave={handleSave} />
```

### Naming Convention
- `dispatch('close')` → prop `onclose`
- `dispatch('save', data)` → prop `onsave` accepting data arg
- `dispatch('clear-filters')` → prop `onclearfilters` (kebab → flat lowercase)
- `dispatch('field-click')` → prop `onfieldclick`
- `dispatch('status-change')` → prop `onstatuschange`
- `dispatch('add-new')` → prop `onaddnew`
- Always use optional chaining: `onclose?.()` (callback may not be provided)

### Event Forwarding (Svelte 4 bare `on:event`)
```svelte
<!-- Before: forwarding -->
<BaseListCard on:click on:select />

<!-- After: spread or explicit -->
<script>
  let { onclick, onselect, ...rest } = $props();
</script>
<BaseListCard {onclick} {onselect} />
```

---

## Pre-Flight

- [ ] **Verify all tests pass before starting**
Run: `cd /Volumes/base/dev/app/e-fees && npm test`
Expected: 633/633 pass, 0 failures

- [ ] **Verify TypeScript clean**
Run: `npm run check`
Expected: 0 errors

---

## Tier 1 — Leaf Components (16 components, no downstream dependencies)

Each task: remove `createEventDispatcher`, add callback props via `$props()`, update all consumers, run tests.

### Task 1.1: ProposalModalNew.svelte (zero live consumers)

**Files:**
- Modify: `src/lib/components/ProposalModalNew.svelte`

- [ ] Find all `dispatch(` calls and the `createEventDispatcher` import
- [ ] Replace with `let { onclose } = $props()` and `onclose?.()`
- [ ] Remove `import { createEventDispatcher } from 'svelte'`
- [ ] No consumer updates needed (no live consumers)
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ProposalModalNew to callback props`

### Task 1.2: ActivityLogModal.svelte

**Files:**
- Modify: `src/lib/components/ActivityLogModal.svelte`
- Modify: `src/routes/Dashboard.svelte` (consumer)

- [ ] Replace dispatcher with `let { onclose } = $props()`
- [ ] In Dashboard.svelte: change `<ActivityLogModal on:close={...}` to `<ActivityLogModal onclose={...}`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ActivityLogModal to callback props`

### Task 1.3: ImportWizard.svelte

**Files:**
- Modify: `src/lib/components/ImportWizard.svelte`
- Modify: `src/routes/Proposals.svelte` (consumer)

- [ ] Replace dispatcher with `let { onclose, onimported } = $props()`
- [ ] In Proposals.svelte: `on:close` → `onclose`, `on:imported` → `onimported`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ImportWizard to callback props`

### Task 1.4: GlobalSearchModal.svelte

**Files:**
- Modify: `src/lib/components/GlobalSearchModal.svelte`
- Modify: `src/lib/components/Layout.svelte` (consumer)

- [ ] Replace dispatcher with `let { onclose } = $props()`
- [ ] In Layout.svelte: `on:close={closeSearch}` → `onclose={closeSearch}`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate GlobalSearchModal to callback props`

### Task 1.5: FirstRunSetup.svelte

**Files:**
- Modify: `src/lib/components/FirstRunSetup.svelte`
- Modify: `src/App.svelte` (consumer)

- [ ] Replace dispatcher with `let { oncomplete } = $props()`
- [ ] In App.svelte: `on:complete={handleFirstRunComplete}` → `oncomplete={handleFirstRunComplete}`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate FirstRunSetup to callback props`

### Task 1.6: FolderSyncModal.svelte

**Files:**
- Modify: `src/lib/components/FolderSyncModal.svelte`
- Modify: `src/lib/components/SettingsModal.svelte` (consumer)

- [ ] Replace dispatcher with `let { onclose } = $props()`
- [ ] In SettingsModal.svelte: `on:close` → `onclose`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate FolderSyncModal to callback props`

### Task 1.7: SettingsModal.svelte

**Files:**
- Modify: `src/lib/components/SettingsModal.svelte`
- Modify: `src/lib/components/Layout.svelte` (consumer, if wired)

- [ ] Replace dispatcher with `let { onclose } = $props()`
- [ ] Update Layout.svelte consumer if it uses `on:close`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate SettingsModal to callback props`

### Task 1.8: InfoCard.svelte

**Files:**
- Modify: `src/lib/components/InfoCard.svelte`
- Modify: `src/lib/components/ProjectDetail.svelte` (consumer)

- [ ] Replace dispatcher with `let { onfieldclick } = $props()`
- [ ] In ProjectDetail.svelte: `on:field-click` → `onfieldclick`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate InfoCard to callback props`

### Task 1.9: ListCard.svelte

**Files:**
- Modify: `src/lib/components/ListCard.svelte`

- [ ] Replace dispatcher with `let { onclick } = $props()`
- [ ] No consumer updates needed (no named event consumers found)
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ListCard to callback props`

### Task 1.10: FeePricingModal.svelte

**Files:**
- Modify: `src/lib/components/pricing/FeePricingModal.svelte`
- Modify: `src/lib/components/ProposalModal.svelte` (consumer)

- [ ] Replace dispatcher with `let { onclose, onsave } = $props()`
- [ ] In ProposalModal.svelte: `on:close` → `onclose`, `on:save` → `onsave`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate FeePricingModal to callback props`

### Task 1.11: StatusChangeModal.svelte

**Files:**
- Modify: `src/lib/components/StatusChangeModal.svelte`
- Modify: `src/lib/components/ProjectModal.svelte` (consumer)

- [ ] Replace dispatcher with `let { onconfirm, oncancel } = $props()`
- [ ] In ProjectModal.svelte: `on:confirm` → `onconfirm`, `on:cancel` → `oncancel`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate StatusChangeModal to callback props`

### Task 1.12: WarningModal.svelte

**Files:**
- Modify: `src/lib/components/WarningModal.svelte`
- Modify: `src/lib/components/ProjectDetail.svelte` (consumer)
- Modify: `src/lib/components/ProposalDetail.svelte` (consumer)

- [ ] Replace dispatcher with `let { onconfirm, onclose, oncancel } = $props()`
- [ ] Update both Detail components: `on:close` → `onclose`, etc.
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate WarningModal to callback props`

### Task 1.13: ProposalCard.svelte

**Files:**
- Modify: `src/lib/components/ProposalCard.svelte`
- Modify: `src/routes/Proposals.svelte` (consumer)

- [ ] Replace dispatcher with `let { onview, onedit } = $props()`
- [ ] In Proposals.svelte: `on:edit` → `onedit`, `on:view` → `onview`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ProposalCard to callback props`

### Task 1.14: ContactCard.svelte

**Files:**
- Modify: `src/lib/components/ContactCard.svelte`
- Modify: `src/routes/Contacts.svelte` (consumer)

- [ ] Replace dispatcher with `let { onview, onedit, onselect } = $props()`
- [ ] In Contacts.svelte: `on:edit` → `onedit`, `on:view` → `onview`, `on:select` → `onselect`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ContactCard to callback props`

### Task 1.15: CompanyCard.svelte

**Files:**
- Modify: `src/lib/components/CompanyCard.svelte`
- Modify: `src/routes/Companies.svelte` (consumer)

- [ ] Replace dispatcher with `let { onview, onedit, onselect } = $props()`
- [ ] In Companies.svelte: `on:edit` → `onedit`, `on:view` → `onview`, `on:select` → `onselect`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate CompanyCard to callback props`

### Task 1.16: ProjectCard.svelte

**Files:**
- Modify: `src/lib/components/ProjectCard.svelte`
- Modify: `src/routes/Projects.svelte` (consumer)

- [ ] Replace dispatcher with `let { onview, onedit, onselect } = $props()`
- [ ] In Projects.svelte: `on:edit` → `onedit`, `on:view` → `onview`, `on:select` → `onselect`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ProjectCard to callback props`

### Tier 1 Checkpoint

- [ ] Run full test suite: `npm test`
- [ ] Run type check: `npm run check`
- [ ] Verify no remaining `createEventDispatcher` in Tier 1 files: `grep -l createEventDispatcher src/lib/components/{ProposalModalNew,ActivityLogModal,ImportWizard,GlobalSearchModal,FirstRunSetup,FolderSyncModal,SettingsModal,InfoCard,ListCard,StatusChangeModal,WarningModal,ProposalCard,ContactCard,CompanyCard,ProjectCard}.svelte src/lib/components/pricing/FeePricingModal.svelte`
- [ ] Expected: 0 matches
- [ ] Commit checkpoint if not already committed per-task

---

## Tier 2 — Shared Components (12 components, multiple consumers each)

### Task 2.1: ActionButton.svelte + 4 Card consumers

**Files:**
- Modify: `src/lib/components/ActionButton.svelte`
- Modify: `src/lib/components/CompanyCard.svelte`
- Modify: `src/lib/components/ContactCard.svelte`
- Modify: `src/lib/components/ProjectCard.svelte`
- Modify: `src/lib/components/ProposalCard.svelte`

- [ ] Replace ActionButton dispatcher with `let { onclick } = $props()`
- [ ] In all 4 Card components: `<ActionButton on:click={...}` → `<ActionButton onclick={...}`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ActionButton to callback props`

### Task 2.2: ResultsCounter.svelte + 4 Route consumers

**Files:**
- Modify: `src/lib/components/ResultsCounter.svelte`
- Modify: `src/routes/Companies.svelte`
- Modify: `src/routes/Contacts.svelte`
- Modify: `src/routes/Projects.svelte`
- Modify: `src/routes/Proposals.svelte`

- [ ] Replace dispatcher with `let { onclearfilters } = $props()`
- [ ] In all 4 routes: `on:clear-filters` → `onclearfilters`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ResultsCounter to callback props`

### Task 2.3: BulkActionBar.svelte + 4 Route consumers

**Files:**
- Modify: `src/lib/components/BulkActionBar.svelte`
- Modify: `src/routes/Companies.svelte`
- Modify: `src/routes/Contacts.svelte`
- Modify: `src/routes/Projects.svelte`
- Modify: `src/routes/Proposals.svelte`

- [ ] Replace dispatcher with `let { onstatuschange, ondelete, onclear } = $props()`
- [ ] In all 4 routes: `on:status-change` → `onstatuschange`, `on:delete` → `ondelete`, `on:clear` → `onclear`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate BulkActionBar to callback props`

### Task 2.4: ContactDetail.svelte

**Files:**
- Modify: `src/lib/components/ContactDetail.svelte`
- Modify: `src/routes/Contacts.svelte`

- [ ] Replace dispatcher with `let { onedit, onclose } = $props()`
- [ ] In Contacts.svelte: `on:close` → `onclose`, `on:edit` → `onedit`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ContactDetail to callback props`

### Task 2.5: CompanyDetail.svelte

**Files:**
- Modify: `src/lib/components/CompanyDetail.svelte`
- Modify: `src/routes/Companies.svelte`

- [ ] Replace dispatcher with `let { onedit, onclose } = $props()`
- [ ] In Companies.svelte: `on:close` → `onclose`, `on:edit` → `onedit`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate CompanyDetail to callback props`

### Task 2.6: ProjectDetail.svelte

**Files:**
- Modify: `src/lib/components/ProjectDetail.svelte`
- Modify: `src/routes/Projects.svelte`
- Modify: `src/routes/ProjectDetailPage.svelte`

- [ ] Replace dispatcher with `let { onedit, onclose } = $props()`
- [ ] Update both consumers
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ProjectDetail to callback props`

### Task 2.7: ProposalDetail.svelte

**Files:**
- Modify: `src/lib/components/ProposalDetail.svelte`
- Modify: `src/routes/Proposals.svelte`
- Modify: `src/routes/ProposalDetailPage.svelte`

- [ ] Replace dispatcher with `let { onedit, onclose } = $props()`
- [ ] Update both consumers
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ProposalDetail to callback props`

### Task 2.8: NewProjectModal.svelte + 3 consumers

**Files:**
- Modify: `src/lib/components/NewProjectModal.svelte`
- Modify: `src/routes/Projects.svelte`
- Modify: `src/routes/DevMode.svelte`
- Modify: `src/lib/components/ProposalModal.svelte`

- [ ] Replace dispatcher with `let { onclose } = $props()`
- [ ] Update all 3 consumers
- [ ] Clean up dead `on:submit` listener in DevMode.svelte
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate NewProjectModal to callback props`

### Task 2.9: ContactModal.svelte + 3 consumers

**Files:**
- Modify: `src/lib/components/ContactModal.svelte`
- Modify: `src/routes/Contacts.svelte`
- Modify: `src/routes/DevMode.svelte`
- Modify: `src/lib/components/ProposalModal.svelte`

- [ ] Replace dispatcher with `let { onclose } = $props()`
- [ ] Update all 3 consumers
- [ ] Clean up dead `on:submit` listener in DevMode.svelte
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ContactModal to callback props`

### Task 2.10: CompanyModal.svelte + 4 consumers

**Files:**
- Modify: `src/lib/components/CompanyModal.svelte`
- Modify: `src/routes/Companies.svelte`
- Modify: `src/routes/DevMode.svelte`
- Modify: `src/routes/Dashboard.svelte`
- Modify: `src/lib/components/ProposalModal.svelte`

- [ ] Replace dispatcher with `let { onclose } = $props()`
- [ ] Update all 4 consumers
- [ ] Clean up dead `on:submit` in DevMode, dead `on:saved` in Dashboard
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate CompanyModal to callback props`

### Task 2.11: ProjectModal.svelte + 2 consumers

**Files:**
- Modify: `src/lib/components/ProjectModal.svelte`
- Modify: `src/routes/Projects.svelte`
- Modify: `src/routes/Dashboard.svelte`

- [ ] Replace dispatcher with `let { onclose } = $props()`
- [ ] Update both consumers
- [ ] Clean up dead `on:saved` in Dashboard if present
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ProjectModal to callback props`

### Task 2.12: base/CrudModal.svelte

**Files:**
- Modify: `src/lib/components/base/CrudModal.svelte`

- [ ] Replace dispatcher with `let { onclose } = $props()`
- [ ] Check for consumers that use `on:close` on CrudModal instances — update them
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate CrudModal to callback props`

### Tier 2 Checkpoint

- [ ] Run full test suite: `npm test`
- [ ] Run type check: `npm run check`
- [ ] Verify no remaining `createEventDispatcher` in Tier 2 files
- [ ] Grep for remaining `on:` directive usage in routes — should be minimal

---

## Tier 3 — Foundational Components (6 components, everything depends on them)

**CRITICAL: These must be migrated in order. Each one affects many files.**

### Task 3.1: DetailPanel.svelte + 4 Detail consumers

**Files:**
- Modify: `src/lib/components/DetailPanel.svelte`
- Modify: `src/lib/components/CompanyDetail.svelte`
- Modify: `src/lib/components/ContactDetail.svelte`
- Modify: `src/lib/components/ProjectDetail.svelte`
- Modify: `src/lib/components/ProposalDetail.svelte`

- [ ] Replace dispatcher with `let { onclose, onedit } = $props()`
- [ ] In all 4 Detail components: `<DetailPanel on:close on:edit>` → `<DetailPanel {onclose} {onedit}>`
- [ ] Note: Detail components already migrated in Tier 2, so they already have callback props — just pass them through
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate DetailPanel to callback props`

### Task 3.2: BaseListCard.svelte + 4 Card consumers

**Files:**
- Modify: `src/lib/components/BaseListCard.svelte`
- Modify: `src/lib/components/CompanyCard.svelte`
- Modify: `src/lib/components/ContactCard.svelte`
- Modify: `src/lib/components/ProjectCard.svelte`
- Modify: `src/lib/components/ProposalCard.svelte`

- [ ] Replace dispatcher with `let { onclick, onselect } = $props()`
- [ ] In all 4 Card components: replace `on:click` / `on:select` forwarding with explicit prop passing
- [ ] Note: Cards already migrated in Tier 1 — update their BaseListCard usage to prop passing
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate BaseListCard to callback props`

### Task 3.3: base/FormField.svelte

**Files:**
- Modify: `src/lib/components/base/FormField.svelte`
- Modify: `src/lib/components/base/CrudModal.svelte` (consumer)

- [ ] Replace dispatcher with `let { onfieldchange } = $props()`
- [ ] Handle self-recursive `<svelte:self>` usage — pass `{onfieldchange}` to recursive instances
- [ ] In CrudModal.svelte: `on:fieldChange` → `onfieldchange`
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate FormField to callback props`

### Task 3.4: TypeaheadSelect.svelte (highest risk — 6 events, 3 modal consumers)

**Files:**
- Modify: `src/lib/components/TypeaheadSelect.svelte`
- Modify: `src/lib/components/NewProjectModal.svelte` (3 instances)
- Modify: `src/lib/components/ProposalModal.svelte` (3 instances)
- Modify: `src/lib/components/base/FormField.svelte` (1 instance)

- [ ] Replace dispatcher with `let { onclear, onselect, oninput, onfocus, onblur, onaddnew } = $props()`
- [ ] In NewProjectModal: update all 3 TypeaheadSelect instances (country, city, area)
- [ ] In ProposalModal: update all 3 TypeaheadSelect instances (project, company, contact)
- [ ] In FormField: update TypeaheadSelect usage
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate TypeaheadSelect to callback props`

### Task 3.5: BaseModal.svelte (6 modal consumers)

**Files:**
- Modify: `src/lib/components/BaseModal.svelte`
- Modify: `src/lib/components/base/CrudModal.svelte`
- Modify: `src/lib/components/FolderSyncModal.svelte`
- Modify: `src/lib/components/NewProjectModal.svelte`
- Modify: `src/lib/components/pricing/FeePricingModal.svelte`
- Modify: `src/lib/components/ProposalModal.svelte`
- Modify: `src/lib/components/StatusChangeModal.svelte`

- [ ] Replace dispatcher with `let { onclose } = $props()`
- [ ] In all 6 consumers: `<BaseModal on:close` → `<BaseModal onclose`
- [ ] Note: Some consumers already have their own `onclose` prop from earlier tiers — they now pass it through to BaseModal
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate BaseModal to callback props`

### Task 3.6: ProposalModal.svelte (largest component, migrate last)

**Files:**
- Modify: `src/lib/components/ProposalModal.svelte`
- Modify: `src/routes/Proposals.svelte`
- Modify: `src/routes/Dashboard.svelte`
- Modify: `src/routes/ProposalDetailPage.svelte`

- [ ] Replace dispatcher with `let { onclose } = $props()`
- [ ] Update all 3 route consumers
- [ ] Clean up dead `on:saved` listener in Dashboard if still present
- [ ] Run `npm test` — verify pass
- [ ] Commit: `refactor(ui): migrate ProposalModal to callback props`

### Tier 3 Checkpoint

- [ ] Run full test suite: `npm test`
- [ ] Run type check: `npm run check`
- [ ] Verify ZERO remaining `createEventDispatcher` in codebase: `grep -rl createEventDispatcher src/`
- [ ] Expected: 0 matches
- [ ] Verify ZERO remaining `on:` event directives on custom components: `grep -rn 'on:[a-z]' src/ --include='*.svelte' | grep -v 'on:click\|on:change\|on:input\|on:keydown\|on:submit\|on:focus\|on:blur\|on:mouseover\|on:mouseout\|on:mouseenter\|on:mouseleave\|on:keyup\|on:scroll\|on:resize'`
- [ ] Note: native DOM event directives (`on:click` on HTML elements) are a separate migration — not in scope here

---

## Dead Listener Cleanup (found during research)

These are cleaned up as part of their respective tasks above, but listed here for tracking:

- [ ] `src/routes/DevMode.svelte` — `on:submit` on CompanyModal, ContactModal, NewProjectModal (not dispatched)
- [ ] `src/routes/Dashboard.svelte` — `on:saved` on ProposalModal, CompanyModal (not dispatched)

---

## Final Verification

- [ ] `npm test` — all 633+ tests pass
- [ ] `npm run check` — 0 TypeScript errors
- [ ] `grep -rl createEventDispatcher src/` — 0 matches
- [ ] `npm run tauri:dev` — app launches and basic navigation works
- [ ] Commit final: `refactor(ui): complete Svelte 5 event dispatcher migration (34 components)`
