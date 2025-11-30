<script lang="ts">
  import Netflix from "@icons-pack/svelte-simple-icons/Netflix.js";
  import Spotify from "@icons-pack/svelte-simple-icons/Spotify.js";
  import Disney from "@icons-pack/svelte-simple-icons/Disney.js";
  import Amazonprime from "@icons-pack/svelte-simple-icons/Amazonprime.js";
  import Youtube from "@icons-pack/svelte-simple-icons/Youtube.js";
  import Twitch from "@icons-pack/svelte-simple-icons/Twitch.js";
  import Adobe from "@icons-pack/svelte-simple-icons/Adobe.js";
  import Microsoft from "@icons-pack/svelte-simple-icons/Microsoft.js";
  import Google from "@icons-pack/svelte-simple-icons/Google.js";
  import Dropbox from "@icons-pack/svelte-simple-icons/Dropbox.js";
  import Icloud from "@icons-pack/svelte-simple-icons/Icloud.js";
  import Githubcopilot from "@icons-pack/svelte-simple-icons/Githubcopilot.js";
  import Figma from "@icons-pack/svelte-simple-icons/Figma.js";
  import Notion from "@icons-pack/svelte-simple-icons/Notion.js";
  import Slack from "@icons-pack/svelte-simple-icons/Slack.js";
  import Zoom from "@icons-pack/svelte-simple-icons/Zoom.js";

  type Props = {
    serviceName: string;
    class?: string;
  };

  let { serviceName, class: className = "w-6 h-6" }: Props = $props();

  // Service-Name zu Icon-Mapping
  const serviceIconMap: Record<string, any> = {
    netflix: Netflix,
    spotify: Spotify,
    "disney plus": Disney,
    "disney+": Disney,
    disney: Disney,
    "amazon prime": Amazonprime,
    "prime video": Amazonprime,
    amazon: Amazonprime,
    youtube: Youtube,
    "youtube premium": Youtube,
    twitch: Twitch,
    adobe: Adobe,
    "adobe creative cloud": Adobe,
    photoshop: Adobe,
    illustrator: Adobe,
    microsoft: Microsoft,
    "microsoft 365": Microsoft,
    "office 365": Microsoft,
    excel: Microsoft,
    word: Microsoft,
    powerpoint: Microsoft,
    google: Google,
    "google one": Google,
    "google drive": Google,
    gmail: Google,
    dropbox: Dropbox,
    icloud: Icloud,
    "icloud storage": Icloud,
    "github copilot": Githubcopilot,
    copilot: Githubcopilot,
    figma: Figma,
    notion: Notion,
    slack: Slack,
    zoom: Zoom,
  };

  // Finde das passende Icon basierend auf dem Service-Namen
  const getIconComponent = (name: string) => {
    const normalizedName = name.toLowerCase().trim();
    
    // Exakte Übereinstimmung suchen
    if (serviceIconMap[normalizedName]) {
      return serviceIconMap[normalizedName];
    }
    
    // Teilübereinstimmung suchen
    for (const [key, icon] of Object.entries(serviceIconMap)) {
      if (normalizedName.includes(key) || key.includes(normalizedName)) {
        return icon;
      }
    }
    
    return null;
  };

  const IconComponent = $derived(getIconComponent(serviceName));
</script>

{#if IconComponent}
  <IconComponent class="{className} text-muted-foreground" />
{:else}
  <!-- Fallback Icon für unbekannte Services -->
  <div class="{className} flex items-center justify-center bg-muted rounded text-muted-foreground text-sm font-medium">
    {serviceName.charAt(0).toUpperCase()}
  </div>
{/if}