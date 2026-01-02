<script lang="ts">
  import { Building2, Users } from '@lucide/svelte';
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold">Welcome to Administration</h2>
    <p class="text-muted-foreground">Manage your organization and users from here</p>
  </div>

  <div class="grid gap-6 md:grid-cols-2">
    <a
      href="/admin/organization"
      class="group flex flex-col gap-4 rounded-lg border p-6 transition-all hover:border-primary hover:shadow-md"
    >
      <div class="flex items-center gap-4">
        <div class="rounded-full bg-primary/10 p-3 transition-colors group-hover:bg-primary/20">
          <Building2 class="h-8 w-8 text-primary" />
        </div>
        <div>
          <h3 class="text-xl font-semibold">Organization</h3>
          <p class="text-sm text-muted-foreground">Manage organization settings</p>
        </div>
      </div>
      <p class="text-sm text-muted-foreground">
        View and update your organization's name, description, and other details.
      </p>
    </a>

    <a
      href="/admin/users"
      class="group flex flex-col gap-4 rounded-lg border p-6 transition-all hover:border-primary hover:shadow-md"
    >
      <div class="flex items-center gap-4">
        <div class="rounded-full bg-primary/10 p-3 transition-colors group-hover:bg-primary/20">
          <Users class="h-8 w-8 text-primary" />
        </div>
        <div>
          <h3 class="text-xl font-semibold">User Management</h3>
          <p class="text-sm text-muted-foreground">Create and manage users</p>
        </div>
      </div>
      <p class="text-sm text-muted-foreground">
        Create new users, assign roles, manage permissions, and control user access.
      </p>
    </a>
  </div>

  <div class="rounded-lg border bg-muted/50 p-4">
    <h3 class="mb-2 font-semibold">Quick Tips</h3>
    <ul class="space-y-1 text-sm text-muted-foreground">
      <li>• All users must be assigned a role when created</li>
      <li>• You can force users to change their password on first login</li>
      <li>• Admin role has full access to all features</li>
      <li>• Permission checks happen automatically on all API calls</li>
    </ul>
  </div>
</div>
