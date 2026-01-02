import type {
  Organization,
  Role,
  User,
  CreateUserRequest,
  UpdateUserRequest,
  UpdateUserRoleRequest,
  UpdateOrganizationRequest,
} from '$lib/types/organization';
import { ApiClient } from './api-client';

export const organizationService = {
  // Organization endpoints
  async getOrganization(): Promise<Organization> {
    const response = await ApiClient.get('/organization');
    if (!response.ok) {
      throw new Error('Failed to fetch organization');
    }
    return response.json();
  },

  async updateOrganization(data: UpdateOrganizationRequest): Promise<Organization> {
    const response = await ApiClient.fetch('/organization', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!response.ok) {
      throw new Error('Failed to update organization');
    }
    return response.json();
  },

  // Roles
  async getRoles(): Promise<Role[]> {
    const response = await ApiClient.get('/organization/roles');
    if (!response.ok) {
      throw new Error('Failed to fetch roles');
    }
    return response.json();
  },

  // User management
  async listUsers(): Promise<User[]> {
    const response = await ApiClient.get('/organization/users');
    if (!response.ok) {
      throw new Error('Failed to fetch users');
    }
    return response.json();
  },

  async getUser(userId: string): Promise<User> {
    const response = await ApiClient.get(`/organization/users/${userId}`);
    if (!response.ok) {
      throw new Error('Failed to fetch user');
    }
    return response.json();
  },

  async createUser(data: CreateUserRequest): Promise<User> {
    const response = await ApiClient.fetch('/organization/users', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!response.ok) {
      throw new Error('Failed to create user');
    }
    return response.json();
  },

  async updateUser(userId: string, data: UpdateUserRequest): Promise<User> {
    const response = await ApiClient.fetch(`/organization/users/${userId}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!response.ok) {
      throw new Error('Failed to update user');
    }
    return response.json();
  },

  async deleteUser(userId: string): Promise<void> {
    const response = await ApiClient.fetch(`/organization/users/${userId}`, {
      method: 'DELETE',
    });
    if (!response.ok) {
      throw new Error('Failed to delete user');
    }
  },

  async updateUserRole(userId: string, data: UpdateUserRoleRequest): Promise<void> {
    const response = await ApiClient.fetch(`/organization/users/${userId}/role`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!response.ok) {
      throw new Error('Failed to update user role');
    }
  },
};
