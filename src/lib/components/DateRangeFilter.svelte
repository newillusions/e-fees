<script lang="ts">
  /**
   * Month-based date range picker.
   * Two <input type="month"> fields with a dash separator.
   */

  interface Props {
    /** Start month in YYYY-MM format (bindable) */
    from: string;
    /** End month in YYYY-MM format (bindable) */
    to: string;
  }

  let { from = $bindable(), to = $bindable() }: Props = $props();
</script>

<div class="emittiv-date-range">
  <input
    type="month"
    class="emittiv-date-input"
    bind:value={from}
    placeholder="From"
  />
  <span class="emittiv-date-range__sep">&ndash;</span>
  <input
    type="month"
    class="emittiv-date-input"
    bind:value={to}
    placeholder="To"
  />
</div>
