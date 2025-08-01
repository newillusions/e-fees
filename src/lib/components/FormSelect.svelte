<script lang="ts">
  export let value = '';
  export let label = '';
  export let id = '';
  export let required = false;
  export let disabled = false;
  export let error = '';
  export let options: Array<{value: string, label: string}> = [];
  export let placeholder = 'Select an option';
  
  // Generate ID if not provided
  const selectId = id || `select-${Math.random().toString(36).substr(2, 9)}`;
  
  // Determine styling based on error state
  $: selectClasses = [
    'w-full bg-emittiv-dark border rounded text-emittiv-white',
    'focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash',
    'transition-all cursor-pointer',
    error ? 'border-red-500' : 'border-emittiv-dark',
    disabled ? 'opacity-50 cursor-not-allowed' : ''
  ].filter(Boolean).join(' ');
</script>

<div class="form-select-container">
  {#if label}
    <label 
      for={selectId} 
      class="block font-medium text-emittiv-lighter" 
      style="font-size: 12px; margin-bottom: 4px;"
    >
      {label}
      {#if required}
        <span class="text-red-400">*</span>
      {/if}
    </label>
  {/if}
  
  <select 
    {id}
    {required}
    {disabled}
    bind:value
    class={selectClasses}
    style="padding: 8px 12px; font-size: 12px; height: 32px;"
  >
    {#if placeholder && !value}
      <option value="" disabled>{placeholder}</option>
    {/if}
    
    {#each options as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
  
  {#if error}
    <div class="text-red-400 text-xs mt-1">{error}</div>
  {/if}
</div>

<style>
  .form-select-container {
    display: flex;
    flex-direction: column;
  }
</style>