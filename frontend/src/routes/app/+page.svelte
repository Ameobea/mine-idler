<script lang="ts">
  import { onMount } from 'svelte';
  import { QueryClient, QueryClientProvider } from '@sveltestack/svelte-query';

  import { AppLoaded, getAppStateInitStarted, initAppState } from '../../state';
  import MainApp from './MainApp.svelte';
  import { goto } from '$app/navigation';
  import { getIsLoggedIn } from '../../api';

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

{#if !$AppLoaded}
  <div>Loading...</div>
{:else}
  <QueryClientProvider client={queryClient}>
    <MainApp />
  </QueryClientProvider>
{/if}
