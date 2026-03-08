<script lang="ts">
  import ScopeBuilder from '$lib/components/scope/ScopeBuilder.svelte';

  let { params = {} }: { params?: { id?: string } } = $props();
</script>

<ScopeBuilder {params} />
