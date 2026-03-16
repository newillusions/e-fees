<script lang="ts">
  import type { Discipline } from '../../../types/database';
  import { generatePricingId, DEFAULT_DISCIPLINES } from '../../../types/database';
  import { formatPercent } from '$lib/utils/format';
  import { sortable } from '$lib/actions/sortable';
  import IconButton from '../IconButton.svelte';
  import PanelCard from '../PanelCard.svelte';

  interface Props {
    disciplines: Discipline[];
    onUpdate: (disciplines: Discipline[]) => void;
    readonly?: boolean;
  }

  let { disciplines = $bindable([]), onUpdate, readonly = false }: Props = $props();

  // Calculate total percentage
  const totalPercentage = $derived(
    disciplines.reduce((sum, d) => sum + d.percentage, 0)
  );

  // Validation state — round to 2 decimals before comparison to handle floating point
  const isValid = $derived(Math.abs(Math.round(totalPercentage * 100) - 10000) === 0);

  // Sorted for display
  const sorted = $derived([...disciplines].sort((a, b) => a.order - b.order));

  /** Generate a short code from a discipline name (first letter of each word, uppercase). */
  function generateCode(name: string): string {
    return name.split(/\s+/).map(w => w.charAt(0).toUpperCase()).join('').slice(0, 4) || 'ND';
  }

  function addDiscipline() {
    const newDiscipline: Discipline = {
      id: generatePricingId('disc'),
      name: 'New Discipline',
      code: generateCode('New Discipline'),
      percentage: 0,
      order: disciplines.length + 1,
    };
    const updated = [...disciplines, newDiscipline];
    onUpdate(updated);
  }

  function removeDiscipline(id: string) {
    const updated = disciplines
      .filter(d => d.id !== id)
      .map((d, i) => ({ ...d, order: i + 1 }));
    onUpdate(updated);
  }

  function updateDiscipline(id: string, field: 'name' | 'percentage' | 'code', value: string | number) {
    const updated = disciplines.map(d => {
      if (d.id !== id) return d;
      const patch: Partial<Discipline> = { [field]: value };
      // Auto-regenerate code when name changes (unless user has manually edited the code)
      if (field === 'name' && typeof value === 'string') {
        patch.code = generateCode(value);
      }
      return { ...d, ...patch };
    });
    onUpdate(updated);
  }

  function loadDefaults() {
    const defaults: Discipline[] = DEFAULT_DISCIPLINES.map((d, i) => ({
      ...d,
      id: generatePricingId('disc'),
    }));
    onUpdate(defaults);
  }

  function distributeEvenly() {
    if (disciplines.length === 0) return;
    const evenPercent = Math.floor((100 / disciplines.length) * 100) / 100;
    const updated = disciplines.map((d, i) => ({
      ...d,
      percentage: i === disciplines.length - 1
        ? Math.round((100 - evenPercent * (disciplines.length - 1)) * 100) / 100
        : evenPercent,
    }));
    onUpdate(updated);
  }

  function handleReorder(reordered: Discipline[]) {
    const updated = reordered.map((d, i) => ({ ...d, order: i + 1 }));
    onUpdate(updated);
  }
</script>

<PanelCard title="Disciplines">
  {#snippet headerActions()}
    {#if !readonly}
      <div class="flex items-center gap-2">
        {#if disciplines.length === 0}
          <button type="button" class="emittiv-text-btn emittiv-text-btn--primary" onclick={loadDefaults}>
            Load Defaults
          </button>
        {:else}
          <button type="button" class="emittiv-text-btn" onclick={distributeEvenly}>
            Distribute Evenly
          </button>
        {/if}
        <IconButton icon="plus" label="Add" variant="primary" size="md" on:click={addDiscipline} />
      </div>
    {/if}
  {/snippet}

  <!-- Disciplines List -->
  {#if disciplines.length === 0}
    <div class="p-4 text-center">
      <p class="text-emittiv-light text-sm mb-1">No disciplines defined.</p>
      {#if !readonly}
        <p class="text-emittiv-dark text-xxs">
          Define the design disciplines involved in this project.
        </p>
      {/if}
    </div>
  {:else}
    <!-- Header row -->
    <div class="emittiv-sortable-header">
      {#if !readonly}<div class="emittiv-sortable-col--handle"></div>{/if}
      <div class="emittiv-sortable-col--grow">Name</div>
      <div class="emittiv-sortable-col--code">Code</div>
      <div class="emittiv-sortable-col--pct">Percentage</div>
      {#if !readonly}<div class="emittiv-sortable-col--action"></div>{/if}
    </div>

    <!-- Sortable rows -->
    {#if !readonly}
      <div
        use:sortable={{
          items: sorted,
          onReorder: handleReorder,
          dragClass: 'emittiv-sortable-dragging',
          overClass: 'emittiv-drag-over',
        }}
      >
        {#each sorted as discipline (discipline.id)}
          <div class="emittiv-sortable-row" data-sortable-id={discipline.id}>
            <div class="emittiv-sortable-col--handle">
              <svg class="emittiv-drag-handle" fill="currentColor" viewBox="0 0 16 16">
                <circle cx="5" cy="3" r="1.2"/>
                <circle cx="11" cy="3" r="1.2"/>
                <circle cx="5" cy="8" r="1.2"/>
                <circle cx="11" cy="8" r="1.2"/>
                <circle cx="5" cy="13" r="1.2"/>
                <circle cx="11" cy="13" r="1.2"/>
              </svg>
            </div>
            <div class="emittiv-sortable-col--grow">
              <input
                type="text"
                class="emittiv-table-input emittiv-table-input--left"
                value={discipline.name}
                onchange={(e) => updateDiscipline(discipline.id, 'name', e.currentTarget.value)}
              />
            </div>
            <div class="emittiv-sortable-col--code">
              <input
                type="text"
                maxlength="4"
                class="emittiv-table-input emittiv-table-input--center"
                value={discipline.code}
                onchange={(e) => updateDiscipline(discipline.id, 'code', e.currentTarget.value.toUpperCase())}
              />
            </div>
            <div class="emittiv-sortable-col--pct">
              <input
                type="number"
                min="0"
                max="100"
                step="0.5"
                class="emittiv-table-input emittiv-table-input--lg"
                value={discipline.percentage}
                onchange={(e) => updateDiscipline(discipline.id, 'percentage', parseFloat(e.currentTarget.value) || 0)}
              />
              <span class="text-emittiv-light">%</span>
            </div>
            <div class="emittiv-sortable-col--action">
              <IconButton
                icon="trash"
                variant="danger"
                size="sm"
                title="Remove discipline"
                on:click={() => removeDiscipline(discipline.id)}
              />
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <!-- Readonly: no sortable action -->
      {#each sorted as discipline (discipline.id)}
        <div class="emittiv-sortable-row">
          <div class="emittiv-sortable-col--grow">
            <span class="text-emittiv-white">{discipline.name}</span>
          </div>
          <div class="emittiv-sortable-col--code">
            <span class="text-emittiv-dark text-xxs">{discipline.code}</span>
          </div>
          <div class="emittiv-sortable-col--pct">
            <span class="text-emittiv-white">{formatPercent(discipline.percentage)}</span>
          </div>
        </div>
      {/each}
    {/if}
  {/if}

  {#snippet footerContent()}
    {#if disciplines.length > 0}
      <div class="flex items-center justify-end gap-2">
        <span class="text-emittiv-light text-xs">Total:</span>
        <span
          class="font-medium text-xs"
          class:total-valid={isValid}
          class:total-invalid={!isValid}
        >
          {formatPercent(totalPercentage, 1)}
        </span>
        {#if isValid}
          <svg class="emittiv-icon-check total-valid" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
          </svg>
        {:else}
          <span class="total-invalid text-xxs">(must equal 100%)</span>
        {/if}
      </div>
    {/if}
  {/snippet}
</PanelCard>
