<script lang="ts">
  import toast, { Toaster, type ToastOptions } from 'svelte-french-toast';
  import { CurToast } from './GlobalToaster';

  $: if ($CurToast) {
    const { message, options: rawOpts, variant } = $CurToast;
    const opts: ToastOptions = {
      style: 'background: #222; border: 1px solid #aaa; color: #ececec',
      ...(rawOpts || []),
    };

    if (variant === 'success') {
      toast.success(message, opts);
    } else if (variant === 'error') {
      toast.error(message, opts);
    } else {
      toast(message, opts);
    }
  }
</script>

<Toaster />
