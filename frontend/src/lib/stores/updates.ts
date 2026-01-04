import { writable, derived } from 'svelte/store';
import { updateService, type VersionInfo } from '$lib/services/updates';

interface UpdateState {
	versionInfo: VersionInfo | null;
	loading: boolean;
	error: string | null;
	lastChecked: Date | null;
	isInstalling: boolean;
}

const initialState: UpdateState = {
	versionInfo: null,
	loading: false,
	error: null,
	lastChecked: null,
	isInstalling: false
};

function createUpdateStore() {
	const { subscribe, set, update } = writable<UpdateState>(initialState);

	// Check for updates every hour
	let intervalId: number | null = null;

	return {
		subscribe,

		async checkForUpdates() {
			update((state) => ({ ...state, loading: true, error: null }));

			try {
				const versionInfo = await updateService.checkForUpdates();
				update((state) => ({
					...state,
					versionInfo,
					loading: false,
					lastChecked: new Date()
				}));
			} catch (error) {
				update((state) => ({
					...state,
					loading: false,
					error: error instanceof Error ? error.message : 'Failed to check for updates'
				}));
			}
		},

		async installUpdate(version: string) {
			update((state) => ({ ...state, isInstalling: true, error: null }));

			try {
				const response = await updateService.installUpdate(version);
				if (response.success) {
					update((state) => ({ ...state, isInstalling: false }));
					return response;
				} else {
					throw new Error(response.message);
				}
			} catch (error) {
				update((state) => ({
					...state,
					isInstalling: false,
					error: error instanceof Error ? error.message : 'Failed to install update'
				}));
				throw error;
			}
		},

		startAutoCheck() {
			// Check immediately
			this.checkForUpdates();

			// Then check every hour
			if (intervalId === null) {
				intervalId = window.setInterval(
					() => {
						this.checkForUpdates();
					},
					60 * 60 * 1000
				); // 1 hour
			}
		},

		stopAutoCheck() {
			if (intervalId !== null) {
				clearInterval(intervalId);
				intervalId = null;
			}
		},

		reset() {
			set(initialState);
		}
	};
}

export const updateStore = createUpdateStore();

// Derived store for easy access to update availability
export const updateAvailable = derived(updateStore, ($updateStore) => {
	return $updateStore.versionInfo?.update_available ?? false;
});
