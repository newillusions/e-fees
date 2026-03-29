<script lang="ts">
  let {
    clickable = true,
    customPadding = false,
    onclick
  }: {
    clickable?: boolean;
    customPadding?: boolean;
    onclick?: (event: MouseEvent | KeyboardEvent) => void;
  } = $props();

  function handleClick(event: MouseEvent) {
    if (clickable) {
      onclick?.(event);
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (clickable && (event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault();
      onclick?.(event);
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
  class="group list-card {clickable ? 'cursor-pointer' : ''} {customPadding ? '' : 'px-4 py-3'}"
  on:click={handleClick}
  on:keydown={handleKeydown}
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
  role={clickable ? 'button' : 'presentation'}
  tabindex={clickable ? 0 : -1}
>
  <slot />
</div>
