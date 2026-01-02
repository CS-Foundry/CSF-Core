import type { LayoutServerLoad } from './$types';
import jwt from 'jsonwebtoken';
import { env } from '$env/dynamic/private';

interface User {
  id: string;
  username: string;
}

export const load: LayoutServerLoad = async ({ cookies }) => {
  console.log('[+layout.server.ts] Executing load function...');
  const token = cookies.get('auth_token');

  if (token) {
    console.log('[+layout.server.ts] Auth token found:', token.substring(0, 15) + '...');
    try {
      const JWT_SECRET = env.JWT_SECRET;
      if (!JWT_SECRET) {
        console.error('[+layout.server.ts] JWT_SECRET is not defined!');
        cookies.delete('auth_token', { path: '/' });
        return { user: null, token: null };
      }
      console.log('[+layout.server.ts] Verifying token...');
      const decoded = jwt.verify(token, JWT_SECRET) as jwt.JwtPayload;
      console.log('[+layout.server.ts] Token verified successfully. Decoded payload:', decoded);

      if (decoded && typeof decoded === 'object') {
        const user: User = {
          id: decoded.user_id,
          username: decoded.username,
        };
        console.log('[+layout.server.ts] Returning user and token:', { user, token });
        return { user, token };
      }
    } catch (error) {
      console.error('[+layout.server.ts] Failed to verify token:', error);
      cookies.delete('auth_token', { path: '/' });
      return { user: null, token: null };
    }
  } else {
    console.log('[+layout.server.ts] No auth token found in cookies.');
  }

  return { user: null, token: null };
};
