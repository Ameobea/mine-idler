<script lang="ts">
  import { onMount } from 'svelte';
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
</script>

{#if !$AppLoaded}
  <div>Loading...</div>
{:else}
  <MainApp />
{/if}
