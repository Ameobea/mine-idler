<script lang="ts">
  import { browser } from '$app/environment';
  import { goto, preloadCode } from '$app/navigation';
  import { onMount } from 'svelte';

  import { getIsLoggedIn, login, register } from '../api';
  import { initAppState } from '../state';

  let username = '';
  let password = '';
  let loginError: string | null = null;
  let isLoggingIn = false;

  onMount(() => {
    if (!browser) {
      return;
    }

    if (getIsLoggedIn()) {
      initAppState();
      goto('/app');
    } else {
      preloadCode('/app');
    }
  });

  const handleSubmit = (action: 'login' | 'register') => {
    loginError = null;
    isLoggingIn = true;

    (action === 'login' ? login : register)(username, password)
      .then(() => {
        initAppState();
        goto('/app');
      })
      .catch((err) => {
        loginError = err.message;
      })
      .finally(() => {
        isLoggingIn = false;
      });
  };
</script>

<h1>MineIdler</h1>
<div class="root">
  <input type="text" bind:value={username} placeholder="Username" />
  <input type="password" bind:value={password} placeholder="Password" />
  <button on:click={() => handleSubmit('login')} disabled={isLoggingIn} type="button">Login</button>
  <button style="margin-top: 8px;" on:click={() => handleSubmit('register')} disabled={isLoggingIn} type="button">
    Register
  </button>

  {#if loginError}
    <div class="error">{loginError}</div>
  {/if}
</div>

<style lang="css">
  h1 {
    text-align: center;
    margin-bottom: 80px;
  }

  .root {
    display: flex;
    flex-direction: column;
  }

  .error {
    color: red;
    margin-top: 20px;
    text-align: center;
    font-size: 14px;
  }
</style>
