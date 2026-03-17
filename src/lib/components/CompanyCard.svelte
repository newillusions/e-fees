<script lang="ts">
  import BaseListCard from './BaseListCard.svelte';
  import ActionButton from './ActionButton.svelte';
  import type { Company } from '../../types';

  let { company, clickable = true, selectable = false, selected = false, onedit, onview, onselect }: {
    company: Company;
    clickable?: boolean;
    selectable?: boolean;
    selected?: boolean;
    onedit?: (company: Company) => void;
    onview?: (company: Company) => void;
    onselect?: (selected: boolean) => void;
  } = $props();

  function handleCardClick() {
    if (clickable) {
      onview?.(company);
    }
  }

  function handleEdit(event: Event) {
    event.stopPropagation();
    onedit?.(company);
  }

  function handleView(event: Event) {
    event.stopPropagation();
    onview?.(company);
  }
</script>

<BaseListCard {clickable} {selectable} {selected} onclick={handleCardClick} {onselect}>
  <!-- Title -->
  <svelte:fragment slot="title">
    <h3
      class="emittiv-card-title"
    >
      {company.name}
    </h3>
  </svelte:fragment>

  <!-- Subtitle -->
  <svelte:fragment slot="subtitle">
    <p class="text-sm text-emittiv-lighter">
      {company.city}, {company.country}
    </p>
  </svelte:fragment>

  <!-- Badge -->
  <svelte:fragment slot="badge">
    <span class="emittiv-badge emittiv-badge--splash">
      {company.abbreviation}
    </span>
  </svelte:fragment>

  <!-- Actions -->
  <svelte:fragment slot="actions">
    <ActionButton type="edit" ariaLabel="Edit company" onclick={handleEdit} />
    <ActionButton type="view" ariaLabel="View company details" onclick={handleView} />
  </svelte:fragment>

  <!-- Extra - Full width body section with all metadata -->
  <svelte:fragment slot="extra">
    <div class="emittiv-card-meta">
      <span>Short:<br />{company.name_short}</span>
      {#if company.reg_no}
        <span>Reg:<br />{company.reg_no}</span>
      {/if}
      {#if company.tax_no}
        <span>VAT:<br />{company.tax_no}</span>
      {/if}
      <span
        >Created:<br />{company.time ? new Date(company.time.created_at)
          .toISOString()
          .slice(2, 10)
          .replace(/-/g, '') : '—'}</span
      >
      <span
        >Updated:<br />{company.time ? new Date(company.time.updated_at)
          .toISOString()
          .slice(2, 10)
          .replace(/-/g, '') : '—'}</span
      >
    </div>
  </svelte:fragment>
</BaseListCard>
