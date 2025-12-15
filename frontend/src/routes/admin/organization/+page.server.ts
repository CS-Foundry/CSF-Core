import type { PageServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ parent }) => {
	const { user } = await parent();

	if (!user) {
		throw redirect(302, '/signin');
	}

	// Permission checks are done by the backend API endpoints
	// If user doesn't have access, API will return 403
	return {
		user
	};
};
