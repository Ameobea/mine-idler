<script lang="ts">
  import { onMount } from 'svelte';

  import '../app.css';
  import { initGlobalToaster } from '../components/Common/GlobalToaster';
  import GlobalToaster from '../components/Common/GlobalToaster.svelte';

  onMount(() => {
    initGlobalToaster();
  });
</script>

<GlobalToaster />
<slot />
