import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

// Redirect to home if already logged in
export const load: PageServerLoad = async ({ cookies }) => {
	const token = cookies.get('auth_token');
	if (token) {
		throw redirect(303, '/');
	}
	return {};
};

export const actions = {
	default: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const username = data.get('username');
		const password = data.get('password');
		const confirmPassword = data.get('confirmPassword');

		if (!username || !password || !confirmPassword) {
			return fail(400, { error: 'All fields are required' });
		}

		if (password !== confirmPassword) {
			return fail(400, { error: 'Passwords do not match' });
		}

		try {
			// Get public key
			const keyResponse = await fetch('http://localhost:8000/api/public-key');
			if (!keyResponse.ok) {
				return fail(500, { error: 'Failed to get encryption key' });
			}
			const { public_key } = await keyResponse.json();

			// Import node-forge for server-side encryption
			const forge = await import('node-forge');
			const publicKeyObj = forge.default.pki.publicKeyFromPem(public_key);
			const encrypted = publicKeyObj.encrypt(password as string, 'RSA-OAEP');
			const encryptedPassword = forge.default.util.encode64(encrypted);

			// Register request
			const registerResponse = await fetch('http://localhost:8000/api/register', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					username,
					encrypted_password: encryptedPassword
				})
			});

			if (!registerResponse.ok) {
				if (registerResponse.status === 409) {
					return fail(409, { error: 'Username already exists' });
				}
				return fail(500, { error: 'Registration failed' });
			}

			const authData = await registerResponse.json();
			
			// Set HTTP-only cookie
			cookies.set('auth_token', authData.token, {
				path: '/',
				httpOnly: true,
				sameSite: 'strict',
				secure: false, // Set to true in production with HTTPS
				maxAge: 60 * 60 * 24 * 7 // 7 days
			});

			// Redirect to home
			throw redirect(303, '/');
		} catch (error) {
			if (error instanceof Response) throw error;
			console.error('Registration error:', error);
			return fail(500, { error: 'An unexpected error occurred' });
		}
	}
} satisfies Actions;
