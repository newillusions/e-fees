<script lang="ts">
  let {
    variant = 'primary',
    size = 'md',
    disabled = false,
    loading = false,
    type = 'button',
    className = ''
  }: {
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    disabled?: boolean;
    loading?: boolean;
    type?: 'button' | 'submit';
    className?: string;
  } = $props();

  // Build class string using emittiv-btn CSS classes (fixed px for desktop scaling)
  const buttonClass = $derived(
    `emittiv-btn emittiv-btn--${size} emittiv-btn--${variant} ${className}`
  );
</script>

<button {type} {disabled} class={buttonClass} aria-busy={loading || undefined} on:click>
  {#if loading}
    <div class="emittiv-spinner" aria-hidden="true"></div>
  {/if}
  <slot />
</button>
