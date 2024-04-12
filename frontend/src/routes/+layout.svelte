<script lang="ts">
  import { onMount } from 'svelte';

  import '../app.css';
  import { initGlobalToaster } from '../components/Common/GlobalToaster';
  import GlobalToaster from '../components/Common/GlobalToaster.svelte';
  import { initSentry } from '../sentry';

  onMount(() => {
    initGlobalToaster();
    initSentry();
  });
</script>

<GlobalToaster />
<slot />
