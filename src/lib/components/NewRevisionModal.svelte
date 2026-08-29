<!--
  New Revision Modal

  Collects the audit-trail fields (author, notes) that `clone_fee_as_revision`
  now requires - see `Revision::new`'s doc comment (crates/e-fees-core/src/models/fee.rs)
  for why `revisions[]` must carry a real entry: `fee.rev` is DB-computed
  from `revisions[*].revision_number`, so a one-click clone with an empty
  history was silently writing `rev = 0` for every revision ever created
  (e-fees revision audit, 2026-08-28).

  Author defaults from settingsStore (the same staff_name/staff_email a new
  fee proposal is stamped with) but stays editable, since the person issuing
  a revision isn't always the person who owns the app session.
-->
<script lang="ts">
  import BaseModal from './BaseModal.svelte';
  import Button from './Button.svelte';
  import { settingsStore } from '$lib/stores/settings';

  let {
    isOpen = $bindable(false),
    proposalNumber = '',
    nextRevisionLabel = '',
    onconfirm,
    onclose
  }: {
    isOpen?: boolean;
    /** e.g. "26-97104" - shown for confirmation context only */
    proposalNumber?: string;
    /** e.g. "FP-02" - the revision label this will create */
    nextRevisionLabel?: string;
    onconfirm?: (details: { authorEmail: string; authorName: string; notes: string }) => void;
    onclose?: () => void;
  } = $props();

  let authorName = $state('');
  let authorEmail = $state('');
  let notes = $state('');
  let submitting = $state(false);

  $effect(() => {
    if (isOpen) {
      authorName = $settingsStore?.staff_name || '';
      authorEmail = $settingsStore?.staff_email || '';
      notes = '';
      submitting = false;
    }
  });

  function closeModal() {
    onclose?.();
  }

  function confirm() {
    submitting = true;
    onconfirm?.({ authorEmail: authorEmail.trim(), authorName: authorName.trim(), notes: notes.trim() });
  }
</script>

<BaseModal {isOpen} onclose={closeModal} title="Create New Revision" maxWidth="480px">
  <div style="padding: 16px;">
    <p style="margin: 0 0 16px; color: var(--lighter, #ccc);">
      {#if proposalNumber && nextRevisionLabel}
        Cloning <strong>{proposalNumber}</strong> as <strong>{nextRevisionLabel}</strong>. The
        current proposal will be marked <strong>Superseded</strong>.
      {:else}
        The current proposal will be marked <strong>Superseded</strong> once the new revision is
        created.
      {/if}
    </p>

    <div style="margin-bottom: 12px;">
      <label for="revision-author-name" style="display: block; margin-bottom: 4px;"
        >Author name</label
      >
      <input
        id="revision-author-name"
        class="emittiv-input"
        type="text"
        bind:value={authorName}
        placeholder="Who is issuing this revision"
      />
    </div>

    <div style="margin-bottom: 12px;">
      <label for="revision-author-email" style="display: block; margin-bottom: 4px;"
        >Author email</label
      >
      <input
        id="revision-author-email"
        class="emittiv-input"
        type="email"
        bind:value={authorEmail}
        placeholder="name@emittiv.com"
      />
    </div>

    <div style="margin-bottom: 16px;">
      <label for="revision-notes" style="display: block; margin-bottom: 4px;">Notes</label>
      <textarea
        id="revision-notes"
        class="emittiv-input"
        rows="3"
        bind:value={notes}
        placeholder="What changed in this revision (scope, pricing, terms)"
      ></textarea>
    </div>

    <div style="display: flex; justify-content: flex-end; gap: 8px;">
      <Button variant="ghost" on:click={closeModal} disabled={submitting}>Cancel</Button>
      <Button variant="primary" on:click={confirm} loading={submitting}>Create Revision</Button>
    </div>
  </div>
</BaseModal>
