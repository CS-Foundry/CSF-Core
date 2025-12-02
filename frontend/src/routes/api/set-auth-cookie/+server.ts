import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request, cookies }) => {
	const { token } = await request.json();
	
	// If token is null or empty, delete the cookie
	if (!token) {
		cookies.delete('auth_token', { path: '/' });
		return json({ success: true });
	}
	
	// Set HTTP-only cookie
	cookies.set('auth_token', token, {
		path: '/',
		httpOnly: true,
		sameSite: 'strict',
		secure: false, // Set to true in production with HTTPS
		maxAge: 60 * 60 * 24 * 7 // 7 days
	});
	
	return json({ success: true });
};
