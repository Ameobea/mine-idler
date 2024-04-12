<script lang="ts">
  import { proto3 } from '@bufbuild/protobuf';

  import { PrivateClient } from '../../api';
  import { UpgradeType, type Upgrades } from '../../protos/mine_pb';
  import Modal from '../Common/Modal.svelte';
  import UpgradeCost from './UpgradeCost.svelte';

  export let open: boolean;
  export let upgrades: Upgrades;
  export let refetchUpgrades: () => void;
  export let upgradeType: UpgradeType;

  $: upgradeCost = (() => {
    switch (upgradeType) {
      case UpgradeType.Storage:
        return upgrades.storageUpgrades!.upgradeCost;
      default:
        throw new Error(`Unknown upgrade type: ${upgradeType}`);
    }
  })();
  $: upgradeTier = (() => {
    switch (upgradeType) {
      case UpgradeType.Storage:
        return upgrades.storageUpgrades!.storageLevel;
      default:
        throw new Error(`Unknown upgrade type: ${upgradeType}`);
    }
  })();
  $: upgradeName = proto3.getEnumType(UpgradeType).findNumber(upgradeType)!.name;

  let isUpgrading = false;
  let upgradeError: string | undefined = undefined;

  const onCancel = () => {
    open = false;
  };

  const onSubmit = async () => {
    if (isUpgrading) {
      return;
    }

    try {
      isUpgrading = true;

      await PrivateClient.upgradeBase({ upgradeType });
      toastSuccess(`Upgraded ${upgradeName} to ${upgradeTier + 1}`);

      refetchUpgrades();
      open = false;
    } catch (err) {
      console.error('Error upgrading:', err);
      upgradeError = err instanceof Error ? err.message : `${err}`;
    } finally {
      isUpgrading = false;
    }
  };
</script>

<Modal {open} {onCancel} {onSubmit}>
  <div class="root">
    <p style="text-align: center;">Upgrade {upgradeName} {upgradeTier}->{upgradeTier + 1}</p>

    <h3>Upgrade Cost</h3>
    <div class="costs-table">
      {#each upgradeCost as cost}
        <UpgradeCost {cost} />
      {/each}
    </div>

    <p style="margin-top: 30px; margin-bottom: 0;">Proceed?</p>

    {#if upgradeError}
      <div class="error">{upgradeError}</div>
    {/if}
  </div>
</Modal>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
  }

  .costs-table {
    display: flex;
    flex-direction: column;
  }

  .error {
    color: red;
  }
</style>
