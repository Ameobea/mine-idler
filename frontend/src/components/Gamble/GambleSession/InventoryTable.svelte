<script lang="ts">
  import type { Item } from '../../../protos/mine_pb';
  import InventoryTableRow from './InventoryTableRow.svelte';

  export let FocusedItemList: Item[];
  export let OtherItemList: Item[];

  function moveLists(itemUUID: string) {
    let removedItem: Item | null = null;
    FocusedItemList = FocusedItemList.filter((e) => {
      if (e.itemUuid !== itemUUID) {
        return true;
      } else {
        removedItem = e;
        return false;
      }
    });
    console.log("Removed",removedItem);
    if (removedItem) {

      OtherItemList = [...OtherItemList,removedItem];
    }
  }
</script>

<div class="root">
  <table>
    <thead>
      <tr>
        <th>Item</th>
        <th>Value</th>
      </tr>
    </thead>
    <tbody>
      {#each FocusedItemList as item}
        <InventoryTableRow {moveLists} {item} />
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
