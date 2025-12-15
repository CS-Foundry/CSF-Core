import { PUBLIC_API_BASE_URL } from '$env/static/public';
import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import { authStore } from '$lib/stores/auth';
import { logger } from '$lib/utils/logger';
import { get } from 'svelte/store';

const API_BASE_URL = PUBLIC_API_BASE_URL;

export interface ApiError extends Error {
	status: number;
	statusText: string;
}

export class ApiClient {
	/**
	 * Makes an authenticated API request with automatic error handling
	 * and logout on 401 errors
	 */
	static async fetch(
		url: string,
		options: RequestInit = {},
		token?: string
	): Promise<Response> {
		logger.debug(`Making request to: ${url}`, { method: options.method || 'GET' });

		// Get token from store if not provided
		if (!token && browser) {
			const authState = get(authStore);
			token = authState.token ?? undefined;
			console.log('[ApiClient] Token from store:', { hasToken: !!token, isAuthenticated: authState.isAuthenticated });
		}

		// Add authorization header if token exists
		const headers: Record<string, string> = {};
		
		if (options.headers) {
			if (options.headers instanceof Headers) {
				options.headers.forEach((value, key) => {
					headers[key] = value;
				});
			} else if (Array.isArray(options.headers)) {
				options.headers.forEach(([key, value]) => {
					headers[key] = value;
				});
			} else {
				Object.assign(headers, options.headers);
			}
		}

		if (token) {
			logger.debug('Attaching token to request headers');
			headers['Authorization'] = `Bearer ${token}`;
		} else {
			logger.warn('No token found to attach to request');
		}

		try {
			logger.debug('Sending fetch request...');
			const response = await fetch(`${API_BASE_URL}${url}`, {
				...options,
				headers,
				credentials: 'include', // Important for cookies
			});

			logger.info(`Received response from: ${url}`, {
				status: response.status,
				statusText: response.statusText,
			});

			// Handle 401 Unauthorized - session expired or invalid
			if (response.status === 401) {
				logger.warn('Received 401 Unauthorized, logging out user');
				await this.handleUnauthorized();
				
				const error = new Error('Unauthorized') as ApiError;
				error.status = 401;
				error.statusText = 'Unauthorized';
				throw error;
			}

			return response;
		} catch (error) {
			// Re-throw API errors
			if ((error as ApiError).status === 401) {
				throw error;
			}

			// Network or other errors
			logger.error(`API request to ${url} failed`, error);
			throw error;
		}
	}

	/**
	 * Helper for GET requests
	 */
	static async get(url: string, token?: string): Promise<Response> {
		return this.fetch(url, { method: 'GET' }, token);
	}

	/**
	 * Helper for POST requests
	 */
	static async post(url: string, body?: unknown, token?: string): Promise<Response> {
		const headers: Record<string, string> = {
			'Content-Type': 'application/json'
		};
		
		return this.fetch(
			url,
			{
				method: 'POST',
				headers,
				body: body ? JSON.stringify(body) : undefined,
			},
			token
		);
	}

	/**
	 * Helper for PUT requests
	 */
	static async put(url: string, body?: unknown, token?: string): Promise<Response> {
		const headers: Record<string, string> = {
			'Content-Type': 'application/json'
		};
		
		return this.fetch(
			url,
			{
				method: 'PUT',
				headers,
				body: body ? JSON.stringify(body) : undefined,
			},
			token
		);
	}

	/**
	 * Helper for DELETE requests
	 */
	static async delete(url: string, token?: string): Promise<Response> {
		return this.fetch(url, { method: 'DELETE' }, token);
	}

	/**
	 * Handles unauthorized access - logs out user and redirects to signin
	 */
	private static async handleUnauthorized() {
		logger.info('Handling unauthorized access - logging out user');
		
		// Clear auth cookie
		try {
			await fetch('/api/set-auth-cookie', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ token: null }),
			});
			logger.debug('Auth cookie cleared');
		} catch (error) {
			logger.error('Failed to clear auth cookie', error);
		}

		// Update auth store
		authStore.logout();
		logger.debug('Auth store updated - user logged out');

		// Redirect to signin page
		if (browser && window.location.pathname !== '/signin') {
			logger.info('Redirecting to signin page');
			goto('/signin');
		}
	}
}
