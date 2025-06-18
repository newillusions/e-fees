<script lang="ts">
  export let name: string;
  export let subtitle: string = '';
  export let location: string = '';
  export let avatar: string = '';
  export let avatarBg: string = 'from-emittiv-splash/20 to-emittiv-splash/10';
  export let avatarBorder: string = 'border-emittiv-splash/30';
  export let avatarText: string = 'text-emittiv-splash';
  export let stats: Array<{label: string, value: number}> = [];
</script>

<!-- Header Info -->
<div class="px-8 pt-8 pb-6">
  <div class="flex items-start gap-4">
    <!-- Avatar/Icon -->
    <div class="w-20 h-16 rounded-2xl bg-gradient-to-br {avatarBg} border {avatarBorder} flex items-center justify-center flex-shrink-0">
      <span class="text-lg font-bold {avatarText}">
        {avatar}
      </span>
    </div>
    
    <!-- Details -->
    <div class="flex-1 min-w-0">
      <h1 class="text-2xl font-heading font-bold text-emittiv-white mb-1">
        {name}
      </h1>
      {#if subtitle}
        <p class="text-sm text-emittiv-lighter mb-1">
          {subtitle}
        </p>
      {/if}
      {#if location}
        <div class="flex items-center gap-1 text-sm text-emittiv-light">
          <svg style="width: 16px; height: 16px;" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 616 0z" />
          </svg>
          <span>{location}</span>
        </div>
      {/if}
    </div>
  </div>
</div>

{#if stats.length > 0}
  <!-- Stats Bar -->
  <div class="px-8 pb-6">
    <div class="flex justify-around gap-4">
      {#each stats as stat}
        <div class="text-center flex-1">
          <div class="text-2xl font-bold text-emittiv-white">{stat.value}</div>
          <div class="text-xs text-emittiv-light uppercase tracking-wider">{stat.label}</div>
        </div>
      {/each}
    </div>
  </div>
{/if}