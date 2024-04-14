<script lang="ts">
 import { GlobalState } from '../../../state';
 import GambleSession from '../../../components/Gamble/GambleSession/GambleSession.svelte';
 import BackToOverviewButton from '../../../components/BackToOverviewButton.svelte';
 import { type MineSession as MineSessionTy } from '../../../components/Mine/types';
 let curGambleSession: MineSessionTy | null = null;
</script>

<div class="root">
    <BackToOverviewButton/>
  
    {#if !$GlobalState.gambleLocations}
      <div>Loading...</div>
    {:else if !curGambleSession}
      <h2>Gamble</h2>
      <p>Select a place to make profit</p>
  
      <div class="location-cards">
        {#each $GlobalState.gambleLocations as location}
          <div class="location-card">
            <div>
              <h3>{location.descriptor.displayName}</h3>
              <p class="description">{location.descriptor.description}</p>
            </div>
            <button
              on:click={() => {
                curGambleSession = {
                  locationName: location.descriptor.name,
                  locationDisplayName: location.descriptor.displayName,
                };
              }}
              disabled={!location.isAvailable}
            >
              Visit
            </button>
          </div>
        {/each}
      </div>
    {:else}
      <GambleSession session={curGambleSession} />
    {/if}
  </div>