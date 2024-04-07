<script lang="ts">
  import type { Item } from '../../../protos/mine_pb';
  import LootTableRow from './LootTableRow.svelte';

  export let loot: Item[];

  $: lootToDisplay = loot.slice(-200).reverse();
</script>

<div class="root">
  <table>
    <thead>
      <tr>
        <th>Item</th>
        <th>Quality</th>
        <th>Value</th>
      </tr>
    </thead>
    <tbody>
      {#each lootToDisplay as item}
        <LootTableRow {item} />
      {/each}
    </tbody>
  </table>
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
    max-height: calc(min(100vh, 800px) - 400px);
    overflow-y: auto;
  }
</style>
