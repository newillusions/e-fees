<script lang="ts">
  export let type = 'text';
  export let value: string | number = '';
  export let placeholder = '';
  export let label = '';
  export let id = '';
  export let required = false;
  export let disabled = false;
  export let error = '';
  export let readonly = false;
  export let maxlength: number | undefined = undefined;
  export let min: number | undefined = undefined;
  export let max: number | undefined = undefined;
  export let autocomplete = '';
  export let className = '';

  // Generate ID if not provided
  const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;

  // Build class list using shared emittiv-input styles
  $: inputClasses = [
    'emittiv-input',
    error ? 'emittiv-input--error' : '',
    readonly ? 'emittiv-input--readonly' : '',
    className
  ]
    .filter(Boolean)
    .join(' ');
</script>

<div class="emittiv-form-field">
  {#if label}
    <label for={inputId} class="emittiv-label" class:emittiv-label--required={required}>
      {label}
    </label>
  {/if}

  <input
    id={inputId}
    {type}
    {placeholder}
    {required}
    {disabled}
    {readonly}
    {maxlength}
    {min}
    {max}
    {autocomplete}
    bind:value
    class={inputClasses}
  />

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
