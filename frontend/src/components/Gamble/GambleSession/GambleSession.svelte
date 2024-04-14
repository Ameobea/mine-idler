<script lang="ts">
    import { useQuery } from "@sveltestack/svelte-query";
    import { SortBy, SortDirection } from "../../../protos/mine_pb";
    import { PrivateClient } from "../../../api";
    import type { Item } from '../../../protos/mine_pb';
    import InventoryTable from "./InventoryTable.svelte";
    import { type MineSession as MineSessionTy } from '../../../components/Mine/types';
    export let session: MineSessionTy;
  let sortBy = SortBy.Value;
  let sortDirection = SortDirection.Descending;

  let queryRes = useQuery(['inventory', sortBy, sortDirection], () =>
    PrivateClient.getInventory({ pageNumber: 0, pageSize: 2000, sortBy, sortDirection })
  );

  $: fetchedItems = $queryRes.data?.items ?? [];
  $: isLoading = $queryRes.isLoading;
  $: error = $queryRes.error;
  $: isError = $queryRes.isError;
  
  let inventory: Item[] = [];
  let bettingInventory: Item[] = [];

  $: if (fetchedItems.length && inventory.length === 0 && bettingInventory.length === 0) {
    inventory = fetchedItems;
  }
</script>

<!-- pull initial inventory -->
<!-- gamble in the back end with table value and remove items -->
<div class="root">
  {#if !isLoading}
    <div class="horiz">
        <!-- what you still have to bet -->
        <InventoryTable bind:FocusedItemList={inventory} bind:OtherItemList = {bettingInventory}></InventoryTable>
        <!-- what you already selected to bet -->
        <InventoryTable bind:FocusedItemList={bettingInventory} bind:OtherItemList = {inventory}></InventoryTable>
    </div>
  {:else}
    <h1> Loading...</h1>
  {/if}
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
  }
  .horiz {
    display: flex;
    flex-direction: row;
  }
</style>
