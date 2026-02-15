<script lang="ts">
  let { icon, title, description, actionText = '', onAction = null }: {
    icon: string;
    title: string;
    description: string;
    actionText?: string;
    onAction?: (() => void) | null;
  } = $props();
</script>

<div class="flex flex-col items-center justify-center py-12 px-4 text-center">
  <div class="w-16 h-16 mx-auto mb-4 text-emittiv-light">
    <svg class="w-full h-full" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d={icon} />
    </svg>
  </div>

  <h3 class="text-xl font-heading font-semibold text-emittiv-white mb-2">{title}</h3>
  <p class="text-emittiv-light mb-6 max-w-md">{description}</p>

  {#if actionText && onAction}
    <button
      on:click={onAction}
      class="px-6 py-3 bg-emittiv-splash text-emittiv-black font-medium rounded-lg hover:bg-orange-600 transition-smooth"
    >
      {actionText}
    </button>
  {/if}
</div>
