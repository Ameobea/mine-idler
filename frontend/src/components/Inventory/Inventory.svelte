<script lang="ts">
  import { onMount } from 'svelte';
  import { Item, SortBy, SortDirection } from '../../protos/mine_pb';
  import { PrivateClient } from '../../api';
  import ItemCard from './ItemCard.svelte';
  import BackToOverviewButton from '../BackToOverviewButton.svelte';

  let sortBy = SortBy.Value;
  let sortDirection = SortDirection.Descending;

  // TODO: use svelte-query
  let items: Item[] = [];

  onMount(() => {
    PrivateClient.getInventory({ pageNumber: 0, pageSize: 500, sortBy, sortDirection }).then((res) => {
      items = res.items;
    });
  });
</script>

<div class="root">
  <BackToOverviewButton />
  <h2>Inventory</h2>
  <div class="item-cards">
    {#each items as item}
      <div class="item-card">
        <ItemCard {item} />
      </div>
    {/each}
  </div>
</div>

<style lang="css">
  .item-cards {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
  }

  h2 {
    text-align: center;
    margin-top: 0;
    margin-bottom: 20px;
  }
</style>
