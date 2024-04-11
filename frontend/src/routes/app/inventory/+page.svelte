<script lang="ts">
  import { SortBy, SortDirection } from '../../../protos/mine_pb';
  import { PrivateClient } from '../../../api';
  import BackToOverviewButton from '../../../components/BackToOverviewButton.svelte';
  import { useQuery } from '@sveltestack/svelte-query';
  import TabList from '../../../components/Common/TabList.svelte';
  import CardsList from '../../../components/Inventory/CardsList.svelte';
  import AggregateView from '../../../components/Inventory/AggregateView.svelte';

  enum InventoryTab {
    Cards = 'cards',
    Aggregate = 'aggregate',
  }

  let activeTab = InventoryTab.Cards;

  let sortBy = SortBy.Value;
  let sortDirection = SortDirection.Descending;

  let queryRes = useQuery(['inventory'], () =>
    PrivateClient.getInventory({ pageNumber: 0, pageSize: 2000, sortBy, sortDirection })
  );

  $: items = $queryRes.data?.items ?? [];
  $: totalItemCount = $queryRes.data?.totalItems;
  $: aggInventory = $queryRes.data?.aggregatedInventory;
</script>

<div class="root">
  <BackToOverviewButton />
  <h2>Inventory</h2>
  <p>
    <b style="padding-right: 4px;">Total Items: </b>{totalItemCount
      ? Intl.NumberFormat('en-US', {}).format(totalItemCount)
      : '?'}
  </p>

  <TabList
    style="margin-bottom: 8px;"
    bind:selectedTab={activeTab}
    tabs={[InventoryTab.Cards, InventoryTab.Aggregate]}
  />

  {#if activeTab === InventoryTab.Cards}
    <CardsList {items} />
  {:else if activeTab === InventoryTab.Aggregate && aggInventory}
    <AggregateView agg={aggInventory} />
  {/if}
</div>

<style lang="css">
  h2 {
    text-align: center;
    margin-top: 0;
    margin-bottom: 20px;
  }
</style>
