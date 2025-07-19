<script lang="ts">
  export let status: string;
  export let type: 'project' | 'proposal' | 'general' = 'general';
  
  function getStatusClasses(status: string, type: string): string {
    const baseClasses = 'px-2 py-1 rounded-lg text-xs font-medium';
    
    switch (type) {
      case 'project':
        switch (status) {
          case 'Active':
            return `${baseClasses} bg-green-500/10 text-green-400`;
          case 'Completed':
            return `${baseClasses} bg-blue-500/10 text-blue-400`;
          case 'On Hold':
            return `${baseClasses} bg-yellow-500/10 text-yellow-400`;
          case 'Cancelled':
            return `${baseClasses} bg-red-500/10 text-red-400`;
          default:
            return `${baseClasses} bg-emittiv-dark text-emittiv-light`;
        }
      
      case 'proposal':
        switch (status) {
          case 'Awarded':
            return `${baseClasses} bg-green-500/10 text-green-400`;
          case 'Lost':
            return `${baseClasses} bg-red-500/10 text-red-400`;
          case 'Sent':
            return `${baseClasses} bg-blue-500/10 text-blue-400`;
          default:
            return `${baseClasses} bg-emittiv-dark text-emittiv-light`;
        }
      
      default:
        return `${baseClasses} bg-emittiv-splash/10 text-emittiv-splash`;
    }
  }
</script>

<span class={getStatusClasses(status, type)}>
  {status}
</span>