<script lang="ts">
  /**
   * PanelCard - Reusable panel structure for pricing module
   * Uses fixed px values for desktop app OS scaling
   * Supports Svelte 5 snippets for customizable header/footer
   */

  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    headerActions?: Snippet;
    footerContent?: Snippet;
    className?: string;
    children?: Snippet;
  }

  let { title, headerActions, footerContent, className = '', children }: Props = $props();
</script>

<div class="emittiv-panel {className}">
  <div class="emittiv-panel__header">
    <h3 class="emittiv-panel__title">{title}</h3>
    {#if headerActions}
      {@render headerActions()}
    {/if}
  </div>

  {#if children}
    <div class="emittiv-panel__content">
      {@render children()}
    </div>
  {/if}

  {#if footerContent}
    <div class="emittiv-panel__footer">
      {@render footerContent()}
    </div>
  {/if}
</div>

<style>
  /* Panel Card - Fixed px for desktop app OS scaling */
  .emittiv-panel {
    background-color: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 4px;
    overflow: hidden;
  }

  .emittiv-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    border-bottom: 1px solid var(--emittiv-dark);
  }

  .emittiv-panel__title {
    font-size: 16px; /* Match text-base used on other pages */
    font-weight: 500;
    color: var(--emittiv-white);
    margin: 0;
  }

  .emittiv-panel__content {
    padding: 12px;
  }

  .emittiv-panel__footer {
    padding: 10px 14px;
    border-top: 1px solid var(--emittiv-dark);
    background-color: rgba(0, 0, 0, 0.2);
  }
</style>
