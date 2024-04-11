<script lang="ts">
  import { goto } from '$app/navigation';

  export let predicate: (() => void) | undefined = undefined;
</script>

<button
  role="link"
  on:click={() => {
    predicate?.();
    goto('/app');
  }}
>
  Back
</button>

<style lang="css">
  button {
    position: absolute;
    top: 20px;
    left: 20px;
  }
</style>
