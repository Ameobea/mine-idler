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
    <p class="item-name" style={`background-color: ${RarityColors[desc.rarityTier]}`}>{desc.displayName}</p>
    <div class="stats">
      <div>Qual: <span class="numeric">{item.quality.toFixed(4)}</span></div>
      <div>Val: <span class="numeric">{item.value.toFixed(2)}</span></div>
    </div>
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
  }

  .item-name {
    font-size: 14px;
    padding-left: 2px;
    padding-right: 2px;
  }

  .numeric {
    font-family: monospace;
  }

  .stats {
    display: flex;
    flex-direction: row;
    padding-left: 2px;
    padding-right: 2px;
    justify-content: space-between;
  }

  .stats > div {
    font-size: 13px;
    padding-left: 1px;
    padding-right: 1px;
  }

  img {
    margin: 0;
    height: 200px;
    width: 200px;
  }
</style>
