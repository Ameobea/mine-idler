<script lang="ts">
  export let tabs: string[] = [];
  export let selectedTab: string = '';
  export let style: string | undefined = undefined;

  $: if (!selectedTab && tabs.length > 0) {
    selectedTab = tabs[0];
  }
</script>

<ul {style} role="tablist" class="root">
  {#each tabs as tab}
    <li role="tab" tabindex={-1} aria-selected={tab === selectedTab} on:click={() => (selectedTab = tab)}>
      {tab}
    </li>
  {/each}
</ul>

<style lang="css">
  .root {
    display: flex;
    flex-direction: row;
    list-style-type: none;
    padding: 0;
    margin: 0;
  }

  li {
    padding: 8px;
    cursor: pointer;
    user-select: none;
    color: #ececec;
  }

  li[aria-selected='true'] {
    background-color: #153370;
  }

  li[aria-selected='false'] {
    background-color: #4b5462;
  }

  li[aria-selected='false']:hover {
    background-color: #42567d;
  }
</style>
