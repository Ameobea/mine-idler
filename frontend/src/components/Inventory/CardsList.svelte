<script lang="ts">
  import type { Item } from '../../protos/mine_pb';
  import ItemCard from './ItemCard.svelte';

  export let items: Item[];
</script>

<div class="item-cards">
  {#each items as item}
    <div class="item-card">
      <ItemCard {item} />
    </div>
  {/each}
</div>

<style lang="css">
  .item-cards {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
  }
</style>
