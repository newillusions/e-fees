<script lang="ts">
  let {
    status,
    type = 'general',
    onclick
  }: {
    status: string;
    type?: 'project' | 'proposal' | 'general';
    onclick?: (status: string) => void;
  } = $props();

  function getStatusClasses(status: string, type: string): string {
    switch (type) {
      case 'project':
        switch (status) {
          case 'Lead':
            return 'emittiv-badge emittiv-badge--gray';
          case 'RFP':
          case 'Submitted':
            return 'emittiv-badge emittiv-badge--blue';
          case 'Awarded':
          case 'Design':
          case 'Construction':
            return 'emittiv-badge emittiv-badge--green';
          case 'Completed':
            return 'emittiv-badge emittiv-badge--purple';
          case 'Lost':
          case 'No Response':
            return 'emittiv-badge emittiv-badge--red';
          case 'Cancelled':
          case 'On Hold':
            return 'emittiv-badge emittiv-badge--gray';
          case 'Superseded':
            return 'emittiv-badge emittiv-badge--orange';
          default:
            return 'emittiv-badge emittiv-badge--gray';
        }

      case 'proposal':
        switch (status) {
          case 'Draft':
            return 'emittiv-badge emittiv-badge--gray';
          case 'Sent':
            return 'emittiv-badge emittiv-badge--blue';
          case 'Negotiation':
            return 'emittiv-badge emittiv-badge--yellow';
          case 'Accepted':
            return 'emittiv-badge emittiv-badge--green';
          case 'Rejected':
          case 'No Response':
            return 'emittiv-badge emittiv-badge--red';
          case 'Superseded':
            return 'emittiv-badge emittiv-badge--orange';
          default:
            return 'emittiv-badge emittiv-badge--gray';
        }

      default:
        return 'emittiv-badge emittiv-badge--splash';
    }
  }

  function handleClick(event: Event) {
    event.stopPropagation();
    onclick?.(status);
  }
</script>

{#if onclick}
  <button
    type="button"
    class="{getStatusClasses(status, type)} emittiv-badge--clickable"
    onclick={handleClick}
  >
    {status}
  </button>
{:else}
  <span class={getStatusClasses(status, type)}>
    {status}
  </span>
{/if}
