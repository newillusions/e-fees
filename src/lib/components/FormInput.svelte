<script lang="ts">
  let { type = 'text', value = $bindable(''), placeholder = '', label = '', id = '', required = false, disabled = false, error = '', readonly = false, maxlength = undefined as number | undefined, min = undefined as number | undefined, max = undefined as number | undefined, autocomplete = undefined as HTMLInputElement['autocomplete'] | undefined, className = '' }: {
    type?: string;
    value?: string | number;
    placeholder?: string;
    label?: string;
    id?: string;
    required?: boolean;
    disabled?: boolean;
    error?: string;
    readonly?: boolean;
    maxlength?: number | undefined;
    min?: number | undefined;
    max?: number | undefined;
    autocomplete?: HTMLInputElement['autocomplete'] | undefined;
    className?: string;
  } = $props();

  // Generate ID if not provided
  const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;

  // Build class list using shared emittiv-input styles
  const inputClasses = $derived([
    'emittiv-input',
    error ? 'emittiv-input--error' : '',
    readonly ? 'emittiv-input--readonly' : '',
    className
  ]
    .filter(Boolean)
    .join(' '));
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
    autocomplete={autocomplete}
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
