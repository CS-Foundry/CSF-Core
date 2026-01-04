export interface VersionInfo {
	current_version: string;
	latest_version: string;
	update_available: boolean;
	changelog: string | null;
	release_url: string;
}

export interface UpdateResponse {
	success: boolean;
	message: string;
}

export class UpdateService {
	private apiUrl: string;

	constructor() {
		this.apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:8080';
	}

	/**
	 * Check for available updates
	 */
	async checkForUpdates(): Promise<VersionInfo> {
		const response = await fetch(`${this.apiUrl}/api/updates/check`, {
			method: 'GET',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json'
			}
		});

		if (!response.ok) {
			throw new Error(`Failed to check for updates: ${response.statusText}`);
		}

		return await response.json();
	}

	/**
	 * Install a specific version
	 */
	async installUpdate(version: string): Promise<UpdateResponse> {
		const response = await fetch(`${this.apiUrl}/api/updates/install`, {
			method: 'POST',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ version })
		});

		if (!response.ok) {
			throw new Error(`Failed to install update: ${response.statusText}`);
		}

		return await response.json();
	}

	/**
	 * Get changelog for a specific version
	 */
	async getChangelog(version: string): Promise<string> {
		const response = await fetch(`${this.apiUrl}/api/updates/changelog/${version}`, {
			method: 'GET',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json'
			}
		});

		if (!response.ok) {
			throw new Error(`Failed to get changelog: ${response.statusText}`);
		}

		return await response.json();
	}
}

export const updateService = new UpdateService();
