<script lang="ts">
  export let hover = true;
  export let padding = 'p-6';
  export let className = '';
</script>

<div 
  class="bg-emittiv-darker border border-emittiv-dark rounded-lg transition-smooth {padding} {className}"
  class:hover-effect={hover}
>
  <slot />
</div>

<style>
  .hover-effect {
    transition: all 300ms cubic-bezier(0.4, 0, 0.2, 1);
  }
  
  .hover-effect:hover {
    @apply border-emittiv-splash transform -translate-y-0.5 shadow-lg;
    box-shadow: 0 10px 25px rgba(255, 153, 0, 0.1);
  }
</style>