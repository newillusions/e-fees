<script lang="ts">
  let { status, type = 'general' }: {
    status: string;
    type?: 'project' | 'proposal' | 'general';
  } = $props();

  function getStatusClasses(status: string, type: string): string {
    switch (type) {
      case 'project':
        switch (status) {
          case 'Draft':
          case 'RFP':
            return 'emittiv-badge emittiv-badge--blue';
          case 'Active':
            return 'emittiv-badge emittiv-badge--green';
          case 'Awarded':
            return 'emittiv-badge emittiv-badge--green';
          case 'Completed':
            return 'emittiv-badge emittiv-badge--purple';
          case 'Lost':
            return 'emittiv-badge emittiv-badge--red';
          case 'Cancelled':
            return 'emittiv-badge emittiv-badge--gray';
          case 'On Hold':
            return 'emittiv-badge emittiv-badge--yellow';
          case 'Revised':
            return 'emittiv-badge emittiv-badge--orange';
          default:
            return 'emittiv-badge emittiv-badge--gray';
        }

      case 'proposal':
        switch (status) {
          case 'Draft':
          case 'Cancelled':
            return 'emittiv-badge emittiv-badge--gray';
          case 'Sent':
            return 'emittiv-badge emittiv-badge--blue';
          case 'Negotiation':
            return 'emittiv-badge emittiv-badge--yellow';
          case 'Awarded':
            return 'emittiv-badge emittiv-badge--green';
          case 'Completed':
            return 'emittiv-badge emittiv-badge--purple';
          case 'Lost':
            return 'emittiv-badge emittiv-badge--red';
          case 'On Hold':
            return 'emittiv-badge emittiv-badge--yellow';
          case 'Revised':
            return 'emittiv-badge emittiv-badge--orange';
          default:
            return 'emittiv-badge emittiv-badge--gray';
        }

      default:
        return 'emittiv-badge emittiv-badge--splash';
    }
  }
</script>

<span class={getStatusClasses(status, type)}>
  {status}
</span>
