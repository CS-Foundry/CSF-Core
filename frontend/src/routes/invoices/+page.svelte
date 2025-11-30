<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { authStore } from "$lib/stores/auth";
  import { InvoiceService } from "$lib/services/invoice";
  import type { Invoice, ManualEntry, InvoiceOverview } from "$lib/types";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import { ChevronLeft, ChevronRight } from "@lucide/svelte";
  import { toast } from "svelte-sonner";
  import ManualEntriesColumn from "$lib/components/invoice/manual-entries-column.svelte";
  import InvoicesColumn from "$lib/components/invoice/invoices-column.svelte";
  import MatchOverview from "$lib/components/invoice/match-overview.svelte";

  let loading = $state(true);
  let selectedYear = $state(new Date().getFullYear());
  let selectedMonth = $state(new Date().getMonth() + 1); // 1-12
  let invoiceOverview: InvoiceOverview | null = $state(null);

  // Dialog states
  let showAddManualEntryDialog = $state(false);
  let showEditManualEntryDialog = $state(false);
  let showUploadInvoiceDialog = $state(false);
  let editingManualEntry: ManualEntry | null = $state(null);

  // Form states
  let manualEntryForm = $state({
    description: "",
    amount: 0,
    date: "",
    category: "Lebensmittel",
  });

  let invoiceForm = $state({
    description: "",
    amount: 0,
    date: "",
    category: "Lebensmittel",
    file: null as File | null,
  });

  const categories = [
    "Lebensmittel",
    "Transport",
    "Unterhaltung",
    "Gesundheit",
    "Bildung",
    "Wohnung",
    "Versicherung",
    "Sonstiges",
  ];

  const monthNames = [
    "Januar",
    "Februar",
    "März",
    "April",
    "Mai",
    "Juni",
    "Juli",
    "August",
    "September",
    "Oktober",
    "November",
    "Dezember",
  ];

  let currentMonthString = $derived(
    `${selectedYear}-${String(selectedMonth).padStart(2, "0")}`
  );

  let manualEntries = $derived(invoiceOverview?.manual_entries ?? []);
  let invoices = $derived(invoiceOverview?.invoices ?? []);

  onMount(async () => {
    if (browser && $authStore.isAuthenticated) {
      await loadInvoiceData();
    }
  });

  $effect(() => {
    if (browser && $authStore.isAuthenticated && $authStore.token) {
      loadInvoiceData();
    }
  });

  async function loadInvoiceData() {
    const token = $authStore.token;
    if (!token) return;

    try {
      loading = true;
      invoiceOverview = await InvoiceService.getInvoiceOverview(
        currentMonthString,
        token
      );
    } catch (error) {
      // No data for this month yet
      invoiceOverview = null;
    } finally {
      loading = false;
    }
  }

  function previousMonth() {
    if (selectedMonth === 1) {
      selectedMonth = 12;
      selectedYear--;
    } else {
      selectedMonth--;
    }
  }

  function nextMonth() {
    if (selectedMonth === 12) {
      selectedMonth = 1;
      selectedYear++;
    } else {
      selectedMonth++;
    }
  }

  function goToCurrentMonth() {
    const now = new Date();
    selectedYear = now.getFullYear();
    selectedMonth = now.getMonth() + 1;
  }

  // Manual Entry handlers
  function handleAddManualEntry() {
    const now = new Date();
    manualEntryForm = {
      description: "",
      amount: 0,
      date: now.toISOString().split("T")[0],
      category: "Lebensmittel",
    };
    showAddManualEntryDialog = true;
  }

  async function submitAddManualEntry() {
    const token = $authStore.token;
    if (!token) return;

    try {
      await InvoiceService.createManualEntry(
        {
          month: currentMonthString,
          description: manualEntryForm.description,
          amount: manualEntryForm.amount,
          date: manualEntryForm.date,
          category: manualEntryForm.category,
        },
        token
      );
      toast.success("Manueller Eintrag erfolgreich hinzugefügt");
      showAddManualEntryDialog = false;
      await loadInvoiceData();
    } catch (error) {
      toast.error("Fehler beim Hinzufügen des Eintrags");
      console.error(error);
    }
  }

  function handleEditManualEntry(id: string) {
    const entry = manualEntries.find((e) => e.id === id);
    if (entry) {
      editingManualEntry = entry;
      manualEntryForm = {
        description: entry.description,
        amount: entry.amount,
        date: entry.date.split(" ")[0], // Extract date part
        category: entry.category,
      };
      showEditManualEntryDialog = true;
    }
  }

  async function handleDeleteManualEntry(id: string) {
    if (!confirm("Möchten Sie diesen Eintrag wirklich löschen?")) return;

    const token = $authStore.token;
    if (!token) return;

    try {
      await InvoiceService.deleteManualEntry(id, token);
      toast.success("Eintrag erfolgreich gelöscht");
      await loadInvoiceData();
    } catch (error) {
      toast.error("Fehler beim Löschen des Eintrags");
      console.error(error);
    }
  }

  // Invoice handlers
  function handleUploadInvoice() {
    const now = new Date();
    invoiceForm = {
      description: "",
      amount: 0,
      date: now.toISOString().split("T")[0],
      category: "Lebensmittel",
      file: null,
    };
    showUploadInvoiceDialog = true;
  }

  async function submitUploadInvoice() {
    const token = $authStore.token;
    if (!token) return;

    try {
      await InvoiceService.createInvoice(
        {
          month: currentMonthString,
          description: invoiceForm.description,
          amount: invoiceForm.amount,
          date: invoiceForm.date,
          category: invoiceForm.category,
          file: invoiceForm.file ?? undefined,
        },
        token
      );
      toast.success("Beleg erfolgreich hochgeladen");
      showUploadInvoiceDialog = false;
      await loadInvoiceData();
    } catch (error) {
      toast.error("Fehler beim Hochladen des Belegs");
      console.error(error);
    }
  }

  async function handleDeleteInvoice(id: string) {
    if (!confirm("Möchten Sie diesen Beleg wirklich löschen?")) return;

    const token = $authStore.token;
    if (!token) return;

    try {
      await InvoiceService.deleteInvoice(id, token);
      toast.success("Beleg erfolgreich gelöscht");
      await loadInvoiceData();
    } catch (error) {
      toast.error("Fehler beim Löschen des Belegs");
      console.error(error);
    }
  }

  async function handleMatchInvoices() {
    const token = $authStore.token;
    if (!token) return;

    try {
      invoiceOverview = await InvoiceService.matchInvoices(
        currentMonthString,
        token
      );
      toast.success("Abgleich erfolgreich durchgeführt");
    } catch (error) {
      toast.error("Fehler beim Abgleich");
      console.error(error);
    }
  }

  function handleFileChange(event: Event) {
    const target = event.target as HTMLInputElement;
    if (target.files && target.files[0]) {
      invoiceForm.file = target.files[0];
    }
  }
</script>

<div class="h-screen w-full p-6 overflow-auto">
  <div class="max-w-[1800px] mx-auto space-y-6">
    <!-- Header mit Monatsauswahl -->
    <div class="flex items-center justify-between">
      <h1 class="text-3xl font-bold">Rechnungsprüfung</h1>
      <div class="flex items-center gap-2">
        <Button variant="outline" size="icon" onclick={previousMonth}>
          <ChevronLeft class="h-4 w-4" />
        </Button>
        <Button
          variant="outline"
          onclick={goToCurrentMonth}
          class="min-w-[200px]"
        >
          {monthNames[selectedMonth - 1]}
          {selectedYear}
        </Button>
        <Button variant="outline" size="icon" onclick={nextMonth}>
          <ChevronRight class="h-4 w-4" />
        </Button>
      </div>
    </div>

    {#if loading}
      <div class="text-center py-12">
        <p class="text-muted-foreground">Lade Daten...</p>
      </div>
    {:else}
      <!-- Match Overview -->
      <MatchOverview overview={invoiceOverview} onMatch={handleMatchInvoices} />

      <!-- Zwei-Spalten-Layout -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <!-- Linke Spalte: Manuelle Einträge -->
        <ManualEntriesColumn
          entries={manualEntries}
          onAdd={handleAddManualEntry}
          onEdit={handleEditManualEntry}
          onDelete={handleDeleteManualEntry}
        />

        <!-- Rechte Spalte: Hochgeladene Rechnungen -->
        <InvoicesColumn
          {invoices}
          onUpload={handleUploadInvoice}
          onDelete={handleDeleteInvoice}
        />
      </div>
    {/if}
  </div>
</div>

<!-- Add Manual Entry Dialog -->
<Dialog.Root bind:open={showAddManualEntryDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Manuellen Eintrag hinzufügen</Dialog.Title>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="manual-description">Beschreibung</Label>
        <Input
          id="manual-description"
          bind:value={manualEntryForm.description}
          placeholder="z.B. Einkauf bei Rewe"
        />
      </div>
      <div class="grid gap-2">
        <Label for="manual-amount">Betrag (€)</Label>
        <Input
          id="manual-amount"
          type="number"
          step="0.01"
          bind:value={manualEntryForm.amount}
          placeholder="0.00"
        />
      </div>
      <div class="grid gap-2">
        <Label for="manual-date">Datum</Label>
        <Input id="manual-date" type="date" bind:value={manualEntryForm.date} />
      </div>
      <div class="grid gap-2">
        <Label for="manual-category">Kategorie</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder={manualEntryForm.category} />
          </Select.Trigger>
          <Select.Content>
            {#each categories as category}
              <Select.Item
                value={category}
                onclick={() => (manualEntryForm.category = category)}
              >
                {category}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
    </div>
    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => (showAddManualEntryDialog = false)}
      >
        Abbrechen
      </Button>
      <Button onclick={submitAddManualEntry}>Speichern</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<!-- Upload Invoice Dialog -->
<Dialog.Root bind:open={showUploadInvoiceDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Beleg hochladen</Dialog.Title>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="invoice-description">Beschreibung</Label>
        <Input
          id="invoice-description"
          bind:value={invoiceForm.description}
          placeholder="z.B. Rechnung Strom"
        />
      </div>
      <div class="grid gap-2">
        <Label for="invoice-amount">Betrag (€)</Label>
        <Input
          id="invoice-amount"
          type="number"
          step="0.01"
          bind:value={invoiceForm.amount}
          placeholder="0.00"
        />
      </div>
      <div class="grid gap-2">
        <Label for="invoice-date">Datum</Label>
        <Input id="invoice-date" type="date" bind:value={invoiceForm.date} />
      </div>
      <div class="grid gap-2">
        <Label for="invoice-category">Kategorie</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder={invoiceForm.category} />
          </Select.Trigger>
          <Select.Content>
            {#each categories as category}
              <Select.Item
                value={category}
                onclick={() => (invoiceForm.category = category)}
              >
                {category}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
      <div class="grid gap-2">
        <Label for="invoice-file">Datei (PDF, Bild)</Label>
        <Input
          id="invoice-file"
          type="file"
          accept=".pdf,.jpg,.jpeg,.png"
          onchange={handleFileChange}
        />
        {#if invoiceForm.file}
          <p class="text-sm text-muted-foreground">
            Ausgewählt: {invoiceForm.file.name}
          </p>
        {/if}
      </div>
    </div>
    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => (showUploadInvoiceDialog = false)}
      >
        Abbrechen
      </Button>
      <Button onclick={submitUploadInvoice}>Hochladen</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
