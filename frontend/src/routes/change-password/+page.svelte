<script lang="ts">
  import * as Card from '$lib/components/ui/card/index.js';
  import { Field, FieldLabel } from '$lib/components/ui/field/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Alert, AlertDescription } from '$lib/components/ui/alert/index.js';
  import { SettingsService } from '$lib/services/settings';
  import { AuthService } from '$lib/services/auth';
  import { authStore } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { KeyIcon } from '@lucide/svelte';

  let newPassword = $state('');
  let confirmPassword = $state('');
  let isLoading = $state(false);
  let errorMessage = $state('');

  async function handlePasswordChange(event: Event) {
    event.preventDefault();

    if (newPassword !== confirmPassword) {
      errorMessage = 'Passwörter stimmen nicht überein';
      return;
    }

    if (newPassword.length < 6) {
      errorMessage = 'Passwort muss mindestens 6 Zeichen lang sein';
      return;
    }

    isLoading = true;
    errorMessage = '';

    try {
      const publicKey = await AuthService.getPublicKey();
      // For forced password change, we don't need the old password
      const encryptedOldPassword = '';
      const encryptedNewPassword = await AuthService.encryptPassword(newPassword, publicKey);

      await SettingsService.changePassword(encryptedOldPassword, encryptedNewPassword);

      // Redirect to home after successful password change
      goto('/');
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Passwortänderung fehlgeschlagen';
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="flex min-h-screen items-center justify-center p-4">
  <Card.Root class="w-full max-w-md">
    <Card.Header class="text-center">
      <div class="mx-auto mb-4 rounded-full bg-primary/10 p-3 w-fit">
        <KeyIcon class="h-8 w-8 text-primary" />
      </div>
      <Card.Title class="text-2xl">Passwort ändern erforderlich</Card.Title>
      <Card.Description>
        Bitte ändern Sie Ihr Passwort, bevor Sie fortfahren. Dies ist erforderlich für Ihre
        Sicherheit.
      </Card.Description>
    </Card.Header>
    <Card.Content>
      <form onsubmit={handlePasswordChange} class="space-y-6">
        {#if errorMessage}
          <Alert variant="destructive">
            <AlertDescription>{errorMessage}</AlertDescription>
          </Alert>
        {/if}

        <Field>
          <FieldLabel for="new-password">Neues Passwort</FieldLabel>
          <Input
            id="new-password"
            type="password"
            bind:value={newPassword}
            placeholder="••••••••"
            disabled={isLoading}
            required
            minlength={6}
          />
        </Field>

        <Field>
          <FieldLabel for="confirm-password">Passwort bestätigen</FieldLabel>
          <Input
            id="confirm-password"
            type="password"
            bind:value={confirmPassword}
            placeholder="••••••••"
            disabled={isLoading}
            required
            minlength={6}
          />
        </Field>

        <Button type="submit" disabled={isLoading} class="w-full">
          {#if isLoading}
            <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"></div>
          {/if}
          Passwort ändern
        </Button>
      </form>
    </Card.Content>
  </Card.Root>
</div>
