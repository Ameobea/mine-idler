<script lang="ts">
  import { onMount } from 'svelte';

  export const prerender = true;

  import '../app.css';
  import { initGlobalToaster } from '../components/Common/GlobalToaster';
  import GlobalToaster from '../components/Common/GlobalToaster.svelte';

  onMount(() => {
    initGlobalToaster();
  });
</script>

<GlobalToaster />
<slot />
