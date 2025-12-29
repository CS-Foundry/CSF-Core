export interface ResourceGroup {
  id: string;
  name: string;
  description?: string;
  organization_id: string;
  created_by?: string;
  created_at: string;
  updated_at: string;
  tags?: Record<string, string>;
  location?: string;
}

export interface CreateResourceGroupRequest {
  name: string;
  description?: string;
  location?: string;
  tags?: Record<string, string>;
}

export interface UpdateResourceGroupRequest {
  name?: string;
  description?: string;
  location?: string;
  tags?: Record<string, string>;
}
