<script lang="ts">
  export let type = 'text';
  export let value = '';
  export let placeholder = '';
  export let label = '';
  export let id = '';
  export let required = false;
  export let disabled = false;
  export let error = '';
  export let readonly = false;
  export let maxlength: number | undefined = undefined;
  export let autocomplete = '';
  
  // Generate ID if not provided
  const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;
  
  // Determine styling based on error state
  $: inputClasses = [
    'w-full bg-emittiv-dark border rounded text-emittiv-white placeholder-emittiv-light',
    'focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash',
    'transition-all',
    error ? 'border-red-500' : 'border-emittiv-dark',
    disabled ? 'opacity-50 cursor-not-allowed' : '',
    readonly ? 'bg-emittiv-darker opacity-60' : ''
  ].filter(Boolean).join(' ');
</script>

<div class="form-input-container">
  {#if label}
    <label 
      for={inputId} 
      class="block font-medium text-emittiv-lighter" 
      style="font-size: 12px; margin-bottom: 4px;"
    >
      {label}
      {#if required}
        <span class="text-red-400">*</span>
      {/if}
    </label>
  {/if}
  
  <input 
    {id}
    {type}
    {placeholder}
    {required}
    {disabled}
    {readonly}
    {maxlength}
    {autocomplete}
    bind:value
    class={inputClasses}
    style="padding: 8px 12px; font-size: 12px; height: 32px;"
  />
  
  {#if error}
    <div class="text-red-400 text-xs mt-1">{error}</div>
  {/if}
</div>

<style>
  .form-input-container {
    display: flex;
    flex-direction: column;
  }
</style>