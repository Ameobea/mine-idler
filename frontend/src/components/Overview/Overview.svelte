<script lang="ts">
  import { AppRoute } from '$lib/AppRoute';

  export let setRoute: (route: AppRoute) => void;
</script>

<div class="root">
  <div class="nav">
    <button on:click={() => setRoute(AppRoute.Mine)}>Mine</button>
    <button on:click={() => setRoute(AppRoute.Inventory)}>Inventory</button>
    <button on:click={() => setRoute(AppRoute.Hiscores)}>Hiscores</button>
  </div>
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
  }

  .nav {
    display: flex;
    flex-direction: row;
    gap: 18px;
    justify-content: center;
  }

  .nav button {
    padding: 8px;
    border: none;
    cursor: pointer;
  }
</style>
