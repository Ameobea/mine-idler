<script lang="ts">
  import { onMount } from 'svelte';
  import type { MineSession } from '../types';
  import { ImageBaseURL, PrivateClient } from '../../../api';
  import type { Item, ItemDescriptor, StartMiningResponse } from '../../../protos/mine_pb';
  import SessionStats from './SessionStats.svelte';
  import LootTable from './LootTable.svelte';
  import { writable, type Writable } from 'svelte/store';
  import { GlobalState } from '../../../state';
  import { delay } from '../../../util';
  import { Code, ConnectError } from '@connectrpc/connect';

  export let session: MineSession;

  let lootStream: AsyncIterable<StartMiningResponse> | null = null;
  let loot: Writable<Item[]> = writable([]);
  let error: { message: string; transient: boolean } | null = null;
  let streamEnded = false;
  let imageHidden = localStorage.getItem('imageHidden') === 'true';

  const setImageHidden = (value: boolean) => {
    imageHidden = value;
    localStorage.setItem('imageHidden', value.toString());
  };

  let lastMinedItem: { item: Item; desc: ItemDescriptor } | null = null;

  const startMining = async () => {
    while (true) {
      try {
        error = null;
        streamEnded = false;

        let lastError;

        lootStream = PrivateClient.startMining({ locationName: session.locationName });

        if (!lootStream) {
          throw lastError;
        }

        for await (const res of lootStream) {
          const newLoot = res.loot;
          if (!newLoot) {
            continue;
          }

          loot.update((prev) => {
            prev.push(newLoot);
            return prev;
          });

          const desc = $GlobalState.itemsById.get(newLoot.id)!;
          lastMinedItem = { item: newLoot, desc };
          setTimeout(() => {
            if (lastMinedItem?.item === newLoot) {
              lastMinedItem = null;
            }
          }, 4000);
        }

        break;
      } catch (err) {
        if (err instanceof ConnectError) {
          if (err.code == Code.ResourceExhausted) {
            error = { message: err.rawMessage, transient: false };
            streamEnded = true;
            break;
          }
        }
        error = { message: `Error while mining: ${err}`, transient: true };
      } finally {
        lootStream = null;
        await delay(1500);
      }
    }
  };

  onMount(startMining);
</script>

<div class="root">
  <h2>Mining at {session.locationDisplayName}</h2>

  {#if error}
    <div class="error">
      <p>{error.message}</p>
      {#if error.transient}
        <p>Trying to restart...</p>
      {/if}
    </div>
  {/if}

  <div class="session-container">
    <div style="width:300px;margin-left: auto;margin-right: auto;">
      <span
        class="toggle-image"
        role="button"
        tabindex="0"
        on:click|preventDefault={() => setImageHidden(!imageHidden)}
        on:keydown|preventDefault={(e) => e.key === 'Enter' && setImageHidden(!imageHidden)}
        style={imageHidden ? 'text-align: center' : undefined}
      >
        {#if imageHidden}
          Show
        {:else}
          Hide
        {/if}
        image
      </span>
      {#if !imageHidden}
        <div class="last-mined-item-image-container">
          {#if lastMinedItem}
            <img
              style="width: 100%; height: 100%;"
              src={`${ImageBaseURL}${lastMinedItem.desc.name}.webp`}
              alt={lastMinedItem.desc.description}
            />
          {:else if lootStream}
            <div class="mining-in-progress">Mining...</div>
          {/if}
        </div>
        <p style="font-weight: bold; text-align: center; display: block; height: 15px;">
          {#if lastMinedItem}
            {lastMinedItem.desc.displayName}
          {/if}
        </p>
      {/if}
    </div>

    <div class="session-display">
      <div style="flex: 0.4">
        <h3>Stats</h3>
        <SessionStats loot={$loot} />
      </div>
      <div style="flex: 0.6">
        <h3>Loot</h3>
        <LootTable loot={$loot} />
      </div>
    </div>
  </div>

  {#if streamEnded && !error}
    <div>
      <p>Mining has stopped.</p>
      <button on:click={() => {}}> Restart mining </button>
    </div>
  {/if}
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
  }

  h2 {
    text-align: center;
    margin-top: 0;
    margin-bottom: 20px;
  }

  .error {
    color: red;
    margin-bottom: 10px;
    text-align: center;
  }

  .session-container {
    display: flex;
    flex-direction: column;
  }

  .session-display {
    display: flex;
    flex-direction: row;
    gap: 16px;
    margin-top: 40px;
  }

  .last-mined-item-image-container {
    width: 300px;
    height: 300px;
    border: 1px solid #999;
  }

  .toggle-image {
    display: block;
    margin-bottom: 3px;
    color: rgb(238, 238, 238);
    font-size: 13.5px;
    text-decoration: underline;
  }

  .toggle-image:hover {
    cursor: pointer;
  }

  .mining-in-progress {
    display: flex;
    justify-content: center;
    align-items: center;
    width: 100%;
    height: 100%;
    font-size: 24px;
    font-weight: bold;
  }
</style>
