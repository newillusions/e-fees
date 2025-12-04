<script lang="ts">
  export let value = '';
  export let label = '';
  export let id = '';
  export let required = false;
  export let disabled = false;
  export let error = '';
  export let options: Array<{ value: string; label: string }> = [];
  export let placeholder = 'Select an option';
  export let className = '';

  // Generate ID if not provided
  const selectId = id || `select-${Math.random().toString(36).substr(2, 9)}`;

  // Build class list using shared emittiv-select styles
  $: selectClasses = ['emittiv-select', error ? 'emittiv-select--error' : '', className]
    .filter(Boolean)
    .join(' ');
</script>

<div class="emittiv-form-field">
  {#if label}
    <label for={selectId} class="emittiv-label" class:emittiv-label--required={required}>
      {label}
    </label>
  {/if}

  <select id={selectId} {required} {disabled} bind:value class={selectClasses}>
    {#if placeholder && !value}
      <option value="" disabled>{placeholder}</option>
    {/if}

    {#each options as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>

  {#if error}
    <div class="emittiv-error">{error}</div>
  {/if}
</div>

<style>
  .emittiv-form-field {
    display: flex;
    flex-direction: column;
  }
</style>
