<script lang="ts">
  export let type: 'text' | 'email' | 'password' | 'number' | 'search' = 'text';
  export let value = '';
  export let placeholder = '';
  export let label = '';
  export let error = '';
  export let disabled = false;
  export let required = false;
  export let className = '';
  
  let inputElement: HTMLInputElement;
  
  export function focus() {
    inputElement?.focus();
  }
</script>

<div class="w-full">
  {#if label}
    <label class="block text-sm font-medium text-emittiv-lighter mb-2" for={label.toLowerCase()}>
      {label}
      {#if required}<span class="text-emittiv-splash">*</span>{/if}
    </label>
  {/if}
  
  <input
    bind:this={inputElement}
    bind:value
    {type}
    {placeholder}
    {disabled}
    {required}
    id={label.toLowerCase()}
    class="w-full px-4 py-2 bg-emittiv-darker border border-emittiv-dark rounded-lg text-emittiv-white placeholder-emittiv-light focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-smooth disabled:opacity-50 disabled:cursor-not-allowed {error ? 'border-red-500' : ''} {className}"
    on:input
    on:change
    on:focus
    on:blur
  />
  
  {#if error}
    <p class="mt-1 text-sm text-red-500">{error}</p>
  {/if}
</div>