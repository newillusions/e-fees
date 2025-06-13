<script lang="ts">
  export let rows = 3;
  export let height = 'h-4';
  export let width = 'w-full';
</script>

<div class="animate-pulse space-y-3">
  {#each Array(rows) as _}
    <div class="skeleton {height} {width}"></div>
  {/each}
</div>