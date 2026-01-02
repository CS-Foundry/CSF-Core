import { ApiClient } from './api-client';

export interface MarketplaceTemplate {
  id: string;
  template_id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  resource_type: string;
  configuration: any;
  popular: boolean;
  install_count: number;
}

export interface InstallTemplateRequest {
  template_id: string;
  name: string;
  resource_group_id: string;
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    let errorMessage = 'Request failed';
    try {
      const errorData = await response.json();
      errorMessage = errorData.error || errorData.message || errorMessage;
    } catch {
      errorMessage = response.statusText || errorMessage;
    }
    throw new Error(errorMessage);
  }

  return response.json();
}

export async function listTemplates(): Promise<MarketplaceTemplate[]> {
  const response = await ApiClient.get('/marketplace/templates');
  return handleResponse<MarketplaceTemplate[]>(response);
}

export async function listPopularTemplates(): Promise<MarketplaceTemplate[]> {
  const response = await ApiClient.get('/marketplace/templates/popular');
  return handleResponse<MarketplaceTemplate[]>(response);
}

export async function getTemplate(templateId: string): Promise<MarketplaceTemplate> {
  const response = await ApiClient.get(`/marketplace/templates/${templateId}`);
  return handleResponse<MarketplaceTemplate>(response);
}

export async function installTemplate(data: InstallTemplateRequest): Promise<any> {
  const response = await ApiClient.post('/marketplace/install', data);
  return handleResponse<any>(response);
}

export async function seedMarketplace(): Promise<any> {
  const response = await ApiClient.post('/marketplace/seed', {});
  return handleResponse<any>(response);
}
