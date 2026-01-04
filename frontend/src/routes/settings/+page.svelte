<script lang="ts">
  import * as Card from '$lib/components/ui/card/index.js';
  import * as Tabs from '$lib/components/ui/tabs/index.js';
  import { Field, FieldLabel, FieldDescription } from '$lib/components/ui/field/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Alert, AlertDescription } from '$lib/components/ui/alert/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { authStore } from '$lib/stores/auth';
  import { SettingsService } from '$lib/services/settings';
  import { AuthService } from '$lib/services/auth';
  import {
    UserIcon,
    ShieldCheckIcon,
    KeyIcon,
    CheckCircle2Icon,
    XCircleIcon,
    Building2,
    Users,
  } from '@lucide/svelte';
  import { onMount } from 'svelte';
  import OrganizationSettings from '$lib/components/settings/OrganizationSettings.svelte';
  import UpdateSettings from '$lib/components/settings/UpdateSettings.svelte';

  let authState = $derived($authStore);
  let email = $state('');
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let isProfileLoading = $state(false);
  let profileMessage = $state('');

  let twoFactorEnabled = $state(false);
  let totpQrCode = $state('');
  let totpSecret = $state('');
  let verificationCode = $state('');
  let disableVerificationCode = $state('');
  let is2FALoading = $state(false);
  let twoFactorMessage = $state('');

  onMount(async () => {
    // Wait a tick to ensure authStore is initialized
    await new Promise((resolve) => setTimeout(resolve, 0));

    console.log('[Settings] authStore state:', $authStore);
    console.log('[Settings] Has token:', !!$authStore.token);

    try {
      const profile = await SettingsService.getProfile();
      email = profile.email || '';
      twoFactorEnabled = profile.two_factor_enabled;
    } catch (error) {
      console.error('Failed to load profile:', error);
      // Don't show error to user on initial load, just use defaults
      // The profile endpoints are protected, if user is not authenticated
      // they will be redirected by the server-side load function
    }
  });

  async function handleEmailChange(event: Event) {
    event.preventDefault();
    isProfileLoading = true;
    profileMessage = '';

    try {
      await SettingsService.changeEmail(email);
      profileMessage = 'E-Mail erfolgreich aktualisiert';
    } catch (error) {
      profileMessage =
        error instanceof Error ? error.message : 'E-Mail-Aktualisierung fehlgeschlagen';
    } finally {
      isProfileLoading = false;
    }
  }

  async function handlePasswordChange(event: Event) {
    event.preventDefault();

    if (newPassword !== confirmPassword) {
      profileMessage = 'Passwörter stimmen nicht überein';
      return;
    }

    if (newPassword.length < 6) {
      profileMessage = 'Passwort muss mindestens 6 Zeichen lang sein';
      return;
    }

    isProfileLoading = true;
    profileMessage = '';

    try {
      const publicKey = await AuthService.getPublicKey();
      const encryptedOldPassword = await AuthService.encryptPassword(currentPassword, publicKey);
      const encryptedNewPassword = await AuthService.encryptPassword(newPassword, publicKey);

      await SettingsService.changePassword(encryptedOldPassword, encryptedNewPassword);

      profileMessage = 'Passwort erfolgreich geändert';
      currentPassword = '';
      newPassword = '';
      confirmPassword = '';
    } catch (error) {
      profileMessage = error instanceof Error ? error.message : 'Passwortänderung fehlgeschlagen';
    } finally {
      isProfileLoading = false;
    }
  }

  async function handleSetup2FA() {
    is2FALoading = true;
    twoFactorMessage = '';

    try {
      const response = await SettingsService.setup2FA();
      totpSecret = response.secret;
      totpQrCode = response.qr_code;
      twoFactorMessage = 'Scannen Sie den QR-Code mit Ihrer Authenticator App';
    } catch (error) {
      twoFactorMessage = error instanceof Error ? error.message : '2FA-Setup fehlgeschlagen';
    } finally {
      is2FALoading = false;
    }
  }

  async function handleEnable2FA(event: Event) {
    event.preventDefault();

    if (!verificationCode || verificationCode.length !== 6) {
      twoFactorMessage = 'Bitte geben Sie einen gültigen 6-stelligen Code ein';
      return;
    }

    is2FALoading = true;
    twoFactorMessage = '';

    try {
      await SettingsService.enable2FA(verificationCode);
      twoFactorEnabled = true;
      verificationCode = '';
      totpQrCode = '';
      totpSecret = '';
      twoFactorMessage = '2FA erfolgreich aktiviert';
    } catch (error) {
      twoFactorMessage = error instanceof Error ? error.message : '2FA-Aktivierung fehlgeschlagen';
    } finally {
      is2FALoading = false;
    }
  }

  async function handleDisable2FA(event: Event) {
    event.preventDefault();

    if (!disableVerificationCode || disableVerificationCode.length !== 6) {
      twoFactorMessage = 'Bitte geben Sie einen gültigen 6-stelligen Code ein';
      return;
    }

    is2FALoading = true;
    twoFactorMessage = '';

    try {
      await SettingsService.disable2FA(disableVerificationCode);
      twoFactorEnabled = false;
      disableVerificationCode = '';
      twoFactorMessage = '2FA erfolgreich deaktiviert';
    } catch (error) {
      twoFactorMessage =
        error instanceof Error ? error.message : '2FA-Deaktivierung fehlgeschlagen';
    } finally {
      is2FALoading = false;
    }
  }
</script>

<div class="flex-1 space-y-6 p-8 pt-6">
  <div class="space-y-1">
    <h2 class="text-3xl font-bold tracking-tight">Einstellungen</h2>
    <p class="text-muted-foreground">Verwalten Sie Ihr Profil und Ihre Sicherheitseinstellungen</p>
  </div>

  <Tabs.Root value="profile" class="space-y-6">
    <Tabs.List class="grid w-full max-w-3xl grid-cols-5">
      <Tabs.Trigger value="profile" class="gap-2">
        <UserIcon class="h-4 w-4" />
        Profil
      </Tabs.Trigger>
      <Tabs.Trigger value="security" class="gap-2">
        <ShieldCheckIcon class="h-4 w-4" />
        Sicherheit
      </Tabs.Trigger>
      <Tabs.Trigger value="updates" class="gap-2">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-4 w-4"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
          />
        </svg>
        Updates
      </Tabs.Trigger>
      <Tabs.Trigger value="organization" class="gap-2">
        <Building2 class="h-4 w-4" />
        Organisation
      </Tabs.Trigger>
      <Tabs.Trigger value="users" class="gap-2">
        <Users class="h-4 w-4" />
        Benutzer
      </Tabs.Trigger>
    </Tabs.List>

    <Tabs.Content value="profile" class="space-y-6">
      <Card.Root>
        <Card.Header>
          <Card.Title>E-Mail ändern</Card.Title>
          <Card.Description>Aktualisieren Sie Ihre E-Mail-Adresse</Card.Description>
        </Card.Header>
        <Card.Content>
          <form onsubmit={handleEmailChange} class="space-y-6">
            {#if profileMessage && !profileMessage.includes('Passwort')}
              <Alert variant={profileMessage.includes('erfolgreich') ? 'default' : 'destructive'}>
                <AlertDescription>{profileMessage}</AlertDescription>
              </Alert>
            {/if}

            <Field>
              <FieldLabel for="email">E-Mail</FieldLabel>
              <Input
                id="email"
                type="email"
                bind:value={email}
                placeholder="ihre@email.de"
                disabled={isProfileLoading}
              />
            </Field>

            <Button type="submit" disabled={isProfileLoading}>
              {#if isProfileLoading}
                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"></div>
              {/if}
              E-Mail aktualisieren
            </Button>
          </form>
        </Card.Content>
      </Card.Root>

      <Card.Root>
        <Card.Header>
          <Card.Title>Passwort ändern</Card.Title>
          <Card.Description>Ändern Sie Ihr Passwort für mehr Sicherheit</Card.Description>
        </Card.Header>
        <Card.Content>
          <form onsubmit={handlePasswordChange} class="space-y-6">
            {#if profileMessage && profileMessage.includes('Passwort')}
              <Alert variant={profileMessage.includes('erfolgreich') ? 'default' : 'destructive'}>
                <AlertDescription>{profileMessage}</AlertDescription>
              </Alert>
            {/if}

            <Field>
              <FieldLabel for="current-password">Aktuelles Passwort</FieldLabel>
              <Input
                id="current-password"
                type="password"
                bind:value={currentPassword}
                placeholder="••••••••"
                disabled={isProfileLoading}
              />
            </Field>

            <Field>
              <FieldLabel for="new-password">Neues Passwort</FieldLabel>
              <Input
                id="new-password"
                type="password"
                bind:value={newPassword}
                placeholder="••••••••"
                disabled={isProfileLoading}
              />
            </Field>

            <Field>
              <FieldLabel for="confirm-password">Passwort bestätigen</FieldLabel>
              <Input
                id="confirm-password"
                type="password"
                bind:value={confirmPassword}
                placeholder="••••••••"
                disabled={isProfileLoading}
              />
            </Field>

            <Button type="submit" disabled={isProfileLoading}>
              {#if isProfileLoading}
                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"></div>
              {/if}
              Passwort ändern
            </Button>
          </form>
        </Card.Content>
      </Card.Root>
    </Tabs.Content>

    <Tabs.Content value="security" class="space-y-6">
      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center gap-2">
            <ShieldCheckIcon class="h-5 w-5" />
            Zwei-Faktor-Authentifizierung (2FA)
          </Card.Title>
          <Card.Description>
            Erhöhen Sie die Sicherheit Ihres Kontos mit TOTP (Time-based One-Time Password)
          </Card.Description>
        </Card.Header>
        <Card.Content class="space-y-6">
          {#if twoFactorMessage}
            <Alert variant={twoFactorMessage.includes('erfolgreich') ? 'default' : 'destructive'}>
              <AlertDescription>{twoFactorMessage}</AlertDescription>
            </Alert>
          {/if}

          <div class="space-y-4 rounded-lg border p-4">
            <div class="flex items-start space-x-4">
              <div class="rounded-full bg-primary/10 p-2">
                <KeyIcon class="h-5 w-5 text-primary" />
              </div>
              <div class="space-y-1 flex-1">
                <div class="flex items-center gap-2">
                  <p class="text-sm font-medium">Authenticator App (TOTP)</p>
                  {#if twoFactorEnabled}
                    <Badge variant="default" class="gap-1">
                      <CheckCircle2Icon class="h-3 w-3" />
                      Aktiv
                    </Badge>
                  {:else}
                    <Badge variant="secondary" class="gap-1">
                      <XCircleIcon class="h-3 w-3" />
                      Inaktiv
                    </Badge>
                  {/if}
                </div>
                <p class="text-sm text-muted-foreground">
                  Verwenden Sie eine Authenticator App wie Google Authenticator oder Authy
                </p>
              </div>
            </div>

            {#if !twoFactorEnabled && !totpQrCode}
              <Button onclick={handleSetup2FA} disabled={is2FALoading} class="w-full">
                {#if is2FALoading}
                  <div
                    class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"
                  ></div>
                {/if}
                2FA aktivieren
              </Button>
            {/if}

            {#if totpQrCode && !twoFactorEnabled}
              <div class="space-y-4">
                <div class="bg-white p-4 rounded-lg border">
                  <div class="flex flex-col items-center gap-4">
                    <div class="bg-white p-2 rounded">
                      <img
                        src="data:image/png;base64,{totpQrCode}"
                        alt="TOTP QR Code"
                        class="w-48 h-48"
                      />
                    </div>
                    <div class="text-center space-y-2">
                      <p class="text-sm font-medium">Secret Key</p>
                      <code class="text-xs bg-muted px-2 py-1 rounded">{totpSecret}</code>
                    </div>
                  </div>
                </div>

                <form onsubmit={handleEnable2FA} class="space-y-4">
                  <Field>
                    <FieldLabel for="verification-code">Verifizierungscode</FieldLabel>
                    <Input
                      id="verification-code"
                      bind:value={verificationCode}
                      placeholder="123456"
                      maxlength={6}
                      disabled={is2FALoading}
                    />
                    <FieldDescription
                      >Geben Sie den 6-stelligen Code aus Ihrer Authenticator App ein</FieldDescription
                    >
                  </Field>

                  <Button type="submit" disabled={is2FALoading} class="w-full">
                    {#if is2FALoading}
                      <div
                        class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"
                      ></div>
                    {/if}
                    Code verifizieren
                  </Button>
                </form>
              </div>
            {/if}

            {#if twoFactorEnabled}
              <div class="space-y-4">
                <Alert>
                  <CheckCircle2Icon class="h-4 w-4" />
                  <AlertDescription>
                    2FA ist aktiviert. Sie benötigen bei der Anmeldung einen Code aus Ihrer
                    Authenticator App.
                  </AlertDescription>
                </Alert>

                <form onsubmit={handleDisable2FA} class="space-y-4">
                  <Field>
                    <FieldLabel for="disable-code">Verifizierungscode zum Deaktivieren</FieldLabel>
                    <Input
                      id="disable-code"
                      bind:value={disableVerificationCode}
                      placeholder="123456"
                      maxlength={6}
                      disabled={is2FALoading}
                    />
                  </Field>

                  <Button
                    variant="destructive"
                    type="submit"
                    disabled={is2FALoading}
                    class="w-full"
                  >
                    {#if is2FALoading}
                      <div
                        class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"
                      ></div>
                    {/if}
                    2FA deaktivieren
                  </Button>
                </form>
              </div>
            {/if}
          </div>
        </Card.Content>
      </Card.Root>
    </Tabs.Content>

    <Tabs.Content value="updates" class="space-y-6">
      <UpdateSettings />
    </Tabs.Content>

    <Tabs.Content value="organization" class="space-y-6">
      <Card.Root>
        <Card.Header>
          <Card.Title>Organisation</Card.Title>
          <Card.Description>Verwalten Sie Ihre Organisationseinstellungen</Card.Description>
        </Card.Header>
        <Card.Content>
          <p class="text-muted-foreground mb-4">
            Besuchen Sie die <a
              href="/admin/organization"
              class="text-primary hover:underline font-medium">Organisationsverwaltungsseite</a
            >, um Ihre Organisation zu verwalten.
          </p>
          <Button href="/admin/organization">
            <Building2 class="mr-2 h-4 w-4" />
            Zur Organisationsverwaltung
          </Button>
        </Card.Content>
      </Card.Root>
    </Tabs.Content>

    <Tabs.Content value="users" class="space-y-6">
      <Card.Root>
        <Card.Header>
          <Card.Title>Benutzerverwaltung</Card.Title>
          <Card.Description>Verwalten Sie Benutzer und deren Rollen</Card.Description>
        </Card.Header>
        <Card.Content>
          <p class="text-muted-foreground mb-4">
            Besuchen Sie die <a href="/admin/users" class="text-primary hover:underline font-medium"
              >Benutzerverwaltungsseite</a
            >, um Benutzer zu erstellen, bearbeiten und löschen.
          </p>
          <Button href="/admin/users">
            <Users class="mr-2 h-4 w-4" />
            Zur Benutzerverwaltung
          </Button>
        </Card.Content>
      </Card.Root>
    </Tabs.Content>
  </Tabs.Root>
</div>
