<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  // Props for flexible card configuration
  let { clickable = true, href = '', customClass = '', selectable = false, selected = false }: {
    clickable?: boolean;
    href?: string;
    customClass?: string;
    selectable?: boolean;
    selected?: boolean;
  } = $props();

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

  function handleCheckbox(event: Event) {
    event.stopPropagation();
    dispatch('select', !selected);
  }
</script>

{#if href}
  <a
    {href}
    class="group block list-card px-4 py-3 {customClass}"
    on:mouseenter={handleMouseEnter}
    on:mouseleave={handleMouseLeave}
  >
    <div class="emittiv-list-card__body">
      {#if selectable}
        <div class="flex items-center flex-shrink-0 pt-0.5">
          <input
            type="checkbox"
            checked={selected}
            on:click={handleCheckbox}
            class="emittiv-checkbox"
          />
        </div>
      {/if}

      <!-- Main content area -->
      <div class="emittiv-list-card__content">
        <!-- Title slot -->
        <div class="emittiv-list-card__title-row">
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
      <div class="emittiv-list-card__actions flex items-center gap-2">
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
    class="group list-card px-4 py-3 {clickable ? 'cursor-pointer' : ''} {selected ? 'emittiv-card--selected' : ''} {customClass}"
    on:click={handleClick}
    on:keydown={handleKeydown}
    on:mouseenter={handleMouseEnter}
    on:mouseleave={handleMouseLeave}
    role={clickable ? 'button' : 'presentation'}
    tabindex={clickable ? 0 : null}
  >
    <div class="emittiv-list-card__body">
      {#if selectable}
        <div class="flex items-center flex-shrink-0 pt-0.5">
          <input
            type="checkbox"
            checked={selected}
            on:click={handleCheckbox}
            class="emittiv-checkbox"
          />
        </div>
      {/if}

      <!-- Main content area -->
      <div class="emittiv-list-card__content">
        <!-- Title slot -->
        <div class="emittiv-list-card__title-row">
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
      <div class="emittiv-list-card__actions flex items-center gap-2">
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
