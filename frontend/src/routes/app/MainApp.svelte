<script lang="ts">
  import { AppRoute, CurRoute } from '$lib/AppRoute';
  import MineHome from '../../components/Mine/MineHome.svelte';
  import Overview from '../../components/Overview/Overview.svelte';
  import Inventory from '../../components/Inventory/Inventory.svelte';
  import Hiscores from '../../components/Hiscores/Hiscores.svelte';

  const setRoute = (newRoute: AppRoute) => CurRoute.set(newRoute);
</script>

<div class="root">
  {#if $CurRoute === AppRoute.Overview}
    <Overview {setRoute} />
  {:else if $CurRoute === AppRoute.Mine}
    <MineHome />
  {:else if $CurRoute === AppRoute.Inventory}
    <Inventory />
  {:else if $CurRoute === AppRoute.Hiscores}
    <Hiscores />
  {/if}
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
  }
</style>
