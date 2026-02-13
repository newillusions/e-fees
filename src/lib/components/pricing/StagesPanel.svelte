<script lang="ts">
  import type { Stage } from '../../../types/database';
  import { generatePricingId, DEFAULT_DESIGN_STAGES, DEFAULT_POST_CONTRACT_STAGES } from '../../../types/database';
  import { formatPercent } from '$lib/utils/format';
  import { sortable } from '$lib/actions/sortable';
  import IconButton from '../IconButton.svelte';
  import PanelCard from '../PanelCard.svelte';

  interface Props {
    stages: Stage[];
    onUpdateStages: (stages: Stage[]) => void;
    readonly?: boolean;
  }

  let {
    stages = $bindable([]),
    onUpdateStages,
    readonly = false
  }: Props = $props();

  // Separate design and post-contract stages (filter creates new array, so sort is safe)
  const designStages = $derived(stages.filter(s => !s.is_post_contract).sort((a, b) => a.order - b.order));
  const postContractStages = $derived(stages.filter(s => s.is_post_contract).sort((a, b) => a.order - b.order));

  // Calculate totals
  const designTotal = $derived(designStages.reduce((sum, s) => sum + s.percentage, 0));
  // Round to 2 decimals before comparison to handle floating point
  const isDesignValid = $derived(Math.abs(Math.round(designTotal * 100) - 10000) === 0);

  /** Generate a short code from a stage name (first letter of each word, uppercase). */
  function generateCode(name: string): string {
    return name.split(/\s+/).map(w => w.charAt(0).toUpperCase()).join('').slice(0, 4) || 'NS';
  }

  // Stage editing
  function addDesignStage() {
    const newStage: Stage = {
      id: generatePricingId('stage'),
      name: 'New Stage',
      code: generateCode('New Stage'),
      percentage: 0,
      order: designStages.length + 1,
      is_post_contract: false,
    };
    const updated = [...stages, newStage];
    stages = updated;
    onUpdateStages(updated);
  }

  function removeStage(id: string) {
    const updated = stages.filter(s => s.id !== id);
    const designReordered = updated
      .filter(s => !s.is_post_contract)
      .map((s, i) => ({ ...s, order: i + 1 }));
    const postReordered = updated
      .filter(s => s.is_post_contract)
      .map((s, i) => ({ ...s, order: 10 + i }));
    const final = [...designReordered, ...postReordered];
    stages = final;
    onUpdateStages(final);
  }

  function updateStage(id: string, field: keyof Stage, value: string | number | boolean) {
    const updated = stages.map(s => {
      if (s.id !== id) return s;
      const patch: Partial<Stage> = { [field]: value };
      // Auto-regenerate code when name changes
      if (field === 'name' && typeof value === 'string') {
        patch.code = generateCode(value);
      }
      return { ...s, ...patch };
    });
    stages = updated;
    onUpdateStages(updated);
  }

  // Pointer-based reorder handlers
  function handleDesignStageReorder(reordered: Stage[]) {
    const postContract = stages.filter(s => s.is_post_contract);
    const updated = [
      ...reordered.map((s, i) => ({ ...s, order: i + 1 })),
      ...postContract,
    ];
    stages = updated;
    onUpdateStages(updated);
  }

  function handlePostContractReorder(reordered: Stage[]) {
    const design = stages.filter(s => !s.is_post_contract);
    const updated = [
      ...design,
      ...reordered.map((s, i) => ({ ...s, order: 10 + i })),
    ];
    stages = updated;
    onUpdateStages(updated);
  }

  function addPostContractStage() {
    const newStage: Stage = {
      id: generatePricingId('stage'),
      name: 'New Service',
      code: generateCode('New Service'),
      percentage: 0,
      order: 10 + postContractStages.length,
      is_post_contract: true,
    };
    const updated = [...stages, newStage];
    stages = updated;
    onUpdateStages(updated);
  }

  function loadDefaults() {
    const designDefaults: Stage[] = DEFAULT_DESIGN_STAGES.map(s => ({
      ...s,
      id: generatePricingId('stage'),
    }));
    const postDefaults: Stage[] = DEFAULT_POST_CONTRACT_STAGES.map(s => ({
      ...s,
      id: generatePricingId('stage'),
    }));
    const all = [...designDefaults, ...postDefaults];
    stages = all;
    onUpdateStages(all);
  }

  function distributeEvenly() {
    if (designStages.length === 0) return;
    const evenPercent = Math.floor((100 / designStages.length) * 100) / 100;
    let designIndex = 0;
    const updated = stages.map(s => {
      if (s.is_post_contract) return s;
      const isLast = designIndex === designStages.length - 1;
      designIndex++;
      return {
        ...s,
        percentage: isLast
          ? Math.round((100 - evenPercent * (designStages.length - 1)) * 100) / 100
          : evenPercent,
      };
    });
    stages = updated;
    onUpdateStages(updated);
  }
</script>

<div class="space-y-3">
  <!-- Design Stages -->
  <PanelCard title="Design Stages">
    {#snippet headerActions()}
      {#if !readonly}
        <div class="flex items-center gap-2">
          {#if stages.length === 0}
            <button type="button" class="emittiv-text-btn emittiv-text-btn--primary" onclick={loadDefaults}>
              Load Defaults
            </button>
          {:else}
            <button type="button" class="emittiv-text-btn" onclick={distributeEvenly}>
              Distribute Evenly
            </button>
          {/if}
          <IconButton icon="plus" label="Add" variant="primary" size="md" on:click={addDesignStage} />
        </div>
      {/if}
    {/snippet}

    {#if designStages.length === 0}
      <div class="p-4 text-center">
        <p class="text-emittiv-light text-sm mb-1">No design stages defined.</p>
        {#if !readonly}
          <p class="text-emittiv-dark text-xxs">
            Define the design stages for this project.
          </p>
        {/if}
      </div>
    {:else}
      <!-- Header row -->
      <div class="emittiv-sortable-header">
        {#if !readonly}<div class="emittiv-sortable-col--handle"></div>{/if}
        <div class="emittiv-sortable-col--grow">Stage</div>
        <div class="emittiv-sortable-col--pct">Percentage</div>
        {#if !readonly}<div class="emittiv-sortable-col--action"></div>{/if}
      </div>

      <!-- Sortable rows -->
      {#if !readonly}
        <div
          use:sortable={{
            items: designStages,
            onReorder: handleDesignStageReorder,
            dragClass: 'emittiv-sortable-dragging',
            overClass: 'emittiv-drag-over',
          }}
        >
          {#each designStages as stage (stage.id)}
            <div class="emittiv-sortable-row" data-sortable-id={stage.id}>
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
                <div class="flex items-center gap-1.5">
                  <input
                    type="text"
                    class="emittiv-table-input emittiv-table-input--left"
                    value={stage.name}
                    onchange={(e) => updateStage(stage.id, 'name', e.currentTarget.value)}
                  />
                  <span class="text-emittiv-dark text-xxs flex-shrink-0">({stage.code})</span>
                </div>
              </div>
              <div class="emittiv-sortable-col--pct">
                <input
                  type="number"
                  min="0"
                  max="100"
                  step="0.5"
                  class="emittiv-table-input emittiv-table-input--lg"
                  value={stage.percentage}
                  onchange={(e) => updateStage(stage.id, 'percentage', parseFloat(e.currentTarget.value) || 0)}
                />
                <span class="text-emittiv-light">%</span>
              </div>
              <div class="emittiv-sortable-col--action">
                <IconButton
                  icon="trash"
                  variant="danger"
                  size="sm"
                  title="Remove stage"
                  on:click={() => removeStage(stage.id)}
                />
              </div>
            </div>
          {/each}
        </div>
      {:else}
        {#each designStages as stage (stage.id)}
          <div class="emittiv-sortable-row">
            <div class="emittiv-sortable-col--grow">
              <span class="text-emittiv-white">{stage.name}</span>
              <span class="text-emittiv-dark text-xxs ml-1">({stage.code})</span>
            </div>
            <div class="emittiv-sortable-col--pct">
              <span class="text-emittiv-white">{formatPercent(stage.percentage)}</span>
            </div>
          </div>
        {/each}
      {/if}
    {/if}

    {#snippet footerContent()}
      {#if designStages.length > 0}
        <div class="flex items-center justify-end gap-2">
          <span class="text-emittiv-light text-xs">Total:</span>
          <span
            class="font-medium text-xs"
            class:text-green-500={isDesignValid}
            class:text-red-500={!isDesignValid}
          >
            {formatPercent(designTotal, 1)}
          </span>
          {#if isDesignValid}
            <svg class="emittiv-icon-check text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
            </svg>
          {:else}
            <span class="text-red-500 text-xxs">(must equal 100%)</span>
          {/if}
        </div>
      {/if}
    {/snippet}
  </PanelCard>

  <!-- Post-Contract Services -->
  <PanelCard title="Post-Contract Services">
    {#snippet headerActions()}
      {#if !readonly}
        <IconButton icon="plus" label="Add Service" variant="primary" size="md" on:click={addPostContractStage} />
      {/if}
    {/snippet}

    {#if postContractStages.length === 0}
      <div class="p-4 text-center">
        <p class="text-emittiv-light text-sm mb-1">No post-contract services defined.</p>
        {#if !readonly}
          <p class="text-emittiv-dark text-xxs">
            Add services like site visits, commissioning, as-built docs.
          </p>
        {/if}
      </div>
    {:else}
      <!-- Header row -->
      <div class="emittiv-sortable-header">
        {#if !readonly}<div class="emittiv-sortable-col--handle"></div>{/if}
        <div class="emittiv-sortable-col--grow">Service</div>
        {#if !readonly}<div class="emittiv-sortable-col--action"></div>{/if}
      </div>

      <!-- Sortable rows -->
      {#if !readonly}
        <div
          use:sortable={{
            items: postContractStages,
            onReorder: handlePostContractReorder,
            dragClass: 'emittiv-sortable-dragging',
            overClass: 'emittiv-drag-over',
          }}
        >
          {#each postContractStages as stage (stage.id)}
            <div class="emittiv-sortable-row" data-sortable-id={stage.id}>
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
                <div class="flex items-center gap-1.5">
                  <input
                    type="text"
                    class="emittiv-table-input emittiv-table-input--left"
                    value={stage.name}
                    onchange={(e) => updateStage(stage.id, 'name', e.currentTarget.value)}
                  />
                  <span class="text-emittiv-dark text-xxs flex-shrink-0">({stage.code})</span>
                </div>
              </div>
              <div class="emittiv-sortable-col--action ml-auto">
                <IconButton
                  icon="trash"
                  variant="danger"
                  size="sm"
                  title="Remove service"
                  on:click={() => removeStage(stage.id)}
                />
              </div>
            </div>
          {/each}
        </div>
      {:else}
        {#each postContractStages as stage (stage.id)}
          <div class="emittiv-sortable-row">
            <div class="emittiv-sortable-col--grow">
              <span class="text-emittiv-white">{stage.name}</span>
              <span class="text-emittiv-dark text-xxs ml-1">({stage.code})</span>
            </div>
          </div>
        {/each}
      {/if}
    {/if}
  </PanelCard>
</div>
