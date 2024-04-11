<script lang="ts">
  import { logout } from '../../api';
</script>

<div class="root">
  <div class="nav">
    <a data-sveltekit-preload-code="hover" href="/app/mine" role="button">Mine</a>
    <a data-sveltekit-preload-code="hover" href="/app/inventory" role="button">Inventory</a>
    <a data-sveltekit-preload-code="hover" href="/app/hiscores" role="button">Hiscores</a>
    <a data-sveltekit-preload-code="hover" href="/app/base" role="button">Base</a>
    <button class="logout-button" type="button" on:click={logout}>Logout</button>
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

  .logout-button {
    background-color: #b20934;
  }
  .logout-button:hover {
    background-color: #940020;
  }
</style>
