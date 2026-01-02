export interface Organization {
  id: string;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface Role {
  id: string;
  name: string;
  description: string | null;
  is_system_role: boolean;
}

export interface User {
  id: string;
  username: string;
  email: string | null;
  role_id: string;
  role_name: string;
  force_password_change: boolean;
  two_factor_enabled: boolean;
  joined_at: string;
}

export interface CreateUserRequest {
  username: string;
  email: string | null;
  password: string;
  role_id: string;
  force_password_change: boolean;
}

export interface UpdateUserRequest {
  email?: string | null;
  force_password_change?: boolean;
}

export interface UpdateUserRoleRequest {
  role_id: string;
}

export interface UpdateOrganizationRequest {
  name: string;
  description: string | null;
}
