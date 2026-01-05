import { ApiClient } from './api-client';

export interface VersionInfo {
	current_version: string;
	latest_version: string;
	update_available: boolean;
	changelog: string | null;
	release_url: string;
	is_prerelease: boolean;
	latest_beta_version: string | null;
	beta_release_url: string | null;
}

export interface UpdateResponse {
	success: boolean;
	message: string;
}

export class UpdateService {
	/**
	 * Check for available updates
	 */
	async checkForUpdates(): Promise<VersionInfo> {
		const response = await ApiClient.get('/updates/check');

		if (!response.ok) {
			throw new Error(`Failed to check for updates: ${response.statusText}`);
		}

		return await response.json();
	}

	/**
	 * Install a specific version
	 */
	async installUpdate(version: string): Promise<UpdateResponse> {
		const response = await ApiClient.post('/updates/install', { version });

		if (!response.ok) {
			throw new Error(`Failed to install update: ${response.statusText}`);
		}

		return await response.json();
	}

	/**
	 * Get changelog for a specific version
	 */
	async getChangelog(version: string): Promise<string> {
		const response = await ApiClient.get(`/updates/changelog/${version}`);

		if (!response.ok) {
			throw new Error(`Failed to get changelog: ${response.statusText}`);
		}

		return await response.json();
	}
}

export const updateService = new UpdateService();
