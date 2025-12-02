<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.js";
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import {
    FieldGroup,
    Field,
    FieldLabel,
    FieldDescription,
  } from "$lib/components/ui/field/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Alert, AlertDescription } from "$lib/components/ui/alert/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { Switch } from "$lib/components/ui/switch/index.js";
  import { authStore } from "$lib/stores/auth";
  import {
    UserIcon,
    ShieldCheckIcon,
    KeyIcon,
    SmartphoneIcon,
    MailIcon,
    CheckCircle2Icon,
    XCircleIcon,
  } from "@lucide/svelte";

  let authState = $derived($authStore);

  // Profile state
  let username = $state(authState.user?.username || "");
  let email = $state("");
  let currentPassword = $state("");
  let newPassword = $state("");
  let confirmPassword = $state("");
  let isProfileLoading = $state(false);
  let profileMessage = $state("");

  // OTP/TOTP state
  let otpEnabled = $state(false);
  let emailOtpEnabled = $state(true);
  let smsOtpEnabled = $state(false);
  let totpEnabled = $state(false);
  let phoneNumber = $state("");
  let totpQrCode = $state("");
  let totpSecret = $state("");
  let verificationCode = $state("");
  let isOtpLoading = $state(false);
  let otpMessage = $state("");

  async function handleProfileUpdate(event: Event) {
    event.preventDefault();
    isProfileLoading = true;
    profileMessage = "";

    try {
      // TODO: Implement API call
      await new Promise((resolve) => setTimeout(resolve, 1000));
      profileMessage = "Profil erfolgreich aktualisiert";
    } catch (error) {
      profileMessage =
        error instanceof Error
          ? error.message
          : "Aktualisierung fehlgeschlagen";
    } finally {
      isProfileLoading = false;
    }
  }

  async function handlePasswordChange(event: Event) {
    event.preventDefault();

    if (newPassword !== confirmPassword) {
      profileMessage = "Passwörter stimmen nicht überein";
      return;
    }

    isProfileLoading = true;
    profileMessage = "";

    try {
      // TODO: Implement API call
      await new Promise((resolve) => setTimeout(resolve, 1000));
      profileMessage = "Passwort erfolgreich geändert";
      currentPassword = "";
      newPassword = "";
      confirmPassword = "";
    } catch (error) {
      profileMessage =
        error instanceof Error
          ? error.message
          : "Passwortänderung fehlgeschlagen";
    } finally {
      isProfileLoading = false;
    }
  }

  async function handleEnableTotp() {
    isOtpLoading = true;
    otpMessage = "";

    try {
      // TODO: Implement API call to generate TOTP secret
      await new Promise((resolve) => setTimeout(resolve, 1000));
      totpSecret = "JBSWY3DPEHPK3PXP";
      totpQrCode = `otpauth://totp/CSF-Core:${username}?secret=${totpSecret}&issuer=CSF-Core`;
      otpMessage = "Scannen Sie den QR-Code mit Ihrer Authenticator App";
    } catch (error) {
      otpMessage =
        error instanceof Error
          ? error.message
          : "TOTP-Aktivierung fehlgeschlagen";
    } finally {
      isOtpLoading = false;
    }
  }

  async function handleVerifyTotp(event: Event) {
    event.preventDefault();
    isOtpLoading = true;
    otpMessage = "";

    try {
      // TODO: Implement API call to verify TOTP code
      await new Promise((resolve) => setTimeout(resolve, 1000));
      totpEnabled = true;
      otpEnabled = true;
      verificationCode = "";
      totpQrCode = "";
      otpMessage = "TOTP erfolgreich aktiviert";
    } catch (error) {
      otpMessage =
        error instanceof Error ? error.message : "Verifizierung fehlgeschlagen";
    } finally {
      isOtpLoading = false;
    }
  }

  async function handleToggleEmailOtp() {
    isOtpLoading = true;
    try {
      // TODO: Implement API call
      await new Promise((resolve) => setTimeout(resolve, 500));
      emailOtpEnabled = !emailOtpEnabled;
    } catch (error) {
      console.error(error);
    } finally {
      isOtpLoading = false;
    }
  }

  async function handleToggleSmsOtp() {
    if (!phoneNumber && !smsOtpEnabled) {
      otpMessage = "Bitte fügen Sie zuerst eine Telefonnummer hinzu";
      return;
    }
    isOtpLoading = true;
    try {
      // TODO: Implement API call
      await new Promise((resolve) => setTimeout(resolve, 500));
      smsOtpEnabled = !smsOtpEnabled;
    } catch (error) {
      console.error(error);
    } finally {
      isOtpLoading = false;
    }
  }

  async function handleDisableTotp() {
    isOtpLoading = true;
    try {
      // TODO: Implement API call
      await new Promise((resolve) => setTimeout(resolve, 500));
      totpEnabled = false;
      totpSecret = "";
      totpQrCode = "";
      if (!emailOtpEnabled && !smsOtpEnabled) {
        otpEnabled = false;
      }
    } catch (error) {
      console.error(error);
    } finally {
      isOtpLoading = false;
    }
  }
</script>

<div class="flex-1 space-y-6 p-8 pt-6">
  <div class="space-y-1">
    <h2 class="text-3xl font-bold tracking-tight">Einstellungen</h2>
    <p class="text-muted-foreground">
      Verwalten Sie Ihr Profil und Ihre Sicherheitseinstellungen
    </p>
  </div>

  <Tabs.Root value="profile" class="space-y-6">
    <Tabs.List class="grid w-full max-w-md grid-cols-2">
      <Tabs.Trigger value="profile" class="gap-2">
        <UserIcon class="h-4 w-4" />
        Profil
      </Tabs.Trigger>
      <Tabs.Trigger value="security" class="gap-2">
        <ShieldCheckIcon class="h-4 w-4" />
        Sicherheit
      </Tabs.Trigger>
    </Tabs.List>

    <!-- Profile Tab -->
    <Tabs.Content value="profile" class="space-y-6">
      <Card.Root>
        <Card.Header>
          <Card.Title>Profil-Informationen</Card.Title>
          <Card.Description>
            Aktualisieren Sie Ihre persönlichen Informationen
          </Card.Description>
        </Card.Header>
        <Card.Content>
          <form onsubmit={handleProfileUpdate} class="space-y-6">
            {#if profileMessage}
              <Alert
                variant={profileMessage.includes("erfolgreich")
                  ? "default"
                  : "destructive"}
              >
                <AlertDescription>{profileMessage}</AlertDescription>
              </Alert>
            {/if}

            <FieldGroup class="space-y-4">
              <Field>
                <FieldLabel for="username">Benutzername</FieldLabel>
                <Input
                  id="username"
                  bind:value={username}
                  placeholder="Ihr Benutzername"
                  disabled={isProfileLoading}
                />
              </Field>

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
            </FieldGroup>

            <Button type="submit" disabled={isProfileLoading}>
              {#if isProfileLoading}
                <div
                  class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"
                ></div>
              {/if}
              Profil aktualisieren
            </Button>
          </form>
        </Card.Content>
      </Card.Root>

      <Card.Root>
        <Card.Header>
          <Card.Title>Passwort ändern</Card.Title>
          <Card.Description>
            Ändern Sie Ihr Passwort für mehr Sicherheit
          </Card.Description>
        </Card.Header>
        <Card.Content>
          <form onsubmit={handlePasswordChange} class="space-y-6">
            <FieldGroup class="space-y-4">
              <Field>
                <FieldLabel for="current-password"
                  >Aktuelles Passwort</FieldLabel
                >
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
                <FieldLabel for="confirm-password">
                  Passwort bestätigen
                </FieldLabel>
                <Input
                  id="confirm-password"
                  type="password"
                  bind:value={confirmPassword}
                  placeholder="••••••••"
                  disabled={isProfileLoading}
                />
              </Field>
            </FieldGroup>

            <Button type="submit" disabled={isProfileLoading}>
              {#if isProfileLoading}
                <div
                  class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"
                ></div>
              {/if}
              Passwort ändern
            </Button>
          </form>
        </Card.Content>
      </Card.Root>
    </Tabs.Content>

    <!-- Security Tab -->
    <Tabs.Content value="security" class="space-y-6">
      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center gap-2">
            <ShieldCheckIcon class="h-5 w-5" />
            Zwei-Faktor-Authentifizierung
          </Card.Title>
          <Card.Description>
            Erhöhen Sie die Sicherheit Ihres Kontos mit zusätzlichen
            Verifizierungsmethoden
          </Card.Description>
        </Card.Header>
        <Card.Content class="space-y-6">
          {#if otpMessage}
            <Alert
              variant={otpMessage.includes("erfolgreich")
                ? "default"
                : "destructive"}
            >
              <AlertDescription>{otpMessage}</AlertDescription>
            </Alert>
          {/if}

          <!-- E-Mail OTP -->
          <div
            class="flex items-center justify-between space-x-4 rounded-lg border p-4"
          >
            <div class="flex items-start space-x-4">
              <div class="rounded-full bg-primary/10 p-2">
                <MailIcon class="h-5 w-5 text-primary" />
              </div>
              <div class="space-y-1 flex-1">
                <div class="flex items-center gap-2">
                  <p class="text-sm font-medium">E-Mail Verifizierung</p>
                  {#if emailOtpEnabled}
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
                  Erhalten Sie Verifizierungscodes per E-Mail
                </p>
              </div>
            </div>
            <Switch
              checked={emailOtpEnabled}
              onCheckedChange={handleToggleEmailOtp}
              disabled={isOtpLoading}
            />
          </div>

          <!-- SMS OTP -->
          <div class="space-y-4 rounded-lg border p-4">
            <div class="flex items-center justify-between space-x-4">
              <div class="flex items-start space-x-4">
                <div class="rounded-full bg-primary/10 p-2">
                  <SmartphoneIcon class="h-5 w-5 text-primary" />
                </div>
                <div class="space-y-1 flex-1">
                  <div class="flex items-center gap-2">
                    <p class="text-sm font-medium">SMS Verifizierung</p>
                    {#if smsOtpEnabled}
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
                    Erhalten Sie Verifizierungscodes per SMS
                  </p>
                </div>
              </div>
              <Switch
                checked={smsOtpEnabled}
                onCheckedChange={handleToggleSmsOtp}
                disabled={isOtpLoading}
              />
            </div>
            {#if !phoneNumber}
              <Field>
                <FieldLabel for="phone">Telefonnummer</FieldLabel>
                <Input
                  id="phone"
                  type="tel"
                  bind:value={phoneNumber}
                  placeholder="+49 123 456789"
                  disabled={isOtpLoading}
                />
              </Field>
            {/if}
          </div>

          <Separator />

          <!-- TOTP/Authenticator App -->
          <div class="space-y-4 rounded-lg border p-4">
            <div class="flex items-start space-x-4">
              <div class="rounded-full bg-primary/10 p-2">
                <KeyIcon class="h-5 w-5 text-primary" />
              </div>
              <div class="space-y-1 flex-1">
                <div class="flex items-center gap-2">
                  <p class="text-sm font-medium">Authenticator App (TOTP)</p>
                  {#if totpEnabled}
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
                  Verwenden Sie eine Authenticator App wie Google Authenticator
                  oder Authy
                </p>
              </div>
            </div>

            {#if !totpEnabled && !totpQrCode}
              <Button
                onclick={handleEnableTotp}
                disabled={isOtpLoading}
                class="w-full"
              >
                {#if isOtpLoading}
                  <div
                    class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"
                  ></div>
                {/if}
                TOTP aktivieren
              </Button>
            {/if}

            {#if totpQrCode && !totpEnabled}
              <div class="space-y-4">
                <div class="bg-white p-4 rounded-lg border">
                  <div class="flex flex-col items-center gap-4">
                    <div class="bg-white p-2 rounded">
                      <img
                        src="https://api.qrserver.com/v1/create-qr-code/?size=200x200&data={encodeURIComponent(
                          totpQrCode
                        )}"
                        alt="TOTP QR Code"
                        class="w-48 h-48"
                      />
                    </div>
                    <div class="text-center space-y-2">
                      <p class="text-sm font-medium">Secret Key</p>
                      <code class="text-xs bg-muted px-2 py-1 rounded">
                        {totpSecret}
                      </code>
                    </div>
                  </div>
                </div>

                <form onsubmit={handleVerifyTotp} class="space-y-4">
                  <Field>
                    <FieldLabel for="verification-code">
                      Verifizierungscode
                    </FieldLabel>
                    <Input
                      id="verification-code"
                      bind:value={verificationCode}
                      placeholder="123456"
                      maxlength={6}
                      disabled={isOtpLoading}
                    />
                    <FieldDescription>
                      Geben Sie den 6-stelligen Code aus Ihrer Authenticator App
                      ein
                    </FieldDescription>
                  </Field>

                  <Button type="submit" disabled={isOtpLoading} class="w-full">
                    {#if isOtpLoading}
                      <div
                        class="animate-spin rounded-full h-4 w-4 border-b-2 border-current mr-2"
                      ></div>
                    {/if}
                    Code verifizieren
                  </Button>
                </form>
              </div>
            {/if}

            {#if totpEnabled}
              <div class="space-y-4">
                <Alert>
                  <CheckCircle2Icon class="h-4 w-4" />
                  <AlertDescription>
                    TOTP ist aktiviert. Verwenden Sie Ihre Authenticator App zur
                    Anmeldung.
                  </AlertDescription>
                </Alert>

                <Button
                  variant="destructive"
                  onclick={handleDisableTotp}
                  disabled={isOtpLoading}
                  class="w-full"
                >
                  TOTP deaktivieren
                </Button>
              </div>
            {/if}
          </div>
        </Card.Content>
      </Card.Root>
    </Tabs.Content>
  </Tabs.Root>
</div>
