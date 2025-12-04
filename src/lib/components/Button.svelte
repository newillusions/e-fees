<script lang="ts">
  export let variant: 'primary' | 'secondary' | 'ghost' | 'danger' = 'primary';
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let disabled = false;
  export let loading = false;
  export let type: 'button' | 'submit' = 'button';
  export let className = '';

  // Base classes with fixed pixel sizing for DPI consistency
  const baseClasses =
    'font-medium rounded transition-all focus:outline-none focus:ring-2 focus:ring-emittiv-splash focus:ring-offset-1 focus:ring-offset-emittiv-black disabled:opacity-50 disabled:cursor-not-allowed inline-flex items-center justify-center';

  const variantClasses = {
    primary:
      'bg-emittiv-splash text-emittiv-black hover:bg-orange-600 active:bg-orange-700 border-none',
    secondary:
      'bg-transparent text-emittiv-light border border-emittiv-dark hover:border-emittiv-light hover:text-emittiv-white',
    ghost:
      'text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark border-none bg-transparent',
    danger: 'bg-red-600 text-white border border-red-500 hover:bg-red-700'
  };

  // Fixed pixel sizes for consistency across DPI settings
  const sizeStyles = {
    sm: 'height: 28px; padding: 6px 12px; font-size: 12px;',
    md: 'height: 32px; padding: 8px 16px; font-size: 13px;',
    lg: 'height: 40px; padding: 10px 20px; font-size: 14px;'
  };
</script>

<button
  {type}
  disabled={disabled || loading}
  class="{baseClasses} {variantClasses[variant]} {className}"
  style={sizeStyles[size]}
  on:click
>
  {#if loading}
    <div class="emittiv-spinner" style="margin-right: 6px;"></div>
  {/if}
  <slot />
</button>
