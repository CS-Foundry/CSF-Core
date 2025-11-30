import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import { ApiClient } from '$lib/services/api-client';
import { logger } from '$lib/utils/logger';

interface User {
	id: string;
	username: string;
}

interface AuthState {
	user: User | null;
	token: string | null;
	isAuthenticated: boolean;
	isLoading: boolean;
}

const initialState: AuthState = {
	user: null,
	token: null,
	isAuthenticated: false,
	isLoading: false
};

function createAuthStore() {
	const { subscribe, set, update } = writable<AuthState>(initialState);

	return {
		subscribe,
		init: (user: User | null, token: string | null) => {
			logger.info('Initializing auth store', { hasUser: !!user, hasToken: !!token });
			if (browser) {
				if (user && token) {
					logger.info('User and token found, setting authenticated state');
					set({
						user,
						token,
						isAuthenticated: true,
						isLoading: false
					});
					} else {
					logger.debug('No user or token, setting initial state');
					set(initialState);
				}
			}
		},
		login: (user: User, token: string) => {
			logger.info('Logging in user', { username: user.username, userId: user.id });
			set({
				user,
				token,
				isAuthenticated: true,
				isLoading: false
			});
			
		},
		logout: () => {
			logger.info('Logging out user');
			
			// Cookie is deleted by the /api/set-auth-cookie endpoint
			set(initialState);
		},
		setLoading: (loading: boolean) => {
			update(state => ({ ...state, isLoading: loading }));
		}
	};
}

export const authStore = createAuthStore();