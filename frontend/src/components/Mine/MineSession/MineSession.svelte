<script lang="ts">
  import { Code, ConnectError } from '@connectrpc/connect';
  import { onDestroy, onMount } from 'svelte';
  import { writable, type Writable } from 'svelte/store';
  import { v4 as uuidv4 } from 'uuid';

  import { ImageBaseURL, PrivateClient } from '../../../api';
  import type { Item, ItemDescriptor, StartMiningResponse } from '../../../protos/mine_pb';
  import { GlobalState } from '../../../state';
  import { delay } from '../../../util';
  import type { MineSession } from '../types';
  import LootTable from './LootTable.svelte';
  import SessionStats from './SessionStats.svelte';

  export let session: MineSession;

  // ripping from stackoverflow. umad?
  function animationInterval(ms: number, signal: { aborted: boolean } | null, callback: (time: number) => void) {
    const start = Date.now();

    function frame(time: number) {
      if (signal?.aborted) return;
      callback(time);
      scheduleFrame(time);
    }

    function scheduleFrame(time: number) {
      const elapsed = time - start;
      const roundedElapsed = Math.round(elapsed / ms) * ms;
      const targetNext = start + roundedElapsed + ms;
      const delay = targetNext - performance.now();
      setTimeout(() => requestAnimationFrame(frame), delay);
    }

    scheduleFrame(start);
  }

  function round(num: number, precision: number) {
    const factor = Math.pow(10, precision);
    return Math.round(num * factor) / factor;
  }

  function clamp(num: number, min: number, max: number) {
    return Math.min(Math.max(num, min), max);
  }

  let lootStream: AsyncIterable<StartMiningResponse> | null = null;
  let loot: Writable<Item[]> = writable([]);
  let error: { message: string; transient: boolean } | null = null;
  let streamEnded = false;
  let imageHidden = localStorage.getItem('imageHidden') === 'true';

  const setImageHidden = (value: boolean) => {
    imageHidden = value;
    localStorage.setItem('imageHidden', value.toString());
  };

  let mineSessionToken: string | null = null;
  let lastMinedItem: { item: Item; desc: ItemDescriptor } | null = null;

  let totalMineTime = Infinity;
  $: totalMineTimeSeconds = totalMineTime / 1000;
  let currentMillis = Date.now();
  let lastReceivedLootAt = currentMillis;
  $: elapsedTime = clamp(round((currentMillis - lastReceivedLootAt) / 1000, 1), 0, totalMineTimeSeconds);
  $: progressAmount = clamp(round(((currentMillis - lastReceivedLootAt) * 100) / totalMineTime, 1), 0, 100);

  const startMining = async () => {
    lastReceivedLootAt = Date.now();

    while (true) {
      try {
        error = null;
        streamEnded = false;

        mineSessionToken = uuidv4();
        lootStream = PrivateClient.startMining({
          locationName: session.locationName,
          mineSessionTokenUuid: mineSessionToken,
        });

        for await (const res of lootStream) {
          totalMineTime = res.millisUntilNextLoot;
          const newLoot = res.loot;
          if (!newLoot) {
            continue;
          }

          lastReceivedLootAt = Date.now() - (res.millisUntilNextLoot - totalMineTime);

          loot.update((prev) => {
            prev.push(newLoot);
            return prev;
          });

          const desc = $GlobalState.itemsById.get(newLoot.itemTypeId)!;
          lastMinedItem = { item: newLoot, desc };
          setTimeout(() => {
            if (lastMinedItem?.item === newLoot) {
              lastMinedItem = null;
            }
          }, 4000);
        }

        streamEnded = true;
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

  const animationCancelSignal = { aborted: false };
  onDestroy(() => {
    animationCancelSignal.aborted = true;
    if (mineSessionToken) {
      PrivateClient.stopMining({ mineSessionTokenUuid: mineSessionToken });
    }
  });

  // update progress bar every .1 sec, which is the precision of the percentage in it
  animationInterval(90, animationCancelSignal, () => {
    currentMillis = Date.now();
  });
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
      {#if lootStream}
        <div class="progress-container">
          <div class="progress-bar">
            <div class="progress-amount" style="width: {progressAmount}%;"></div>
            <div class="progress-percent-text">{progressAmount}%</div>
          </div>
          <div class="progress-text">{elapsedTime}s / {totalMineTimeSeconds}s</div>
        </div>
      {/if}

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
    margin-top: -8px;
    margin-bottom: 20px;
    padding: 0;
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
    margin-bottom: 2px;
    color: rgb(238, 238, 238);
    font-size: 13px;
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

  .progress-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-bottom: 8px;
  }

  .progress-bar {
    width: 100%;
    border: solid 1px #ddd;
    border-radius: 10px;
    overflow: hidden;
    position: relative;
  }

  .progress-amount {
    height: 20px;
    background-color: #007bff;
    transition: width 0.1s ease-in-out;
    display: flex;
    justify-content: center;
    align-items: center;
    color: #fff;
    font-weight: bold;
  }

  .progress-percent-text {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    top: 0;
    bottom: 0;
  }

  .progress-text {
    font-size: 18px;
  }
</style>
