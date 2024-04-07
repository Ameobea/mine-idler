<script lang="ts">
  import { RarityColors } from '$lib/item';
  import type { Item } from '../../../protos/mine_pb';
  import { GlobalState } from '../../../state';

  export let loot: Item[];

  // TODO: This should be optimized to be incremental

  $: totalValue = loot.reduce((acc, item) => acc + item.value, 0);
  $: countsByRarityTier = loot.reduce(
    (acc, item) => {
      const desc = $GlobalState.itemsById.get(item.id)!;
      acc[desc.rarityTier] = (acc[desc.rarityTier] || 0) + 1;
      return acc;
    },
    {} as Record<number, number>
  );
</script>

<div class="root">
  <div><b>Total Value</b>: {totalValue.toFixed(2)}</div>
  <div style="margin-top: 10px;">
    {#each new Array(6) as _, tier}
      <div style="background-color: {RarityColors[tier]}; padding-left: 2px; padding-right: 2px;">
        Tier {tier}: {countsByRarityTier[tier] ?? 0}
      </div>
    {/each}
  </div>
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
  }
</style>
