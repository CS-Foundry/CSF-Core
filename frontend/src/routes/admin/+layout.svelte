<script lang="ts">
  import { page } from '$app/stores';
  import { Building2, Users, Settings } from '@lucide/svelte';

  const navItems = [
    {
      href: '/admin/organization',
      icon: Building2,
      label: 'Organization',
    },
    {
      href: '/admin/users',
      icon: Users,
      label: 'Users',
    },
    {
      href: '/settings',
      icon: Settings,
      label: 'Back to Settings',
    },
  ];

  let currentPath = $derived($page.url.pathname);
</script>

<div class="min-h-screen bg-background">
  <div class="border-b">
    <div class="container mx-auto px-6 py-4">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-bold">Administration</h1>
          <p class="text-sm text-muted-foreground">Manage your organization and users</p>
        </div>
      </div>
    </div>
  </div>

  <div class="container mx-auto px-6 py-6">
    <div class="grid gap-6 lg:grid-cols-[240px_1fr]">
      <!-- Sidebar Navigation -->
      <aside class="space-y-2">
        <nav class="space-y-1">
          {#each navItems as item}
            <a
              href={item.href}
              class="flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors {currentPath ===
              item.href
                ? 'bg-primary text-primary-foreground'
                : 'hover:bg-accent hover:text-accent-foreground'}"
            >
              <svelte:component this={item.icon} class="h-4 w-4" />
              {item.label}
            </a>
          {/each}
        </nav>
      </aside>

      <!-- Main Content -->
      <main>
        <slot />
      </main>
    </div>
  </div>
</div>
