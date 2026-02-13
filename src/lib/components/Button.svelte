<script lang="ts">
  export let variant: 'primary' | 'secondary' | 'ghost' | 'danger' = 'primary';
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let disabled = false;
  export let loading = false;
  export let type: 'button' | 'submit' = 'button';
  export let className = '';

  // Build class string using emittiv-btn CSS classes (fixed px for desktop scaling)
  $: buttonClass = `emittiv-btn emittiv-btn--${size} emittiv-btn--${variant} ${className}`;
</script>

<button
  {type}
  {disabled}
  class={buttonClass}
  on:click
>
  {#if loading}
    <div class="emittiv-spinner"></div>
  {/if}
  <slot />
</button>
