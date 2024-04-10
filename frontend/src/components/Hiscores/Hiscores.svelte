<script lang="ts">
  import { useQuery } from '@sveltestack/svelte-query';
  import { PublicClient } from '../../api';
  import BackToOverviewButton from '../BackToOverviewButton.svelte';

  const hiscoresRes = useQuery('hiscores', () => PublicClient.getHiscores({}).then((res) => res.hiscores));
</script>

<div class="root">
  <BackToOverviewButton />
  <h2>Hiscores</h2>
  {#if $hiscoresRes.isLoading}
    <div>Loading...</div>
  {:else if $hiscoresRes.data}
    <table>
      <thead>
        <tr>
          <th>Rank</th>
          <th>Username</th>
          <th>Total Value</th>
        </tr>
      </thead>
      <tbody>
        {#each $hiscoresRes.data as { username, totalValue }, i}
          <tr>
            <td>{i + 1}</td>
            <td>{username}</td>
            <td>{Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(totalValue).slice(1)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style lang="css">
  .root {
    display: flex;
    flex-direction: column;
  }
</style>
