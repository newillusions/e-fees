<script lang="ts">
  let { value = $bindable(''), label = '', id = '', required = false, disabled = false, error = '', options = [] as Array<{ value: string; label: string }>, placeholder = 'Select an option', className = '' }: {
    value?: string;
    label?: string;
    id?: string;
    required?: boolean;
    disabled?: boolean;
    error?: string;
    options?: Array<{ value: string; label: string }>;
    placeholder?: string;
    className?: string;
  } = $props();

  // Generate ID if not provided
  const selectId = id || `select-${Math.random().toString(36).substr(2, 9)}`;

  // Build class list using shared emittiv-select styles
  const selectClasses = $derived(['emittiv-select', error ? 'emittiv-select--error' : '', className]
    .filter(Boolean)
    .join(' '));
</script>

<div class="emittiv-form-field">
  {#if label}
    <label for={selectId} class="emittiv-label" class:emittiv-label--required={required}>
      {label}
    </label>
  {/if}

  <select
    id={selectId}
    {required}
    {disabled}
    bind:value
    class={selectClasses}
    aria-invalid={error ? 'true' : undefined}
    aria-describedby={error ? `${selectId}-error` : undefined}
  >
    {#if placeholder && !value}
      <option value="" disabled>{placeholder}</option>
    {/if}

    {#each options as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>

  {#if error}
    <div id="{selectId}-error" class="emittiv-error" aria-live="polite">{error}</div>
  {/if}
</div>

<style>
  .emittiv-form-field {
    display: flex;
    flex-direction: column;
  }
</style>
