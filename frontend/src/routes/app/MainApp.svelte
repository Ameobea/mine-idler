<script lang="ts">
  import { AppRoute } from '$lib/AppRoute';
  import MineHome from '../../components/Mine/MineHome.svelte';
  import Overview from '../../components/Overview/Overview.svelte';
  import Inventory from '../../components/Inventory/Inventory.svelte';

  let route: AppRoute = AppRoute.Overview;

  const setRoute = (newRoute: AppRoute) => {
    route = newRoute;
  };
</script>

<div class="root">
  {#if route === AppRoute.Overview}
    <Overview {setRoute} />
  {:else if route === AppRoute.Mine}
    <MineHome />
  {:else if route === AppRoute.Inventory}
    <Inventory />
  {/if}
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
  }
</style>
