<script lang="ts">
  import { AppRoute, CurRoute } from '$lib/AppRoute';

  export let predicate: (() => void) | undefined = undefined;
</script>

<button
  on:click={() => {
    predicate?.();
    CurRoute.set(AppRoute.Overview);
  }}>Back</button
>

<style lang="css">
  button {
    position: absolute;
    top: 20px;
    left: 20px;
  }
</style>
