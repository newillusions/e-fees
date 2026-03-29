<script lang="ts">
  let {
    icon,
    title,
    description,
    actionText = '',
    onAction = null
  }: {
    icon: string;
    title: string;
    description: string;
    actionText?: string;
    onAction?: (() => void) | null;
  } = $props();
</script>

<div class="emittiv-empty-state">
  <div class="emittiv-empty-state__icon">
    <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d={icon} />
    </svg>
  </div>

  <h3 class="emittiv-empty-state__title">{title}</h3>
  <p class="emittiv-empty-state__description">{description}</p>

  {#if actionText && onAction}
    <button on:click={onAction} class="emittiv-btn emittiv-btn--primary emittiv-btn--md">
      {actionText}
    </button>
  {/if}
</div>

<style>
  .emittiv-empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 16px;
    text-align: center;
  }

  .emittiv-empty-state__icon {
    width: 64px;
    height: 64px;
    margin-bottom: 16px;
    color: var(--emittiv-light);
  }

  .emittiv-empty-state__icon svg {
    width: 100%;
    height: 100%;
  }

  .emittiv-empty-state__title {
    font-size: 20px;
    font-family: var(--font-heading);
    font-weight: 600;
    color: var(--emittiv-white);
    margin-bottom: 8px;
  }

  .emittiv-empty-state__description {
    color: var(--emittiv-light);
    margin-bottom: 24px;
    max-width: 448px;
  }
</style>
