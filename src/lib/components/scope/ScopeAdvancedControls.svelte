<script lang="ts">
  import type { AssembleRequest } from '$lib/types/scope';
  import { getDeliverables } from '$lib/api/scope';
  import { SvelteSet } from 'svelte/reactivity';

  let {
    feeId,
    stages = [],
    onassemble,
    loading = false,
  }: {
    feeId: string;
    stages?: import('$lib/api/feeStages').FeeStage[];
    onassemble: (request: AssembleRequest) => void;
    loading?: boolean;
  } = $props();

  // Internal state
  let disciplines: string[] = $state([]);
  let selectedDisciplines = $state<Set<string>>(new SvelteSet());
  let selectedStages = $state<Set<string>>(new SvelteSet());
  let disciplinesLoaded = $state(false);

  // Fetch distinct disciplines from deliverables
  async function loadDisciplines() {
    if (disciplinesLoaded) return;
    try {
      const resp = await getDeliverables();
      const uniqueDisciplines = new Set<string>();
      for (const d of resp.data) {
        if (d.discipline) uniqueDisciplines.add(d.discipline);
      }
      disciplines = [...uniqueDisciplines].sort();
      // Default: select all
      selectedDisciplines = new SvelteSet(disciplines);
      disciplinesLoaded = true;
    } catch {
      // Fail silently — advanced controls become unavailable
      disciplines = [];
    }
  }

  // Load disciplines on first render
  $effect(() => {
    loadDisciplines();
  });

  // Stages sorted by order for selection
  let sortedStages = $derived(
    [...stages].sort((a, b) => a.order - b.order)
  );

  function toggleDiscipline(d: string) {
    const next = new SvelteSet(selectedDisciplines);
    if (next.has(d)) next.delete(d);
    else next.add(d);
    selectedDisciplines = next;
  }

  function toggleStage(name: string) {
    const next = new SvelteSet(selectedStages);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    selectedStages = next;
  }

  function handleAssemble() {
    const request: AssembleRequest = {
      fee_id: feeId,
      disciplines: [...selectedDisciplines],
      stages: selectedStages.size > 0 ? [...selectedStages] : undefined,
    };
    onassemble(request);
  }
</script>

<div class="emittiv-scope-advanced">
  <!-- Disciplines -->
  {#if disciplines.length > 0}
    <div class="emittiv-scope-advanced__group">
      <label class="emittiv-scope-advanced__label">Disciplines</label>
      <div class="emittiv-scope-advanced__checkboxes">
        {#each disciplines as d (d)}
          <label class="emittiv-scope-advanced__checkbox-item">
            <input
              type="checkbox"
              class="emittiv-checkbox"
              checked={selectedDisciplines.has(d)}
              onchange={() => toggleDiscipline(d)}
            />
            <span>{d}</span>
          </label>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Stages (optional filter) -->
  {#if sortedStages.length > 0}
    <div class="emittiv-scope-advanced__group">
      <label class="emittiv-scope-advanced__label">Stages (optional filter)</label>
      <div class="emittiv-scope-advanced__checkboxes">
        {#each sortedStages as s (s.id)}
          <label class="emittiv-scope-advanced__checkbox-item">
            <input
              type="checkbox"
              class="emittiv-checkbox"
              checked={selectedStages.has(s.name)}
              onchange={() => toggleStage(s.name)}
            />
            <span>{s.name}</span>
          </label>
        {/each}
      </div>
    </div>
  {/if}

  <button
    class="emittiv-btn emittiv-btn--secondary emittiv-btn--sm"
    onclick={handleAssemble}
    disabled={loading || selectedDisciplines.size === 0}
  >
    {loading ? 'Assembling...' : 'Assemble'}
  </button>
</div>
