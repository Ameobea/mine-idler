<script lang="ts">
  import { PrivateClient } from '../../../api';
  import BackToOverviewButton from '../../../components/BackToOverviewButton.svelte';
  import MineSession from '../../../components/Mine/MineSession/MineSession.svelte';
  import { type MineSession as MineSessionTy } from '../../../components/Mine/types';
  import { GlobalState } from '../../../state';

  let curMineSession: MineSessionTy | null = null;
</script>

<div class="root">
  <BackToOverviewButton predicate={() => PrivateClient.stopMining({})} />

  {#if !$GlobalState.mineLocations}
    <div>Loading...</div>
  {:else if !curMineSession}
    <h2>Mine</h2>
    <p>Select a location to start mining at</p>

    <div class="location-cards">
      {#each $GlobalState.mineLocations as location}
        <div class="location-card">
          <div>
            <h3>{location.descriptor.displayName}</h3>
            <p class="description">{location.descriptor.description}</p>
          </div>
          <button
            on:click={() => {
              curMineSession = {
                locationName: location.descriptor.name,
                locationDisplayName: location.descriptor.displayName,
              };
            }}
            disabled={!location.isAvailable}
          >
            Start Mining
          </button>
        </div>
      {/each}
    </div>
  {:else}
    <MineSession session={curMineSession} />
  {/if}
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
  }

  .location-cards {
    display: flex;
    flex-direction: row;
    gap: 16px;
    margin-top: 20px;
  }

  .location-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 16px;
    border: 1px solid #ccc;
    border-radius: 8px;
    margin-bottom: 16px;
    width: 270px;
  }

  .location-card h3 {
    padding-top: 4px;
    margin-bottom: 12px;
  }

  .location-card .description {
    font-size: 13.5px;
    white-space: pre-line;
  }

  .location-card button {
    margin-left: auto;
    margin-right: auto;
    display: flex;
  }
</style>
