<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let clickable = true;
  export let customPadding = false;
  
  function handleClick(event: MouseEvent) {
    if (clickable) {
      dispatch('click', event);
    }
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (clickable && (event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault();
      dispatch('click', event);
    }
  }
  
  function handleMouseEnter(event: MouseEvent) {
    // Using CSS classes for hover effects - no manual style changes needed
  }
  
  function handleMouseLeave(event: MouseEvent) {
    // Using CSS classes for hover effects - no manual style changes needed
  }
</script>

<div 
  class="group list-card {clickable ? 'cursor-pointer' : ''} {customPadding ? '' : 'px-3 py-2'}"
  on:click={handleClick}
  on:keydown={handleKeydown}
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
  role={clickable ? 'button' : 'presentation'}
  tabindex={clickable ? 0 : -1}
>
  <slot />
</div>