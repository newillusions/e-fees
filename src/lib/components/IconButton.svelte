<script lang="ts">
  /**
   * IconButton - Small action button with icon and optional label
   * Uses fixed px values for desktop app OS scaling
   */

  export let icon: 'plus' | 'minus' | 'trash' | 'check' | 'x' | 'edit' | 'split' = 'plus';
  export let label = '';
  export let variant: 'primary' | 'secondary' | 'danger' = 'secondary';
  export let size: 'sm' | 'md' = 'sm';
  export let disabled = false;
  export let title = '';
  export let className = '';

  // SVG paths for each icon
  const icons = {
    plus: 'M12 4v16m8-8H4',
    minus: 'M5 12h14',
    trash: 'M6 18L18 6M6 6l12 12',
    check: 'M5 13l4 4L19 7',
    x: 'M6 18L18 6M6 6l12 12',
    edit: 'M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z',
    split: 'M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4'
  };

  // Build class string - uses fixed px in CSS
  $: buttonClass = `emittiv-icon-btn emittiv-icon-btn--${variant} emittiv-icon-btn--${size} ${className}`;
</script>

<button
  type="button"
  class={buttonClass}
  {disabled}
  {title}
  on:click
>
  <svg class="emittiv-icon-btn__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={icons[icon]} />
  </svg>
  {#if label}
    <span class="emittiv-icon-btn__label">{label}</span>
  {/if}
</button>

<style>
  /* Icon Button - Fixed px for desktop app OS scaling */
  .emittiv-icon-btn {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 2px;
    border-radius: 4px;
    font-weight: 500;
    transition: all 0.15s ease-in-out;
    cursor: pointer;
    border: none;
    background: transparent;
  }

  .emittiv-icon-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Icon sizes - Fixed px */
  .emittiv-icon-btn--sm .emittiv-icon-btn__icon {
    width: 12px;
    height: 12px;
  }

  .emittiv-icon-btn--md .emittiv-icon-btn__icon {
    width: 16px;
    height: 16px;
  }

  /* Label sizing - Fixed px */
  .emittiv-icon-btn--sm .emittiv-icon-btn__label {
    font-size: 10px;
  }

  .emittiv-icon-btn--md .emittiv-icon-btn__label {
    font-size: 12px;
  }

  /* Variants */
  .emittiv-icon-btn--primary {
    color: var(--emittiv-splash);
  }

  .emittiv-icon-btn--primary:hover:not(:disabled) {
    color: #e68a00;
  }

  .emittiv-icon-btn--secondary {
    color: var(--emittiv-light);
  }

  .emittiv-icon-btn--secondary:hover:not(:disabled) {
    color: var(--emittiv-white);
  }

  .emittiv-icon-btn--danger {
    color: var(--emittiv-dark);
  }

  .emittiv-icon-btn--danger:hover:not(:disabled) {
    color: #ef4444;
  }
</style>
