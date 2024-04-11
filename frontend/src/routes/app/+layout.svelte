<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { QueryClient, QueryClientProvider } from '@sveltestack/svelte-query';

  import { getIsLoggedIn } from '../../api';
  import { AppLoaded, getAppStateInitStarted, initAppState } from '../../state';

  onMount(() => {
    if (!getIsLoggedIn()) {
      goto('/');
      return;
    }

    if (!getAppStateInitStarted()) {
      initAppState();
    }
  });

  const queryClient = new QueryClient();
</script>

{#if $AppLoaded}
  <QueryClientProvider client={queryClient}>
    <slot />
  </QueryClientProvider>
{/if}
