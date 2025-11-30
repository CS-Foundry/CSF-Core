<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { authStore } from "$lib/stores/auth";
  import { getLocalTimeZone, today } from "@internationalized/date";
  import Subscriptions from "$lib/components/expenses/subscritpions.svelte";
  import {
    SubscriptionService,
    type Subscription,
    type CreateSubscriptionRequest,
  } from "$lib/services/subscriptions";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { toast } from "svelte-sonner";

  let subscriptions: Subscription[] = $state([]);
  let loading = $state(true);
  let selectedDate = $state(today(getLocalTimeZone()));

  // Dialog state
  let showAddDialog = $state(false);
  let showEditDialog = $state(false);
  let editingSubscription: Subscription | null = $state(null);

  // Form state
  let formData = $state({
    name: "",
    amount: 0,
    billing_cycle: "monthly",
    next_billing_date: "",
    category: "Streaming",
  });

  const categories = [
    "Streaming",
    "Software",
    "Cloud Services",
    "Fitness",
    "Zeitschriften",
    "Sonstiges",
  ];

  const billingCycles = [
    { value: "monthly", label: "Monatlich" },
    { value: "yearly", label: "Jährlich" },
    { value: "quarterly", label: "Quartalsweise" },
    { value: "weekly", label: "Wöchentlich" },
  ];

  onMount(async () => {
    if (browser) {
      // Load subscriptions if authenticated
      if ($authStore.isAuthenticated && $authStore.token) {
        await loadSubscriptions();
      }
    }
  });

  async function loadSubscriptions() {
    const token = $authStore.token;
    if (!token) return;

    try {
      loading = true;
      subscriptions = await SubscriptionService.getSubscriptions(token);
    } catch (error) {
      console.error("Failed to load subscriptions:", error);
      toast.error("Fehler beim Laden der Abonnements");
    } finally {
      loading = false;
    }
  }

  function handleAddSubscription() {
    const now = new Date();
    formData = {
      name: "",
      amount: 0,
      billing_cycle: "monthly",
      next_billing_date: now.toISOString().slice(0, 16), // Format: YYYY-MM-DDTHH:MM
      category: "Streaming",
    };
    showAddDialog = true;
  }

  async function submitAddSubscription() {
    const token = $authStore.token;
    if (!token) return;

    try {
      // Convert datetime-local format to backend format (YYYY-MM-DD HH:MM:SS)
      const dateForBackend =
        formData.next_billing_date.replace("T", " ") + ":00";

      const request: CreateSubscriptionRequest = {
        name: formData.name,
        amount: formData.amount,
        billing_cycle: formData.billing_cycle,
        next_billing_date: dateForBackend,
        category: formData.category,
      };

      await SubscriptionService.createSubscription(request, token);
      showAddDialog = false;
      await loadSubscriptions();
      toast.success("Abonnement erfolgreich erstellt");
    } catch (error) {
      console.error("Failed to create subscription:", error);
      toast.error("Fehler beim Erstellen des Abonnements");
    }
  }

  function handleEditSubscription(id: string) {
    const subscription = subscriptions.find((s) => s.id === id);
    if (subscription) {
      editingSubscription = subscription;
      // Convert backend format (YYYY-MM-DD HH:MM:SS) to datetime-local format (YYYY-MM-DDTHH:MM)
      const dateForInput = subscription.next_billing_date
        .slice(0, 16)
        .replace(" ", "T");
      formData = {
        name: subscription.name,
        amount: parseFloat(subscription.amount.toString()),
        billing_cycle: subscription.billing_cycle,
        next_billing_date: dateForInput,
        category: subscription.category,
      };
      showEditDialog = true;
    }
  }

  async function submitEditSubscription() {
    if (!editingSubscription) return;

    const token = $authStore.token;
    if (!token) return;

    try {
      // Convert datetime-local format to backend format (YYYY-MM-DD HH:MM:SS)
      const dateForBackend =
        formData.next_billing_date.replace("T", " ") + ":00";

      await SubscriptionService.updateSubscription(
        editingSubscription.id,
        {
          name: formData.name,
          amount: formData.amount,
          billing_cycle: formData.billing_cycle,
          next_billing_date: dateForBackend,
          category: formData.category,
        },
        token
      );
      showEditDialog = false;
      await loadSubscriptions();
      toast.success("Abonnement erfolgreich aktualisiert");
    } catch (error) {
      console.error("Failed to update subscription:", error);
      toast.error("Fehler beim Aktualisieren des Abonnements");
    }
  }

  async function handleDeleteSubscription(id: string) {
    if (!confirm("Möchten Sie dieses Abonnement wirklich löschen?")) return;

    const token = $authStore.token;
    if (!token) return;

    try {
      await SubscriptionService.deleteSubscription(id, token);
      await loadSubscriptions();
      toast.success("Abonnement erfolgreich gelöscht");
    } catch (error) {
      console.error("Failed to delete subscription:", error);
      toast.error("Fehler beim Löschen des Abonnements");
    }
  }

  // Transform subscriptions for the component
  let transformedSubscriptions = $derived(
    subscriptions.map((s) => {
      const nextDate = new Date(s.next_billing_date);
      return {
        id: parseInt(s.id.replace(/-/g, "").slice(0, 8), 16),
        name: s.name,
        amount: parseFloat(s.amount.toString()),
        billingDay: nextDate.getDate(),
        category: s.category,
      };
    })
  );

  // Calculate total monthly subscriptions cost
  let totalSubscriptions = $derived(
    subscriptions.reduce((sum, s) => {
      const amount = parseFloat(s.amount.toString());
      // Convert to monthly cost based on billing cycle
      switch (s.billing_cycle) {
        case "yearly":
          return sum + amount / 12;
        case "quarterly":
          return sum + amount / 3;
        case "weekly":
          return sum + amount * 4;
        default: // monthly
          return sum + amount;
      }
    }, 0)
  );
</script>

<svelte:head>
  <title>Abonnements - FinanceVault</title>
</svelte:head>

<div class="container mx-auto py-6">
  {#if loading}
    <div class="flex items-center justify-center h-64">
      <p>Lade Abonnements...</p>
    </div>
  {:else}
    <Subscriptions
      subscriptions={transformedSubscriptions}
      {selectedDate}
      {totalSubscriptions}
      onAddSubscription={handleAddSubscription}
      onEditSubscription={(id) => {
        const subscription =
          subscriptions[transformedSubscriptions.findIndex((s) => s.id === id)];
        if (subscription) handleEditSubscription(subscription.id);
      }}
      onDeleteSubscription={(id) => {
        const subscription =
          subscriptions[transformedSubscriptions.findIndex((s) => s.id === id)];
        if (subscription) handleDeleteSubscription(subscription.id);
      }}
    />
  {/if}
</div>

<!-- Add Subscription Dialog -->
<Dialog.Root bind:open={showAddDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Neues Abonnement hinzufügen</Dialog.Title>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="name">Name</Label>
        <Input
          id="name"
          bind:value={formData.name}
          placeholder="z.B. Netflix"
        />
      </div>
      <div class="grid gap-2">
        <Label for="amount">Betrag (€)</Label>
        <Input
          id="amount"
          type="number"
          step="0.01"
          bind:value={formData.amount}
          placeholder="0.00"
        />
      </div>
      <div class="grid gap-2">
        <Label for="billing_cycle">Abrechnungszyklus</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder="Abrechnungszyklus wählen" />
          </Select.Trigger>
          <Select.Content>
            {#each billingCycles as cycle}
              <Select.Item
                value={cycle.value}
                onclick={() => (formData.billing_cycle = cycle.value)}
              >
                {cycle.label}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
      <div class="grid gap-2">
        <Label for="next_billing_date">Nächste Abbuchung</Label>
        <Input
          id="next_billing_date"
          type="datetime-local"
          bind:value={formData.next_billing_date}
        />
      </div>
      <div class="grid gap-2">
        <Label for="category">Kategorie</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder="Kategorie wählen" />
          </Select.Trigger>
          <Select.Content>
            {#each categories as category}
              <Select.Item
                value={category}
                onclick={() => (formData.category = category)}
              >
                {category}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (showAddDialog = false)}>
        Abbrechen
      </Button>
      <Button onclick={submitAddSubscription}>Speichern</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<!-- Edit Subscription Dialog -->
<Dialog.Root bind:open={showEditDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Abonnement bearbeiten</Dialog.Title>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="edit-name">Name</Label>
        <Input id="edit-name" bind:value={formData.name} />
      </div>
      <div class="grid gap-2">
        <Label for="edit-amount">Betrag (€)</Label>
        <Input
          id="edit-amount"
          type="number"
          step="0.01"
          bind:value={formData.amount}
        />
      </div>
      <div class="grid gap-2">
        <Label for="edit-billing_cycle">Abrechnungszyklus</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder={formData.billing_cycle} />
          </Select.Trigger>
          <Select.Content>
            {#each billingCycles as cycle}
              <Select.Item
                value={cycle.value}
                onclick={() => (formData.billing_cycle = cycle.value)}
              >
                {cycle.label}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
      <div class="grid gap-2">
        <Label for="edit-next_billing_date">Nächste Abbuchung</Label>
        <Input
          id="edit-next_billing_date"
          type="datetime-local"
          bind:value={formData.next_billing_date}
        />
      </div>
      <div class="grid gap-2">
        <Label for="edit-category">Kategorie</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder={formData.category} />
          </Select.Trigger>
          <Select.Content>
            {#each categories as category}
              <Select.Item
                value={category}
                onclick={() => (formData.category = category)}
              >
                {category}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (showEditDialog = false)}>
        Abbrechen
      </Button>
      <Button onclick={submitEditSubscription}>Aktualisieren</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
