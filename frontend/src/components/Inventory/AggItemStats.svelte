<script lang="ts">
  import type { AggregatedItemCount } from '../../protos/mine_pb';
  import { GlobalState } from '../../state';
  import QualityHistogram from './QualityHistogram.svelte';

  export let count: AggregatedItemCount;
  $: desc = $GlobalState.itemsById.get(count.itemId)!;
</script>

<div class="item-name padded">{desc.displayName}</div>
<div class="padded">{Intl.NumberFormat('en-US', {}).format(count.totalCount)}</div>
<div class="padded">{count.totalQuality.toFixed(2)}</div>
<div class="padded">{Intl.NumberFormat('en-US', {style: 'currency', currency: 'USD'}).format(count.totalValue).slice(1)}</div>
<div><QualityHistogram totalQuality={count.totalQuality} buckets={count.qualityHistogram!.buckets} /></div>

<style lang="css">
  .padded {
    padding-left: 8px;
    padding-right: 8px;
  }

  .item-name {
    font-weight: bold;
  }

  div {
    display: flex;
    align-items: center;
  }
</style>
