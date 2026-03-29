<script lang="ts">
  let {
    title = 'Information',
    fields = [],
    columns = 3,
    onfieldclick
  }: {
    title?: string;
    fields?: Array<{
      label: string;
      value: string | number | undefined;
      type?: 'text' | 'date' | 'id' | 'link';
      clickable?: boolean;
    }>;
    columns?: number;
    onfieldclick?: (detail: {
      field: {
        label: string;
        value: string | number | undefined;
        type?: string;
        clickable?: boolean;
      };
      index: number;
    }) => void;
  } = $props();

  function formatValue(value: string | number | undefined, type: string = 'text'): string {
    if (!value) return '—';

    switch (type) {
      case 'date':
        return new Date(value).toLocaleDateString('en-US', {
          month: 'short',
          day: 'numeric',
          year: 'numeric'
        });
      case 'id':
        return String(value).replace(/^[^:]+:/, ''); // Remove table prefix from IDs
      default:
        return String(value);
    }
  }
</script>

<section>
  <h2 class="emittiv-section-title">{title}</h2>
  <div class="bg-emittiv-black rounded-xl px-3 py-2 border border-emittiv-dark">
    <div
      class="grid gap-2 text-sm"
      class:grid-cols-1={columns === 1}
      class:grid-cols-2={columns === 2}
      class:grid-cols-3={columns === 3}
      class:grid-cols-4={columns === 4}
    >
      {#each fields as field, index}
        <div>
          <div class="text-xs text-emittiv-light">{field.label}</div>
          {#if field.clickable && field.value && field.value !== '—'}
            <button
              class="emittiv-link text-sm text-left"
              onclick={() => onfieldclick?.({ field, index })}
              title="Click to open"
            >
              {formatValue(field.value, field.type)}
            </button>
          {:else}
            <div class="text-sm text-emittiv-white">{formatValue(field.value, field.type)}</div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</section>
