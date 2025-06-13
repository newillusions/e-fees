<script lang="ts">
  export let variant: 'primary' | 'secondary' | 'ghost' = 'primary';
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let disabled = false;
  export let type: 'button' | 'submit' = 'button';
  export let className = '';
  
  const baseClasses = 'font-medium rounded-lg transition-smooth focus:outline-none focus:ring-2 focus:ring-emittiv-splash focus:ring-offset-2 focus:ring-offset-emittiv-black disabled:opacity-50 disabled:cursor-not-allowed';
  
  const variantClasses = {
    primary: 'bg-emittiv-splash text-emittiv-black hover:bg-orange-600 active:bg-orange-700',
    secondary: 'bg-emittiv-dark text-emittiv-white border border-emittiv-light hover:bg-emittiv-lighter hover:text-emittiv-black',
    ghost: 'text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark'
  };
  
  const sizeClasses = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-4 py-2 text-base',
    lg: 'px-6 py-3 text-lg'
  };
</script>

<button 
  {type}
  {disabled}
  class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]} {className}"
  on:click
>
  <slot />
</button>