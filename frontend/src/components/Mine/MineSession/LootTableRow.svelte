<script lang="ts">
  import type { Item } from '../../../protos/mine_pb';
  import { GlobalState } from '../../../state';

  export let item: Item;

  $: desc = $GlobalState.itemsById.get(item.id)!;
</script>

<tr class={`rarity-tier-${desc.rarityTier}`}>
  <td>{desc.displayName}</td>
  <td class="numeric">{item.quality.toFixed(4)}</td>
  <td class="numeric">{item.value.toFixed(2)}</td>
</tr>

<style lang="css">
  .rarity-tier-0 td {
    background-color: #485760;
  }
  .rarity-tier-1 td {
    background-color: #3a6b90;
  }
  .rarity-tier-2 td {
    background-color: #5d3479;
  }
  .rarity-tier-3 td {
    background-color: #9d27a3;
  }
  .rarity-tier-4 td {
    background-color: #750b1f;
  }
  .rarity-tier-5 td {
    background-color: #f7b500;
  }

  .numeric {
    font-family: monospace;
  }
</style>
