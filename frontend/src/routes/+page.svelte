<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/stores/auth";

  onMount(() => {
    // Redirect authenticated users to monthly expenses
    if ($authStore.isAuthenticated) {
      goto("/expenses/monthly");
    }
  });
</script>

<div class="flex items-center justify-center h-screen">
  <div class="text-center">
    <h1 class="text-4xl font-bold">FinanceVault</h1>
    <p class="mt-4 text-muted-foreground">
      {#if !$authStore.isAuthenticated}
        Bitte melden Sie sich an, um fortzufahren.
      {:else}
        Weiterleitung...
      {/if}
    </p>
  </div>
</div>
