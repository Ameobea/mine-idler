<script lang="ts">
  // Adapted from https://svelte.dev/examples/modal

  export let open: boolean;
  export let modalStyle: string | undefined = undefined;
  export let onCancel: (() => void) | undefined = undefined;
  export let onSubmit: (() => void) | undefined = undefined;

  let dialog: HTMLDialogElement;

  $: if (dialog && open) {
    dialog.showModal();
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
<dialog style={modalStyle} bind:this={dialog} on:close={onCancel} on:click|self={() => dialog.close()}>
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div on:click|stopPropagation class="container">
    <div>
      <slot name="header" />
      <slot />
    </div>

    <div class="buttons-container">
      <!-- svelte-ignore a11y-autofocus -->
      <button
        class="secondary cancel-button"
        autofocus
        on:click={() => {
          dialog.close();
          onCancel?.();
        }}
      >
        Cancel
      </button>
      <!-- svelte-ignore a11y-autofocus -->
      <button class="submit-button" on:click={onSubmit}> Submit </button>
    </div>
  </div>
</dialog>

<style>
  dialog {
    display: flex;
    max-width: 32em;
    border-radius: 0.2em;
    padding: 0;
    min-width: 300px;
    min-height: 250px;
    margin: auto;
  }

  dialog::backdrop {
    background: rgba(0, 0, 0, 0.3);
  }

  .container {
    padding: 1em;
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }

  dialog[open] {
    animation: zoom 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  @keyframes zoom {
    from {
      transform: scale(0.95);
    }

    to {
      transform: scale(1);
    }
  }

  dialog[open]::backdrop {
    animation: fade 0.2s ease-out;
  }

  @keyframes fade {
    from {
      opacity: 0;
    }

    to {
      opacity: 1;
    }
  }

  .buttons-container {
    display: flex;
    justify-content: space-around;
    gap: 8px;
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid #cccccc12;
  }

  button {
    display: block;
  }
</style>
