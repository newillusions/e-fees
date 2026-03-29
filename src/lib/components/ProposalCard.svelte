<script lang="ts">
  import BaseListCard from './BaseListCard.svelte';
  import ActionButton from './ActionButton.svelte';
  import StatusBadge from './StatusBadge.svelte';
  import type { Fee } from '../../types';
  import CurrencyAmount from './CurrencyAmount.svelte';

  let {
    proposal,
    clickable = true,
    projectName = '',
    companyName = '',
    contactName = '',
    selectable = false,
    selected = false,
    onedit,
    onview,
    onselect,
    onstatusclick
  }: {
    proposal: Fee;
    clickable?: boolean;
    projectName?: string;
    companyName?: string;
    contactName?: string;
    selectable?: boolean;
    selected?: boolean;
    onedit?: (proposal: Fee) => void;
    onview?: (proposal: Fee) => void;
    onselect?: (selected: boolean) => void;
    onstatusclick?: (status: string) => void;
  } = $props();

  function handleCardClick() {
    if (clickable) {
      onview?.(proposal);
    }
  }

  function handleEdit(event: Event) {
    event.stopPropagation();
    onedit?.(proposal);
  }

  function handleView(event: Event) {
    event.stopPropagation();
    onview?.(proposal);
  }
</script>

<BaseListCard {clickable} {selectable} {selected} onclick={handleCardClick} {onselect}>
  <!-- Title -->
  <svelte:fragment slot="title">
    <h3 class="emittiv-card-title">
      {proposal.number} - {projectName || 'Unknown Project'}
    </h3>
  </svelte:fragment>

  <!-- Subtitle -->
  <svelte:fragment slot="subtitle">
    <p class="text-sm text-emittiv-lighter">
      {companyName || 'N/A'}
      {#if contactName}
        • {contactName}
      {/if}
    </p>
  </svelte:fragment>

  <!-- Badge -->
  <svelte:fragment slot="badge">
    <StatusBadge status={proposal.status} type="proposal" onclick={onstatusclick} />
  </svelte:fragment>

  <!-- Actions -->
  <svelte:fragment slot="actions">
    <ActionButton type="edit" ariaLabel="Edit proposal" onclick={handleEdit} />
    <ActionButton type="view" ariaLabel="View proposal details" onclick={handleView} />
  </svelte:fragment>

  <!-- Extra - Full width body section with all metadata -->
  <svelte:fragment slot="extra">
    <div class="space-y-1">
      {#if proposal.pricing?.grand_total}
        <div class="text-emittiv-splash font-bold text-sm">
          <CurrencyAmount amount={proposal.pricing.grand_total} config={proposal.pricing.config} />
        </div>
      {/if}
      {#if proposal.package}
        <div class="text-sm text-emittiv-light">
          Package: {proposal.package}
        </div>
      {/if}
      <div class="emittiv-card-meta">
        <span class="emittiv-card-meta__item"
          ><span class="emittiv-card-meta__label">Rev</span>{proposal.rev}</span
        >
        <span class="emittiv-card-meta__item"
          ><span class="emittiv-card-meta__label">Staff</span>{proposal.staff_name || 'N/A'}</span
        >
        <span class="emittiv-card-meta__item"
          ><span class="emittiv-card-meta__label">Issue Date</span>{proposal.issue_date.length === 6
            ? proposal.issue_date
            : new Date(proposal.issue_date).toISOString().slice(2, 10).replace(/-/g, '')}</span
        >
        <span class="emittiv-card-meta__item"
          ><span class="emittiv-card-meta__label">Created</span>{proposal.time
            ? new Date(proposal.time.created_at).toISOString().slice(2, 10).replace(/-/g, '')
            : '—'}</span
        >
      </div>
    </div>
  </svelte:fragment>
</BaseListCard>
