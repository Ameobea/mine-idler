<script lang="ts">
  import { RarityColors } from '$lib/item';
  import { ImageBaseURL } from '../../api';
  import type { Item } from '../../protos/mine_pb';
  import { GlobalState } from '../../state';

  export let item: Item;

  $: desc = $GlobalState.itemsById.get(item.id)!;
</script>

<div class="root">
  <img src={`${ImageBaseURL}${desc.name}.webp`} alt={desc.displayName} />
  <div>
    <p style={`background-color: ${RarityColors[desc.rarityTier]}`}>{desc.displayName}</p>
    <p>Quality: {item.quality.toFixed(4)}</p>
  </div>
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
    height: 240px;
    width: 200px;
    border: 1px solid #777;
  }

  p {
    margin: 0;
    font-size: 13px;
  }

  img {
    margin: 0;
    height: 200px;
    width: 200px;
  }
</style>
