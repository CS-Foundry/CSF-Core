<script lang="ts">
  import { onMount } from "svelte";
  import { organizationService } from "$lib/services/organization";
  import type { User, Role } from "$lib/types/organization";
  import { Button } from "$lib/components/ui/button";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from "$lib/components/ui/dialog";
  import { Trash2, Edit, Plus, Shield } from "@lucide/svelte";

  let users = $state<User[]>([]);
  let roles = $state<Role[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  // Create user dialog
  let createDialogOpen = $state(false);
  let newUser = $state({
    username: "",
    email: "",
    password: "",
    role_id: "",
    force_password_change: false,
  });

  // Edit user dialog
  let editDialogOpen = $state(false);
  let editingUser = $state<User | null>(null);
  let editForm = $state({
    email: "",
    force_password_change: false,
  });

  // Edit role dialog
  let roleDialogOpen = $state(false);
  let roleEditingUser = $state<User | null>(null);
  let newRoleId = $state("");

  // Delete confirmation
  let deleteDialogOpen = $state(false);
  let deletingUser = $state<User | null>(null);

  async function loadData() {
    loading = true;
    error = null;
    try {
      [users, roles] = await Promise.all([
        organizationService.listUsers(),
        organizationService.getRoles(),
      ]);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load data";
    } finally {
      loading = false;
    }
  }

  async function handleCreateUser() {
    if (!newUser.username || !newUser.password || !newUser.role_id) {
      error = "Please fill in all required fields";
      return;
    }

    loading = true;
    error = null;
    try {
      await organizationService.createUser({
        username: newUser.username,
        email: newUser.email || null,
        password: newUser.password,
        role_id: newUser.role_id,
        force_password_change: newUser.force_password_change,
      });
      await loadData();
      createDialogOpen = false;
      newUser = {
        username: "",
        email: "",
        password: "",
        role_id: "",
        force_password_change: false,
      };
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create user";
    } finally {
      loading = false;
    }
  }

  async function handleUpdateUser() {
    if (!editingUser) return;

    loading = true;
    error = null;
    try {
      await organizationService.updateUser(editingUser.id, {
        email: editForm.email || null,
        force_password_change: editForm.force_password_change,
      });
      await loadData();
      editDialogOpen = false;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to update user";
    } finally {
      loading = false;
    }
  }

  async function handleUpdateRole() {
    if (!roleEditingUser || !newRoleId) return;

    loading = true;
    error = null;
    try {
      await organizationService.updateUserRole(roleEditingUser.id, {
        role_id: newRoleId,
      });
      await loadData();
      roleDialogOpen = false;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to update role";
    } finally {
      loading = false;
    }
  }

  async function handleDeleteUser() {
    if (!deletingUser) return;

    loading = true;
    error = null;
    try {
      await organizationService.deleteUser(deletingUser.id);
      await loadData();
      deleteDialogOpen = false;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to delete user";
    } finally {
      loading = false;
    }
  }

  function openEditDialog(user: User) {
    editingUser = user;
    editForm = {
      email: user.email || "",
      force_password_change: user.force_password_change,
    };
    editDialogOpen = true;
  }

  function openRoleDialog(user: User) {
    roleEditingUser = user;
    newRoleId = user.role_id;
    roleDialogOpen = true;
  }

  function openDeleteDialog(user: User) {
    deletingUser = user;
    deleteDialogOpen = true;
  }

  onMount(() => {
    loadData();
  });
</script>

<div class="mb-6 flex items-center justify-between">
  <div>
    <h2 class="text-2xl font-bold">User Management</h2>
    <p class="text-muted-foreground">Manage users and their roles</p>
  </div>
  <Button onclick={() => (createDialogOpen = true)}>
    <Plus class="mr-2 h-4 w-4" />
    Add User
  </Button>
</div>

{#if error}
  <div
    class="mb-4 rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive"
  >
    {error}
  </div>
{/if}

{#if loading}
  <div class="text-center">Loading...</div>
{:else}
  <div class="rounded-md border">
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Username</TableHead>
          <TableHead>Email</TableHead>
          <TableHead>Role</TableHead>
          <TableHead>2FA</TableHead>
          <TableHead>Force Password Change</TableHead>
          <TableHead>Joined</TableHead>
          <TableHead class="text-right">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {#each users as user (user.id)}
          <TableRow>
            <TableCell class="font-medium">{user.username}</TableCell>
            <TableCell>{user.email || "-"}</TableCell>
            <TableCell>
              <div class="flex items-center gap-2">
                <Shield class="h-4 w-4 text-muted-foreground" />
                {user.role_name}
              </div>
            </TableCell>
            <TableCell>
              {#if user.two_factor_enabled}
                <span class="text-green-600">Enabled</span>
              {:else}
                <span class="text-muted-foreground">Disabled</span>
              {/if}
            </TableCell>
            <TableCell>
              {#if user.force_password_change}
                <span class="text-orange-600">Yes</span>
              {:else}
                <span class="text-muted-foreground">No</span>
              {/if}
            </TableCell>
            <TableCell
              >{new Date(user.joined_at).toLocaleDateString()}</TableCell
            >
            <TableCell class="text-right">
              <div class="flex justify-end gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => openRoleDialog(user)}
                >
                  <Shield class="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => openEditDialog(user)}
                >
                  <Edit class="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => openDeleteDialog(user)}
                  class="text-destructive hover:text-destructive"
                >
                  <Trash2 class="h-4 w-4" />
                </Button>
              </div>
            </TableCell>
          </TableRow>
        {/each}
      </TableBody>
    </Table>
  </div>
{/if}

<!-- Create User Dialog -->
<Dialog bind:open={createDialogOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Create New User</DialogTitle>
      <DialogDescription>Add a new user to the organization</DialogDescription>
    </DialogHeader>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="username">Username *</Label>
        <Input id="username" bind:value={newUser.username} />
      </div>
      <div class="grid gap-2">
        <Label for="email">Email</Label>
        <Input id="email" type="email" bind:value={newUser.email} />
      </div>
      <div class="grid gap-2">
        <Label for="password">Password *</Label>
        <Input id="password" type="password" bind:value={newUser.password} />
      </div>
      <div class="grid gap-2">
        <Label for="role">Role *</Label>
        <select
          id="role"
          bind:value={newUser.role_id}
          class="flex h-9 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
        >
          <option value="">Select a role</option>
          {#each roles as role (role.id)}
            <option value={role.id}>{role.name}</option>
          {/each}
        </select>
      </div>
      <div class="flex items-center space-x-2">
        <input
          id="force-pwd"
          type="checkbox"
          bind:checked={newUser.force_password_change}
          class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
        />
        <Label for="force-pwd">Force password change on first login</Label>
      </div>
    </div>
    <DialogFooter>
      <Button variant="outline" onclick={() => (createDialogOpen = false)}
        >Cancel</Button
      >
      <Button onclick={handleCreateUser} disabled={loading}>Create User</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<!-- Edit User Dialog -->
<Dialog bind:open={editDialogOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Edit User</DialogTitle>
      <DialogDescription>Update user information</DialogDescription>
    </DialogHeader>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="edit-email">Email</Label>
        <Input id="edit-email" type="email" bind:value={editForm.email} />
      </div>
      <div class="flex items-center space-x-2">
        <input
          id="edit-force-pwd"
          type="checkbox"
          bind:checked={editForm.force_password_change}
          class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
        />
        <Label for="edit-force-pwd">Force password change</Label>
      </div>
    </div>
    <DialogFooter>
      <Button variant="outline" onclick={() => (editDialogOpen = false)}
        >Cancel</Button
      >
      <Button onclick={handleUpdateUser} disabled={loading}>Save Changes</Button
      >
    </DialogFooter>
  </DialogContent>
</Dialog>

<!-- Edit Role Dialog -->
<Dialog bind:open={roleDialogOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Change User Role</DialogTitle>
      <DialogDescription>Assign a new role to this user</DialogDescription>
    </DialogHeader>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="new-role">Role</Label>
        <select
          id="new-role"
          bind:value={newRoleId}
          class="flex h-9 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
        >
          <option value="">Select a role</option>
          {#each roles as role (role.id)}
            <option value={role.id}>{role.name}</option>
          {/each}
        </select>
      </div>
    </div>
    <DialogFooter>
      <Button variant="outline" onclick={() => (roleDialogOpen = false)}
        >Cancel</Button
      >
      <Button onclick={handleUpdateRole} disabled={loading}>Update Role</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<!-- Delete Confirmation Dialog -->
<Dialog bind:open={deleteDialogOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Delete User</DialogTitle>
      <DialogDescription>
        Are you sure you want to delete {deletingUser?.username}? This action
        cannot be undone.
      </DialogDescription>
    </DialogHeader>
    <DialogFooter>
      <Button variant="outline" onclick={() => (deleteDialogOpen = false)}
        >Cancel</Button
      >
      <Button
        variant="destructive"
        onclick={handleDeleteUser}
        disabled={loading}
      >
        Delete
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
