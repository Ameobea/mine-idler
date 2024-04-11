<script lang="ts">
  import type { AggregatedInventory } from '../../protos/mine_pb';
  import AggItemStats from './AggItemStats.svelte';

  export let agg: AggregatedInventory;
</script>

<div class="root">
  <div class="header">Name</div>
  <div class="header">Total Count</div>
  <div class="header">Total Quality</div>
  <div class="header">Total Value</div>
  <div class="header">Quality Distribution</div>
  {#each agg.itemCounts as count}
    <AggItemStats {count} />
  {/each}
</div>

<style lang="css">
  .root {
    display: grid;
    grid-template-columns: max-content 130px 130px 130px 180px;
  }

  .header {
    font-weight: bold;
    padding: 8px 4px;
    margin-bottom: 4px;
    border-bottom: 1px solid #cccccc22;
  }
</style>
