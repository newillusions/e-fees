<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  // Props for flexible card configuration
  export let clickable = true;
  export let href: string = '';
  export let customClass = '';
  
  // Mouse event handlers - now using CSS classes for consistent hover effects
  function handleMouseEnter(event: MouseEvent) {
    // Using CSS classes for hover effects - no manual style changes needed
  }
  
  function handleMouseLeave(event: MouseEvent) {
    // Using CSS classes for hover effects - no manual style changes needed
  }
  
  function handleClick() {
    if (clickable && !href) {
      dispatch('click');
    }
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (clickable && (event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault();
      handleClick();
    }
  }
</script>

{#if href}
  <a 
    {href}
    class="group block list-card px-3 py-2 {customClass}"
    on:mouseenter={handleMouseEnter}
    on:mouseleave={handleMouseLeave}
  >
    <div class="flex items-start justify-between gap-3">
      <!-- Main content area -->
      <div class="flex-1 min-w-0">
        <!-- Title slot -->
        <div class="flex items-center gap-2">
          <slot name="title" />
        </div>
        
        <!-- Subtitle slot (optional) -->
        {#if $$slots.subtitle}
          <div class="mt-1">
            <slot name="subtitle" />
          </div>
        {/if}
        
        <!-- Metadata slot (optional) -->
        {#if $$slots.metadata}
          <div class="mt-1">
            <slot name="metadata" />
          </div>
        {/if}
      </div>
      
      <!-- Actions area -->
      <div class="flex items-center gap-2 flex-shrink-0">
        <!-- Badge slot (optional) -->
        {#if $$slots.badge}
          <slot name="badge" />
        {/if}
        
        <!-- Actions slot -->
        <slot name="actions" />
      </div>
    </div>
    
    <!-- Extra content slot for special cases (optional) -->
    {#if $$slots.extra}
      <div class="mt-2">
        <slot name="extra" />
      </div>
    {/if}
  </a>
{:else}
  <div 
    class="group list-card px-3 py-2 {clickable ? 'cursor-pointer' : ''} {customClass}"
    on:click={handleClick}
    on:keydown={handleKeydown}
    on:mouseenter={handleMouseEnter}
    on:mouseleave={handleMouseLeave}
    role={clickable ? 'button' : 'presentation'}
    tabindex={clickable ? 0 : null}
  >
    <div class="flex items-start justify-between gap-3">
      <!-- Main content area -->
      <div class="flex-1 min-w-0">
        <!-- Title slot -->
        <div class="flex items-center gap-2">
          <slot name="title" />
        </div>
        
        <!-- Subtitle slot (optional) -->
        {#if $$slots.subtitle}
          <div class="mt-1">
            <slot name="subtitle" />
          </div>
        {/if}
        
        <!-- Metadata slot (optional) -->
        {#if $$slots.metadata}
          <div class="mt-1">
            <slot name="metadata" />
          </div>
        {/if}
      </div>
      
      <!-- Actions area -->
      <div class="flex items-center gap-2 flex-shrink-0">
        <!-- Badge slot (optional) -->
        {#if $$slots.badge}
          <slot name="badge" />
        {/if}
        
        <!-- Actions slot -->
        <slot name="actions" />
      </div>
    </div>
    
    <!-- Extra content slot for special cases (optional) -->
    {#if $$slots.extra}
      <div class="mt-2">
        <slot name="extra" />
      </div>
    {/if}
  </div>
{/if}