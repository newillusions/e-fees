<script lang="ts">
  let { rows = 3, height = 'h-4', width = 'w-full' }: {
    rows?: number;
    height?: string;
    width?: string;
  } = $props();
</script>

<div class="animate-pulse space-y-3">
  {#each Array(rows) as _}
    <div class="skeleton {height} {width}"></div>
  {/each}
</div>
