<script lang="ts">
  let {
    hover = true,
    padding = 'p-6',
    className = ''
  }: { hover?: boolean; padding?: string; className?: string } = $props();
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
    border-color: var(--emittiv-splash);
    transform: translateY(-2px);
    box-shadow: 0 10px 25px rgba(255, 153, 0, 0.1);
  }
</style>
