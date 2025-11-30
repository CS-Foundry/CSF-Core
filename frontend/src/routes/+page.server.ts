import { redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions = {
	logout: async ({ cookies }) => {
		// Remove auth cookie
		cookies.delete('auth_token', { path: '/' });
		
		// Redirect to signin
		throw redirect(303, '/signin');
	}
} satisfies Actions;
