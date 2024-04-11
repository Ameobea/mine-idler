<script lang="ts">
  import type { ItemCost } from '../../protos/mine_pb';
  import { GlobalState } from '../../state';

  export let cost: ItemCost;
  $: costDisplay = Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' })
    .format(cost.totalQuality)
    .slice(1);

  $: desc = $GlobalState.itemsById.get(cost.itemId)!;
</script>

<div class="root">
  <div><b>{desc.displayName}</b></div>
  <div class="cost">{costDisplay}</div>
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: row;
    flex: 1;
  }

  .root > div {
    flex: 1;
  }

  .cost {
    text-align: right;
  }
</style>
