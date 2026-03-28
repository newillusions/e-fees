<script lang="ts">
  import { onMount } from 'svelte';
  import type { StageDictEntry } from '$lib/api/stages';
  import { getStageDictionary, filterStageDictionary, generateStageCode } from '$lib/api/stages';

  interface Props {
    value: string;
    onselect: (entry: { name: string; code: string; percentage: number }) => void;
    onchange: (value: string) => void;
    inputClass?: string;
  }

  let { value, onselect, onchange, inputClass = '' }: Props = $props();

  let dictionary: StageDictEntry[] = $state([]);
  let suggestions: StageDictEntry[] = $state([]);
  let showDropdown = $state(false);
  let highlightIndex = $state(-1);
  let inputEl: HTMLInputElement | null = $state(null);

  // Default percentages by canonical name (typical lighting design split)
  const DEFAULT_PERCENTAGES: Record<string, number> = {
    preliminaries: 5,
    concept: 25,
    schematic: 30,
    detailed: 30,
    ift: 10,
  };

  onMount(async () => {
    dictionary = await getStageDictionary();
  });

  function handleInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    onchange(val);
    if (val.trim().length > 0 && dictionary.length > 0) {
      suggestions = filterStageDictionary(dictionary, val);
      showDropdown = suggestions.length > 0;
      highlightIndex = -1;
    } else {
      showDropdown = false;
    }
  }

  function handleFocus() {
    if (value.trim() === '' && dictionary.length > 0) {
      suggestions = dictionary;
      showDropdown = true;
    }
  }

  function selectEntry(entry: StageDictEntry) {
    const percentage = DEFAULT_PERCENTAGES[entry.canonical_name] ?? 0;
    onselect({
      name: entry.default_label,
      code: generateStageCode(entry.default_label),
      percentage,
    });
    showDropdown = false;
    highlightIndex = -1;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!showDropdown) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlightIndex = Math.min(highlightIndex + 1, suggestions.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlightIndex = Math.max(highlightIndex - 1, -1);
    } else if (e.key === 'Enter' && highlightIndex >= 0) {
      e.preventDefault();
      selectEntry(suggestions[highlightIndex]);
    } else if (e.key === 'Escape') {
      showDropdown = false;
    }
  }

  function handleBlur() {
    // Delay to allow click on dropdown item
    setTimeout(() => { showDropdown = false; }, 150);
  }
</script>

<div class="stage-autocomplete">
  <input
    bind:this={inputEl}
    type="text"
    class={inputClass || 'emittiv-table-input emittiv-table-input--left'}
    {value}
    oninput={handleInput}
    onfocus={handleFocus}
    onblur={handleBlur}
    onkeydown={handleKeydown}
    autocomplete="off"
  />
  {#if showDropdown}
    <div class="stage-autocomplete-dropdown">
      {#each suggestions as entry, i (entry.canonical_name)}
        <button
          type="button"
          class="stage-autocomplete-item"
          class:stage-autocomplete-item--active={i === highlightIndex}
          onmousedown={() => selectEntry(entry)}
        >
          <span class="stage-autocomplete-label">{entry.default_label}</span>
          {#if entry.aliases.length > 0}
            <span class="stage-autocomplete-aliases">{entry.aliases.slice(0, 3).join(', ')}</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .stage-autocomplete {
    position: relative;
    flex: 1;
  }

  .stage-autocomplete-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 50;
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 4px;
    margin-top: 2px;
    max-height: 200px;
    overflow-y: auto;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }

  .stage-autocomplete-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 6px 8px;
    border: none;
    background: none;
    color: var(--emittiv-lighter);
    font-size: 12px;
    cursor: pointer;
    text-align: left;
  }

  .stage-autocomplete-item:hover,
  .stage-autocomplete-item--active {
    background: var(--emittiv-dark);
    color: var(--emittiv-white);
  }

  .stage-autocomplete-aliases {
    font-size: 10px;
    color: var(--emittiv-light);
    margin-left: 8px;
    flex-shrink: 0;
  }
</style>
