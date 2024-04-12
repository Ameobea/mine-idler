<script lang="ts">
  import { useQuery } from '@sveltestack/svelte-query';

  import { PrivateClient } from '../../../api';
  import BackToOverviewButton from '../../../components/BackToOverviewButton.svelte';
  import UpgradeModal from '../../../components/Base/UpgradeModal.svelte';
  import { UpgradeType } from '../../../protos/mine_pb';

  let queryRes = useQuery('base', () => PrivateClient.getBase({}));

  let upgradeType = UpgradeType.Storage;
  let upgradeModalOpen = false;

  const upgradeStorage = () => {
    upgradeModalOpen = true;
    upgradeType = UpgradeType.Storage;
  }
</script>

<div class="root">
  <BackToOverviewButton />

  {#if upgradeModalOpen && $queryRes.data?.upgrades}
    <UpgradeModal
      bind:open={upgradeModalOpen}
      upgrades={$queryRes.data.upgrades}
      refetchUpgrades={() => $queryRes.refetch()}
      upgradeType={upgradeType}
    />
  {/if}

  {#if $queryRes.isLoading}
    <p>Loading...</p>
  {:else if $queryRes.error}
    <div class="error">
      <p>{$queryRes.error}</p>
    </div>
  {:else if $queryRes.data?.upgrades}
    <div class="base">
      <h2>Upgrades</h2>
      <div class="upgrades-container">
        <div>
          <p><b>Storage Level</b>: {$queryRes.data.upgrades.storageUpgrades!.storageLevel}</p>
          <p>Total Capacity: {Intl.NumberFormat('en-US', {}).format($queryRes.data.upgrades.storageUpgrades!.storageCapacity)}</p>
          <button on:click={upgradeStorage}>Upgrade</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
  }

  .upgrades-container {
    display: flex;
    flex-direction: row;
    margin-top: 20px;
    gap: 8px;
  }

  .upgrades-container > div {
    display: flex;
    flex-direction: column;
    padding: 12px;
    border: 1px solid #cccccc88;
    min-width: 300px;
    max-width: 300px;
  }
</style>
